// FTS Query Normalization for ConPort MCP server
// Handles preparing queries for SQLite full-text search

use regex::Regex;
use std::collections::HashSet;

/// Escape patterns for FTS special characters
const FTS_ESCAPE_CHARS: &[char] = &['"', '(', ')', '*', ':', '^', '-', '+', '~'];

/// Prepare an FTS query by escaping special characters and handling column prefixes
pub struct FtsQueryNormalizer {
    allowed_columns: Option<HashSet<String>>,
}

impl FtsQueryNormalizer {
    /// Create a new normalizer with optional allowed columns
    pub fn new(allowed_columns: Option<Vec<String>>) -> Self {
        let cols = allowed_columns.map(|v| v.into_iter().collect());
        Self { allowed_columns: cols }
    }

    /// Prepare an FTS query string for safe use
    pub fn prepare_fts_query(&self, query: &str) -> String {
        self.normalize_query(query)
    }

    /// Normalize a query string for FTS
    fn normalize_query(&self, query: &str) -> String {
        let mut result = query.to_string();

        // Escape FTS special characters
        for ch in FTS_ESCAPE_CHARS {
            result = result.replace(*ch, &format!("\\{}", ch));
        }

        // Handle column prefixes (e.g., "summary:foo" -> "summary:foo")
        // But validate that only allowed columns are used
        if self.allowed_columns.is_some() {
            result = self.validate_column_prefixes(&result);
        }

        // Collapse multiple whitespace
        let re = Regex::new(r"\s+").unwrap();
        result = re.replace_all(&result, " ").to_string();

        result.trim().to_string()
    }

    /// Validate and sanitize column prefixes in the query
    fn validate_column_prefixes(&self, query: &str) -> String {
        let allowed = match &self.allowed_columns {
            Some(cols) => cols,
            None => return query.to_string(),
        };

        // Regex to find column:pattern patterns
        let re = Regex::new(r"(\w+):(\S+)").unwrap();
        
        let mut result = query.to_string();
        
        for cap in re.captures_iter(query) {
            if let Some(col) = cap.get(1) {
                let col_name = col.as_str();
                if !allowed.contains(col_name) {
                    // Remove the invalid column prefix
                    let pattern = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                    result = result.replace(&cap[0], pattern);
                }
            }
        }

        result
    }
}

/// Prepare an FTS query string with default settings
pub fn PrepareFtsQuery(query: &str) -> String {
    let normalizer = FtsQueryNormalizer::new(None);
    normalizer.prepare_fts_query(query)
}

/// Prepare an FTS query with column validation
pub fn PrepareFtsQueryWithColumns(query: &str, allowed_columns: Vec<String>) -> String {
    let normalizer = FtsQueryNormalizer::new(Some(allowed_columns));
    normalizer.prepare_fts_query(query)
}

// ============================================================================
// Tag-based FTS Query Helpers
// ============================================================================

/// Convert tags to FTS query format
pub fn tags_to_fts_query(tags: &[String], match_all: bool) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let operator = if match_all { " AND " } else { " OR " };
    
    tags.iter()
        .map(|tag| {
            let escaped = escape_tag_for_fts(tag);
            format!("\"{}\"", escaped)
        })
        .collect::<Vec<_>>()
        .join(operator)
}

/// Escape a tag for use in FTS queries
fn escape_tag_for_fts(tag: &str) -> String {
    let mut result = tag.to_string();
    for ch in FTS_ESCAPE_CHARS {
        result = result.replace(*ch, &format!("\\{}", ch));
    }
    result
}

// ============================================================================
// Search Query Parsing
// ============================================================================

/// Parse a search query into terms and operators
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub terms: Vec<String>,
    pub exclude_terms: Vec<String>,
    pub phrase_searches: Vec<String>,
}

