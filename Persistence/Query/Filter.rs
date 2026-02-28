// Tag Filtering Logic for ConPort MCP server
// Handles building SQL for tag-based filtering with include_all and include_any support

use serde::{Deserialize, Serialize};

/// Tag filter mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TagFilterMode {
    /// Include items with ALL tags (AND logic)
    IncludeAll,
    /// Include items with ANY tag (OR logic)
    IncludeAny,
}

/// Result of building a tag filter SQL query
#[derive(Debug, Clone)]
pub struct TagFilterSql {
    pub where_clause: String,
    pub params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl TagFilterSql {
    pub fn new(where_clause: String, params: Vec<Box<dyn rusqlite::ToSql>>) -> Self {
        Self {
            where_clause,
            params,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.where_clause.is_empty()
    }
}

/// Tag filter input parameters
#[derive(Debug, Clone)]
pub struct TagFilterInput {
    pub tags: Vec<String>,
    pub mode: TagFilterMode,
}

impl TagFilterInput {
    pub fn new_include_all(tags: Vec<String>) -> Self {
        Self {
            tags,
            mode: TagFilterMode::IncludeAll,
        }
    }

    pub fn new_include_any(tags: Vec<String>) -> Self {
        Self {
            tags,
            mode: TagFilterMode::IncludeAny,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tags.is_empty() {
            return Err("Tag filter requires at least one tag".to_string());
        }
        Ok(())
    }
}

/// Build SQL WHERE clause for tag filtering
/// 
/// # Arguments
/// * `tags` - List of tags to filter by
/// * `include_all` - If true, items must have ALL tags (AND). If false, items must have ANY tag (OR)
/// * `tags_column` - The column name containing the JSON tags array
/// 
/// # Returns
/// A tuple of (WHERE clause string, parameter values)
pub fn BuildTagFilterSql(
    tags: &[String],
    include_all: bool,
    tags_column: &str,
) -> Result<TagFilterSql, String> {
    if tags.is_empty() {
        return Ok(TagFilterSql::new(String::new(), vec![]));
    }

    // Validate tags
    for tag in tags {
        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }
    }

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    let where_clause = if include_all {
        // AND logic: item must have ALL tags
        let clauses: Vec<String> = tags
            .iter()
            .map(|tag| {
                params.push(Box::new(format!("%\"{}\"%", tag)));
                format!("{} LIKE ?", tags_column)
            })
            .collect();
        
        format!("({})", clauses.join(" AND "))
    } else {
        // OR logic: item must have ANY tag
        let conditions: Vec<String> = tags
            .iter()
            .map(|tag| {
                params.push(Box::new(format!("%\"{}\"%", tag)));
                format!("{} LIKE ?", tags_column)
            })
            .collect();
        
        format!("({})", conditions.join(" OR "))
    };

    Ok(TagFilterSql::new(where_clause, params))
}

/// Build tag filter for JSON array column using json_each
/// This is more precise for SQLite JSON columns
pub fn BuildTagFilterSqlJsonEach(
    tags: &[String],
    include_all: bool,
    tags_column: &str,
) -> Result<TagFilterSql, String> {
    if tags.is_empty() {
        return Ok(TagFilterSql::new(String::new(), vec![]));
    }

    // Validate tags
    for tag in tags {
        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }
    }

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    
    // Build the WHERE clause using json_each
    let subquery = if include_all {
        // AND logic: use HAVING to ensure all tags are present
        let having_clause = format!(
            "HAVING COUNT(DISTINCT value) = {}",
            tags.len()
        );
        
        let mut conditions = String::new();
        for tag in tags {
            params.push(Box::new(tag.clone()));
            if !conditions.is_empty() {
                conditions.push_str(" AND ");
            }
            conditions.push_str(&format!("value = ?"));
        }
        
        let subquery = format!(
            "SELECT 1 FROM json_each({}) AS je WHERE {} {}",
            tags_column, conditions, having_clause
        );
        
        format!("EXISTS ({})", subquery)
    } else {
        // OR logic: any match is sufficient
        let conditions: Vec<String> = tags
            .iter()
            .map(|tag| {
                params.push(Box::new(tag.clone()));
                "value = ?".to_string()
            })
            .collect();
        
        let subquery = format!(
            "SELECT 1 FROM json_each({}) AS je WHERE {} LIMIT 1",
            tags_column,
            conditions.join(" OR ")
        );
        
        format!("EXISTS ({})", subquery)
    };

    Ok(TagFilterSql::new(subquery, params))
}

/// Validate mutual exclusivity of include_all and include_any
/// 
/// # Arguments
/// * `include_all` - Option containing tags for include_all filter
/// * `include_any` - Option containing tags for include_any filter
/// 
/// # Returns
/// Ok if valid, Err with message if mutually exclusive
pub fn validate_mutual_exclusivity(
    include_all: &Option<Vec<String>>,
    include_any: &Option<Vec<String>>,
) -> Result<(), String> {
    let has_include_all = include_all
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    
    let has_include_any = include_any
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    if has_include_all && has_include_any {
        return Err(
            "Cannot use both tags_filter_include_all and tags_filter_include_any simultaneously. \
             Use one or the other, not both.".to_string()
        );
    }

    Ok(())
}

/// Determine tag filter mode and get tags from input
pub fn resolve_tag_filter(
    include_all: &Option<Vec<String>>,
    include_any: &Option<Vec<String>>,
) -> Option<TagFilterInput> {
    // First validate mutual exclusivity
    if let Err(_) = validate_mutual_exclusivity(include_all, include_any) {
        return None;
    }

    if let Some(tags) = include_all {
        if !tags.is_empty() {
            return Some(TagFilterInput::new_include_all(tags.clone()));
        }
    }

    if let Some(tags) = include_any {
        if !tags.is_empty() {
            return Some(TagFilterInput::new_include_any(tags.clone()));
        }
    }

    None
}

// ============================================================================
// Combined Filter Building
// ============================================================================

/// Build complete filter SQL with tags, pagination, and sorting
pub struct FilterSqlBuilder {
    base_query: String,
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl FilterSqlBuilder {
    pub fn new(base_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            conditions: vec![],
            params: vec![],
        }
    }

