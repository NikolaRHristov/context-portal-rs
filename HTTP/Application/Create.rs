// Create HTTP application for the ConPort MCP server
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;

// Import database connection type
use crate::Persistence::Database::Connect::Connect;

pub async fn Create(
    db: Arc<Connect>,
) -> Result<Router, crate::Error::Kind::Kind> {
    let App = Router::new()
        // Root and health
        .route("/", get(root))
        .route("/health", get(health))
        
        // Context endpoints (using direct module imports for now)
        .route("/api/v1/context", get(crate::HTTP::Handler::Context::List))
        .route("/api/v1/context/product", get(crate::HTTP::Handler::Context::GetProductContext))
        .route("/api/v1/context/product", put(crate::HTTP::Handler::Context::UpdateProductContext))
        .route("/api/v1/context/active", get(crate::HTTP::Handler::Context::GetActiveContext))
        .route("/api/v1/context/active", put(crate::HTTP::Handler::Context::UpdateActiveContext))
        
        // Decision endpoints
        .route("/api/v1/decisions", get(crate::HTTP::Handler::Decision::List))
        .route("/api/v1/decisions", post(crate::HTTP::Handler::Decision::Create))
        .route("/api/v1/decisions/:id", get(crate::HTTP::Handler::Decision::Get))
        .route("/api/v1/decisions/:id", delete(crate::HTTP::Handler::Decision::Delete))
        
        // Progress endpoints
        .route("/api/v1/progress", get(crate::HTTP::Handler::Progress::List))
        .route("/api/v1/progress", post(crate::HTTP::Handler::Progress::Create))
        .route("/api/v1/progress/:id", get(crate::HTTP::Handler::Progress::Get))
        .route("/api/v1/progress/:id", put(crate::HTTP::Handler::Progress::Update))
        .route("/api/v1/progress/:id", delete(crate::HTTP::Handler::Progress::Delete))
        
        // Custom Data endpoints
        .route("/api/v1/custom-data", get(crate::HTTP::Handler::CustomDataHandler::List))
        .route("/api/v1/custom-data", post(crate::HTTP::Handler::CustomDataHandler::Create))
        .route("/api/v1/custom-data", delete(crate::HTTP::Handler::CustomDataHandler::Delete))
        
        // System Pattern endpoints
        .route("/api/v1/system-patterns", get(crate::HTTP::Handler::SystemPattern::List))
        .route("/api/v1/system-patterns", post(crate::HTTP::Handler::SystemPattern::Create))
        .route("/api/v1/system-patterns/:id", delete(crate::HTTP::Handler::SystemPattern::Delete))
        
        // Schema endpoint
        .route("/api/v1/schema", get(Schema))
        
        // Add database state to all routes
        .with_state(db);

    Ok(App)
}

async fn root() -> &'static str {
    "ConPort MCP Server - Running"
}

async fn health() -> &'static str {
    "OK"
}

/// Schema endpoint - returns all available API endpoints
async fn Schema() -> &'static str {
    r#"{"/api/v1/context": "GET, POST", "/api/v1/decisions": "GET, POST", "/api/v1/progress": "GET, POST, PUT, DELETE", "/api/v1/custom-data": "GET, POST, DELETE"}"#
}