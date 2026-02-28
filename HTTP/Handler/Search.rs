// Search HTTP handler for the ConPort MCP server
use crate::HTTP::Protocol::Response::Error;
use crate::Persistence::Database::Connect;
use axum::{
    extract::State,
    Json,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub Id: String,
    #[serde(rename = "Type")]
    pub TypeField: String,
    pub Name: String,
    pub Description: String,
    pub Score: f32,
    pub Metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone)]
pub struct SearchDecisionsRequest {
    pub Query: String,
    pub Limit: Option<usize>,
    pub Threshold: Option<f32>,
    pub WorkspaceId: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct SearchContextRequest {
    pub Query: String,
    pub Limit: Option<usize>,
    pub Threshold: Option<f32>,
    pub WorkspaceId: Option<String>,
}

/// Semantic search request with full filter support
#[derive(Deserialize, Clone)]
pub struct SemanticSearchRequest {
    pub Query: String,
    pub WorkspaceId: String,
    #[serde(default)]
    pub Limit: Option<usize>,
    #[serde(default)]
    pub Threshold: Option<f32>,
    #[serde(default)]
    pub FilterItemTypes: Option<Vec<String>>,
    #[serde(default)]
    pub FilterTagsIncludeAny: Option<Vec<String>>,
    #[serde(default)]
    pub FilterTagsIncludeAll: Option<Vec<String>>,
    #[serde(default)]
    pub FilterCustomDataCategories: Option<Vec<String>>,
}

impl Default for SemanticSearchRequest {
    fn default() -> Self {
        Self {
            Query: String::new(),
            WorkspaceId: String::new(),
            Limit: Some(10),
            Threshold: Some(0.0),
            FilterItemTypes: None,
            FilterTagsIncludeAny: None,
            FilterTagsIncludeAll: None,
            FilterCustomDataCategories: None,
        }
    }
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub Results: Vec<SearchResult>,
    pub Total: usize,
    pub Query: String,
}

// ============================================================================
// HTTP Handlers (axum-compatible)
// ============================================================================

/// HTTP handler for searching decisions
pub async fn SearchDecisions(
    State(_state): State<Arc<Connect>>,
    Json(payload): Json<SearchDecisionsRequest>,
) -> Result<Json<SearchResponse>, crate::Error::Kind::Kind> {
    let limit = payload.Limit.unwrap_or(10);
    let _threshold = payload.Threshold.unwrap_or(0.0);
    
    // Generate embedding for the query
    let _embedding = crate::AI::Embedding::Generate::Generate::Execute(&payload.Query)
        .await
        .map_err(|e| crate::Error::Kind::Kind::Embedding(e.to_string()))?;
    
    // TODO: Query vector store for similar decisions
    // For now, return empty results
    Ok(Json(SearchResponse {
        Results: vec![],
        Total: 0,
        Query: payload.Query,
    }))
}

/// HTTP handler for searching context
pub async fn SearchContext(
    State(_state): State<Arc<Connect>>,
    Json(payload): Json<SearchContextRequest>,
) -> Result<Json<SearchResponse>, crate::Error::Kind::Kind> {
    let limit = payload.Limit.unwrap_or(10);
    let _threshold = payload.Threshold.unwrap_or(0.0);
    
    // Generate embedding for the query
    let _embedding = crate::AI::Embedding::Generate::Generate::Execute(&payload.Query)
        .await
        .map_err(|e| crate::Error::Kind::Kind::Embedding(e.to_string()))?;
    
    // TODO: Query vector store for similar contexts
    // For now, return empty results
    Ok(Json(SearchResponse {
        Results: vec![],
        Total: 0,
        Query: payload.Query,
    }))
}

/// HTTP handler for semantic search with full filters
pub async fn SemanticSearch(
    State(_state): State<Arc<Connect>>,
    Json(_payload): Json<SemanticSearchRequest>,
) -> Result<Json<SearchResponse>, crate::Error::Kind::Kind> {
    // TODO: Implement full semantic search with vector store
    // For now, return empty results
    Ok(Json(SearchResponse {
        Results: vec![],
        Total: 0,
        Query: String::new(),
    }))
}

// HandleSearchDecisions - Semantic search for decisions
pub struct HandleSearchDecisions;

impl HandleSearchDecisions {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        Payload: SearchDecisionsRequest,
    ) -> Result<SearchResponse, Error> {
        let _ = DbState;
        let Limit = Payload.Limit.unwrap_or(10);
        let _Threshold = Payload.Threshold.unwrap_or(0.0);
        let _ = Limit;

        // Generate embedding for the query
        let _Embedding = crate::AI::Embedding::Generate::Generate::Execute(&Payload.Query)
            .await
            .map_err(|e: crate::Error::Kind::Kind| Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string()))?;

