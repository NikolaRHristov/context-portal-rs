// History type for the ConPort MCP server
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Context item type for history tracking
pub mod item_type {
	pub const PRODUCT_CONTEXT:&str = "product_context";
	pub const ACTIVE_CONTEXT:&str = "active_context";

	pub fn all() -> Vec<&'static str> { vec![PRODUCT_CONTEXT, ACTIVE_CONTEXT] }

	pub fn is_valid(s:&str) -> bool { all().contains(&s) }
}

/// Main ContextHistory structure for tracking context changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextHistory {
	pub Id:i64,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Version:i64,
	pub Content:Value,
	pub ChangeSource:Option<String>,
}

impl ContextHistory {
	pub fn New(WorkspaceId:String, Version:i64, Content:Value, ChangeSource:Option<String>) -> Self {
		Self {
			Id:0, // Will be set by database
			WorkspaceId,
			Timestamp:chrono::Utc::now().to_rfc3339(),
			Version,
			Content,
			ChangeSource,
		}
	}

	pub fn with_id(Id:i64, WorkspaceId:String, Version:i64, Content:Value, ChangeSource:Option<String>) -> Self {
		Self {
			Id,
			WorkspaceId,
			Timestamp:chrono::Utc::now().to_rfc3339(),
			Version,
			Content,
			ChangeSource,
		}
	}

	/// Serialize content to JSON string for database storage
	pub fn content_to_json(&self) -> String {
		serde_json::to_string(&self.Content).unwrap_or_else(|_| "{}".to_string())
	}

	/// Deserialize content from JSON string from database
	pub fn content_from_json(json:&str) -> Value { serde_json::from_str(json).unwrap_or(Value::Null) }
}

/// HTTP response type for ContextHistory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
	pub id:i64,
	pub workspace_id:String,
	pub timestamp:String,
	pub version:i64,
	pub content:Value,
	pub change_source:Option<String>,
}

impl From<ContextHistory> for HistoryResponse {
	fn from(h:ContextHistory) -> Self {
		Self {
			id:h.Id,
			workspace_id:h.WorkspaceId,
			timestamp:h.Timestamp,
			version:h.Version,
			content:h.Content,
			change_source:h.ChangeSource,
		}
	}
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for retrieving history of a context item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetItemHistoryArgs {
	pub workspace_id:String,
	pub item_type:String, // "product_context" or "active_context"
	#[serde(default)]
	pub limit:Option<i64>,
	#[serde(default)]
	pub before_timestamp:Option<String>,
	#[serde(default)]
	pub after_timestamp:Option<String>,
	#[serde(default)]
	pub version:Option<i64>,
}

impl GetItemHistoryArgs {
	pub fn limit_value(&self) -> i64 { self.limit.unwrap_or(50) }

	pub fn validate(&self) -> Result<(), String> {
		// Validate item_type
		if !item_type::is_valid(&self.item_type) {
			return Err("item_type must be 'product_context' or 'active_context'".to_string());
		}

		// Validate limit
		if let Some(limit) = self.limit {
			if limit < 1 {
				return Err("limit must be greater than or equal to 1".to_string());
			}
		}

		// Validate version
		if let Some(version) = self.version {
			if version < 1 {
				return Err("version must be greater than or equal to 1".to_string());
			}
		}

		Ok(())
	}
}

/// Arguments for getting recent activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecentActivitySummaryArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub hours_ago:Option<i64>,
	#[serde(default)]
	pub since_timestamp:Option<String>,
	#[serde(default)]
	pub limit_per_type:Option<i64>,
}

impl GetRecentActivitySummaryArgs {
	pub fn limit_per_type_value(&self) -> i64 { self.limit_per_type.unwrap_or(5) }

	pub fn validate(&self) -> Result<(), String> {
		// Cannot use both time filters simultaneously
		if self.hours_ago.is_some() && self.since_timestamp.is_some() {
			return Err("Provide either 'hours_ago' or 'since_timestamp', not both".to_string());
		}

		// Validate hours_ago
		if let Some(hours) = self.hours_ago {
			if hours < 1 {
				return Err("hours_ago must be greater than or equal to 1".to_string());
			}
		}

		// Validate limit_per_type
		if let Some(limit) = self.limit_per_type {
			if limit < 1 {
				return Err("limit_per_type must be greater than or equal to 1".to_string());
			}
		}

		Ok(())
	}
}

/// Recent activity summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivitySummary {
	pub product_context:Vec<HistoryResponse>,
	pub active_context:Vec<HistoryResponse>,
	pub decisions:Vec<Value>, // Simplified for now
	pub progress_entries:Vec<Value>,
	pub system_patterns:Vec<Value>,
	pub custom_data:Vec<Value>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_context_history_creation() {
		let history = ContextHistory::New(
			"workspace1".to_string(),
			1,
			serde_json::json!({"key": "value"}),
			Some("initialization".to_string()),
		);

		assert!(history.Id > 0);
		assert_eq!(history.Version, 1);
		assert_eq!(history.Content, serde_json::json!({"key": "value"}));
		assert_eq!(history.ChangeSource, Some("initialization".to_string()));
	}

	#[test]
	fn test_context_history_content_json() {
		let history = ContextHistory::New("workspace1".to_string(), 1, serde_json::json!({"setting": "dark"}), None);

		let json = history.content_to_json();
		assert!(!json.is_empty());

		let content = ContextHistory::content_from_json(&json);
		assert_eq!(content, serde_json::json!({"setting": "dark"}));
	}

	#[test]
	fn test_item_type_validation() {
		assert!(item_type::is_valid("product_context"));
		assert!(item_type::is_valid("active_context"));
		assert!(!item_type::is_valid("invalid"));
	}

	#[test]
	fn test_get_item_history_args_validation() {
		let valid_args = GetItemHistoryArgs {
			workspace_id:"ws1".to_string(),
			item_type:"product_context".to_string(),
			limit:Some(10),
			before_timestamp:None,
			after_timestamp:None,
			version:None,
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = GetItemHistoryArgs {
			workspace_id:"ws1".to_string(),
			item_type:"invalid_type".to_string(),
			limit:None,
			before_timestamp:None,
			after_timestamp:None,
			version:None,
		};
		assert!(invalid_args.validate().is_err());

		let invalid_limit = GetItemHistoryArgs {
			workspace_id:"ws1".to_string(),
			item_type:"active_context".to_string(),
			limit:Some(0),
			before_timestamp:None,
			after_timestamp:None,
			version:None,
		};
		assert!(invalid_limit.validate().is_err());
	}

	#[test]
	fn test_get_recent_activity_summary_args_validation() {
		let valid_args = GetRecentActivitySummaryArgs {
			workspace_id:"ws1".to_string(),
			hours_ago:Some(24),
			since_timestamp:None,
			limit_per_type:Some(5),
		};
		assert!(valid_args.validate().is_ok());

		// Both time filters - should fail
		let invalid_args = GetRecentActivitySummaryArgs {
			workspace_id:"ws1".to_string(),
			hours_ago:Some(24),
			since_timestamp:Some("2024-01-01T00:00:00Z".to_string()),
			limit_per_type:None,
		};
		assert!(invalid_args.validate().is_err());

		// Invalid hours_ago
		let invalid_hours = GetRecentActivitySummaryArgs {
			workspace_id:"ws1".to_string(),
			hours_ago:Some(0),
			since_timestamp:None,
			limit_per_type:None,
		};
		assert!(invalid_hours.validate().is_err());
	}
}