impl SearchQuery {
    /// Parse a query string into structured components
    pub fn parse(query: &str) -> Self {
        let mut terms = Vec::new();
        let mut exclude_terms = Vec::new();
        let mut phrase_searches = Vec::new();

        // Regex to find quoted phrases
        let phrase_re = Regex::new(r#""([^"]+)""#).unwrap();
        
        // Extract quoted phrases first
        for cap in phrase_re.captures_iter(query) {
            if let Some(phrase) = cap.get(1) {
                phrase_searches.push(phrase.as_str().to_string());
            }
        }

        // Remove quoted phrases from query for further processing
        let remaining = phrase_re.replace_all(query, "");

        // Split by whitespace and process
        for word in remaining.split_whitespace() {
            if word.starts_with('-') && word.len() > 1 {
                exclude_terms.push(word[1..].to_string());
            } else if !word.is_empty() {
                terms.push(word.to_string());
            }
        }

        Self {
            terms,
            exclude_terms,
            phrase_searches,
        }
    }

    /// Convert to FTS query format
    pub fn to_fts_query(&self) -> String {
        let mut parts = Vec::new();

        // Add regular terms
        for term in &self.terms {
            parts.push(format!("\"{}\"", term));
        }

        // Add phrase searches
        for phrase in &self.phrase_searches {
            parts.push(format!("\"{}\"", phrase));
        }

        // Add exclusion terms with NOT
        for term in &self.exclude_terms {
            parts.push(format!("NOT \"{}\"", term));
        }

        parts.join(" ")
    }
}

/// Build FTS WHERE clause for a table
pub fn build_fts_where_clause(
    query: &str,
    columns: &[&str],
) -> String {
    let normalized = PrepareFtsQuery(query);
    let search_query = SearchQuery::parse(&normalized);
    let fts_query = search_query.to_fts_query();

    if fts_query.is_empty() {
        return String::new();
    }

    // Build OR clause across columns
    let column_clauses: Vec<String> = columns
        .iter()
        .map(|col| format!("{} MATCH ?", col))
        .collect();

    format!("({})", column_clauses.join(" OR "))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_fts_query() {
        let query = "hello world";
        let result = PrepareFtsQuery(query);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_prepare_fts_query_with_special_chars() {
        let query = "test (query)";
        let result = PrepareFtsQuery(query);
        assert_eq!(result, "test \\(query\\)");
    }

    #[test]
    fn test_tags_to_fts_query_or() {
        let tags = vec!["rust".to_string(), "database".to_string()];
        let result = tags_to_fts_query(&tags, false);
        assert!(result.contains("\"rust\""));
        assert!(result.contains("\"database\""));
        assert!(result.contains(" OR "));
    }

    #[test]
    fn test_tags_to_fts_query_and() {
        let tags = vec!["rust".to_string(), "database".to_string()];
        let result = tags_to_fts_query(&tags, true);
        assert!(result.contains(" AND "));
    }

    #[test]
    fn test_search_query_parse() {
        let query = "test -exclude \"phrase search\"";
        let parsed = SearchQuery::parse(query);
        
        assert!(parsed.terms.contains(&"test".to_string()));
        assert!(parsed.exclude_terms.contains(&"exclude".to_string()));
        assert!(parsed.phrase_searches.contains(&"phrase search".to_string()));
    }

    #[test]
    fn test_search_query_to_fts() {
        let query = "test \"hello world\"";
        let parsed = SearchQuery::parse(query);
        let fts = parsed.to_fts_query();
        
        assert!(fts.contains("\"test\""));
        assert!(fts.contains("\"hello world\""));
    }

    #[test]
    fn test_column_validation() {
        let normalizer = FtsQueryNormalizer::new(Some(vec!["summary".to_string(), "description".to_string()]));
        
        // Valid column prefix should remain
        let query = "summary:test";
        let result = normalizer.prepare_fts_query(query);
        assert!(result.contains("summary:test"));
        
        // Invalid column prefix should be removed
        let query2 = "invalid:hello";
        let result2 = normalizer.prepare_fts_query(query2);
        assert!(!result2.contains("invalid:"));
    }
}