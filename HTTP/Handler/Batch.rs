// Batch HTTP handlers for the ConPort MCP server
// Provides handlers for batch operations on ConPort items

use std::sync::Arc;

use axum::{Json, extract::State};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{HTTP::Protocol::Response::Error, Persistence::Database::Operations};

/// Request for batch logging items
#[derive(Debug, Deserialize)]
pub struct BatchLogItemsRequest {
	pub WorkspaceId:String,
	pub Items:Vec<BatchItem>,
}

/// Single item in a batch request
#[derive(Debug, Deserialize)]
pub struct BatchItem {
	#[serde(rename = "Type")]
	pub TypeField:String,
	pub Data:serde_json::Value,
}

/// Response for batch operation
#[derive(Debug, Serialize)]
pub struct BatchLogItemsResponse {
	pub Success:bool,
	pub ItemsProcessed:usize,
	pub ItemsSucceeded:usize,
	pub ItemsFailed:usize,
	pub Errors:Vec<BatchError>,
}

/// Error for a single item in batch
#[derive(Debug, Serialize)]
pub struct BatchError {
	pub Index:usize,
	pub ItemType:String,
	pub Error:String,
}

/// HandleBatchLogItems - Log multiple items in a single request
pub struct HandleBatchLogItems;

impl HandleBatchLogItems {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		Payload:BatchLogItemsRequest,
	) -> Result<BatchLogItemsResponse, Error> {
		let TotalItems = Payload.Items.len();
		let mut ItemsSucceeded = 0;
		let mut ItemsFailed = 0;
		let mut Errors = Vec::new();

		// Process each item
		for (Index, Item) in Payload.Items.iter().enumerate() {
			match Self::ProcessItem(DbState, VectorStore, EmbeddingModel, &Payload.WorkspaceId, Item).await {
				Ok(_) => {
					ItemsSucceeded += 1;
				},
				Err(e) => {
					ItemsFailed += 1;
					Errors.push(BatchError { Index, ItemType:Item.TypeField.clone(), Error:e.to_string() });
				},
			}
		}

		Ok(BatchLogItemsResponse {
			Success:ItemsFailed == 0,
			ItemsProcessed:TotalItems,
			ItemsSucceeded,
			ItemsFailed,
			Errors,
		})
	}

	/// Process a single item based on its type
	async fn ProcessItem(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		WorkspaceId:&str,
		Item:&BatchItem,
	) -> Result<(), crate::Error::Kind::Kind> {
		match Item.TypeField.as_str() {
			"decision" => Self::ProcessDecision(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data).await,
			"progress_entry" => {
				Self::ProcessProgressEntry(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data).await
			},
			"system_pattern" => {
				Self::ProcessSystemPattern(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data).await
			},
			"custom_data" => {
				Self::ProcessCustomData(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data).await
			},
			_ => Err(crate::Error::Kind::Kind::InvalidItemType(Item.TypeField.clone())),
		}
	}

	/// Process a decision item
	async fn ProcessDecision(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		WorkspaceId:&str,
		Data:&serde_json::Value,
	) -> Result<(), crate::Error::Kind::Kind> {
		// Parse decision data
		let Summary = Data.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let Rationale = Data.get("rationale").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let ImplementationDetails = Data
			.get("implementation_details")
			.and_then(|v| v.as_str())
			.unwrap_or("")
			.to_string();
		let Id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		// Generate embedding for search
		let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Summary).await?;

		// Save to SQLite database
		let conn = DbState
			.Connection
			.lock()
			.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;
		let timestamp = crate::Persistence::Database::Connect::current_timestamp();
		let tags_json = Data.get("tags").and_then(|v| serde_json::to_string(v).ok()).unwrap_or_default();

		conn.execute(
			"INSERT OR REPLACE INTO Decisions (Id, WorkspaceId, Summary, Rationale, ImplementationDetails, Tags, \
			 CreatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			params![Id, WorkspaceId, Summary, Rationale, ImplementationDetails, tags_json, timestamp],
		)
		.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;

		// Add to vector store
		let Mutex = VectorStore.lock().await;
		Mutex.Add(WorkspaceId, &Id, &Embedding, "decision")?;

		tracing::debug!("Processed decision: {}", Id);
		Ok(())
	}

	/// Process a progress entry item
	async fn ProcessProgressEntry(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		WorkspaceId:&str,
		Data:&serde_json::Value,
	) -> Result<(), crate::Error::Kind::Kind> {
		let Description = Data.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let Status = Data.get("status").and_then(|v| v.as_str()).unwrap_or("TODO").to_string();
		let ParentId = Data.get("parent_id").and_then(|v| v.as_str()).map(|s| s.to_string());
		let Id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		// Generate embedding
		let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Description).await?;

		// Save to SQLite database
		let conn = DbState
			.Connection
			.lock()
			.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;
		let timestamp = crate::Persistence::Database::Connect::current_timestamp();

		conn.execute(
			"INSERT INTO ProgressEntries (WorkspaceId, Timestamp, Status, Description, ParentId, CreatedAt, \
			 UpdatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			params![WorkspaceId, timestamp, Status, Description, ParentId, timestamp, timestamp],
		)
		.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;

		let Mutex = VectorStore.lock().await;
		Mutex.Add(WorkspaceId, &Id, &Embedding, "progress_entry")?;

		tracing::debug!("Processed progress entry: {}", Id);
		Ok(())
	}

	/// Process a system pattern item
	async fn ProcessSystemPattern(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		WorkspaceId:&str,
		Data:&serde_json::Value,
	) -> Result<(), crate::Error::Kind::Kind> {
		let Name = Data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let Description = Data.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let Id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		let Content = format!("{}: {}", Name, Description);
		let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Content).await?;

		// Save to SQLite database
		let conn = DbState
			.Connection
			.lock()
			.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;
		let timestamp = crate::Persistence::Database::Connect::current_timestamp();
		let tags_json = Data.get("tags").and_then(|v| serde_json::to_string(v).ok()).unwrap_or_default();

		conn.execute(
			"INSERT OR REPLACE INTO SystemPatterns (WorkspaceId, Timestamp, Name, Description, Tags, CreatedAt) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			params![WorkspaceId, timestamp, Name, Description, tags_json, timestamp],
		)
		.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;

		let Mutex = VectorStore.lock().await;
		Mutex.Add(WorkspaceId, &Id, &Embedding, "system_pattern")?;

		tracing::debug!("Processed system pattern: {}", Id);
		Ok(())
	}

	/// Process a custom data item
	async fn ProcessCustomData(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		WorkspaceId:&str,
		Data:&serde_json::Value,
	) -> Result<(), crate::Error::Kind::Kind> {
		let Key = Data.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
		let Category = Data.get("category").and_then(|v| v.as_str()).unwrap_or("default").to_string();
		// Store value as JSON string
		let Value = Data.get("value").map(|v| v.to_string()).unwrap_or_default();
		let Id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		let Content = format!("[{}] {}: {}", Category, Key, Value);
		let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Content).await?;

		// Save to SQLite database
		let conn = DbState
			.Connection
			.lock()
			.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;
		let timestamp = crate::Persistence::Database::Connect::current_timestamp();

		conn.execute(
			"INSERT OR REPLACE INTO CustomData (WorkspaceId, Timestamp, Category, Key, Value, CreatedAt) VALUES (?1, \
			 ?2, ?3, ?4, ?5, ?6)",
			params![WorkspaceId, timestamp, Category, Key, Value, timestamp],
		)
		.map_err(|e| crate::Error::Kind::Kind::DatabaseError(e.to_string()))?;

		let Mutex = VectorStore.lock().await;
		Mutex.Add(WorkspaceId, &Id, &Embedding, "custom_data")?;

		tracing::debug!("Processed custom data: {}", Id);
		Ok(())
	}

	/// Generate embedding for text content
	async fn GenerateEmbedding(
		EmbeddingModel:&Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
		Content:&str,
	) -> Result<Vec<f32>, crate::Error::Kind::Kind> {
		let ModelGuard = EmbeddingModel.lock().await;

		if let Some(Model) = ModelGuard.as_ref() {
			Model.Generate(Content)
		} else {
			// Fallback to simple embedding
			Ok(crate::AI::Embedding::Generate::Generate::Execute(Content).await?)
		}
	}
}

