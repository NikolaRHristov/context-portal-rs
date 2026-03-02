// Context HTTP handler for the ConPort MCP server
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use crate::Persistence::Database::Operations as DbOps;

/// Query args for context endpoints
#[derive(Debug, Deserialize)]
pub struct ContextQueryArgs {
    pub workspace_id:String,
}

#[derive(Serialize, Deserialize)]
pub struct ContextResponse {
    pub Id:i64,
    pub Content:serde_json::Value,
}

impl From<(i64, String)> for ContextResponse {
    fn from((id, content):(i64, String)) -> Self {
        let parsed:serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Object(Default::default()));
        Self { Id:id, Content:parsed }
    }
}

pub async fn List(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Query(args):Query<ContextQueryArgs>,
) -> Result<Json<Vec<ContextResponse>>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    // Get product context (Id = 1)
    let product_content:String = conn
        .query_row("SELECT Content FROM ProductContext WHERE Id = 1", [], |row| row.get(0))
        .unwrap_or_else(|_| "{}".to_string());

    // Get active context (Id = 1)
    let active_content:String = conn
        .query_row("SELECT Content FROM ActiveContext WHERE Id = 1", [], |row| row.get(0))
        .unwrap_or_else(|_| "{}".to_string());

    Ok(Json(vec![
        ContextResponse::from((1, product_content)),
        ContextResponse::from((2, active_content)),
    ]))
}

pub async fn Get(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Path(Id):Path<i64>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    let table = match Id {
        1 => "ProductContext",
        2 => "ActiveContext",
        _ => return Err(crate::Error::Kind::Kind::NotFound(format!("Context {} not found", Id))),
    };

    let content:String = conn
        .query_row(&format!("SELECT Content FROM {} WHERE Id = 1", table), [], |row| row.get(0))
        .map_err(|_| crate::Error::Kind::Kind::NotFound(format!("Context {} not found", Id)))?;

    Ok(Json(ContextResponse::from((Id, content))))
}

pub async fn Create(
    State(_State):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(_Payload):Json<CreateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Contexts are pre-seeded, so creation not needed
    unimplemented!()
}

pub async fn Delete(
    State(_State):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Path(Id):Path<i64>,
) -> Result<(), crate::Error::Kind::Kind> {
    // Contexts shouldn't be deleted
    Err(crate::Error::Kind::Kind::NotFound(format!("Context {} not found", Id)))
}

#[derive(Deserialize)]
pub struct CreateContextRequest {
    pub Content:serde_json::Value,
}

/// Get product context - returns the overall project
/// goals/features/architecture
pub async fn GetProductContext(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    // Use database operation
    let context = DbOps::get_product_context(&conn)
        .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    let content_str = serde_json::to_string(&context.Content).unwrap_or_else(|_| "{}".to_string());
    Ok(Json(ContextResponse::from((1, content_str))))
}

/// Update product context - accepts full content or patch_content
pub async fn UpdateProductContext(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(payload):Json<UpdateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    // Convert HTTP request to database operation args
    let args = DbOps::UpdateContextArgs {
        workspace_id: payload.workspace_id.clone(),
        content: payload.Content,
        patch_content: payload.PatchContent,
    };

    // Use database operation
    let context = DbOps::update_product_context(
        &conn,
        &args.workspace_id.unwrap_or_default(),
        &args,
    )
    .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    let content_str = serde_json::to_string(&context.Content).unwrap_or_else(|_| "{}".to_string());
    Ok(Json(ContextResponse::from((1, content_str))))
}

/// Get active context - returns current working focus/recent changes
pub async fn GetActiveContext(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    // Use database operation
    let context = DbOps::get_active_context(&conn)
        .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    let content_str = serde_json::to_string(&context.Content).unwrap_or_else(|_| "{}".to_string());
    Ok(Json(ContextResponse::from((2, content_str))))
}

/// Update active context - accepts full content or patch_content
pub async fn UpdateActiveContext(
    State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(payload):Json<UpdateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    let conn = state
        .Connection
        .lock()
        .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

    // Convert HTTP request to database operation args
    let args = DbOps::UpdateContextArgs {
        workspace_id: payload.workspace_id.clone(),
        content: payload.Content,
        patch_content: payload.PatchContent,
    };

    // Use database operation
    let context = DbOps::update_active_context(
        &conn,
        &args.workspace_id.unwrap_or_default(),
        &args,
    )
    .map_err(|e| crate::Error::Kind::Kind::Database(e))?;

    let content_str = serde_json::to_string(&context.Content).unwrap_or_else(|_| "{}".to_string());
    Ok(Json(ContextResponse::from((2, content_str))))
}

#[derive(Deserialize)]
pub struct UpdateContextRequest {
    pub workspace_id:Option<String>,
    pub Content:Option<serde_json::Value>,
    pub PatchContent:Option<serde_json::Value>,
}