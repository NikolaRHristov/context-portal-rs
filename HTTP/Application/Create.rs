// Create HTTP application for the ConPort MCP server
use axum::{
    routing::get,
    Router,
};

pub async fn Create() -> Result<Router, crate::Error::Kind::Kind> {
    let App = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    Ok(App)
}

async fn root() -> &'static str {
    "ConPort MCP Server"
}

async fn health() -> &'static str {
    "OK"
}