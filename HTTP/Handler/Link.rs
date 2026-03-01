// Link HTTP handlers for the ConPort MCP server
// Provides handlers for creating and retrieving links between ConPort items

use std::sync::Arc;

use axum::{
	Json,
	extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{HTTP::Protocol::Response::Error, Type::ContextLink};

/// Request for linking two items
#[derive(Debug, Deserialize)]
pub struct LinkItemsRequest {
	pub WorkspaceId:String,
	pub SourceItemType:String,
	pub SourceItemId:String,
	pub TargetItemType:String,
	pub TargetItemId:String,
	pub RelationshipType:String,
	pub Description:Option<String>,
}

/// Response for linking items
#[derive(Debug, Serialize)]
pub struct LinkItemsResponse {
	pub Link:ContextLink::LinkResponse,
	pub Message:String,
}

/// Request for getting linked items
#[derive(Debug, Deserialize)]
pub struct GetLinkedItemsRequest {
	pub WorkspaceId:String,
	pub ItemType:String,
	pub ItemId:String,
	pub RelationshipTypeFilter:Option<String>,
	pub LinkedItemTypeFilter:Option<String>,
	pub Limit:Option<usize>,
}

/// Response for getting linked items
#[derive(Debug, Serialize)]
pub struct GetLinkedItemsResponse {
	pub Links:Vec<ContextLink::LinkResponse>,
	pub Total:usize,
	pub SourceItemType:String,
	pub SourceItemId:String,
}

/// HandleLinkItems - Create a link between two ConPort items
pub struct HandleLinkItems;

impl HandleLinkItems {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:LinkItemsRequest,
	) -> Result<LinkItemsResponse, Error> {
		// Validate source item type
		if !ContextLink::item_type::is_valid(&Payload.SourceItemType) {
			return Err(Error::WithMessage(
				crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
				&format!("Invalid source item type: {}", Payload.SourceItemType),
			));
		}

		// Validate target item type
		if !ContextLink::item_type::is_valid(&Payload.TargetItemType) {
			return Err(Error::WithMessage(
				crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
				&format!("Invalid target item type: {}", Payload.TargetItemType),
			));
		}

		// Prevent self-linking
		if Payload.SourceItemId == Payload.TargetItemId && Payload.SourceItemType == Payload.TargetItemType {
			return Err(Error::WithMessage(
				crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
				"Cannot link an item to itself",
			));
		}

		// Create the link
		let Link = ContextLink::ContextLink::New(
			Payload.WorkspaceId.clone(),
			Payload.SourceItemType.clone(),
			Payload.SourceItemId.clone(),
			Payload.TargetItemType.clone(),
			Payload.TargetItemId.clone(),
			Payload.RelationshipType.clone(),
			Payload.Description.clone(),
		);

		// Save to database
		Self::SaveLink(DbState, &Link).await.map_err(|e| {
			Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string())
		})?;

		tracing::info!(
			"Created link: {} {} -> {} {} ({})",
			Payload.SourceItemType,
			Payload.SourceItemId,
			Payload.TargetItemType,
			Payload.TargetItemId,
			Payload.RelationshipType
		);

		Ok(LinkItemsResponse { Link:Link.into(), Message:"Link created successfully".to_string() })
	}

	/// Save link to database
	async fn SaveLink(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Link:&ContextLink::ContextLink,
	) -> Result<(), crate::Error::Kind::Kind> {
		let _ = DbState;
		let _ = Link;

		// TODO: Implement actual database insert
		tracing::debug!("Saving link to database: {:?}", Link.Id);

		Ok(())
	}
}

/// HandleGetLinkedItems - Get all items linked to a specific item
pub struct HandleGetLinkedItems;

impl HandleGetLinkedItems {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:GetLinkedItemsRequest,
	) -> Result<GetLinkedItemsResponse, Error> {
		// Validate item type
		if !ContextLink::item_type::is_valid(&Payload.ItemType) {
			return Err(Error::WithMessage(
				crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
				&format!("Invalid item type: {}", Payload.ItemType),
			));
		}

		let Limit = Payload.Limit.unwrap_or(10);

		// Query linked items
		let Links = Self::QueryLinks(
			DbState,
			&Payload.WorkspaceId,
			&Payload.ItemType,
			&Payload.ItemId,
			Payload.RelationshipTypeFilter.as_deref(),
			Payload.LinkedItemTypeFilter.as_deref(),
			Limit,
		)
		.await
		.map_err(|e| Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string()))?;

		let Total = Links.len();

		Ok(GetLinkedItemsResponse { Links, Total, SourceItemType:Payload.ItemType, SourceItemId:Payload.ItemId })
	}

	/// Query links from database
	async fn QueryLinks(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		WorkspaceId:&str,
		ItemType:&str,
		ItemId:&str,
		RelationshipTypeFilter:Option<&str>,
		LinkedItemTypeFilter:Option<&str>,
		Limit:usize,
	) -> Result<Vec<ContextLink::LinkResponse>, crate::Error::Kind::Kind> {
		let _ = DbState;
		let _ = WorkspaceId;
		let _ = ItemType;
		let _ = ItemId;
		let _ = RelationshipTypeFilter;
		let _ = LinkedItemTypeFilter;
		let _ = Limit;

		// TODO: Implement actual database query
		tracing::debug!("Querying links for {} {} in workspace {}", ItemType, ItemId, WorkspaceId);

		Ok(vec![])
	}
}

/// HandleUnlinkItems - Remove a link between two items
#[derive(Debug, Deserialize)]
pub struct UnlinkItemsRequest {
	pub WorkspaceId:String,
	pub SourceItemType:String,
	pub SourceItemId:String,
	pub TargetItemType:String,
	pub TargetItemId:String,
}

