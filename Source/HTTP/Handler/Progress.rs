// Progress HTTP handler for the ConPort MCP server
use std::sync::Arc;

use axum::{
	Json,
	extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{
	Persistence::Database::Operations as DbOps,
	Type::Progress::{
		DeleteProgressByIdArgs,
		GetProgressArgs,
		LogProgressArgs,
		Progress,
		ProgressResponse,
		UpdateProgressArgs,
	},
};

/// List progress entries with optional filters
pub async fn List(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<GetProgressArgs>,
) -> Result<Json<Vec<ProgressResponse>>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let progress_entries =
		DbOps::get_progress(&conn, &args.workspace_id, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(
		progress_entries
			.into_iter()
			.map(|p| {
				ProgressResponse {
					id:p.Id,
					workspace_id:p.WorkspaceId,
					timestamp:p.Timestamp,
					status:p.Status,
					description:p.Description,
					parent_id:p.ParentId,
					created_at:p.CreatedAt,
					updated_at:p.UpdatedAt,
				}
			})
			.collect(),
	))
}

/// Get a single progress entry by ID
pub async fn Get(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Path(id):Path<i64>,
	Query(args):Query<GetProgressArgs>,
) -> Result<Json<ProgressResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	// Get all progress and filter by ID
	let progress_entries = DbOps::get_progress(
		&conn,
		&args.workspace_id,
		&GetProgressArgs {
			workspace_id:args.workspace_id,
			status_filter:None,
			parent_id_filter:None,
			limit:Some(1000), // Get enough to find by ID
		},
	)
	.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	// Find by ID (convert string ID to i64 for comparison)
	if let Some(p) = progress_entries
		.into_iter()
		.find(|p| p.Id.parse::<i64>().map(|i| i == id).unwrap_or(false))
	{
		Ok(Json(ProgressResponse {
			id:p.Id,
			workspace_id:p.WorkspaceId,
			timestamp:p.Timestamp,
			status:p.Status,
			description:p.Description,
			parent_id:p.ParentId,
			created_at:p.CreatedAt,
			updated_at:p.UpdatedAt,
		}))
	} else {
		Err(crate::Error::Kind::Kind::NotFound(format!("Progress {} not found", id)))
	}
}

/// Create a new progress entry
pub async fn Create(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<LogProgressArgs>,
) -> Result<Json<ProgressResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let progress = DbOps::log_progress(&conn, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	// Handle auto-linking if provided
	if let (Some(linked_type), Some(linked_id)) = (&args.linked_item_type, &args.linked_item_id) {
		let link_args = crate::Type::ContextLink::LinkConportItemsArgs {
			workspace_id:args.workspace_id.clone(),
			source_item_type:"progress_entry".to_string(),
			source_item_id:progress.Id.clone(),
			target_item_type:linked_type.clone(),
			target_item_id:linked_id.clone(),
			relationship_type:args.link_relationship_type.unwrap_or_else(|| "relates_to_progress".to_string()),
			description:None,
		};

		if let Err(e) = link_args.validate() {
			// Log the linking error but don't fail the progress creation
			tracing::warn!("Failed to validate link for progress: {}", e);
		} else {
			// Create the link
			let _ = DbOps::create_link(&conn, &link_args);
		}
	}

	Ok(Json(ProgressResponse {
		id:progress.Id,
		workspace_id:progress.WorkspaceId,
		timestamp:progress.Timestamp,
		status:progress.Status,
		description:progress.Description,
		parent_id:progress.ParentId,
		created_at:progress.CreatedAt,
		updated_at:progress.UpdatedAt,
	}))
}

/// Update an existing progress entry
pub async fn Update(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<UpdateProgressArgs>,
) -> Result<Json<UpdateResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let updated =
		DbOps::update_progress(&conn, &args.workspace_id, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if updated {
		Ok(Json(UpdateResponse {
			status:"success".to_string(),
			message:format!("Progress entry ID {} updated successfully.", args.progress_id),
		}))
	} else {
		Ok(Json(UpdateResponse {
			status:"success".to_string(),
			message:format!("Progress entry ID {} not found for update.", args.progress_id),
		}))
	}
}

/// Delete a progress entry by ID
pub async fn Delete(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Path(progress_id):Path<i64>,
	Query(args):Query<DeleteProgressByIdArgs>,
) -> Result<Json<DeleteResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let deleted = DbOps::delete_progress(&conn, &args.workspace_id, args.progress_id)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if deleted {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("Progress entry ID {} deleted successfully.", args.progress_id),
		}))
	} else {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("Progress entry ID {} not found in database.", args.progress_id),
		}))
	}
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(serde::Serialize)]
pub struct UpdateResponse {
	pub status:String,
	pub message:String,
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
	pub status:String,
	pub message:String,
}
