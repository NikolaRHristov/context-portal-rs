// ContextLink type for the ConPort MCP server
use serde::{Deserialize, Serialize};

/// Valid item types that can be linked
pub mod item_type {
	pub const DECISION:&str = "decision";
	pub const PROGRESS_ENTRY:&str = "progress_entry";
	pub const SYSTEM_PATTERN:&str = "system_pattern";
	pub const CUSTOM_DATA:&str = "custom_data";
	pub const PRODUCT_CONTEXT:&str = "product_context";
	pub const ACTIVE_CONTEXT:&str = "active_context";

	pub fn all() -> Vec<&'static str> {
		vec![
			DECISION,
			PROGRESS_ENTRY,
			SYSTEM_PATTERN,
			CUSTOM_DATA,
			PRODUCT_CONTEXT,
			ACTIVE_CONTEXT,
		]
	}

	pub fn is_valid(s:&str) -> bool { all().contains(&s) }
}

/// Main ContextLink structure for linking ConPort items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLink {
	pub Id:String,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub SourceItemType:String,
	pub SourceItemId:String,
	pub TargetItemType:String,
	pub TargetItemId:String,
	pub RelationshipType:String,
	pub Description:Option<String>,
}

impl ContextLink {
	pub fn New(
		WorkspaceId:String,
		SourceItemType:String,
		SourceItemId:String,
		TargetItemType:String,
		TargetItemId:String,
		RelationshipType:String,
		Description:Option<String>,
	) -> Self {
		Self {
			Id:uuid::Uuid::new_v4().to_string(),
			WorkspaceId,
			Timestamp:chrono::Utc::now().to_rfc3339(),
			SourceItemType,
			SourceItemId,
			TargetItemType,
			TargetItemId,
			RelationshipType,
			Description,
		}
	}

	pub fn with_id(
		Id:String,
		WorkspaceId:String,
		SourceItemType:String,
		SourceItemId:String,
		TargetItemType:String,
		TargetItemId:String,
		RelationshipType:String,
		Description:Option<String>,
	) -> Self {
		Self {
			Id,
			WorkspaceId,
			Timestamp:chrono::Utc::now().to_rfc3339(),
			SourceItemType,
			SourceItemId,
			TargetItemType,
			TargetItemId,
			RelationshipType,
			Description,
		}
	}
}

/// HTTP response type for ContextLink
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
	pub id:String,
	pub workspace_id:String,
	pub timestamp:String,
	pub source_item_type:String,
	pub source_item_id:String,
	pub target_item_type:String,
	pub target_item_id:String,
	pub relationship_type:String,
	pub description:Option<String>,
}

impl From<ContextLink> for LinkResponse {
	fn from(l:ContextLink) -> Self {
		Self {
			id:l.Id,
			workspace_id:l.WorkspaceId,
			timestamp:l.Timestamp,
			source_item_type:l.SourceItemType,
			source_item_id:l.SourceItemId,
			target_item_type:l.TargetItemType,
			target_item_id:l.TargetItemId,
			relationship_type:l.RelationshipType,
			description:l.Description,
		}
	}
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for creating a link between two ConPort items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConportItemsArgs {
	pub workspace_id:String,
	pub source_item_type:String,
	pub source_item_id:String,
	pub target_item_type:String,
	pub target_item_id:String,
	pub relationship_type:String,
	#[serde(default)]
	pub description:Option<String>,
}

impl LinkConportItemsArgs {
	pub fn validate(&self) -> Result<(), String> {
		// Validate source item type
		if !item_type::is_valid(&self.source_item_type) {
			return Err(format!("Invalid source_item_type: {}", self.source_item_type));
		}

		// Validate target item type
		if !item_type::is_valid(&self.target_item_type) {
			return Err(format!("Invalid target_item_type: {}", self.target_item_type));
		}

		// Validate IDs are not empty
		if self.source_item_id.is_empty() {
			return Err("source_item_id is required".to_string());
		}
		if self.target_item_id.is_empty() {
			return Err("target_item_id is required".to_string());
		}

		// Validate relationship type
		if self.relationship_type.is_empty() {
			return Err("relationship_type is required".to_string());
		}

		Ok(())
	}
}

/// Arguments for retrieving links for a ConPort item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLinkedItemsArgs {
	pub workspace_id:String,
	pub item_type:String,
	pub item_id:String,
	#[serde(default)]
	pub relationship_type_filter:Option<String>,
	#[serde(default)]
	pub linked_item_type_filter:Option<String>,
	#[serde(default)]
	pub limit:Option<i64>,
}

impl GetLinkedItemsArgs {
	pub fn limit_value(&self) -> i64 { self.limit.unwrap_or(50) }

