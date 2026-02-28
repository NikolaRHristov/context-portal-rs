// History HTTP handlers for the ConPort MCP server
// Provides handlers for retrieving context history and recent activity

use crate::HTTP::Protocol::Response::Error;
use crate::Type::History;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request for getting item history
#[derive(Debug, Deserialize)]
pub struct GetItemHistoryRequest {
    pub WorkspaceId: String,
    #[serde(rename = "Type")]
    pub TypeField: String,
    pub Limit: Option<usize>,
    pub BeforeTimestamp: Option<String>,
    pub AfterTimestamp: Option<String>,
    pub Version: Option<i64>,
}

/// Response for getting item history
#[derive(Debug, Serialize)]
pub struct GetItemHistoryResponse {
    pub Items: Vec<History::HistoryResponse>,
    pub Total: usize,
    pub WorkspaceId: String,
}

/// Request for getting recent activity
#[derive(Debug, Deserialize)]
pub struct GetRecentActivityRequest {
    pub WorkspaceId: String,
    pub HoursAgo: Option<i64>,
    pub LimitPerType: Option<usize>,
    pub ItemTypes: Option<Vec<String>>,
}

/// Response for getting recent activity
#[derive(Debug, Serialize)]
pub struct RecentActivityItem {
    pub ItemType: String,
    pub Id: String,
    pub Timestamp: String,
    pub Summary: String,
}

#[derive(Debug, Serialize)]
pub struct GetRecentActivityResponse {
    pub Items: Vec<RecentActivityItem>,
    pub Total: usize,
}

/// HandleGetItemHistory - Get history of a context item
pub struct HandleGetItemHistory;

impl HandleGetItemHistory {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        Payload: GetItemHistoryRequest,
    ) -> Result<GetItemHistoryResponse, Error> {
        // Validate item type
        if !History::item_type::is_valid(&Payload.TypeField) {
            return Err(Error::WithMessage(
                crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
                &format!("Invalid item type: {}", Payload.TypeField),
            ));
        }

        let Limit = Payload.Limit.unwrap_or(10);

        // Query database for history
        // In production, this would query the actual database
        let Items = Self::QueryHistory(
            DbState,
            &Payload.WorkspaceId,
            &Payload.TypeField,
            Limit,
            Payload.BeforeTimestamp.as_deref(),
            Payload.AfterTimestamp.as_deref(),
            Payload.Version,
        )
        .await
        .map_err(|e| Error::WithMessage(
            crate::HTTP::Protocol::Response::ErrorCode::InternalError,
            &e.to_string(),
        ))?;

        let Total = Items.len();

        Ok(GetItemHistoryResponse {
            Items,
            Total,
            WorkspaceId: Payload.WorkspaceId,
        })
    }

    /// Query history from database
    async fn QueryHistory(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        WorkspaceId: &str,
        ItemType: &str,
        Limit: usize,
        BeforeTimestamp: Option<&str>,
        AfterTimestamp: Option<&str>,
        Version: Option<i64>,
    ) -> Result<Vec<History::HistoryResponse>, crate::Error::Kind::Kind> {
        // Build SQL query based on parameters
        let _ = DbState;
        let _ = WorkspaceId;
        let _ = ItemType;
        let _ = BeforeTimestamp;
        let _ = AfterTimestamp;
        let _ = Version;

        // TODO: Implement actual database query
        // For now, return empty results
        tracing::debug!(
            "Querying history for workspace {} type {} limit {}",
            WorkspaceId,
            ItemType,
            Limit
        );

        Ok(vec![])
    }
}

/// HandleGetRecentActivity - Get recent activity across all types
pub struct HandleGetRecentActivity;

impl HandleGetRecentActivity {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        Payload: GetRecentActivityRequest,
    ) -> Result<GetRecentActivityResponse, Error> {
        let HoursAgo = Payload.HoursAgo.unwrap_or(24);
        let LimitPerType = Payload.LimitPerType.unwrap_or(5);

        // Get all item types if not specified
        let ItemTypes = Payload.ItemTypes.unwrap_or_else(|| History::item_type::all().iter().map(|s| s.to_string()).collect());

        // Query recent activity for each type
        let mut AllItems = Vec::new();

        for ItemType in &ItemTypes {
            let Items = Self::QueryRecentActivity(
                DbState,
                &Payload.WorkspaceId,
                ItemType,
                HoursAgo,
                LimitPerType,
            )
            .await
            .map_err(|e| Error::WithMessage(
                crate::HTTP::Protocol::Response::ErrorCode::InternalError,
                &e.to_string(),
            ))?;

            AllItems.extend(Items);
        }

        // Sort by timestamp (most recent first)
        AllItems.sort_by(|a, b| b.Timestamp.cmp(&a.Timestamp));

        let Total = AllItems.len();

        Ok(GetRecentActivityResponse { Items: AllItems, Total })
    }

    /// Query recent activity from database
    async fn QueryRecentActivity(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        WorkspaceId: &str,
        ItemType: &str,
        HoursAgo: i64,
        Limit: usize,
    ) -> Result<Vec<RecentActivityItem>, crate::Error::Kind::Kind> {
        let _ = DbState;
        let _ = WorkspaceId;
        let _ = ItemType;
        let _ = HoursAgo;
        let _ = Limit;

        // TODO: Implement actual database query
        tracing::debug!(
            "Querying recent activity for workspace {} type {} hours {} limit {}",
            WorkspaceId,
            ItemType,
            HoursAgo,
            Limit
        );

        Ok(vec![])
    }
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for history handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
    vec![
        crate::HTTP::Protocol::Request::Tool {
            Name: "get_item_history".to_string(),
            Description: "Get version history for Product or Active Context".to_string(),
            InputSchema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workspace_id": {"type": "string"},
                    "type": {"type": "string", "enum": ["product_context", "active_context"]},
                    "limit": {"type": "integer"},
                    "before_timestamp": {"type": "string"},
                    "after_timestamp": {"type": "string"},
                    "version": {"type": "integer"}
                },
                "required": ["workspace_id", "type"]
            }),
        },
        crate::HTTP::Protocol::Request::Tool {
            Name: "get_recent_activity".to_string(),
            Description: "Get recent activity summary across all ConPort items".to_string(),
            InputSchema: serde_json::json!({
                "type": "object",
                "properties": {
                    "workspace_id": {"type": "string"},
                    "hours_ago": {"type": "integer"},
                    "limit_per_type": {"type": "integer"},
                    "item_types": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["workspace_id"]
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
    fn test_get_item_history_request_parsing() {
        let Json = r#"{"workspace_id": "test", "type": "product_context", "limit": 5}"#;
        let Request: GetItemHistoryRequest = serde_json::from_str(Json).unwrap();
        assert_eq!(Request.WorkspaceId, "test");
        assert_eq!(Request.TypeField, "product_context");
        assert_eq!(Request.Limit, Some(5));
    }

    #[test]
    fn test_get_recent_activity_request_parsing() {
        let Json = r#"{"workspace_id": "test", "hours_ago": 48}"#;
        let Request: GetRecentActivityRequest = serde_json::from_str(Json).unwrap();
        assert_eq!(Request.WorkspaceId, "test");
        assert_eq!(Request.HoursAgo, Some(48));
    }

    #[test]
    fn test_invalid_item_type() {
        let Json = r#"{"workspace_id": "test", "type": "invalid"}"#;
        let Result: Result<GetItemHistoryRequest, _> = serde_json::from_str(Json);
        // Should parse but validation happens in Execute
        assert!(Result.is_ok());
    }
}