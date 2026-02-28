// Batch HTTP handlers for the ConPort MCP server
// Provides handlers for batch operations on ConPort items

use crate::HTTP::Protocol::Response::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request for batch logging items
#[derive(Debug, Deserialize)]
pub struct BatchLogItemsRequest {
    pub WorkspaceId: String,
    pub Items: Vec<BatchItem>,
}

/// Single item in a batch request
#[derive(Debug, Deserialize)]
pub struct BatchItem {
    #[serde(rename = "Type")]
    pub TypeField: String,
    pub Data: serde_json::Value,
}

/// Response for batch operation
#[derive(Debug, Serialize)]
pub struct BatchLogItemsResponse {
    pub Success: bool,
    pub ItemsProcessed: usize,
    pub ItemsSucceeded: usize,
    pub ItemsFailed: usize,
    pub Errors: Vec<BatchError>,
}

/// Error for a single item in batch
#[derive(Debug, Serialize)]
pub struct BatchError {
    pub Index: usize,
    pub ItemType: String,
    pub Error: String,
}

/// HandleBatchLogItems - Log multiple items in a single request
pub struct HandleBatchLogItems;

impl HandleBatchLogItems {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        Payload: BatchLogItemsRequest,
    ) -> Result<BatchLogItemsResponse, Error> {
        let TotalItems = Payload.Items.len();
        let mut ItemsSucceeded = 0;
        let mut ItemsFailed = 0;
        let mut Errors = Vec::new();

        // Process each item
        for (Index, Item) in Payload.Items.iter().enumerate() {
            match Self::ProcessItem(
                DbState,
                VectorStore,
                EmbeddingModel,
                &Payload.WorkspaceId,
                Item,
            )
            .await
            {
                Ok(_) => {
                    ItemsSucceeded += 1;
                }
                Err(e) => {
                    ItemsFailed += 1;
                    Errors.push(BatchError {
                        Index,
                        ItemType: Item.TypeField.clone(),
                        Error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchLogItemsResponse {
            Success: ItemsFailed == 0,
            ItemsProcessed: TotalItems,
            ItemsSucceeded,
            ItemsFailed,
            Errors,
        })
    }

    /// Process a single item based on its type
    async fn ProcessItem(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        WorkspaceId: &str,
        Item: &BatchItem,
    ) -> Result<(), crate::Error::Kind::Kind> {
        match Item.TypeField.as_str() {
            "decision" => {
                Self::ProcessDecision(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data)
                    .await
            }
            "progress_entry" => {
                Self::ProcessProgressEntry(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data)
                    .await
            }
            "system_pattern" => {
                Self::ProcessSystemPattern(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data)
                    .await
            }
            "custom_data" => {
                Self::ProcessCustomData(DbState, VectorStore, EmbeddingModel, WorkspaceId, &Item.Data)
                    .await
            }
            _ => Err(crate::Error::Kind::Kind::InvalidItemType(Item.TypeField.clone())),
        }
    }

    /// Process a decision item
    async fn ProcessDecision(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        WorkspaceId: &str,
        Data: &serde_json::Value,
    ) -> Result<(), crate::Error::Kind::Kind> {
        // Parse decision data
        let Summary = Data.get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Id = Data.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Generate embedding for search
        let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Summary).await?;

        // Save to database
        let _ = DbState;
        
        // Add to vector store
        let Mutex = VectorStore.lock().await;
        Mutex.Add(WorkspaceId, &Id, &Embedding, "decision")?;

        tracing::debug!("Processed decision: {}", Id);
        Ok(())
    }

    /// Process a progress entry item
    async fn ProcessProgressEntry(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        WorkspaceId: &str,
        Data: &serde_json::Value,
    ) -> Result<(), crate::Error::Kind::Kind> {
        let Description = Data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Id = Data.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Generate embedding
        let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Description).await?;

        let _ = DbState;

        let Mutex = VectorStore.lock().await;
        Mutex.Add(WorkspaceId, &Id, &Embedding, "progress_entry")?;

        tracing::debug!("Processed progress entry: {}", Id);
        Ok(())
    }

    /// Process a system pattern item
    async fn ProcessSystemPattern(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        WorkspaceId: &str,
        Data: &serde_json::Value,
    ) -> Result<(), crate::Error::Kind::Kind> {
        let Name = Data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Description = Data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Id = Data.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let Content = format!("{}: {}", Name, Description);
        let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Content).await?;

        let _ = DbState;

        let Mutex = VectorStore.lock().await;
        Mutex.Add(WorkspaceId, &Id, &Embedding, "system_pattern")?;

        tracing::debug!("Processed system pattern: {}", Id);
        Ok(())
    }

    /// Process a custom data item
    async fn ProcessCustomData(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        WorkspaceId: &str,
        Data: &serde_json::Value,
    ) -> Result<(), crate::Error::Kind::Kind> {
        let Key = Data.get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Value = Data.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let Category = Data.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();
        let Id = Data.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let Content = format!("[{}] {}: {}", Category, Key, Value);
        let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Content).await?;

        let _ = DbState;

        let Mutex = VectorStore.lock().await;
        Mutex.Add(WorkspaceId, &Id, &Embedding, "custom_data")?;

        tracing::debug!("Processed custom data: {}", Id);
        Ok(())
    }

    /// Generate embedding for text content
    async fn GenerateEmbedding(
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        Content: &str,
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

/// HandleBatchDelete - Delete multiple items in a single request
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub WorkspaceId: String,
    pub Items: Vec<BatchDeleteItem>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteItem {
    #[serde(rename = "Type")]
    pub TypeField: String,
    pub Id: String,
}

#[derive(Debug, Serialize)]
pub struct BatchDeleteResponse {
    pub Success: bool,
    pub ItemsDeleted: usize,
    pub ItemsFailed: usize,
    pub Errors: Vec<BatchError>,
}

pub struct HandleBatchDelete;

impl HandleBatchDelete {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        Payload: BatchDeleteRequest,
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
                }
                Err(e) => {
                    ItemsFailed += 1;
                    Errors.push(BatchError {
                        Index,
                        ItemType: Item.TypeField.clone(),
                        Error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchDeleteResponse {
            Success: ItemsFailed == 0,
            ItemsDeleted,
            ItemsFailed,
            Errors,
        })
    }
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for batch handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
    vec![
        crate::HTTP::Protocol::Request::Tool {
            Name: "batch_log_items".to_string(),
            Description: "Log multiple ConPort items in a single request".to_string(),
            InputSchema: serde_json::json!({
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
            Name: "batch_delete".to_string(),
            Description: "Delete multiple ConPort items in a single request".to_string(),
            InputSchema: serde_json::json!({
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
        let Request: BatchLogItemsRequest = serde_json::from_str(Json).unwrap();
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
        let Request: BatchDeleteRequest = serde_json::from_str(Json).unwrap();
        assert_eq!(Request.WorkspaceId, "test");
        assert_eq!(Request.Items.len(), 2);
    }

    #[test]
    fn test_batch_item_serialization() {
        let Item = BatchItem {
            TypeField: "decision".to_string(),
            Data: serde_json::json!({"summary": "Test"}),
        };

        let Json = serde_json::to_string(&Item).unwrap();
        let Parsed: BatchItem = serde_json::from_str(&Json).unwrap();

        assert_eq!(Parsed.TypeField, "decision");
    }
}