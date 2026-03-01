// Error context and propagation
// Provides ErrorContext struct with workspace_id, operation details
// Includes ToErrorCode(), ToErrorResponse() methods for Error/Kind

use serde::{Deserialize, Serialize};

use crate::Error::Kind::Kind;

/// Error context with workspace_id and operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
	/// The workspace ID where the error occurred
	pub workspace_id:Option<String>,
	/// The operation that was being performed
	pub operation:String,
	/// Additional context details
	pub details:Option<String>,
	/// Timestamp when the error occurred (ISO 8601)
	pub timestamp:String,
}

impl ErrorContext {
	/// Create a new error context
	pub fn new(operation:impl Into<String>) -> Self {
		Self {
			workspace_id:None,
			operation:operation.into(),
			details:None,
			timestamp:chrono::Utc::now().to_rfc3339(),
		}
	}

	/// Create with workspace ID
	pub fn with_workspace(mut self, workspace_id:impl Into<String>) -> Self {
		self.workspace_id = Some(workspace_id.into());
		self
	}

	/// Create with additional details
	pub fn with_details(mut self, details:impl Into<String>) -> Self {
		self.details = Some(details.into());
		self
	}

	/// Create from current context
	pub fn from_current(operation:impl Into<String>) -> Self { Self::new(operation) }
}

/// HTTP error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
	/// Error code for programmatic handling
	pub error_code:String,
	/// Human-readable error message
	pub message:String,
	/// Error context
	pub context:Option<ErrorContext>,
	/// Inner error details (for debugging)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub inner:Option<String>,
}

impl ErrorResponse {
	/// Create a new error response
	pub fn new(error_code:impl Into<String>, message:impl Into<String>) -> Self {
		Self { error_code:error_code.into(), message:message.into(), context:None, inner:None }
	}

	/// Add error context
	pub fn with_context(mut self, context:ErrorContext) -> Self {
		self.context = Some(context);
		self
	}

	/// Add inner error details
	pub fn with_inner(mut self, inner:impl Into<String>) -> Self {
		self.inner = Some(inner.into());
		self
	}
}

/// Error code mapping for Error/Kind
impl Kind {
	/// Convert Error/Kind to error code string
	pub fn to_error_code(&self) -> String {
		match self {
			Kind::Database(_) => "DATABASE_ERROR",
			Kind::Configuration(_) => "CONFIGURATION_ERROR",
			Kind::WorkspaceDetection(_) => "WORKSPACE_DETECTION_ERROR",
			Kind::VectorStore(_) => "VECTOR_STORE_ERROR",
			Kind::Http(_) => "HTTP_ERROR",
			Kind::Embedding(_) => "EMBEDDING_ERROR",
			Kind::Serialization(_) => "SERIALIZATION_ERROR",
			Kind::NotFound(_) => "NOT_FOUND",
			Kind::InvalidInput(_) => "INVALID_INPUT",
		}
		.to_string()
	}

	/// Convert Error/Kind to ErrorResponse
	pub fn to_error_response(&self) -> ErrorResponse { ErrorResponse::new(self.to_error_code(), self.to_string()) }

	/// Convert Error/Kind to ErrorResponse with context
	pub fn to_error_response_with_context(&self, context:ErrorContext) -> ErrorResponse {
		self.to_error_response().with_context(context)
	}
}

/// Log with context helper for structured logging
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $ctx:expr, $($arg:tt)*) => {
        tracing::$level!(
            workspace_id = $ctx.workspace_id.as_deref(),
            operation = %$ctx.operation,
            details = $ctx.details.as_deref(),
            $($arg)*
        )
    };
}

/// Log error with context helper
#[macro_export]
macro_rules! log_error_with_context {
    ($ctx:expr, $error:expr) => {
        tracing::error!(
            workspace_id = $ctx.workspace_id.as_deref(),
            operation = %$ctx.operation,
            details = $ctx.details.as_deref(),
            error = %$error,
            error_code = %$error.to_error_code()
        )
    };
}

/// Extension trait for Result to add context-aware error handling
pub trait ResultExt<T, E> {
	/// Add workspace context to error
	fn with_workspace_context(self, workspace_id:impl Into<String>, operation:impl Into<String>) -> Result<T, E>;

	/// Map error to include context
	fn with_error_context<F>(self, operation:impl Into<String>, f:F) -> Result<T, E>
	where
		F: FnOnce(&E) -> String;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
	fn with_workspace_context(self, workspace_id:impl Into<String>, operation:impl Into<String>) -> Result<T, E> {
		// This is a marker method - actual implementation would depend on error type
		// For now, we just return the result as-is
		// Real implementation would wrap errors with context
		self
	}

	fn with_error_context<F>(self, operation:impl Into<String>, f:F) -> Result<T, E>
	where
		F: FnOnce(&E) -> String, {
		self
	}
}

/// Create error context from workspace ID and operation
pub fn create_error_context(workspace_id:Option<&str>, operation:&str) -> ErrorContext {
	let mut ctx = ErrorContext::new(operation);
	if let Some(id) = workspace_id {
		ctx = ctx.with_workspace(id);
	}
	ctx
}

/// Helper to convert any error to ErrorResponse
pub fn to_error_response<E:std::fmt::Display>(error:&E) -> ErrorResponse {
	ErrorResponse::new("INTERNAL_ERROR", error.to_string())
}

/// Helper to convert any error to ErrorResponse with context
pub fn to_error_response_with_context<E:std::fmt::Display>(
	error:&E,
	workspace_id:Option<&str>,
	operation:&str,
) -> ErrorResponse {
	let ctx = create_error_context(workspace_id, operation);
	ErrorResponse::new("INTERNAL_ERROR", error.to_string()).with_context(ctx)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_error_context_creation() {
		let ctx = ErrorContext::new("test_operation");
		assert_eq!(ctx.operation, "test_operation");
		assert!(ctx.workspace_id.is_none());
	}

	#[test]
	fn test_error_context_with_workspace() {
		let ctx = ErrorContext::new("test_operation")
			.with_workspace("workspace_123")
			.with_details("additional info");

		assert_eq!(ctx.workspace_id, Some("workspace_123".to_string()));
		assert_eq!(ctx.details, Some("additional info".to_string()));
	}

	#[test]
	fn test_error_kind_to_code() {
		let err = Kind::Database("connection failed".to_string());
		assert_eq!(err.to_error_code(), "DATABASE_ERROR");

		let err = Kind::NotFound("item not found".to_string());
		assert_eq!(err.to_error_code(), "NOT_FOUND");
	}

	#[test]
	fn test_error_response_creation() {
		let resp = ErrorResponse::new("TEST_ERROR", "Test message");
		assert_eq!(resp.error_code, "TEST_ERROR");
		assert_eq!(resp.message, "Test message");
	}
}
