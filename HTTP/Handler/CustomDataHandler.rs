// CustomData HTTP handler for the ConPort MCP server
use axum::{
    extract::{Query, State},
    Json,
};
use std::sync::Arc;

use crate::Type::CustomData::{
    CustomData, LogCustomDataArgs, GetCustomDataArgs, DeleteCustomDataArgs,
    SearchCustomDataValueArgs, CustomDataResponse,
};
use crate::Persistence::Database::Operations as DbOps;

/// List custom data with optional category/key filters
pub async fn List(
    State(state): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Query(args): Query<GetCustomDataArgs>,
) -> Result<Json<Vec<CustomDataResponse>>, crate::Error::Kind::Kind> {
    // Validate input
    args.validate()
        .map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

    let conn = state.Connection.lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    let data_list = DbOps::get_custom_data(&conn, &args.workspace_id, &args)
        .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    Ok(Json(data_list.into_iter().map(|d| CustomDataResponse {
        id: d.Id,
        workspace_id: d.WorkspaceId,
        timestamp: d.Timestamp,
        category: d.Category,
        key: d.Key,
        value: d.Value,
        created_at: d.CreatedAt,
    }).collect()))
}

/// Create new custom data entry
pub async fn Create(
    State(state): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(args): Json<LogCustomDataArgs>,
) -> Result<Json<CustomDataResponse>, crate::Error::Kind::Kind> {
    // Validate input
    args.validate()
        .map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

    let conn = state.Connection.lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    let data = DbOps::log_custom_data(&conn, &args)
        .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    Ok(Json(CustomDataResponse {
        id: data.Id,
        workspace_id: data.WorkspaceId,
        timestamp: data.Timestamp,
        category: data.Category,
        key: data.Key,
        value: data.Value,
        created_at: data.CreatedAt,
    }))
}

/// Delete custom data by category and key
pub async fn Delete(
    State(state): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Query(args): Query<DeleteCustomDataArgs>,
) -> Result<Json<DeleteResponse>, crate::Error::Kind::Kind> {
    // Validate input
    args.validate()
        .map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

    let conn = state.Connection.lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    let deleted = DbOps::delete_custom_data(&conn, &args.workspace_id, &args.category, &args.key)
        .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    if deleted {
        Ok(Json(DeleteResponse {
            status: "success".to_string(),
            message: format!("Custom data '{}/{}' deleted.", args.category, args.key),
        }))
    } else {
        Ok(Json(DeleteResponse {
            status: "success".to_string(),
            message: format!("Custom data '{}/{}' not found for deletion.", args.category, args.key),
        }))
    }
}

/// Search custom data using FTS5
pub async fn Search(
    State(state): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Query(args): Query<SearchCustomDataValueArgs>,
) -> Result<Json<Vec<CustomDataResponse>>, crate::Error::Kind::Kind> {
    // Validate input
    args.validate()
        .map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

    // Prepare the FTS query
    let safe_query = DbOps::prepare_fts_query(
        &args.query_term,
        Some(&["category", "key", "value_text"]),
        Some("value_text"),
    );

    let conn = state.Connection.lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    let data_list = DbOps::search_custom_data_fts(
        &conn,
        &args.workspace_id,
        &safe_query,
        args.category_filter.as_deref(),
        args.limit_value(),
    ).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    Ok(Json(data_list.into_iter().map(|d| CustomDataResponse {
        id: d.Id,
        workspace_id: d.WorkspaceId,
        timestamp: d.Timestamp,
        category: d.Category,
        key: d.Key,
        value: d.Value,
        created_at: d.CreatedAt,
    }).collect()))
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    pub status: String,
    pub message: String,
}