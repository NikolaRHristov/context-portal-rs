// Context HTTP handler for the ConPort MCP server
use axum::{
    extract::Path,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct ContextResponse {
    pub Id: String,
    pub Name: String,
    pub Content: String,
    pub WorkspacePath: String,
    pub CreatedAt: String,
    pub UpdatedAt: String,
}

pub async fn List(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
) -> Result<Json<Vec<ContextResponse>>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would query the database
    Ok(Json(vec![]))
}

pub async fn Get(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Path(Id): Path<String>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would query the database
    Err(crate::Error::Kind::Kind::NotFound(format!("Context {} not found", Id)))
}

pub async fn Create(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(_Payload): Json<CreateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would create in database
    unimplemented!()
}

pub async fn Delete(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Path(Id): Path<String>,
) -> Result<(), crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would delete from database
    Err(crate::Error::Kind::Kind::NotFound(format!("Context {} not found", Id)))
}

#[derive(Deserialize)]
pub struct CreateContextRequest {
    pub Name: String,
    pub Content: String,
    pub WorkspacePath: String,
}

/// Get product context - returns the overall project goals/features/architecture
pub async fn GetProductContext(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would query product context from DB
    Ok(Json(ContextResponse {
        Id: "product".to_string(),
        Name: "Product Context".to_string(),
        Content: "{}".to_string(),
        WorkspacePath: "".to_string(),
        CreatedAt: chrono::Utc::now().to_rfc3339(),
        UpdatedAt: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Update product context - accepts full content or patch_content
pub async fn UpdateProductContext(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(_Payload): Json<UpdateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would update product context
    Ok(Json(ContextResponse {
        Id: "product".to_string(),
        Name: "Product Context".to_string(),
        Content: "{}".to_string(),
        WorkspacePath: "".to_string(),
        CreatedAt: chrono::Utc::now().to_rfc3339(),
        UpdatedAt: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Get active context - returns current working focus/recent changes
pub async fn GetActiveContext(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would query active context from DB
    Ok(Json(ContextResponse {
        Id: "active".to_string(),
        Name: "Active Context".to_string(),
        Content: "{}".to_string(),
        WorkspacePath: "".to_string(),
        CreatedAt: chrono::Utc::now().to_rfc3339(),
        UpdatedAt: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Update active context - accepts full content or patch_content
pub async fn UpdateActiveContext(
    State(_State): State<Arc<crate::Persistence::Database::Connect::Connect>>,
    Json(_Payload): Json<UpdateContextRequest>,
) -> Result<Json<ContextResponse>, crate::Error::Kind::Kind> {
    // Placeholder - actual implementation would update active context
    Ok(Json(ContextResponse {
        Id: "active".to_string(),
        Name: "Active Context".to_string(),
        Content: "{}".to_string(),
        WorkspacePath: "".to_string(),
        CreatedAt: chrono::Utc::now().to_rfc3339(),
        UpdatedAt: chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(Deserialize)]
pub struct UpdateContextRequest {
    pub Content: Option<serde_json::Value>,
    pub PatchContent: Option<serde_json::Value>,
}