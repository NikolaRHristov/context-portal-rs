// Context type for the ConPort MCP server
use serde::{Deserialize, Serialize};

/// Update mode for context operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum UpdateMode {
	/// Full update - replaces entire content
	FullUpdate,
	/// Patch update - merges with existing content
	PatchUpdate,
}

impl Default for UpdateMode {
	fn default() -> Self { UpdateMode::FullUpdate }
}

/// Context entity with JSON content support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
	pub Id:String,
	pub Name:String,
	#[serde(default)]
	pub Content:serde_json::Value,
	pub WorkspacePath:String,
	pub CreatedAt:String,
	pub UpdatedAt:String,
}

impl Context {
	/// Create a new context with string content (backwards compatible)
	pub fn new_string(Id:String, Name:String, Content:String, WorkspacePath:String) -> Self {
		Self::new(Id, Name, serde_json::Value::String(Content), WorkspacePath)
	}

	/// Create a new context with JSON content
	pub fn new(Id:String, Name:String, Content:serde_json::Value, WorkspacePath:String) -> Self {
		let timestamp = chrono::Utc::now().to_rfc3339();
		Self {
			Id,
			Name,
			Content,
			WorkspacePath,
			CreatedAt:timestamp.clone(),
			UpdatedAt:timestamp,
		}
	}

	/// Create a new context from a JSON object
	pub fn from_json(Id:String, Name:String, Content:serde_json::Value, WorkspacePath:String) -> Self {
		Self::new(Id, Name, Content, WorkspacePath)
	}

	/// Get content as string (for backwards compatibility)
	pub fn content_as_string(&self) -> String {
		match &self.Content {
			serde_json::Value::String(s) => s.clone(),
			other => other.to_string(),
		}
	}

	/// Update content with specified mode
	pub fn update_content(&mut self, new_content:serde_json::Value, mode:UpdateMode) {
		match mode {
			UpdateMode::FullUpdate => {
				self.Content = new_content;
			},
			UpdateMode::PatchUpdate => {
				// Merge new content into existing
				self.Content = Self::merge_json(self.Content.clone(), new_content);
			},
		}
		self.UpdatedAt = chrono::Utc::now().to_rfc3339();
	}

	/// Merge two JSON values (patch update)
	fn merge_json(base:serde_json::Value, patch:serde_json::Value) -> serde_json::Value {
		match (base, patch) {
			(serde_json::Value::Object(mut base_map), serde_json::Value::Object(patch_map)) => {
				for (key, value) in patch_map {
					base_map.insert(
						key,
						Self::merge_json(base_map.remove(&key).unwrap_or(serde_json::Value::Null), value),
					);
				}
				serde_json::Value::Object(base_map)
			},
			_ => patch,
		}
	}
}

/// Context arguments for creating/updating
#[derive(Debug, Deserialize)]
pub struct CreateContextArgs {
	pub workspace_id:String,
	pub name:String,
	#[serde(default)]
	pub content:serde_json::Value,
}

/// Arguments for updating context
#[derive(Debug, Deserialize)]
pub struct UpdateContextArgs {
	pub workspace_id:String,
	pub context_id:String,
	pub name:Option<String>,
	#[serde(default)]
	pub content:Option<serde_json::Value>,
	#[serde(default)]
	pub update_mode:Option<UpdateMode>,
}

/// Arguments for getting context
#[derive(Debug, Deserialize)]
pub struct GetContextArgs {
	pub workspace_id:String,
	pub context_id:Option<String>,
}

/// Context response
#[derive(Debug, Serialize)]
pub struct ContextResponse {
	pub Context:Option<Context>,
	pub Error:Option<String>,
}

/// List of contexts response
#[derive(Debug, Serialize)]
pub struct ContextListResponse {
	pub Contexts:Vec<Context>,
	pub Total:usize,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_context_creation_with_string() {
		let ctx = Context::new_string(
			"test-id".to_string(),
			"Test Context".to_string(),
			"Hello World".to_string(),
			"/workspace".to_string(),
		);
		assert_eq!(ctx.content_as_string(), "Hello World");
	}

	#[test]
	fn test_context_creation_with_json() {
		let json_content = serde_json::json!({
			"title": "Test",
			"items": ["a", "b", "c"]
		});
		let ctx = Context::new(
			"test-id".to_string(),
			"Test Context".to_string(),
			json_content.clone(),
			"/workspace".to_string(),
		);
		assert_eq!(ctx.Content, json_content);
	}

	#[test]
	fn test_full_update_mode() {
		let mut ctx = Context::new_string(
			"test-id".to_string(),
			"Test".to_string(),
			"original".to_string(),
			"/workspace".to_string(),
		);

		ctx.update_content(serde_json::json!("updated"), UpdateMode::FullUpdate);

		assert_eq!(ctx.content_as_string(), "updated");
	}

	#[test]
	fn test_patch_update_mode() {
		let mut ctx = Context::new(
			"test-id".to_string(),
			"Test".to_string(),
			serde_json::json!({"a": 1, "b": 2}),
			"/workspace".to_string(),
		);

		ctx.update_content(serde_json::json!({"b": 3, "c": 4}), UpdateMode::PatchUpdate);

		let expected = serde_json::json!({"a": 1, "b": 3, "c": 4});
		assert_eq!(ctx.Content, expected);
	}

	#[test]
	fn test_update_mode_default() {
		let mode:UpdateMode = serde_json::from_str("null").unwrap_or_default();
		assert_eq!(mode, UpdateMode::FullUpdate);
	}

	#[test]
	fn test_context_serialization() {
		let ctx = Context::new_string(
			"test-id".to_string(),
			"Test Context".to_string(),
			"Hello World".to_string(),
			"/workspace".to_string(),
		);

		let json = serde_json::to_string(&ctx).unwrap();
		let parsed:Context = serde_json::from_str(&json).unwrap();

		assert_eq!(parsed.Id, "test-id");
		assert_eq!(parsed.Name, "Test Context");
	}
}
