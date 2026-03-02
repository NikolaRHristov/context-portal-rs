// Error kinds for the ConPort MCP server
use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Kind {
	#[error("Database error: {0}")]
	Database(String),

	#[error("Configuration error: {0}")]
	Configuration(String),

	#[error("Workspace detection error: {0}")]
	WorkspaceDetection(String),

	#[error("Vector store error: {0}")]
	VectorStore(String),

	#[error("HTTP error: {0}")]
	Http(String),

	#[error("Embedding error: {0}")]
	Embedding(String),

	#[error("Serialization error: {0}")]
	Serialization(String),

	#[error("Not found: {0}")]
	NotFound(String),

	#[error("Invalid input: {0}")]
	InvalidInput(String),

	#[error("Invalid item type: {0}")]
	InvalidItemType(String),

	#[error("Database error: {0}")]
	DatabaseError(String),

	#[error("Model error: {0}")]
	Model(String),

	#[error("Server error: {0}")]
	Server(String),

	#[error("Transport error: {0}")]
	Transport(String),
}

impl IntoResponse for Kind {
	fn into_response(self) -> Response {
		let (status, message) = match self {
			Kind::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::Configuration(e) => (StatusCode::BAD_GATEWAY, e),
			Kind::WorkspaceDetection(e) => (StatusCode::BAD_GATEWAY, e),
			Kind::VectorStore(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::Http(e) => (StatusCode::BAD_REQUEST, e),
			Kind::Embedding(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::Serialization(e) => (StatusCode::BAD_REQUEST, e),
			Kind::NotFound(e) => (StatusCode::NOT_FOUND, e),
			Kind::InvalidInput(e) => (StatusCode::BAD_REQUEST, e),
			Kind::InvalidItemType(e) => (StatusCode::BAD_REQUEST, e),
			Kind::Model(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::Server(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
			Kind::Transport(e) => (StatusCode::SERVICE_UNAVAILABLE, e),
		};

		let body = Json(serde_json::json!({
			"error": message,
		}));

		(status, body).into_response()
	}
}
