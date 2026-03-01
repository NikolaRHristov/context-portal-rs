// Schema definition module for the ConPort MCP server
// Provides JSON schema definitions and validation for all ConPort types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema version
pub const SCHEMA_VERSION:&str = "1.0.0";

/// Schema for context entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for ContextSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the context".to_string(),
				},
				SchemaField {
					Name:"Name".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Human-readable name of the context".to_string(),
				},
				SchemaField {
					Name:"Content".to_string(),
					Type:"json".to_string(),
					Required:true,
					Description:"JSON content of the context".to_string(),
				},
				SchemaField {
					Name:"WorkspacePath".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Path to the workspace".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
				SchemaField {
					Name:"UpdatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of last update".to_string(),
				},
			],
		}
	}
}

/// Schema field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
	pub Name:String,
	pub Type:String,
	pub Required:bool,
	pub Description:String,
}

/// Schema for decision entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for DecisionSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the decision".to_string(),
				},
				SchemaField {
					Name:"WorkspaceId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Workspace identifier".to_string(),
				},
				SchemaField {
					Name:"Summary".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Brief summary of the decision".to_string(),
				},
				SchemaField {
					Name:"Rationale".to_string(),
					Type:"string".to_string(),
					Required:false,
					Description:"Reasoning behind the decision".to_string(),
				},
				SchemaField {
					Name:"ImplementationDetails".to_string(),
					Type:"string".to_string(),
					Required:false,
					Description:"Implementation details".to_string(),
				},
				SchemaField {
					Name:"Tags".to_string(),
					Type:"array".to_string(),
					Required:false,
					Description:"Tags for categorization".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
			],
		}
	}
}

/// Schema for progress entry entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for ProgressSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the progress entry".to_string(),
				},
				SchemaField {
					Name:"WorkspaceId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Workspace identifier".to_string(),
				},
				SchemaField {
					Name:"Status".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Status (TODO, IN_PROGRESS, DONE)".to_string(),
				},
				SchemaField {
					Name:"Description".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Description of the progress".to_string(),
				},
				SchemaField {
					Name:"ParentId".to_string(),
					Type:"string".to_string(),
					Required:false,
					Description:"Parent task ID for subtasks".to_string(),
				},
				SchemaField {
					Name:"Timestamp".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
				SchemaField {
					Name:"UpdatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of last update".to_string(),
				},
			],
		}
	}
}

/// Schema for system pattern entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPatternSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for SystemPatternSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the system pattern".to_string(),
				},
				SchemaField {
					Name:"WorkspaceId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Workspace identifier".to_string(),
				},
				SchemaField {
					Name:"Name".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Name of the pattern".to_string(),
				},
				SchemaField {
					Name:"Description".to_string(),
					Type:"string".to_string(),
					Required:false,
					Description:"Description of the pattern".to_string(),
				},
				SchemaField {
					Name:"Tags".to_string(),
					Type:"array".to_string(),
					Required:false,
					Description:"Tags for categorization".to_string(),
				},
				SchemaField {
					Name:"Timestamp".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
			],
		}
	}
}

/// Schema for custom data entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDataSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for CustomDataSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the custom data".to_string(),
				},
				SchemaField {
					Name:"WorkspaceId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Workspace identifier".to_string(),
				},
				SchemaField {
					Name:"Category".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Category for the custom data".to_string(),
				},
				SchemaField {
					Name:"Key".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Key for the custom data".to_string(),
				},
				SchemaField {
					Name:"Value".to_string(),
					Type:"json".to_string(),
					Required:true,
					Description:"Value (any JSON type)".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
				SchemaField {
					Name:"UpdatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of last update".to_string(),
				},
			],
		}
	}
}

/// Schema for context links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLinkSchema {
	pub Version:String,
	pub Fields:Vec<SchemaField>,
}

impl Default for ContextLinkSchema {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Fields:vec![
				SchemaField {
					Name:"Id".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Unique identifier for the link".to_string(),
				},
				SchemaField {
					Name:"WorkspaceId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Workspace identifier".to_string(),
				},
				SchemaField {
					Name:"SourceItemType".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Type of source item".to_string(),
				},
				SchemaField {
					Name:"SourceItemId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ID of source item".to_string(),
				},
				SchemaField {
					Name:"TargetItemType".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Type of target item".to_string(),
				},
				SchemaField {
					Name:"TargetItemId".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ID of target item".to_string(),
				},
				SchemaField {
					Name:"RelationshipType".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"Type of relationship".to_string(),
				},
				SchemaField {
					Name:"Description".to_string(),
					Type:"string".to_string(),
					Required:false,
					Description:"Optional description".to_string(),
				},
				SchemaField {
					Name:"CreatedAt".to_string(),
					Type:"string".to_string(),
					Required:true,
					Description:"ISO 8601 timestamp of creation".to_string(),
				},
			],
		}
	}
}

/// All schemas combined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllSchemas {
	pub Version:String,
	pub Context:ContextSchema,
	pub Decision:DecisionSchema,
	pub Progress:ProgressSchema,
	pub SystemPattern:SystemPatternSchema,
	pub CustomData:CustomDataSchema,
	pub ContextLink:ContextLinkSchema,
}