	pub fn validate(&self) -> Result<(), String> {
		// Validate item type
		if !item_type::is_valid(&self.item_type) {
			return Err(format!("Invalid item_type: {}", self.item_type));
		}

		// Validate item_id
		if self.item_id.is_empty() {
			return Err("item_id is required".to_string());
		}

		// Validate limit
		if let Some(limit) = self.limit {
			if limit < 1 {
				return Err("limit must be greater than or equal to 1".to_string());
			}
		}

		Ok(())
	}
}

/// Response type for link operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkOperationResponse {
	pub status:String,
	pub message:String,
	pub link:Option<LinkResponse>,
}

// ============================================================================
// Common Relationship Types
// ============================================================================

pub mod relationship {
	pub const RELATES_TO:&str = "relates_to";
	pub const IMPLEMENTS:&str = "implements";
	pub const BLOCKED_BY:&str = "blocked_by";
	pub const DEPENDS_ON:&str = "depends_on";
	pub const DERIVED_FROM:&str = "derived_from";
	pub const RELATES_TO_PROGRESS:&str = "relates_to_progress";
	pub const PARENT_OF:&str = "parent_of";
	pub const CHILD_OF:&str = "child_of";

	pub fn all() -> Vec<&'static str> {
		vec![
			RELATES_TO,
			IMPLEMENTS,
			BLOCKED_BY,
			DEPENDS_ON,
			DERIVED_FROM,
			RELATES_TO_PROGRESS,
			PARENT_OF,
			CHILD_OF,
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_context_link_creation() {
		let link = ContextLink::New(
			"workspace1".to_string(),
			"decision".to_string(),
			"123".to_string(),
			"progress_entry".to_string(),
			"456".to_string(),
			"implements".to_string(),
			Some("This progress implements decision 123".to_string()),
		);

		assert!(!link.Id.is_empty());
		assert_eq!(link.SourceItemType, "decision");
		assert_eq!(link.SourceItemId, "123");
		assert_eq!(link.TargetItemType, "progress_entry");
		assert_eq!(link.TargetItemId, "456");
		assert_eq!(link.RelationshipType, "implements");
	}

	#[test]
	fn test_item_type_validation() {
		assert!(item_type::is_valid("decision"));
		assert!(item_type::is_valid("progress_entry"));
		assert!(item_type::is_valid("system_pattern"));
		assert!(item_type::is_valid("custom_data"));
		assert!(!item_type::is_valid("invalid"));
	}

	#[test]
	fn test_link_conport_items_args_validation() {
		let valid_args = LinkConportItemsArgs {
			workspace_id:"ws1".to_string(),
			source_item_type:"decision".to_string(),
			source_item_id:"123".to_string(),
			target_item_type:"progress_entry".to_string(),
			target_item_id:"456".to_string(),
			relationship_type:"implements".to_string(),
			description:Some("Description".to_string()),
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = LinkConportItemsArgs {
			workspace_id:"ws1".to_string(),
			source_item_type:"invalid".to_string(),
			source_item_id:"123".to_string(),
			target_item_type:"progress_entry".to_string(),
			target_item_id:"456".to_string(),
			relationship_type:"relates_to".to_string(),
			description:None,
		};
		assert!(invalid_args.validate().is_err());

		let invalid_empty = LinkConportItemsArgs {
			workspace_id:"ws1".to_string(),
			source_item_type:"decision".to_string(),
			source_item_id:"".to_string(),
			target_item_type:"progress_entry".to_string(),
			target_item_id:"456".to_string(),
			relationship_type:"relates_to".to_string(),
			description:None,
		};
		assert!(invalid_empty.validate().is_err());
	}

	#[test]
	fn test_get_linked_items_args_validation() {
		let valid_args = GetLinkedItemsArgs {
			workspace_id:"ws1".to_string(),
			item_type:"decision".to_string(),
			item_id:"123".to_string(),
			relationship_type_filter:None,
			linked_item_type_filter:None,
			limit:Some(10),
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = GetLinkedItemsArgs {
			workspace_id:"ws1".to_string(),
			item_type:"invalid".to_string(),
			item_id:"123".to_string(),
			relationship_type_filter:None,
			linked_item_type_filter:None,
			limit:None,
		};
		assert!(invalid_args.validate().is_err());

		let invalid_limit = GetLinkedItemsArgs {
			workspace_id:"ws1".to_string(),
			item_type:"decision".to_string(),
			item_id:"123".to_string(),
			relationship_type_filter:None,
			linked_item_type_filter:None,
			limit:Some(0),
		};
		assert!(invalid_limit.validate().is_err());
	}
}