// ============================================================================
// BatchLogItemsToDatabase - Direct database storage without vector store
// ============================================================================

/// Request for batch logging items directly to database
pub struct BatchLogItemsToDatabaseRequest {
	pub WorkspaceId:String,
	pub Items:Vec<BatchItem>,
}

/// Response for batch database operation
#[derive(Debug, Serialize)]
pub struct BatchLogItemsToDatabaseResponse {
	pub Success:bool,
	pub ItemsProcessed:usize,
	pub ItemsSucceeded:usize,
	pub ItemsFailed:usize,
	pub Errors:Vec<BatchError>,
	pub Results:Vec<BatchItemResult>,
}

/// Result for a single item
#[derive(Debug, Serialize)]
pub struct BatchItemResult {
	pub Index:usize,
	pub ItemType:String,
	pub Id:String,
	pub Stored:bool,
}

/// HandleBatchLogItemsToDatabase - Log items directly to database without
/// vector store
pub struct HandleBatchLogItemsToDatabase;

impl HandleBatchLogItemsToDatabase {
	/// Execute batch insert to database
	pub fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		Payload:BatchLogItemsToDatabaseRequest,
	) -> Result<BatchLogItemsToDatabaseResponse, Error> {
		let TotalItems = Payload.Items.len();
		let mut ItemsSucceeded = 0;
		let mut ItemsFailed = 0;
		let mut Errors = Vec::new();
		let mut Results = Vec::new();

		// Get database connection
		let conn = DbState.Connection.lock().map_err(|e| {
			Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string())
		})?;

		let timestamp = crate::Persistence::Database::Connect::current_timestamp();

		// Process each item
		for (Index, Item) in Payload.Items.iter().enumerate() {
			let result = Self::StoreItem(&conn, &Payload.WorkspaceId, &timestamp, Item);

			match result {
				Ok(id) => {
					ItemsSucceeded += 1;
					Results.push(BatchItemResult { Index, ItemType:Item.TypeField.clone(), Id:id, Stored:true });
				},
				Err(e) => {
					ItemsFailed += 1;
					Errors.push(BatchError { Index, ItemType:Item.TypeField.clone(), Error:e.to_string() });
					Results.push(BatchItemResult {
						Index,
						ItemType:Item.TypeField.clone(),
						Id:String::new(),
						Stored:false,
					});
				},
			}
		}

		Ok(BatchLogItemsToDatabaseResponse {
			Success:ItemsFailed == 0,
			ItemsProcessed:TotalItems,
			ItemsSucceeded,
			ItemsFailed,
			Errors,
			Results,
		})
	}

	/// Store a single item based on its type
	fn StoreItem(
		conn:&rusqlite::Mutex<rusqlite::Connection>,
		WorkspaceId:&str,
		timestamp:&str,
		Item:&BatchItem,
	) -> Result<String, String> {
		match Item.TypeField.as_str() {
			"decision" => Self::StoreDecision(conn, WorkspaceId, timestamp, &Item.Data),
			"progress_entry" => Self::StoreProgressEntry(conn, WorkspaceId, timestamp, &Item.Data),
			"system_pattern" => Self::StoreSystemPattern(conn, WorkspaceId, timestamp, &Item.Data),
			"custom_data" => Self::StoreCustomData(conn, WorkspaceId, timestamp, &Item.Data),
			_ => Err(format!("Invalid item type: {}", Item.TypeField)),
		}
	}

	/// Store a decision item
	fn StoreDecision(
		conn:&rusqlite::Mutex<rusqlite::Connection>,
		WorkspaceId:&str,
		timestamp:&str,
		Data:&serde_json::Value,
	) -> Result<String, String> {
		let summary = Data.get("summary").and_then(|v| v.as_str()).unwrap_or("");
		let rationale = Data.get("rationale").and_then(|v| v.as_str()).unwrap_or("");
		let implementation_details = Data.get("implementation_details").and_then(|v| v.as_str()).unwrap_or("");
		let id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
		let tags_json = Data.get("tags").and_then(|v| serde_json::to_string(v).ok()).unwrap_or_default();

		let conn_guard = conn.lock().map_err(|e| e.to_string())?;
		conn_guard
			.execute(
				"INSERT OR REPLACE INTO Decisions (Id, WorkspaceId, Summary, Rationale, ImplementationDetails, Tags, \
				 CreatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
				params![
					id,
					WorkspaceId,
					summary,
					rationale,
					implementation_details,
					tags_json,
					timestamp
				],
			)
			.map_err(|e| e.to_string())?;

		Ok(id)
	}

	/// Store a progress entry
	fn StoreProgressEntry(
		conn:&rusqlite::Mutex<rusqlite::Connection>,
		WorkspaceId:&str,
		timestamp:&str,
		Data:&serde_json::Value,
	) -> Result<String, String> {
		let description = Data.get("description").and_then(|v| v.as_str()).unwrap_or("");
		let status = Data.get("status").and_then(|v| v.as_str()).unwrap_or("TODO");
		let parent_id = Data.get("parent_id").and_then(|v| v.as_str()).map(|s| s.to_string());
		let id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		let conn_guard = conn.lock().map_err(|e| e.to_string())?;
		conn_guard
			.execute(
				"INSERT INTO ProgressEntries (WorkspaceId, Timestamp, Status, Description, ParentId, CreatedAt, \
				 UpdatedAt) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
				params![WorkspaceId, timestamp, status, description, parent_id, timestamp, timestamp],
			)
			.map_err(|e| e.to_string())?;

		Ok(id)
	}

	/// Store a system pattern
	fn StoreSystemPattern(
		conn:&rusqlite::Mutex<rusqlite::Connection>,
		WorkspaceId:&str,
		timestamp:&str,
		Data:&serde_json::Value,
	) -> Result<String, String> {
		let name = Data.get("name").and_then(|v| v.as_str()).unwrap_or("");
		let description = Data.get("description").and_then(|v| v.as_str()).unwrap_or("");
		let id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
		let tags_json = Data.get("tags").and_then(|v| serde_json::to_string(v).ok()).unwrap_or_default();

		let conn_guard = conn.lock().map_err(|e| e.to_string())?;
		conn_guard
			.execute(
				"INSERT OR REPLACE INTO SystemPatterns (WorkspaceId, Timestamp, Name, Description, Tags, CreatedAt) \
				 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
				params![WorkspaceId, timestamp, name, description, tags_json, timestamp],
			)
			.map_err(|e| e.to_string())?;

		Ok(id)
	}

	/// Store custom data
	fn StoreCustomData(
		conn:&rusqlite::Mutex<rusqlite::Connection>,
		WorkspaceId:&str,
		timestamp:&str,
		Data:&serde_json::Value,
	) -> Result<String, String> {
		let key = Data.get("key").and_then(|v| v.as_str()).unwrap_or("");
		let category = Data.get("category").and_then(|v| v.as_str()).unwrap_or("default");
		let value = Data.get("value").map(|v| v.to_string()).unwrap_or_default();
		let id = Data
			.get("id")
			.and_then(|v| v.as_str())
			.map(|s| s.to_string())
			.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

		let conn_guard = conn.lock().map_err(|e| e.to_string())?;
		conn_guard
			.execute(
				"INSERT OR REPLACE INTO CustomData (WorkspaceId, Timestamp, Category, Key, Value, CreatedAt) VALUES \
				 (?1, ?2, ?3, ?4, ?5, ?6)",
				params![WorkspaceId, timestamp, category, key, value, timestamp],
			)
			.map_err(|e| e.to_string())?;

		Ok(id)
	}
}

