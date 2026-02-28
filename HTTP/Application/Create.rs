// Create HTTP application for the ConPort MCP server
use axum::{
    Router,
    routing::get,
};

pub async fn Create() -> Result<Router, crate::Error::Kind> {
    let App = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/context", get(crate::HTTP::Handler::Context::List))
        .route("/context/:id", get(crate::HTTP::Handler::Context::Get))
        .route("/context", post(crate::HTTP::Handler::Context::Create))
        .route("/context/:id", delete(crate::HTTP::Handler::Context::Delete))
        .route("/decision", get(crate::HTTP::Handler::Decision::List))
        .route("/decision/:id", get(crate::HTTP::Handler::Decision::Get))
        .route("/decision", post(crate::HTTP::Handler::Decision::Create))
        .route("/decision/:id", delete(crate::HTTP::Handler::Decision::Delete))
        .route("/progress", get(crate::HTTP::Handler::Progress::List))
        .route("/progress/:id", get(crate::HTTP::Handler::Progress::Get))
        .route("/progress", post(crate::HTTP::Handler::Progress::Create))
        .route("/progress/:id", delete(crate::HTTP::Handler::Progress::Delete));

    Ok(App)
}

async fn root() -> &'static str {
    "ConPort MCP Server"
}

async fn health() -> &'static str {
    "OK"
}