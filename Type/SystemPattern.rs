// SystemPattern type for the ConPort MCP server
use serde::{Deserialize, Serialize};

/// Main SystemPattern structure representing a reusable pattern/discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPattern {
    pub Id: String,
    pub WorkspaceId: String,
    pub Timestamp: String,
    pub Name: String,
    pub Description: Option<String>,
    pub Tags: Option<Vec<String>>,
    pub CreatedAt: String,
}

impl SystemPattern {
    pub fn New(
        WorkspaceId: String,
        Name: String,
        Description: Option<String>,
        Tags: Option<Vec<String>>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            Id: uuid::Uuid::new_v4().to_string(),
            WorkspaceId,
            Timestamp: now.clone(),
            Name,
            Description,
            Tags,
            CreatedAt: now,
        }
    }

    pub fn with_id(Id: String, WorkspaceId: String, Name: String, Description: Option<String>, Tags: Option<Vec<String>>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            Id,
            WorkspaceId,
            Timestamp: now.clone(),
            Name,
            Description,
            Tags,
            CreatedAt: now,
        }
    }

    /// Serialize tags to JSON string for database storage
    pub fn tags_to_json(&self) -> Option<String> {
        self.Tags.as_ref().map(|tags| {
            serde_json::to_string(tags).unwrap_or_default()
        })
    }

    /// Deserialize tags from JSON string from database
    pub fn tags_from_json(json: &str) -> Option<Vec<String>> {
        if json.is_empty() {
            return None;
        }
        serde_json::from_str(json).ok()
    }
}

/// HTTP response type for SystemPattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPatternResponse {
    pub id: String,
    pub workspace_id: String,
    pub timestamp: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: String,
}

impl From<SystemPattern> for SystemPatternResponse {
    fn from(p: SystemPattern) -> Self {
        Self {
            id: p.Id,
            workspace_id: p.WorkspaceId,
            timestamp: p.Timestamp,
            name: p.Name,
            description: p.Description,
            tags: p.Tags,
            created_at: p.CreatedAt,
        }
    }
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for logging/creating a new system pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSystemPatternArgs {
    pub workspace_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

impl LogSystemPatternArgs {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is required".to_string());
        }
        Ok(())
    }
}

/// Arguments for retrieving system patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSystemPatternsArgs {
    pub workspace_id: String,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub tags_filter_include_all: Option<Vec<String>>,
    #[serde(default)]
    pub tags_filter_include_any: Option<Vec<String>>,
}

impl GetSystemPatternsArgs {
    pub fn limit_value(&self) -> i64 {
        self.limit.unwrap_or(50)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Cannot use both tag filters simultaneously
        if self.tags_filter_include_all.is_some() && self.tags_filter_include_any.is_some() {
            return Err("Cannot use 'tags_filter_include_all' and 'tags_filter_include_any' simultaneously".to_string());
        }
        
        if let Some(limit) = self.limit {
            if limit < 1 {
                return Err("limit must be greater than or equal to 1".to_string());
            }
        }
        
        Ok(())
    }
}

/// Arguments for deleting a system pattern by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSystemPatternByIdArgs {
    pub workspace_id: String,
    pub pattern_id: i64,
}

impl DeleteSystemPatternByIdArgs {
    pub fn validate(&self) -> Result<(), String> {
        if self.pattern_id < 1 {
            return Err("pattern_id must be greater than or equal to 1".to_string());
        }
        Ok(())
    }
}

/// Response type for delete operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSystemPatternResponse {
    pub status: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_pattern_creation() {
        let pattern = SystemPattern::New(
            "workspace1".to_string(),
            "Error Handling Pattern".to_string(),
            Some("A pattern for handling errors".to_string()),
            Some(vec!["error".to_string(), "handling".to_string()]),
        );
        
        assert!(!pattern.Id.is_empty());
        assert_eq!(pattern.Name, "Error Handling Pattern");
        assert_eq!(pattern.Description, Some("A pattern for handling errors".to_string()));
        assert_eq!(pattern.Tags, Some(vec!["error".to_string(), "handling".to_string()]));
    }

    #[test]
    fn test_system_pattern_tags_json() {
        let pattern = SystemPattern::New(
            "workspace1".to_string(),
            "Test Pattern".to_string(),
            None,
            Some(vec!["tag1".to_string(), "tag2".to_string()]),
        );
        
        let json = pattern.tags_to_json();
        assert!(json.is_some());
        
        let tags = SystemPattern::tags_from_json(&json.unwrap());
        assert_eq!(tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));
    }

    #[test]
    fn test_log_system_pattern_args_validation() {
        let valid_args = LogSystemPatternArgs {
            workspace_id: "ws1".to_string(),
            name: "Test Pattern".to_string(),
            description: Some("Description".to_string()),
            tags: Some(vec!["tag1".to_string()]),
        };
        assert!(valid_args.validate().is_ok());

        let invalid_args = LogSystemPatternArgs {
            workspace_id: "ws1".to_string(),
            name: "".to_string(),
            description: None,
            tags: None,
        };
        assert!(invalid_args.validate().is_err());
    }

    #[test]
    fn test_get_system_patterns_args_validation() {
        let valid_args = GetSystemPatternsArgs {
            workspace_id: "ws1".to_string(),
            limit: Some(10),
            tags_filter_include_all: None,
            tags_filter_include_any: None,
        };
        assert!(valid_args.validate().is_ok());

        // Both tags filters - should fail
        let invalid_args = GetSystemPatternsArgs {
            workspace_id: "ws1".to_string(),
            limit: None,
            tags_filter_include_all: Some(vec!["tag1".to_string()]),
            tags_filter_include_any: Some(vec!["tag2".to_string()]),
        };
        assert!(invalid_args.validate().is_err());

        // Invalid limit
        let invalid_limit = GetSystemPatternsArgs {
            workspace_id: "ws1".to_string(),
            limit: Some(0),
            tags_filter_include_all: None,
            tags_filter_include_any: None,
        };
        assert!(invalid_limit.validate().is_err());
    }
}