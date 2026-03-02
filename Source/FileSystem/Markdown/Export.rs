// Markdown Export for ConPort MCP server
// Handles exporting context, decisions, and progress to markdown format

use serde::{Deserialize, Serialize};

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
	Markdown,
	Json,
	Yaml,
}

impl Default for ExportFormat {
	fn default() -> Self { ExportFormat::Markdown }
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
	pub include_metadata:bool,
	pub include_timestamps:bool,
	pub format:ExportFormat,
	pub include_tags:bool,
}

impl Default for ExportOptions {
	fn default() -> Self {
		Self {
			include_metadata:true,
			include_timestamps:true,
			format:ExportFormat::Markdown,
			include_tags:true,
		}
	}
}

impl ExportOptions {
	pub fn new() -> Self { Self::default() }

	pub fn with_format(mut self, format:ExportFormat) -> Self {
		self.format = format;
		self
	}

	pub fn with_metadata(mut self, include:bool) -> Self {
		self.include_metadata = include;
		self
	}

	pub fn with_timestamps(mut self, include:bool) -> Self {
		self.include_timestamps = include;
		self
	}

	pub fn with_tags(mut self, include:bool) -> Self {
		self.include_tags = include;
		self
	}
}

// ============================================================================
// Context Export
// ============================================================================

/// Context export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextExport {
	pub name:String,
	pub content:String,
	pub workspace_path:String,
	pub created_at:Option<String>,
	pub updated_at:Option<String>,
}

impl ContextExport {
	pub fn new(name:String, content:String, workspace_path:String) -> Self {
		Self { name, content, workspace_path, created_at:None, updated_at:None }
	}

	pub fn with_timestamps(mut self, created_at:String, updated_at:String) -> Self {
		self.created_at = Some(created_at);
		self.updated_at = Some(updated_at);
		self
	}
}

/// Export context to markdown format
pub fn ExportContext(context:&ContextExport, options:&ExportOptions) -> String {
	let mut output = String::new();

	// Header
	output.push_str(&format!("# {}\n\n", context.name));

	// Metadata section
	if options.include_metadata {
		output.push_str("## Metadata\n\n");
		output.push_str(&format!("- **Workspace**: `{}`\n", context.workspace_path));

		if options.include_timestamps {
			if let Some(created) = &context.created_at {
				output.push_str(&format!("- **Created**: {}\n", created));
			}
			if let Some(updated) = &context.updated_at {
				output.push_str(&format!("- **Updated**: {}\n", updated));
			}
		}
		output.push_str("\n");
	}

	// Content section
	output.push_str("## Content\n\n");
	output.push_str(&context.content);
	output.push_str("\n");

	output
}

/// Export multiple contexts to a single markdown document
pub fn ExportContexts(contexts:&[ContextExport], options:&ExportOptions) -> String {
	if contexts.is_empty() {
		return "# No Contexts Found\n\nNo contexts were found to export.\n".to_string();
	}

	let mut output = String::new();
	output.push_str("# ConPort Contexts Export\n\n");
	output.push_str(&format!("Exported {} context(s)\n\n", contexts.len()));

	for (i, context) in contexts.iter().enumerate() {
		if i > 0 {
			output.push_str("\n---\n\n");
		}
		output.push_str(&ExportContext(context, options));
	}

	output
}

// ============================================================================
// Decision Export
// ============================================================================

/// Decision export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionExport {
	pub id:String,
	pub summary:String,
	pub rationale:Option<String>,
	pub implementation_details:Option<String>,
	pub tags:Vec<String>,
	pub timestamp:Option<String>,
}

impl DecisionExport {
	pub fn new(id:String, summary:String) -> Self {
		Self {
			id,
			summary,
			rationale:None,
			implementation_details:None,
			tags:vec![],
			timestamp:None,
		}
	}

	pub fn with_rationale(mut self, rationale:String) -> Self {
		self.rationale = Some(rationale);
		self
	}

	pub fn with_implementation(mut self, details:String) -> Self {
		self.implementation_details = Some(details);
		self
	}

	pub fn with_tags(mut self, tags:Vec<String>) -> Self {
		self.tags = tags;
		self
	}

	pub fn with_timestamp(mut self, timestamp:String) -> Self {
		self.timestamp = Some(timestamp);
		self
	}
}