        // TODO: Query vector store for similar decisions
        // For now, return empty results
        let Results = vec![];

        Ok(SearchResponse {
            Results,
            Total: 0,
            Query: Payload.Query,
        })
    }
}

// HandleSearchContext - Semantic search for context
pub struct HandleSearchContext;

impl HandleSearchContext {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        Payload: SearchContextRequest,
    ) -> Result<SearchResponse, Error> {
        let _ = DbState;
        let Limit = Payload.Limit.unwrap_or(10);
        let _Threshold = Payload.Threshold.unwrap_or(0.0);
        let _ = Limit;

        // Generate embedding for the query
        let _Embedding = crate::AI::Embedding::Generate::Generate::Execute(&Payload.Query)
            .await
            .map_err(|e: crate::Error::Kind::Kind| Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string()))?;

        // TODO: Query vector store for similar contexts
        // For now, return empty results
        let Results = vec![];

        Ok(SearchResponse {
            Results,
            Total: 0,
            Query: Payload.Query,
        })
    }
}

// ============================================================================
// Semantic Search with Filters
// ============================================================================

/// HandleSemanticSearch - Full-featured semantic search with filters
pub struct HandleSemanticSearch;

impl HandleSemanticSearch {
    pub async fn Execute(
        DbState: &Arc<Connect>,
        VectorStore: &Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        Payload: SemanticSearchRequest,
    ) -> Result<SearchResponse, Error> {
        // Generate embedding for the query
        let Embedding = Self::GenerateEmbedding(EmbeddingModel, &Payload.Query).await?;
        
        let limit = Payload.Limit.unwrap_or(10);
        let threshold = Payload.Threshold.unwrap_or(0.0);
        
        // Search vector store
        let vector_results = {
            let mutex = VectorStore.lock().await;
            match mutex.Search(&Payload.WorkspaceId, &Embedding, limit * 2, threshold) {
                Ok(results) => results,
                Err(e) => {
                    return Err(Error::WithMessage(
                        crate::HTTP::Protocol::Response::ErrorCode::InternalError,
                        &format!("Vector search failed: {}", e),
                    ));
                }
            }
        };
        
        // Apply filters and build results
        let mut results = Vec::new();
        
        for item in vector_results {
            // Filter by item types if specified
            if let Some(ref types) = Payload.FilterItemTypes {
                if !types.is_empty() && !types.contains(&item.ItemType) {
                    continue;
                }
            }
            
            // Get additional metadata from database for filtering
            let metadata = Self::GetItemMetadata(DbState, &Payload.WorkspaceId, &item.ItemType, &item.Id).await;
            
            // Filter by tags if specified
            if let Some(ref tags_any) = Payload.FilterTagsIncludeAny {
                if let Some(ref meta) = metadata {
                    let item_tags: Vec<String> = meta.get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    
                    // Check if any of the requested tags match
                    if !tags_any.iter().any(|t| item_tags.contains(t)) {
                        continue;
                    }
                }
            }
            
            if let Some(ref tags_all) = Payload.FilterTagsIncludeAll {
                if let Some(ref meta) = metadata {
                    let item_tags: Vec<String> = meta.get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    
                    // Check if all requested tags are present
                    if !tags_all.iter().all(|t| item_tags.contains(t)) {
                        continue;
                    }
                }
            }
            
            // Filter by custom data categories
            if item.ItemType == "custom_data" {
                if let Some(ref categories) = Payload.FilterCustomDataCategories {
                    if let Some(ref meta) = metadata {
                        let category = meta.get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        if !categories.is_empty() && !categories.contains(&category.to_string()) {
                            continue;
                        }
                    }
                }
            }
            
            results.push(SearchResult {
                Id: item.Id,
                TypeField: item.ItemType,
                Name: item.Name,
                Description: item.Description,
                Score: item.Score,
                Metadata: metadata,
            });
            
            if results.len() >= limit {
                break;
            }
        }
        
        let total = results.len();
        
        Ok(SearchResponse {
            Results: results,
            Total: total,
            Query: Payload.Query,
        })
    }
    