    /// Add workspace filter
    pub fn with_workspace(mut self, workspace_id: &str) -> Self {
        self.conditions.push("WorkspaceId = ?".to_string());
        self.params.push(Box::new(workspace_id.to_string()));
        self
    }

    /// Add status filter
    pub fn with_status(mut self, status: &str) -> Self {
        self.conditions.push("Status = ?".to_string());
        self.params.push(Box::new(status.to_string()));
        self
    }

    /// Add parent ID filter
    pub fn with_parent_id(mut self, parent_id: i64) -> Self {
        self.conditions.push("ParentId = ?".to_string());
        self.params.push(Box::new(parent_id));
        self
    }

    /// Add tag filter
    pub fn with_tag_filter(mut self, tag_filter: &TagFilterInput, tags_column: &str) -> Result<Self, String> {
        let sql = BuildTagFilterSqlJsonEach(&tag_filter.tags, 
            tag_filter.mode == TagFilterMode::IncludeAll,
            tags_column
        )?;
        
        if !sql.is_empty() {
            self.conditions.push(sql.where_clause);
            for param in sql.params {
                self.params.push(param);
            }
        }
        
        Ok(self)
    }

    /// Add sorting
    pub fn with_sorting(self, field: &str, direction: &str) -> Self {
        // Note: Sorting should be added after building the WHERE clause
        // This is handled in the build method
        self
    }

    /// Add pagination
    pub fn with_pagination(self, offset: Option<usize>, limit: Option<usize>) -> Self {
        // Note: Pagination should be added after ORDER BY
        // This is handled in the build method
        self
    }

    /// Build the final SQL query
    pub fn build(self, order_by: Option<&str>, offset: Option<usize>, limit: Option<usize>) -> String {
        let mut query = self.base_query.clone();

        // Add WHERE clause
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        // Add ORDER BY
        if let Some(order) = order_by {
            query.push_str(" ORDER BY ");
            query.push_str(order);
        }

        // Add LIMIT/OFFSET
        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        query
    }

    /// Get the params for the query
    pub fn params(self) -> Vec<Box<dyn rusqlite::ToSql>> {
        self.params
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tag_filter_include_all() {
        let tags = vec!["rust".to_string(), "database".to_string()];
        let result = BuildTagFilterSql(&tags, true, "Tags");
        
        assert!(result.is_ok());
        let sql = result.unwrap();
        assert!(sql.where_clause.contains("AND"));
        assert_eq!(sql.params.len(), 2);
    }

    #[test]
    fn test_build_tag_filter_include_any() {
        let tags = vec!["rust".to_string(), "database".to_string()];
        let result = BuildTagFilterSql(&tags, false, "Tags");
        
        assert!(result.is_ok());
        let sql = result.unwrap();
        assert!(sql.where_clause.contains("OR"));
        assert_eq!(sql.params.len(), 2);
    }

    #[test]
    fn test_empty_tags() {
        let tags: Vec<String> = vec![];
        let result = BuildTagFilterSql(&tags, true, "Tags");
        
        assert!(result.is_ok());
        let sql = result.unwrap();
        assert!(sql.is_empty());
    }

    #[test]
    fn test_validate_mutual_exclusivity() {
        let include_all = Some(vec!["tag1".to_string()]);
        let include_any = Some(vec!["tag2".to_string()]);
        
        let result = validate_mutual_exclusivity(&include_all, &include_any);
        assert!(result.is_err());
        
        let only_all = Some(vec!["tag1".to_string()]);
        let none: Option<Vec<String>> = None;
        
        let result2 = validate_mutual_exclusivity(&only_all, &none);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_resolve_tag_filter() {
        let include_all = Some(vec!["tag1".to_string(), "tag2".to_string()]);
        let none: Option<Vec<String>> = None;
        
        let result = resolve_tag_filter(&include_all, &none);
        assert!(result.is_some());
        
        let filter = result.unwrap();
        assert_eq!(filter.mode, TagFilterMode::IncludeAll);
        assert_eq!(filter.tags.len(), 2);
    }

    #[test]
    fn test_filter_sql_builder() {
        let builder = FilterSqlBuilder::new("SELECT * FROM Items")
            .with_workspace("/test/workspace")
            .with_status("active");
        
        let query = builder.build(Some("CreatedAt DESC"), Some(0), Some(10));
        
        assert!(query.contains("WHERE"));
        assert!(query.contains("WorkspaceId = ?"));
        assert!(query.contains("Status = ?"));
        assert!(query.contains("ORDER BY"));
        assert!(query.contains("LIMIT"));
    }
}