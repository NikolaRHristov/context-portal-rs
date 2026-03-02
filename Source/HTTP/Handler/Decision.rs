// Decision HTTP handler for the ConPort MCP server
use std::sync::Arc;

use axum::{
	Json,
	extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{
	Persistence::Database::Operations as DbOps,
	Type::Decision::{Decision, GetDecisionsArgs, LogDecisionArgs, UpdateDecisionArgs},
};

/// Query args for decision endpoints
#[derive(Debug, Deserialize)]
pub struct DecisionQueryArgs {
	pub workspace_id:String,
	pub limit:Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct DecisionResponse {
	pub Id:String,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Summary:String,
	pub Rationale:Option<String>,
	pub ImplementationDetails:Option<String>,
	pub Tags:Option<Vec<String>>,
	pub CreatedAt:String,
}

impl From<Decision> for DecisionResponse {
	fn from(d:Decision) -> Self {
		let tags:Option<Vec<String>> = d.Tags.as_ref().and_then(|t| serde_json::from_str(t).ok());
		Self {
			Id:d.Id.to_string(),
			WorkspaceId:d.WorkspaceId,
			Timestamp:d.Timestamp,
			Summary:d.Summary,
			Rationale:d.Rationale,
			ImplementationDetails:d.ImplementationDetails,
			Tags:tags,
			CreatedAt:d.CreatedAt,
		}
	}
}

pub async fn List(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<DecisionQueryArgs>,
) -> Result<Json<Vec<DecisionResponse>>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let get_args = GetDecisionsArgs { workspace_id:args.workspace_id, limit:args.limit };

	let decisions = DbOps::get_decisions(&conn, &get_args.workspace_id, &get_args)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(decisions.into_iter().map(DecisionResponse::from).collect()))
}

pub async fn Get(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Path(id):Path<i64>,
	Query(args):Query<DecisionQueryArgs>,
) -> Result<Json<DecisionResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let decision = DbOps::get_decision_by_id(&conn, &args.workspace_id, id)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?
		.ok_or_else(|| crate::Error::Kind::Kind::NotFound(format!("Decision {} not found", id)))?;

	Ok(Json(DecisionResponse::from(decision)))
}

pub async fn Create(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<LogDecisionArgs>,
) -> Result<Json<DecisionResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let decision = DbOps::log_decision(&conn, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(DecisionResponse::from(decision)))
}

pub async fn Update(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<UpdateDecisionArgs>,
) -> Result<Json<UpdateResponse>, crate::Error::Kind::Kind> {
	// Validate input
	args.validate().map_err(|e| crate::Error::Kind::Kind::InvalidInput(e))?;

	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let updated =
		DbOps::update_decision(&conn, &args.workspace_id, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if updated {
		Ok(Json(UpdateResponse {
			status:"success".to_string(),
			message:format!("Decision ID {} updated successfully.", args.decision_id),
		}))
	} else {
		Ok(Json(UpdateResponse {
			status:"success".to_string(),
			message:format!("Decision ID {} not found for update.", args.decision_id),
		}))
	}
}

pub async fn Delete(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Path(id):Path<i64>,
	Query(args):Query<DecisionQueryArgs>,
) -> Result<Json<DeleteResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let deleted =
		DbOps::delete_decision(&conn, &args.workspace_id, id).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if deleted {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("Decision ID {} deleted successfully.", id),
		}))
	} else {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("Decision ID {} not found in database.", id),
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