impl Default for AllSchemas {
	fn default() -> Self {
		Self {
			Version:SCHEMA_VERSION.to_string(),
			Context:ContextSchema::default(),
			Decision:DecisionSchema::default(),
			Progress:ProgressSchema::default(),
			SystemPattern:SystemPatternSchema::default(),
			CustomData:CustomDataSchema::default(),
			ContextLink:ContextLinkSchema::default(),
		}
	}
}

// ============================================================================
// GetSchema functions - Return JSON schema definitions
// ============================================================================

/// Get the context JSON schema
pub fn GetContextSchema() -> Value { serde_json::to_value(ContextSchema::default()).unwrap_or_default() }

/// Get the decision JSON schema
pub fn GetDecisionSchema() -> Value { serde_json::to_value(DecisionSchema::default()).unwrap_or_default() }

/// Get the progress JSON schema
pub fn GetProgressSchema() -> Value { serde_json::to_value(ProgressSchema::default()).unwrap_or_default() }

/// Get the system pattern JSON schema
pub fn GetSystemPatternSchema() -> Value { serde_json::to_value(SystemPatternSchema::default()).unwrap_or_default() }

/// Get the custom data JSON schema
pub fn GetCustomDataSchema() -> Value { serde_json::to_value(CustomDataSchema::default()).unwrap_or_default() }

/// Get the context link JSON schema
pub fn GetContextLinkSchema() -> Value { serde_json::to_value(ContextLinkSchema::default()).unwrap_or_default() }

/// Get all schemas combined
pub fn GetAllSchemas() -> Value { serde_json::to_value(AllSchemas::default()).unwrap_or_default() }

// ============================================================================
// Validation functions
// ============================================================================

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
	pub Field:String,
	pub Message:String,
}

/// Validate a context against its schema
pub fn ValidateContext(ctx:&crate::Type::Context::Context) -> Result<(), Vec<ValidationError>> {
	let mut errors = Vec::new();

	if ctx.Id.is_empty() {
		errors.push(ValidationError { Field:"Id".to_string(), Message:"Id cannot be empty".to_string() });
	}

	if ctx.Name.is_empty() {
		errors.push(ValidationError { Field:"Name".to_string(), Message:"Name cannot be empty".to_string() });
	}

	if ctx.WorkspacePath.is_empty() {
		errors.push(ValidationError {
			Field:"WorkspacePath".to_string(),
			Message:"WorkspacePath cannot be empty".to_string(),
		});
	}

	if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Validate a JSON value against a field type
pub fn ValidateFieldType(value:&Value, expected_type:&str) -> bool {
	match expected_type {
		"string" => value.is_string(),
		"number" => value.is_number(),
		"boolean" => value.is_boolean(),
		"array" => value.is_array(),
		"object" => value.is_object(),
		"json" => true, // Any JSON value is valid
		"null" => value.is_null(),
		_ => false,
	}
}

/// Check if a value is valid for a given field
pub fn ValidateField(value:&Value, field:&SchemaField) -> Result<(), ValidationError> {
	if field.Required && value.is_null() {
		return Err(ValidationError { Field:field.Name.clone(), Message:format!("Field {} is required", field.Name) });
	}

	if !value.is_null() && !ValidateFieldType(value, &field.Type) {
		return Err(ValidationError {
			Field:field.Name.clone(),
			Message:format!("Field {} has invalid type. Expected {}", field.Name, field.Type),
		});
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_schema_versions() {
		let schema = ContextSchema::default();
		assert_eq!(schema.Version, SCHEMA_VERSION);
	}

	#[test]
	fn test_get_context_schema() {
		let schema = GetContextSchema();
		assert!(schema.get("Version").is_some());
	}

	#[test]
	fn test_validate_context_valid() {
		let ctx = crate::Type::Context::Context::new_string(
			"test-id".to_string(),
			"Test".to_string(),
			"content".to_string(),
			"/workspace".to_string(),
		);
		assert!(ValidateContext(&ctx).is_ok());
	}

	#[test]
	fn test_validate_context_invalid() {
		let ctx = crate::Type::Context::Context {
			Id:"".to_string(),
			Name:"".to_string(),
			Content:serde_json::Value::Null,
			WorkspacePath:"".to_string(),
			CreatedAt:"".to_string(),
			UpdatedAt:"".to_string(),
		};
		let result = ValidateContext(&ctx);
		assert!(result.is_err());
		assert_eq!(result.unwrap_err().len(), 3);
	}

	#[test]
	fn test_validate_field_type() {
		assert!(ValidateFieldType(&serde_json::json!("test"), "string"));
		assert!(ValidateFieldType(&serde_json::json!(123), "number"));
		assert!(ValidateFieldType(&serde_json::json!(true), "boolean"));
		assert!(ValidateFieldType(&serde_json::json!([]), "array"));
		assert!(ValidateFieldType(&serde_json::json!({}), "object"));
		assert!(ValidateFieldType(&serde_json::json!(null), "null"));
		assert!(ValidateFieldType(&serde_json::json!({"a": 1}), "json"));
	}

	#[test]
	fn test_all_schemas() {
		let all = AllSchemas::default();
		assert_eq!(all.Version, SCHEMA_VERSION);
	}
}
