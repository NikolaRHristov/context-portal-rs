// CustomData HTTP handler for the ConPort MCP server
use std::sync::Arc;

use axum::{
	Json,
	extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{
	Persistence::Database::Operations as DbOps,
	Type::CustomData::{CustomData, GetCustomDataArgs, LogCustomDataArgs},
};

/// Query args for custom data endpoints
#[derive(Debug, Deserialize)]
pub struct CustomDataQueryArgs {
	pub workspace_id:String,
	pub category:Option<String>,
	pub key:Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomDataResponse {
	pub Id:String,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Category:String,
	pub Key:String,
	pub Value:serde_json::Value,
	pub CreatedAt:String,
}

impl From<CustomData> for CustomDataResponse {
	fn from(data:CustomData) -> Self {
		Self {
			Id:data.Id,
			WorkspaceId:data.WorkspaceId,
			Timestamp:data.Timestamp,
			Category:data.Category,
			Key:data.Key,
			Value:data.Value,
			CreatedAt:data.CreatedAt,
		}
	}
}

/// List custom data entries
pub async fn List(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<CustomDataQueryArgs>,
) -> Result<Json<Vec<CustomDataResponse>>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let get_args = GetCustomDataArgs { workspace_id:args.workspace_id, category:args.category, key:args.key };

	let entries = DbOps::get_custom_data(&conn, &get_args.workspace_id, &get_args)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(entries.into_iter().map(CustomDataResponse::from).collect()))
}

/// Get a single custom data entry
pub async fn Get(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<CustomDataQueryArgs>,
) -> Result<Json<CustomDataResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let get_args = GetCustomDataArgs { workspace_id:args.workspace_id, category:args.category, key:args.key };

	let entries = DbOps::get_custom_data(&conn, &get_args.workspace_id, &get_args)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	entries
		.into_iter()
		.next()
		.map(|e| Json(CustomDataResponse::from(e)))
		.ok_or_else(|| crate::Error::Kind::Kind::NotFound(format!("CustomData not found")))
}

/// Create a new custom data entry
pub async fn Create(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(args):Json<LogCustomDataArgs>,
) -> Result<Json<CustomDataResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let entry = DbOps::log_custom_data(&conn, &args).map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	Ok(Json(CustomDataResponse::from(entry)))
}

/// Delete a custom data entry
pub async fn Delete(
	State(state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Query(args):Query<CustomDataQueryArgs>,
) -> Result<Json<DeleteResponse>, crate::Error::Kind::Kind> {
	let conn = state
		.Connection
		.lock()
		.map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

	let category = args
		.category
		.ok_or_else(|| crate::Error::Kind::Kind::InvalidInput("category is required".to_string()))?;
	let key = args
		.key
		.ok_or_else(|| crate::Error::Kind::Kind::InvalidInput("key is required".to_string()))?;

	let deleted = DbOps::delete_custom_data(&conn, &args.workspace_id, &category, &key)
		.map_err(|e| crate::Error::Kind::Kind::Database(e))?;

	if deleted {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("CustomData ({}, {}) deleted successfully.", category, key),
		}))
	} else {
		Ok(Json(DeleteResponse {
			status:"success".to_string(),
			message:format!("CustomData ({}, {}) not found in database.", category, key),
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

// ============================================================================
// MCP Protocol Handler Types (for backwards compatibility)
// ============================================================================

use crate::HTTP::Protocol::Response::Error;

#[derive(Deserialize)]
pub struct SetCustomDataRequest {
	pub workspace_id:String,
	pub category:String,
	pub key:String,
	pub value:serde_json::Value,
}

#[derive(Deserialize)]
pub struct GetCustomDataRequest {
	pub workspace_id:String,
	pub category:String,
	pub key:String,
}

// HandleSetCustomData - Standalone handler function for MCP protocol
pub struct HandleSetCustomData;

impl HandleSetCustomData {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:SetCustomDataRequest,
	) -> Result<CustomDataResponse, Error> {
		let conn = DbState.Connection.lock().map_err(|e| Error::DatabaseError(&e.to_string()))?;

		let args = LogCustomDataArgs {
			workspace_id:Payload.workspace_id,
			category:Payload.category,
			key:Payload.key,
			value:Payload.value,
		};

		let entry = DbOps::log_custom_data(&conn, &args).map_err(|e| Error::DatabaseError(&e))?;

		Ok(CustomDataResponse::from(entry))
	}
}

// HandleGetCustomData - Standalone handler function for MCP protocol
pub struct HandleGetCustomData;

impl HandleGetCustomData {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:GetCustomDataRequest,
	) -> Result<CustomDataResponse, Error> {
		let conn = DbState.Connection.lock().map_err(|e| Error::DatabaseError(&e.to_string()))?;

		let args = GetCustomDataArgs {
			workspace_id:Payload.workspace_id,
			category:Some(Payload.category),
			key:Some(Payload.key),
		};

		let entries = DbOps::get_custom_data(&conn, &args.workspace_id, &args).map_err(|e| Error::DatabaseError(&e))?;

		entries
			.into_iter()
			.next()
			.map(CustomDataResponse::from)
			.ok_or_else(|| Error::NotFound(&format!("CustomData not found")))
	}
}

// HandleDeleteCustomData - Standalone handler function for MCP protocol
pub struct HandleDeleteCustomData;

impl HandleDeleteCustomData {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:GetCustomDataRequest,
	) -> Result<(), Error> {
		let conn = DbState.Connection.lock().map_err(|e| Error::DatabaseError(&e.to_string()))?;

		let deleted = DbOps::delete_custom_data(&conn, &Payload.workspace_id, &Payload.category, &Payload.key)
			.map_err(|e| Error::DatabaseError(&e))?;

		if deleted {
			Ok(())
		} else {
			Err(Error::NotFound(&format!("CustomData not found")))
		}
	}
}
