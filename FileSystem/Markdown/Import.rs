// Markdown Import for ConPort MCP server
// Handles importing context, decisions, and progress from markdown format

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Merge strategy for importing data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// Merge with existing data (combine updates)
    Merge,
    /// Replace existing data with imported data
    Replace,
    /// Skip existing items, only add new ones
    SkipExisting,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::Merge
    }
}

/// Import options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub merge_strategy: MergeStrategy,
    pub workspace_id: String,
    pub preserve_timestamps: bool,
    pub validate_schema: bool,
}

impl ImportOptions {
    pub fn new(workspace_id: String) -> Self {
        Self {
            merge_strategy: MergeStrategy::default(),
            workspace_id,
            preserve_timestamps: false,
            validate_schema: true,
        }
    }

    pub fn with_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    pub fn with_timestamps(mut self, preserve: bool) -> Self {
        self.preserve_timestamps = preserve;
        self
    }

    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_schema = validate;
        self
    }
}

/// Parse result for imported items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult<T> {
    pub items: Vec<T>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl<T> ParseResult<T> {
    pub fn new() -> Self {
        Self {
            items: vec![],
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            items,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

// ============================================================================
// Context Parsing
// ============================================================================

/// Parsed context from markdown
#[derive(Debug, Clone)]
pub struct ParsedContext {
    pub name: String,
    pub content: String,
    pub workspace_path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Parse context from markdown content
pub fn ParseContext(markdown: &str, options: &ImportOptions) -> ParseResult<ParsedContext> {
    let mut result = ParseResult::new();
    
    // Extract title (first # heading)
    let title_re = Regex::new(r"^#\s+(.+)$").unwrap();
    let mut name = String::new();
    
    // Extract metadata section
    let metadata_re = Regex::new(r"##\s+Metadata\s*\n((?:.+\n)*)").unwrap();
    let mut workspace_path: Option<String> = None;
    let mut created_at: Option<String> = None;
    let mut updated_at: Option<String> = None;
    
    // Extract content section
    let content_re = Regex::new(r"##\s+Content\s*\n((?:[\s\S]*?))").unwrap();
    let mut content = String::new();

    for line in markdown.lines() {
        // Check for title
        if let Some(cap) = title_re.captures(line) {
            name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        }
        
        // Check for metadata fields
        if line.starts_with("- **Workspace**") {
            if let Some(path) = line.split('`').nth(1) {
                workspace_path = Some(path.to_string());
            }
        }
        if line.starts_with("- **Created**") {
            if let Some(ts) = line.split_whitespace().last() {
                created_at = Some(ts.to_string());
            }
        }
        if line.starts_with("- **Updated**") {
            if let Some(ts) = line.split_whitespace().last() {
                updated_at = Some(ts.to_string());
            }
        }
    }

    // Extract content after "## Content" header
    if let Some(cap) = content_re.captures(markdown) {
        content = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
    } else {
        // If no content section, use everything after title as content
        let parts: Vec<&str> = markdown.splitn(2, '\n').collect();
        if parts.len() > 1 {
            // Skip the title line and metadata
            let after_title = parts[1];
            // Remove metadata section if present
            if let Some(idx) = after_title.find("## Metadata") {
                content = after_title[idx..].lines()
                    .skip(2) // Skip "## Metadata" and following
                    .filter(|l| !l.starts_with("## "))
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                content = after_title.trim().to_string();
            }
        }
    }

    // Validate required fields
    if name.is_empty() {
        result.add_error("Context name is required".to_string());
    }
    if content.is_empty() {
        result.add_warning("Context content is empty".to_string());
    }

    if result.is_valid() {
        result.items.push(ParsedContext {
            name,
            content,
            workspace_path: options.workspace_id.clone().into(),
            created_at,
            updated_at,
        });
    }

    result
}

/// Parse multiple contexts from a single markdown document
pub fn ParseContexts(markdown: &str, options: &ImportOptions) -> ParseResult<ParsedContext> {
    let mut result = ParseResult::new();
    
    // Split by horizontal rules or top-level headings
    let sections: Vec<&str> = markdown.split("\n---\n").collect();
    
    for section in sections {
        let section_result = ParseContext(section, options);
        result.items.extend(section_result.items);
        result.errors.extend(section_result.errors);
        result.warnings.extend(section_result.warnings);
    }

    // If no sections were split, try parsing as single context
    if result.items.is_empty() && !markdown.trim().is_empty() {
        return ParseContext(markdown, options);
    }

    result
}

// ============================================================================
// Decision Parsing
// ============================================================================

/// Parsed decision from markdown
#[derive(Debug, Clone)]
pub struct ParsedDecision {
    pub summary: String,
    pub rationale: Option<String>,
    pub implementation_details: Option<String>,
    pub tags: Vec<String>,
    pub timestamp: Option<String>,
}

/// Parse decision from markdown content
pub fn ParseDecision(markdown: &str, _options: &ImportOptions) -> ParseResult<ParsedDecision> {
    let mut result = ParseResult::new();
    
    // Extract decision heading (## Decision: ...)
    let decision_re = Regex::new(r"^##\s+Decision:\s+(.+)$").unwrap();
    let mut summary = String::new();
    
    // Extract rationale section
    let rationale_re = Regex::new(r"###\s+Rationale\s*\n((?:[\s\S]*?)(?=###|\Z))").unwrap();
    let mut rationale: Option<String> = None;
    
    // Extract implementation section
    let impl_re = Regex::new(r"###\s+Implementation\s*\n((?:[\s\S]*?)(?=###|\Z))").unwrap();
    let mut implementation_details: Option<String> = None;
    
    // Extract tags from metadata
    let tags_re = Regex::new(r"-\s*\*\*Tags\*\*:\s*(.+)").unwrap();
    let mut tags: Vec<String> = vec![];
    
    // Extract date from metadata
    let date_re = Regex::new(r"-\s*\*\*Date\*\*:\s*(.+)").unwrap();
    let mut timestamp: Option<String> = None;

    for line in markdown.lines() {
        // Check for decision title
        if let Some(cap) = decision_re.captures(line) {
            summary = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        }
        
        // Check for tags
        if let Some(cap) = tags_re.captures(line) {
            if let Some(tags_str) = cap.get(1) {
                tags = tags_str.as_str()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        // Check for date
        if let Some(cap) = date_re.captures(line) {
            timestamp = cap.get(1).map(|m| m.as_str().to_string());
        }
    }

    // Extract rationale content
    if let Some(cap) = rationale_re.captures(markdown) {
        rationale = Some(cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default());
    }

    // Extract implementation content
    if let Some(cap) = impl_re.captures(markdown) {
        implementation_details = Some(cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default());
    }

    // Validate required fields
    if summary.is_empty() {
        result.add_error("Decision summary is required".to_string());
    }

    if result.is_valid() {
        result.items.push(ParsedDecision {
            summary,
            rationale,
            implementation_details,
            tags,
            timestamp,
        });
    }

    result
}

/// Parse multiple decisions from markdown
pub fn ParseDecisions(markdown: &str, options: &ImportOptions) -> ParseResult<ParsedDecision> {
    let mut result = ParseResult::new();
    
    // Split by horizontal rules or decision headings
    let sections: Vec<&str> = markdown
        .split("\n---\n")
        .filter(|s| s.contains("## Decision:"))
        .collect();
    
    for section in sections {
        let section_result = ParseDecision(section, options);
        result.items.extend(section_result.items);
        result.errors.extend(section_result.errors);
        result.warnings.extend(section_result.warnings);
    }

    // If no sections found, try whole document
    if result.items.is_empty() && markdown.contains("## Decision:") {
        return ParseDecision(markdown, options);
    }

    result
}

// ============================================================================
// Progress Parsing
// ============================================================================

/// Parsed progress item from markdown
#[derive(Debug, Clone)]
pub struct ParsedProgress {
    pub status: String,
    pub description: String,
    pub parent_id: Option<i64>,
    pub linked_item_type: Option<String>,
    pub linked_item_id: Option<String>,
    pub timestamp: Option<String>,
}

/// Parse progress item from markdown
pub fn ParseProgress(markdown: &str, _options: &ImportOptions) -> ParseResult<ParsedProgress> {
    let mut result = ParseResult::new();
    
    // Status patterns: 🔲 TODO, 🔄 IN_PROGRESS, ✅ DONE
    let status_re = Regex::new(r"([🔲🔄✅])\s+\*\*([A-Z_]+)\*\*\s+(.+)").unwrap();
    
    // Metadata patterns
    let parent_re = Regex::new(r"-\s*\*\*Parent ID\*\*:\s*`(\d+)`").unwrap();
    let linked_re = Regex::new(r"-\s*\*\*Linked\*\*:\s*(\w+)\s+`([^`]+)`").unwrap();
    let date_re = Regex::new(r"-\s*\*\*Date\*\*:\s*(.+)").unwrap();

    for line in markdown.lines() {
        if let Some(cap) = status_re.captures(line) {
            let status_emoji = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let status_text = cap.get(2).map(|m| m.as_str()).unwrap_or("TODO");
            let description = cap.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
            
            let status = match status_emoji {
                "🔲" => "TODO",
                "🔄" => "IN_PROGRESS",
                "✅" => "DONE",
                _ => status_text,
            }.to_string();

            let mut parent_id: Option<i64> = None;
            let mut linked_item_type: Option<String> = None;
            let mut linked_item_id: Option<String> = None;
            let mut timestamp: Option<String> = None;

            // Look for metadata in subsequent lines (indented)
            let lines: Vec<&str> = markdown.lines().collect();
            if let Some(idx) = lines.iter().position(|l| l.contains(&description)) {
                for j in (idx + 1)..lines.len() {
                    let meta_line = lines[j];
                    if !meta_line.starts_with("  -") {
                        break;
                    }
                    
                    if let Some(cap) = parent_re.captures(meta_line) {
                        parent_id = cap.get(1).and_then(|m| m.as_str().parse().ok());
                    }
                    if let Some(cap) = linked_re.captures(meta_line) {
                        linked_item_type = cap.get(1).map(|m| m.as_str().to_string());
                        linked_item_id = cap.get(2).map(|m| m.as_str().to_string());
                    }
                    if let Some(cap) = date_re.captures(meta_line) {
                        timestamp = cap.get(1).map(|m| m.as_str().to_string());
                    }
                }
            }

            result.items.push(ParsedProgress {
                status,
                description,
                parent_id,
                linked_item_type,
                linked_item_id,
                timestamp,
            });
        }
    }

    if result.items.is_empty() {
        result.add_warning("No progress items found in markdown".to_string());
    }

    result
}

/// Parse progress list from markdown
pub fn ParseProgressList(markdown: &str, options: &ImportOptions) -> ParseResult<ParsedProgress> {
    // Try to find progress sections and parse each
    let mut result = ParseResult::new();
    
    // Split by ## headings for each status section
    let sections: Vec<&str> = markdown.split("## ").filter(|s| {
        s.contains("TODO") || s.contains("In Progress") || s.contains("DONE")
    }).collect();
    
    for section in sections {
        let section_result = ParseProgress(&format!("## {}", section), options);
        result.items.extend(section_result.items);
        result.errors.extend(section_result.errors);
        result.warnings.extend(section_result.warnings);
    }

    // If no sections, try parsing whole document
    if result.items.is_empty() {
        return ParseProgress(markdown, options);
    }

    result
}

// ============================================================================
// Generic Import
// ============================================================================

/// Import data type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImportDataType {
    Context,
    Decision,
    Progress,
    Auto,
}

/// Auto-detect the data type from markdown content
pub fn detect_data_type(markdown: &str) -> ImportDataType {
    if markdown.contains("# ConPort Contexts Export") || markdown.contains("## Metadata") && markdown.contains("Workspace") {
        ImportDataType::Context
    } else if markdown.contains("# ConPort Decisions Export") || markdown.contains("## Decision:") {
        ImportDataType::Decision
    } else if markdown.contains("# ConPort Progress Export") || markdown.contains("TODO") || markdown.contains("IN_PROGRESS") {
        ImportDataType::Progress
    } else {
        ImportDataType::Auto
    }
}

/// Parse markdown based on detected or specified data type
pub fn ParseMarkdown(markdown: &str, data_type: ImportDataType, options: &ImportOptions) -> Result<ParseResult<()>, String> {
    let detected = if data_type == ImportDataType::Auto {
        detect_data_type(markdown)
    } else {
        data_type
    };

    match detected {
        ImportDataType::Context => {
            let result = ParseContexts(markdown, options);
            Ok(ParseResult {
                items: result.items.into_iter().map(|_| ()).collect(),
                errors: result.errors,
                warnings: result.warnings,
            })
        }
        ImportDataType::Decision => {
            let result = ParseDecisions(markdown, options);
            Ok(ParseResult {
                items: result.items.into_iter().map(|_| ()).collect(),
                errors: result.errors,
                warnings: result.warnings,
            })
        }
        ImportDataType::Progress => {
            let result = ParseProgressList(markdown, options);
            Ok(ParseResult {
                items: result.items.into_iter().map(|_| ()).collect(),
                errors: result.errors,
                warnings: result.warnings,
            })
        }
        ImportDataType::Auto => Err("Unable to auto-detect data type".to_string()),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_import_options_default() {
    let options = ImportOptions::new("/test/workspace".to_string());
    assert_eq!(options.merge_strategy, MergeStrategy::Merge);
    assert!(!options.preserve_timestamps);
    assert!(options.validate_schema);
}

#[test]
fn test_parse_context() {
    let markdown = r#"# Test Context

## Metadata

- **Workspace**: `/test/workspace`
- **Created**: 2024-01-01T00:00:00Z

## Content

This is the test content.
"#;

    let options = ImportOptions::new("/test/workspace".to_string());
    let result = ParseContext(markdown, &options);
    
    assert!(result.is_valid());
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].name, "Test Context");
    assert_eq!(result.items[0].content, "This is the test content.");
}

#[test]
fn test_parse_decision() {
    let markdown = r#"## Decision: Use SQLite

### Metadata

- **ID**: `1`
- **Date**: 2024-01-01T00:00:00Z
- **Tags**: database, storage

### Rationale

SQLite is simple and file-based.

### Implementation

Use rusqlite crate.
"#;

    let options = ImportOptions::new("/test/workspace".to_string());
    let result = ParseDecision(markdown, &options);
    
    assert!(result.is_valid());
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].summary, "Use SQLite");
    assert!(result.items[0].rationale.is_some());
    assert!(result.items[0].tags.contains(&"database".to_string()));
}

#[test]
fn test_parse_progress() {
    let markdown = r#"- 🔲 **TODO** Implement FTS search
  - **ID**: `1`
  - **Date**: 2024-01-01T00:00:00Z
"#;

    let options = ImportOptions::new("/test/workspace".to_string());
    let result = ParseProgress(markdown, &options);
    
    assert!(!result.items.is_empty());
    assert_eq!(result.items[0].status, "TODO");
    assert_eq!(result.items[0].description, "Implement FTS search");
}

#[test]
fn test_detect_data_type() {
    assert_eq!(detect_data_type("# ConPort Contexts Export"), ImportDataType::Context);
    assert_eq!(detect_data_type("# ConPort Decisions Export"), ImportDataType::Decision);
    assert_eq!(detect_data_type("# ConPort Progress Export"), ImportDataType::Progress);
}