/// HandleBatchDelete - Delete multiple items in a single request
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
	pub WorkspaceId:String,
	pub Items:Vec<BatchDeleteItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteItem {
	#[serde(rename = "Type")]
	pub TypeField:String,
	pub Id:String,
}

#[derive(Debug, Serialize)]
pub struct BatchDeleteResponse {
	pub Success:bool,
	pub ItemsDeleted:usize,
	pub ItemsFailed:usize,
	pub Errors:Vec<BatchError>,
}

pub struct HandleBatchDelete;

impl HandleBatchDelete {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		Payload:BatchDeleteRequest,
	) -> Result<BatchDeleteResponse, Error> {
		let _ = DbState;

		let mut ItemsDeleted = 0;
		let mut ItemsFailed = 0;
		let mut Errors = Vec::new();

		let Mutex = VectorStore.lock().await;

		for (Index, Item) in Payload.Items.iter().enumerate() {
			match Mutex.Delete(&Payload.WorkspaceId, &Item.Id) {
				Ok(_) => {
					ItemsDeleted += 1;
				},
				Err(e) => {
					ItemsFailed += 1;
					Errors.push(BatchError { Index, ItemType:Item.TypeField.clone(), Error:e.to_string() });
				},
			}
		}

		Ok(BatchDeleteResponse { Success:ItemsFailed == 0, ItemsDeleted, ItemsFailed, Errors })
	}
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for batch handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
	vec![
		crate::HTTP::Protocol::Request::Tool {
			Name:"batch_log_items".to_string(),
			Description:"Log multiple ConPort items in a single request (with vector embedding)".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"items": {
						"type": "array",
						"items": {
							"type": "object",
							"properties": {
								"type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data"]},
								"data": {"type": "object"}
							},
							"required": ["type", "data"]
						}
					}
				},
				"required": ["workspace_id", "items"]
			}),
		},
		crate::HTTP::Protocol::Request::Tool {
			Name:"batch_log_items_to_database".to_string(),
			Description:"Log multiple ConPort items directly to database without vector embedding".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"items": {
						"type": "array",
						"items": {
							"type": "object",
							"properties": {
								"type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data"]},
								"data": {"type": "object"}
							},
							"required": ["type", "data"]
						}
					}
				},
				"required": ["workspace_id", "items"]
			}),
		},
		crate::HTTP::Protocol::Request::Tool {
			Name:"batch_delete".to_string(),
			Description:"Delete multiple ConPort items in a single request".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"items": {
						"type": "array",
						"items": {
							"type": "object",
							"properties": {
								"type": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data"]},
								"id": {"type": "string"}
							},
							"required": ["type", "id"]
						}
					}
				},
				"required": ["workspace_id", "items"]
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
	fn test_batch_log_items_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "items": [
                {"type": "decision", "data": {"summary": "Test decision"}}},
                {"type": "progress_entry", "data": {"description": "Test progress"}}
            ]
        }"#;
		let Request:BatchLogItemsRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.WorkspaceId, "test");
		assert_eq!(Request.Items.len(), 2);
		assert_eq!(Request.Items[0].TypeField, "decision");
	}

	#[test]
	fn test_batch_delete_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "items": [
                {"type": "decision", "id": "dec-1"},
                {"type": "progress_entry", "id": "prog-1"}
            ]
        }"#;
		let Request:BatchDeleteRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.WorkspaceId, "test");
		assert_eq!(Request.Items.len(), 2);
	}

	#[test]
	fn test_batch_item_serialization() {
		let Item = BatchItem { TypeField:"decision".to_string(), Data:serde_json::json!({"summary": "Test"}) };

		let Json = serde_json::to_string(&Item).unwrap();
		let Parsed:BatchItem = serde_json::from_str(&Json).unwrap();

		assert_eq!(Parsed.TypeField, "decision");
	}
}

// ============================================================================
// HTTP Handlers (axum-compatible)
// ============================================================================

/// Batch log multiple items
pub async fn BatchLogItems(
	State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(payload):Json<BatchLogItemsRequest>,
) -> Result<Json<BatchLogItemsResponse>, crate::Error::Kind::Kind> {
	// TODO: Implement actual batch logging to database
	Ok(Json(BatchLogItemsResponse {
		Success:true,
		ItemsProcessed:payload.Items.len(),
		ItemsSucceeded:payload.Items.len(),
		ItemsFailed:0,
		Errors:vec![],
	}))
}

/// Batch delete items
pub async fn BatchDelete(
	State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
	Json(payload):Json<BatchDeleteRequest>,
) -> Result<Json<serde_json::Value>, crate::Error::Kind::Kind> {
	// TODO: Implement actual batch delete
	Ok(Json(serde_json::json!({
		"success": true,
		"items_deleted": payload.Items.len()
	})))
}