    /// Generate embedding for search query
    async fn GenerateEmbedding(
        EmbeddingModel: &Arc<tokio::sync::Mutex<Option<crate::AI::Embedding::Model::Model>>>,
        Content: &str,
    ) -> Result<Vec<f32>, Error> {
        let ModelGuard = EmbeddingModel.lock().await;
        
        let embedding = if let Some(Model) = ModelGuard.as_ref() {
            Model.Generate(Content)
                .map_err(|e| Error::WithMessage(
                    crate::HTTP::Protocol::Response::ErrorCode::InternalError,
                    &e.to_string(),
                ))?
        } else {
            // Fallback to simple embedding
            crate::AI::Embedding::Generate::Generate::Execute(Content)
                .await
                .map_err(|e: crate::Error::Kind::Kind| Error::WithMessage(
                    crate::HTTP::Protocol::Response::ErrorCode::InternalError,
                    &e.to_string(),
                ))?
        };
        
        Ok(embedding)
    }
    
    /// Get additional metadata for an item from the database
    async fn GetItemMetadata(
        DbState: &Arc<Connect>,
        WorkspaceId: &str,
        ItemType: &str,
        ItemId: &str,
    ) -> Option<serde_json::Value> {
        let conn = match DbState.Connection.lock() {
            Ok(c) => c,
            Err(_) => return None,
        };
        
        match ItemType {
            "decision" => {
                let mut stmt = match conn.prepare(
                    "SELECT Summary, Rationale, Tags FROM Decisions WHERE Id = ? AND WorkspaceId = ?"
                ) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                
                let result: Result<(String, String, String), _> = stmt.query_row(
                    params![ItemId, WorkspaceId],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
                
                if let Ok((summary, rationale, tags)) = result {
                    let tags: Vec<String> = serde_json::from_str(&tags).unwrap_or_default();
                    Some(serde_json::json!({
                        "summary": summary,
                        "rationale": rationale,
                        "tags": tags,
                    }))
                } else {
                    None
                }
            }
            "progress_entry" => {
                let mut stmt = match conn.prepare(
                    "SELECT Status, Description, ParentId FROM ProgressEntries WHERE Id = ? AND WorkspaceId = ?"
                ) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                
                let result: Result<(String, String, Option<String>), _> = stmt.query_row(
                    params![ItemId, WorkspaceId],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
                
                if let Ok((status, description, parent_id)) = result {
                    Some(serde_json::json!({
                        "status": status,
                        "description": description,
                        "parent_id": parent_id,
                    }))
                } else {
                    None
                }
            }
            "system_pattern" => {
                let mut stmt = match conn.prepare(
                    "SELECT Name, Description, Tags FROM SystemPatterns WHERE Id = ? AND WorkspaceId = ?"
                ) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                
                let result: Result<(String, String, String), _> = stmt.query_row(
                    params![ItemId, WorkspaceId],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
                
                if let Ok((name, description, tags)) = result {
                    let tags: Vec<String> = serde_json::from_str(&tags).unwrap_or_default();
                    Some(serde_json::json!({
                        "name": name,
                        "description": description,
                        "tags": tags,
                    }))
                } else {
                    None
                }
            }
            "custom_data" => {
                let mut stmt = match conn.prepare(
                    "SELECT Category, Key, Value FROM CustomData WHERE Id = ? AND WorkspaceId = ?"
                ) {
                    Ok(s) => s,
                    Err(_) => return None,
                };
                
                let result: Result<(String, String, String), _> = stmt.query_row(
                    params![ItemId, WorkspaceId],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
                
                if let Ok((category, key, value)) = result {
                    let value: serde_json::Value = serde_json::from_str(&value).unwrap_or(serde_json::Value::Null);
                    Some(serde_json::json!({
                        "category": category,
                        "key": key,
                        "value": value,
                    }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for search handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
    vec![
        crate::HTTP::Protocol::Request::Tool {
            Name: "search_decisions".to_string(),
            Description: "Semantic search for decisions".to_string(),
            InputSchema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100},
                    "threshold": {"type": "number", "minimum": 0, "maximum": 1},
                    "workspace_id": {"type": "string"}
                },
                "required": ["query"]
            }),
        },
        crate::HTTP::Protocol::Request::Tool {
            Name: "search_context".to_string(),
            Description: "Semantic search for context items".to_string(),
            InputSchema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100},
                    "threshold": {"type": "number", "minimum": 0, "maximum": 1},
                    "workspace_id": {"type": "string"}
                },
                "required": ["query"]
            }),
        },
        crate::HTTP::Protocol::Request::Tool {
            Name: "semantic_search".to_string(),
            Description: "Full-featured semantic search with filters".to_string(),
            InputSchema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "workspace_id": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100},
                    "threshold": {"type": "number", "minimum": 0, "maximum": 1},
                    "filter_item_types": {
                        "type": "array",
                        "items": {"type": "string", "enum": ["decision", "progress_entry", "system_pattern", "custom_data"]},
                        "description": "Filter results by item types"
                    },
                    "filter_tags_include_any": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Return items that have ANY of these tags"
                    },
                    "filter_tags_include_all": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Return items that have ALL of these tags"
                    },
                    "filter_custom_data_categories": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Filter custom data by categories"
                    }
                },
                "required": ["query", "workspace_id"]
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_search_request_parsing() {
        let json = r#"{
            "query": "test search",
            "workspace_id": "test-workspace",
            "limit": 20,
            "threshold": 0.5,
            "filter_item_types": ["decision", "progress_entry"],
            "filter_tags_include_any": ["important", "urgent"],
            "filter_tags_include_all": ["reviewed"],
            "filter_custom_data_categories": ["config", "settings"]
        }"#;
        
        let request: SemanticSearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.Query, "test search");
        assert_eq!(request.WorkspaceId, "test-workspace");
        assert_eq!(request.Limit, Some(20));
        assert_eq!(request.Threshold, Some(0.5));
        assert_eq!(request.FilterItemTypes.unwrap().len(), 2);
        assert_eq!(request.FilterTagsIncludeAny.unwrap().len(), 2);
        assert_eq!(request.FilterTagsIncludeAll.unwrap().len(), 1);
        assert_eq!(request.FilterCustomDataCategories.unwrap().len(), 2);
    }

    #[test]
    fn test_semantic_search_request_defaults() {
        let json = r#"{
            "query": "test",
            "workspace_id": "test"
        }"#;
        
        let request: SemanticSearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.Limit, None);
        assert_eq!(request.Threshold, None);
        assert!(request.FilterItemTypes.is_none());
        assert!(request.FilterTagsIncludeAny.is_none());
        assert!(request.FilterTagsIncludeAll.is_none());
        assert!(request.FilterCustomDataCategories.is_none());
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            Id: "test-id".to_string(),
            TypeField: "decision".to_string(),
            Name: "Test Decision".to_string(),
            Description: "A test decision".to_string(),
            Score: 0.95,
            Metadata: Some(serde_json::json!({"tags": ["test"]})),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("decision"));
    }
}