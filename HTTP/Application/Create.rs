// Create HTTP application for the ConPort MCP server
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;

// Import handlers
use crate::HTTP::Handler::Context;
use crate::HTTP::Handler::Decision;
use crate::HTTP::Handler::Progress;
use crate::HTTP::Handler::Pattern;
use crate::HTTP::Handler::CustomDataHandler;
use crate::HTTP::Handler::SystemPattern;
use crate::HTTP::Handler::Search;
use crate::HTTP::Handler::History;
use crate::HTTP::Handler::Link;
use crate::HTTP::Handler::Batch;
use crate::HTTP::Handler::ImportExport;

// Import database connection type
use crate::Persistence::Database::Connect::Connect;

pub async fn Create(
    db: Arc<Connect>,
) -> Result<Router, crate::Error::Kind::Kind> {
    let App = Router::new()
        // Root and health
        .route("/", get(root))
        .route("/health", get(health))
        
        // Context endpoints (product_context and active_context)
        .route("/api/v1/context", get(Context::List))
        .route("/api/v1/context", post(Context::Create))
        .route("/api/v1/context/product", get(Context::GetProductContext))
        .route("/api/v1/context/product", put(Context::UpdateProductContext))
        .route("/api/v1/context/active", get(Context::GetActiveContext))
        .route("/api/v1/context/active", put(Context::UpdateActiveContext))
        .route("/api/v1/context/:id", delete(Context::Delete))
        
        // Decision endpoints
        .route("/api/v1/decisions", get(Decision::List))
        .route("/api/v1/decisions", post(Decision::Create))
        .route("/api/v1/decisions/:id", get(Decision::Get))
        .route("/api/v1/decisions/:id", delete(Decision::Delete))
        
        // Progress endpoints
        .route("/api/v1/progress", get(Progress::List))
        .route("/api/v1/progress", post(Progress::Create))
        .route("/api/v1/progress/:id", get(Progress::Get))
        .route("/api/v1/progress/:id", put(Progress::Update))
        .route("/api/v1/progress/:id", delete(Progress::Delete))
        
        // Pattern endpoints
        .route("/api/v1/patterns", get(Pattern::List))
        .route("/api/v1/patterns", post(Pattern::Create))
        .route("/api/v1/patterns/:id", get(Pattern::Get))
        .route("/api/v1/patterns/:id", put(Pattern::Update))
        .route("/api/v1/patterns/:id", delete(Pattern::Delete))
        
        // Custom Data endpoints
        .route("/api/v1/custom-data", get(CustomDataHandler::List))
        .route("/api/v1/custom-data", post(CustomDataHandler::Create))
        .route("/api/v1/custom-data", delete(CustomDataHandler::Delete))
        .route("/api/v1/custom-data/search", post(CustomDataHandler::Search))
        
        // System Pattern endpoints
        .route("/api/v1/system-patterns", get(SystemPattern::List))
        .route("/api/v1/system-patterns", post(SystemPattern::Create))
        .route("/api/v1/system-patterns/:id", delete(SystemPattern::Delete))
        
        // Search endpoints
        .route("/api/v1/search/decisions", post(Search::SearchDecisions))
        .route("/api/v1/search/context", post(Search::SearchContext))
        .route("/api/v1/search/semantic", post(Search::SemanticSearch))
        
        // History endpoints
        .route("/api/v1/history", get(History::GetItemHistory))
        .route("/api/v1/activity", get(History::GetRecentActivity))
        
        // Link endpoints
        .route("/api/v1/links", post(Link::LinkItems))
        .route("/api/v1/links", get(Link::GetLinkedItems))
        .route("/api/v1/links", delete(Link::UnlinkItems))
        
        // Batch endpoints
        .route("/api/v1/batch", post(Batch::BatchLogItems))
        .route("/api/v1/batch/delete", post(Batch::BatchDelete))
        
        // Import/Export endpoints
        .route("/api/v1/export/markdown", post(ImportExport::ExportMarkdown))
        .route("/api/v1/import/markdown", post(ImportExport::ImportMarkdown))
        
        // Schema endpoint
        .route("/api/v1/schema", get(Schema))
        
        // Workspace detection endpoint
        .route("/api/v1/workspace/detect", get(WorkspaceDetect))
        
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
    r#"{"/api/v1/context": "GET, POST", "/api/v1/decisions": "GET, POST", "/api/v1/progress": "GET, POST, PUT, DELETE", "/api/v1/custom-data": "GET, POST, DELETE", "/api/v1/search/*": "POST", "/api/v1/history": "GET", "/api/v1/links": "GET, POST, DELETE", "/api/v1/batch": "POST"}"#
}

/// Workspace detection endpoint - returns workspace info
async fn WorkspaceDetect() -> &'static str {
    r#"{"detected": true, "method": "current_directory"}"#
}