// SystemPattern HTTP handler for the ConPort MCP server
use std::sync::Arc;

use axum::{
	Json,
	extract::{Path, Query, State},
};

use crate::{
	Persistence::Database::Operations as DbOps,
	Type::SystemPattern::{
		DeleteSystemPatternByIdArgs,
		GetSystemPatternsArgs,
		LogSystemPatternArgs,
		SystemPatternResponse,
	},
};

/// List system patterns with optional tag filters
pub async fn List(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<GetSystemPatternsArgs>,
) -> Result<Json<Vec<SystemPatternResponse>>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let patterns = DbOps::get_system_patterns(&conn, &args.workspace_id, &args)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(
		patterns
			.into_iter()
			.map(|p| {
				SystemPatternResponse {
					id:p.Id,
					workspace_id:p.WorkspaceId,
					timestamp:p.Timestamp,
					name:p.Name,
					description:p.Description,
					tags:p.Tags,
					created_at:p.CreatedAt,
				}
			})
			.collect(),
	))
}

/// Create a new system pattern
pub async fn Create(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<LogSystemPatternArgs>,
) -> Result<Json<SystemPatternResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let pattern = DbOps::log_system_pattern(&conn, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(SystemPatternResponse {
		id:pattern.Id,
		workspace_id:pattern.WorkspaceId,
		timestamp:pattern.Timestamp,
		name:pattern.Name,
		description:pattern.Description,
		tags:pattern.Tags,
		created_at:pattern.CreatedAt,
	}))
}

/// Delete a system pattern by ID
pub async fn Delete(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Path(_pattern_id):Path<i64>,
	Query(args):Query<DeleteSystemPatternByIdArgs>,
) -> Result<Json<DeleteResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let deleted = DbOps::delete_system_pattern(&conn, &args.workspace_id, args.pattern_id)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if deleted {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("System pattern ID {} deleted successfully.", args.pattern_id),
		}))
	} else {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("System pattern ID {} not found in database.", args.pattern_id),
		}))
	}
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(serde::Serialize)]
pub struct DeleteResponse {
	pub status:String,
	pub message:String,
}