/// Export decision to markdown format
pub fn ExportDecision(decision:&DecisionExport, options:&ExportOptions) -> String {
	let mut output = String::new();

	// Header with ID
	output.push_str(&format!("## Decision: {}\n\n", decision.summary));

	// Metadata
	if options.include_metadata || options.include_timestamps || options.include_tags {
		output.push_str("### Metadata\n\n");

		output.push_str(&format!("- **ID**: `{}`\n", decision.id));

		if options.include_timestamps {
			if let Some(ts) = &decision.timestamp {
				output.push_str(&format!("- **Date**: {}\n", ts));
			}
		}

		if options.include_tags && !decision.tags.is_empty() {
			let tags_str = decision.tags.join(", ");
			output.push_str(&format!("- **Tags**: {}\n", tags_str));
		}
		output.push_str("\n");
	}

	// Rationale
	if let Some(rationale) = &decision.rationale {
		output.push_str("### Rationale\n\n");
		output.push_str(rationale);
		output.push_str("\n\n");
	}

	// Implementation Details
	if let Some(impl_details) = &decision.implementation_details {
		output.push_str("### Implementation\n\n");
		output.push_str(impl_details);
		output.push_str("\n");
	}

	output
}

/// Export multiple decisions to markdown format
pub fn ExportDecisions(decisions:&[DecisionExport], options:&ExportOptions) -> String {
	if decisions.is_empty() {
		return "# No Decisions Found\n\nNo decisions were found to export.\n".to_string();
	}

	let mut output = String::new();
	output.push_str("# ConPort Decisions Export\n\n");
	output.push_str(&format!("Exported {} decision(s)\n\n", decisions.len()));

	for decision in decisions {
		output.push_str(&ExportDecision(decision, options));
		output.push_str("\n---\n\n");
	}

	output
}

// ============================================================================
// Progress Export
// ============================================================================

/// Progress status enum for export
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
	Todo,
	InProgress,
	Done,
}

impl std::fmt::Display for ProgressStatus {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ProgressStatus::Todo => write!(f, "TODO"),
			ProgressStatus::InProgress => write!(f, "IN_PROGRESS"),
			ProgressStatus::Done => write!(f, "DONE"),
		}
	}
}

/// Progress export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressExport {
	pub id:String,
	pub status:ProgressStatus,
	pub description:String,
	pub parent_id:Option<i64>,
	pub linked_item_type:Option<String>,
	pub linked_item_id:Option<String>,
	pub timestamp:Option<String>,
}

impl ProgressExport {
	pub fn new(id:String, status:ProgressStatus, description:String) -> Self {
		Self {
			id,
			status,
			description,
			parent_id:None,
			linked_item_type:None,
			linked_item_id:None,
			timestamp:None,
		}
	}

	pub fn with_parent(mut self, parent_id:i64) -> Self {
		self.parent_id = Some(parent_id);
		self
	}

	pub fn with_link(mut self, item_type:String, item_id:String) -> Self {
		self.linked_item_type = Some(item_type);
		self.linked_item_id = Some(item_id);
		self
	}

	pub fn with_timestamp(mut self, timestamp:String) -> Self {
		self.timestamp = Some(timestamp);
		self
	}
}

/// Status badge for progress items
fn status_badge(status:&ProgressStatus) -> String {
	let (emoji, text) = match status {
		ProgressStatus::Todo => ("🔲", "TODO"),
		ProgressStatus::InProgress => ("🔄", "IN_PROGRESS"),
		ProgressStatus::Done => ("✅", "DONE"),
	};
	format!("{} **{}**", emoji, text)
}

/// Export progress to markdown format
pub fn ExportProgress(progress:&ProgressExport, options:&ExportOptions) -> String {
	let mut output = String::new();

	// Status and description
	output.push_str(&format!("- {} {}\n", status_badge(&progress.status), progress.description));

	// Metadata (indented)
	if options.include_metadata {
		output.push_str("  - **ID**: `");
		output.push_str(&progress.id);
		output.push_str("`\n");

		if let Some(parent) = progress.parent_id {
			output.push_str(&format!("  - **Parent ID**: `{}`\n", parent));
		}

		if let (Some(item_type), Some(item_id)) = (&progress.linked_item_type, &progress.linked_item_id) {
			output.push_str(&format!("  - **Linked**: {} `{}`\n", item_type, item_id));
		}

		if options.include_timestamps {
			if let Some(ts) = &progress.timestamp {
				output.push_str(&format!("  - **Date**: {}\n", ts));
			}
		}
	}

	output
}