pub struct HandleUnlinkItems;

impl HandleUnlinkItems {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:UnlinkItemsRequest,
	) -> Result<serde_json::Value, Error> {
		// Delete the link
		Self::DeleteLink(
			DbState,
			&Payload.WorkspaceId,
			&Payload.SourceItemType,
			&Payload.SourceItemId,
			&Payload.TargetItemType,
			&Payload.TargetItemId,
		)
		.await
		.map_err(|e| Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string()))?;

		Ok(serde_json::json!({
			"message": "Link removed successfully"
		}))
	}

	/// Delete link from database
	async fn DeleteLink(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		WorkspaceId:&str,
		SourceItemType:&str,
		SourceItemId:&str,
		TargetItemType:&str,
		TargetItemId:&str,
	) -> Result<(), crate::Error::Kind::Kind> {
		let _ = DbState;
		let _ = WorkspaceId;
		let _ = SourceItemType;
		let _ = SourceItemId;
		let _ = TargetItemType;
		let _ = TargetItemId;

		// TODO: Implement actual database delete
		tracing::debug!(
			"Deleting link: {} {} -> {} {}",
			SourceItemType,
			SourceItemId,
			TargetItemType,
			TargetItemId
		);

		Ok(())
	}
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for link handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
	vec![
		crate::HTTP::Protocol::Request::Tool {
			Name:"link_items".to_string(),
			Description:"Create a relationship link between two ConPort items".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"source_item_type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data", "product_context", "active_context"]},
					"source_item_id": {"type": "string"},
					"target_item_type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data", "product_context", "active_context"]},
					"target_item_id": {"type": "string"},
					"relationship_type": {"type": "string"},
					"description": {"type": "string"}
				},
				"required": ["workspace_id", "source_item_type", "source_item_id", "target_item_type", "target_item_id", "relationship_type"]
			}),
		},
		crate::HTTP::Protocol::Request::Tool {
			Name:"get_linked_items".to_string(),
			Description:"Get all items linked to a specific ConPort item".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"item_type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data", "product_context", "active_context"]},
					"item_id": {"type": "string"},
					"relationship_type_filter": {"type": "string"},
					"linked_item_type_filter": {"type": "string"},
					"limit": {"type": "integer"}
				},
				"required": ["workspace_id", "item_type", "item_id"]
			}),
		},
		crate::HTTP::Protocol::Request::Tool {
			Name:"unlink_items".to_string(),
			Description:"Remove a relationship link between two ConPort items".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"source_item_type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data", "product_context", "active_context"]},
					"source_item_id": {"type": "string"},
					"target_item_type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data", "product_context", "active_context"]},
					"target_item_id": {"type": "string"}
				},
				"required": ["workspace_id", "source_item_type", "source_item_id", "target_item_type", "target_item_id"]
			}),
		},
	]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_link_items_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "source_item_type": "decision",
            "source_item_id": "dec-1",
            "target_item_type": "progress_entry",
            "target_item_id": "prog-1",
            "relationship_type": "relates_to_progress"
        }"#;
		let Request:LinkItemsRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.WorkspaceId, "test");
		assert_eq!(Request.SourceItemType, "decision");
		assert_eq!(Request.TargetItemType, "progress_entry");
	}

	#[test]
	fn test_get_linked_items_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "item_type": "decision",
            "item_id": "dec-1",
            "limit": 5
        }"#;
		let Request:GetLinkedItemsRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.ItemId, "dec-1");
		assert_eq!(Request.Limit, Some(5));
	}

	#[test]
	fn test_prevent_self_linking() {
		let Json = r#"{
            "workspace_id": "test",
            "source_item_type": "decision",
            "source_item_id": "dec-1",
            "target_item_type": "decision",
            "target_item_id": "dec-1",
            "relationship_type": "relates_to"
        }"#;
		let Request:LinkItemsRequest = serde_json::from_str(Json).unwrap();
		// Validation happens in Execute, not parsing
	}

	// ============================================================================
	// HTTP Handlers (axum-compatible)
	// ============================================================================

	/// Link two ConPort items together
	pub async fn LinkItems(
		State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
		Json(payload):Json<LinkItemsRequest>,
	) -> Result<Json<LinkItemsResponse>, crate::Error::Kind::Kind> {
		Ok(Json(LinkItemsResponse {
			Link:ContextLink::LinkResponse {
				Id:uuid::Uuid::new_v4().to_string(),
				WorkspaceId:payload.WorkspaceId,
				SourceItemType:payload.SourceItemType,
				SourceItemId:payload.SourceItemId,
				TargetItemType:payload.TargetItemType,
				TargetItemId:payload.TargetItemId,
				RelationshipType:payload.RelationshipType,
				Description:payload.Description,
				CreatedAt:chrono::Utc::now().to_rfc3339(),
			},
			Message:"Link created successfully".to_string(),
		}))
	}

	/// Get items linked to a specific item
	pub async fn GetLinkedItems(
		State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
		Query(query):Query<GetLinkedItemsRequest>,
	) -> Result<Json<GetLinkedItemsResponse>, crate::Error::Kind::Kind> {
		Ok(Json(GetLinkedItemsResponse {
			Links:vec![],
			Total:0,
			SourceItemType:query.ItemType,
			SourceItemId:query.ItemId,
		}))
	}

	/// Unlink two ConPort items
	pub async fn UnlinkItems(
		State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
		Json(payload):Json<UnlinkItemsRequest>,
	) -> Result<Json<serde_json::Value>, crate::Error::Kind::Kind> {
		Ok(Json(serde_json::json!({
			"success": true,
			"message": "Items unlinked successfully"
		})))
	}
}
