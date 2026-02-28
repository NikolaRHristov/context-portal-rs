// Decision HTTP handler for the ConPort MCP server
use axum::{
    extract::Path,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct DecisionResponse {
    pub Id: String,
    pub ContextId: String,
    pub Description: String,
    pub Rationale: String,
    pub CreatedAt: String,
}

pub async fn List(
    State(_State): State<Arc<crate::Persistence::Database::Connect>>,
) -> Result<Json<Vec<DecisionResponse>>, crate::Error::Kind> {
    // Placeholder - actual implementation would query the database
    Ok(Json(vec![]))
}

pub async fn Get(
    State(_State): State<Arc<crate::Persistence::Database::Connect>>,
    Path(Id): Path<String>,
) -> Result<Json<DecisionResponse>, crate::Error::Kind> {
    // Placeholder - actual implementation would query the database
    Err(crate::Error::Kind::NotFound(format!("Decision {} not found", Id)))
}

pub async fn Create(
    State(_State): State<Arc<crate::Persistence::Database::Connect>>,
    Json(_Payload): Json<CreateDecisionRequest>,
) -> Result<Json<DecisionResponse>, crate::Error::Kind> {
    // Placeholder - actual implementation would create in database
    unimplemented!()
}

pub async fn Delete(
    State(_State): State<Arc<crate::Persistence::Database::Connect>>,
    Path(Id): Path<String>,
) -> Result<(), crate::Error::Kind> {
    // Placeholder - actual implementation would delete from database
    Err(crate::Error::Kind::NotFound(format!("Decision {} not found", Id)))
}

#[derive(Deserialize)]
pub struct CreateDecisionRequest {
    pub ContextId: String,
    pub Description: String,
    pub Rationale: String,
}