/// Export multiple progress items to markdown format
pub fn ExportProgressList(progress_items:&[ProgressExport], options:&ExportOptions) -> String {
	if progress_items.is_empty() {
		return "# No Progress Found\n\nNo progress entries were found to export.\n".to_string();
	}

	let mut output = String::new();
	output.push_str("# ConPort Progress Export\n\n");
	output.push_str(&format!("Exported {} progress entry(s)\n\n", progress_items.len()));

	// Group by status
	let mut todo_items:Vec<&ProgressExport> = vec![];
	let mut in_progress_items:Vec<&ProgressExport> = vec![];
	let mut done_items:Vec<&ProgressExport> = vec![];

	for item in progress_items {
		match item.status {
			ProgressStatus::Todo => todo_items.push(item),
			ProgressStatus::InProgress => in_progress_items.push(item),
			ProgressStatus::Done => done_items.push(item),
		}
	}

	// TODO section
	if !todo_items.is_empty() {
		output.push_str("## 🔲 TODO\n\n");
		for item in todo_items {
			output.push_str(&ExportProgress(item, options));
		}
		output.push_str("\n");
	}

	// In Progress section
	if !in_progress_items.is_empty() {
		output.push_str("## 🔄 In Progress\n\n");
		for item in in_progress_items {
			output.push_str(&ExportProgress(item, options));
		}
		output.push_str("\n");
	}

	// Done section
	if !done_items.is_empty() {
		output.push_str("## ✅ Done\n\n");
		for item in done_items {
			output.push_str(&ExportProgress(item, options));
		}
	}

	output
}

// ============================================================================
// Generic Export
// ============================================================================

/// Export result containing the formatted output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
	pub content:String,
	pub item_count:usize,
	pub export_type:String,
}

impl ExportResult {
	pub fn new(content:String, item_count:usize, export_type:&str) -> Self {
		Self { content, item_count, export_type:to_string(export_type) }
	}
}

/// Convert string to owned string (helper for export type)
fn to_string(s:&str) -> String { s.to_string() }

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_export_options_default() {
		let options = ExportOptions::default();
		assert!(options.include_metadata);
		assert!(options.include_timestamps);
		assert!(options.include_tags);
		assert_eq!(options.format, ExportFormat::Markdown);
	}

	#[test]
	fn test_context_export() {
		let context = ContextExport::new(
			"Test Context".to_string(),
			"This is the content".to_string(),
			"/test/workspace".to_string(),
		);

		let options = ExportOptions::default();
		let output = ExportContext(&context, &options);

		assert!(output.contains("Test Context"));
		assert!(output.contains("This is the content"));
		assert!(output.contains("/test/workspace"));
	}

	#[test]
	fn test_decision_export() {
		let decision = DecisionExport::new("1".to_string(), "Use SQLite for storage".to_string())
			.with_rationale("SQLite is simple and file-based".to_string())
			.with_tags(vec!["database".to_string(), "storage".to_string()]);

		let options = ExportOptions::default();
		let output = ExportDecision(&decision, &options);

		assert!(output.contains("Use SQLite for storage"));
		assert!(output.contains("SQLite is simple"));
		assert!(output.contains("database"));
	}

	#[test]
	fn test_progress_export() {
		let progress = ProgressExport::new("1".to_string(), ProgressStatus::Todo, "Implement FTS search".to_string())
			.with_timestamp("2024-01-01T00:00:00Z".to_string());

		let options = ExportOptions::default();
		let output = ExportProgress(&progress, &options);

		assert!(output.contains("Implement FTS search"));
		assert!(output.contains("🔲"));
	}

	#[test]
	fn test_progress_grouping() {
		let items = vec![
			ProgressExport::new("1".to_string(), ProgressStatus::Todo, "Task 1".to_string()),
			ProgressExport::new("2".to_string(), ProgressStatus::Done, "Task 2".to_string()),
			ProgressExport::new("3".to_string(), ProgressStatus::Todo, "Task 3".to_string()),
		];

		let options = ExportOptions::default();
		let output = ExportProgressList(&items, &options);

		assert!(output.contains("TODO"));
		assert!(output.contains("DONE"));
	}
}
