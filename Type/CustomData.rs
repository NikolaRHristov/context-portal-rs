// CustomData type for the ConPort MCP server
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Main CustomData structure representing arbitrary key-value data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
	pub Id:String,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Category:String,
	pub Key:String,
	pub Value:Value,
	pub CreatedAt:String,
}

impl CustomData {
	pub fn New(WorkspaceId:String, Category:String, Key:String, Value:Value) -> Self {
		let now = chrono::Utc::now().to_rfc3339();
		Self {
			Id:uuid::Uuid::new_v4().to_string(),
			WorkspaceId,
			Timestamp:now.clone(),
			Category,
			Key,
			Value,
			CreatedAt:now,
		}
	}

	pub fn with_id(Id:String, WorkspaceId:String, Category:String, Key:String, Value:Value) -> Self {
		let now = chrono::Utc::now().to_rfc3339();
		Self { Id, WorkspaceId, Timestamp:now.clone(), Category, Key, Value, CreatedAt:now }
	}

	/// Serialize value to JSON string for database storage
	pub fn value_to_json(&self) -> String { serde_json::to_string(&self.Value).unwrap_or_default() }

	/// Deserialize value from JSON string from database
	pub fn value_from_json(json:&str) -> Value { serde_json::from_str(json).unwrap_or(Value::Null) }
}

/// HTTP response type for CustomData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDataResponse {
	pub id:String,
	pub workspace_id:String,
	pub timestamp:String,
	pub category:String,
	pub key:String,
	pub value:Value,
	pub created_at:String,
}

impl From<CustomData> for CustomDataResponse {
	fn from(c:CustomData) -> Self {
		Self {
			id:c.Id,
			workspace_id:c.WorkspaceId,
			timestamp:c.Timestamp,
			category:c.Category,
			key:c.Key,
			value:c.Value,
			created_at:c.CreatedAt,
		}
	}
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for logging/creating custom data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCustomDataArgs {
	pub workspace_id:String,
	pub category:String,
	pub key:String,
	pub value:Value,
}

impl LogCustomDataArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.category.is_empty() {
			return Err("category is required".to_string());
		}
		if self.key.is_empty() {
			return Err("key is required".to_string());
		}
		Ok(())
	}
}

/// Arguments for retrieving custom data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCustomDataArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub category:Option<String>,
	#[serde(default)]
	pub key:Option<String>,
}

impl GetCustomDataArgs {
	pub fn validate(&self) -> Result<(), String> {
		// Key requires category
		if self.key.is_some() && self.category.is_none() {
			return Err("key filter requires category to also be specified".to_string());
		}
		Ok(())
	}
}

/// Arguments for deleting custom data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCustomDataArgs {
	pub workspace_id:String,
	pub category:String,
	pub key:String,
}

impl DeleteCustomDataArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.category.is_empty() {
			return Err("category is required".to_string());
		}
		if self.key.is_empty() {
			return Err("key is required".to_string());
		}
		Ok(())
	}
}

/// Arguments for searching custom data using FTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCustomDataValueArgs {
	pub workspace_id:String,
	pub query_term:String,
	#[serde(default)]
	pub category_filter:Option<String>,
	#[serde(default)]
	pub limit:Option<i64>,
}

impl SearchCustomDataValueArgs {
	pub fn limit_value(&self) -> i64 { self.limit.unwrap_or(10) }

	pub fn validate(&self) -> Result<(), String> {
		if self.query_term.is_empty() {
			return Err("query_term is required".to_string());
		}

		if let Some(limit) = self.limit {
			if limit < 1 {
				return Err("limit must be greater than or equal to 1".to_string());
			}
		}

		Ok(())
	}
}

/// Response type for delete operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCustomDataResponse {
	pub status:String,
	pub message:String,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_custom_data_creation() {
		let data = CustomData::New(
			"workspace1".to_string(),
			"config".to_string(),
			"api_key".to_string(),
			Value::String("secret123".to_string()),
		);

		assert!(!data.Id.is_empty());
		assert_eq!(data.Category, "config");
		assert_eq!(data.Key, "api_key");
		assert_eq!(data.Value, Value::String("secret123".to_string()));
	}

	#[test]
	fn test_custom_data_value_json() {
		let data = CustomData::New(
			"workspace1".to_string(),
			"settings".to_string(),
			"preferences".to_string(),
			serde_json::json!({"theme": "dark", "notifications": true}),
		);

		let json = data.value_to_json();
		assert!(!json.is_empty());

		let value = CustomData::value_from_json(&json);
		assert_eq!(value, serde_json::json!({"theme": "dark", "notifications": true}));
	}

	#[test]
	fn test_log_custom_data_args_validation() {
		let valid_args = LogCustomDataArgs {
			workspace_id:"ws1".to_string(),
			category:"config".to_string(),
			key:"api_key".to_string(),
			value:Value::String("value".to_string()),
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = LogCustomDataArgs {
			workspace_id:"ws1".to_string(),
			category:"".to_string(),
			key:"key".to_string(),
			value:Value::Null,
		};
		assert!(invalid_args.validate().is_err());
	}

	#[test]
	fn test_get_custom_data_args_validation() {
		let valid_args = GetCustomDataArgs {
			workspace_id:"ws1".to_string(),
			category:Some("config".to_string()),
			key:Some("api_key".to_string()),
		};
		assert!(valid_args.validate().is_ok());

		// Key without category - should fail
		let invalid_args =
			GetCustomDataArgs { workspace_id:"ws1".to_string(), category:None, key:Some("api_key".to_string()) };
		assert!(invalid_args.validate().is_err());
	}

	#[test]
	fn test_search_custom_data_args_validation() {
		let valid_args = SearchCustomDataValueArgs {
			workspace_id:"ws1".to_string(),
			query_term:"test".to_string(),
			category_filter:None,
			limit:Some(10),
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = SearchCustomDataValueArgs {
			workspace_id:"ws1".to_string(),
			query_term:"".to_string(),
			category_filter:None,
			limit:None,
		};
		assert!(invalid_args.validate().is_err());
	}
}
