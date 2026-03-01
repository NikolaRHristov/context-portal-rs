// Import/Export HTTP handlers for the ConPort MCP server
// Provides handlers for exporting and importing ConPort data as markdown

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::HTTP::Protocol::Response::Error;

/// Request for exporting data to markdown
#[derive(Debug, Deserialize)]
pub struct ExportMarkdownRequest {
	pub WorkspaceId:String,
	pub ItemTypes:Vec<String>,
	pub OutputPath:Option<String>,
	pub IncludeMetadata:Option<bool>,
}

/// Response for export operation
#[derive(Debug, Serialize)]
pub struct ExportMarkdownResponse {
	pub Success:bool,
	pub FilesCreated:Vec<String>,
	pub TotalItems:usize,
	pub OutputDirectory:String,
	pub Message:String,
}

/// Request for importing data from markdown
#[derive(Debug, Deserialize)]
pub struct ImportMarkdownRequest {
	pub WorkspaceId:String,
	pub InputPath:String,
	pub MergeStrategy:Option<String>, // "merge", "replace", "skip_existing"
	pub ValidateOnly:Option<bool>,
}

/// Response for import operation
#[derive(Debug, Serialize)]
pub struct ImportMarkdownResponse {
	pub Success:bool,
	pub ItemsImported:usize,
	pub ItemsSkipped:usize,
	pub ItemsFailed:usize,
	pub Errors:Vec<String>,
	pub Message:String,
}

/// Markdown file entry for import/export
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownEntry {
	pub ItemType:String,
	pub ItemId:String,
	pub Frontmatter:serde_json::Value,
	pub Content:String,
}

/// HandleExportMarkdown - Export ConPort data as markdown files
pub struct HandleExportMarkdown;

impl HandleExportMarkdown {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		Payload:ExportMarkdownRequest,
	) -> Result<ExportMarkdownResponse, Error> {
		let IncludeMetadata = Payload.IncludeMetadata.unwrap_or(true);
		let OutputPath = Payload
			.OutputPath
			.unwrap_or_else(|| format!("./conport_export_{}", Payload.WorkspaceId));

		// Create output directory
		let OutputDir = std::path::Path::new(&OutputPath);
		if !OutputDir.exists() {
			std::fs::create_dir_all(OutputDir).map_err(|e| {
				Error::WithMessage(
					crate::HTTP::Protocol::Response::ErrorCode::InternalError,
					&format!("Failed to create output directory: {}", e),
				)
			})?;
		}

		let mut FilesCreated = Vec::new();
		let mut TotalItems = 0;

		// Export each item type
		for ItemType in &Payload.ItemTypes {
			let (Items, Filename) =
				Self::ExportItemType(DbState, VectorStore, &Payload.WorkspaceId, ItemType, IncludeMetadata)
					.await
					.map_err(|e| {
						Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string())
					})?;

			if !Items.is_empty() {
				let FilePath = OutputDir.join(&Filename);
				let Content = Self::FormatMarkdown(&Items);

				std::fs::write(&FilePath, &Content).map_err(|e| {
					Error::WithMessage(
						crate::HTTP::Protocol::Response::ErrorCode::InternalError,
						&format!("Failed to write file: {}", e),
					)
				})?;

				FilesCreated.push(Filename);
				TotalItems += Items.len();
			}
		}

		Ok(ExportMarkdownResponse {
			Success:true,
			FilesCreated,
			TotalItems,
			OutputDirectory:OutputPath,
			Message:format!("Successfully exported {} items", TotalItems),
		})
	}

	/// Export items of a specific type
	async fn ExportItemType(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		WorkspaceId:&str,
		ItemType:&str,
		IncludeMetadata:bool,
	) -> Result<(Vec<MarkdownEntry>, String), crate::Error::Kind::Kind> {
		let _ = DbState;
		let _ = VectorStore;
		let _ = WorkspaceId;
		let _ = ItemType;
		let _ = IncludeMetadata;

		// Determine filename based on item type
		let Filename = format!("{}.md", ItemType);

		// TODO: Implement actual export logic
		tracing::debug!("Exporting {} to {}", ItemType, Filename);

		Ok((vec![], Filename))
	}

	/// Format items as markdown
	fn FormatMarkdown(Items:&[MarkdownEntry]) -> String {
		let mut Output = String::new();

		for Item in Items {
			Output.push_str(&format!("---\n"));
			Output.push_str(&format!("item_type: {}\n", Item.ItemType));
			Output.push_str(&format!("item_id: {}\n", Item.ItemId));

			if let Ok(Frontmatter) = serde_yaml::to_string(&Item.Frontmatter) {
				Output.push_str(&Frontmatter);
			}

			Output.push_str("---\n\n");
			Output.push_str(&Item.Content);
			Output.push_str("\n\n");
		}

		Output
	}
}

/// HandleImportMarkdown - Import ConPort data from markdown files
pub struct HandleImportMarkdown;

impl HandleImportMarkdown {
	pub async fn Execute(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		Payload:ImportMarkdownRequest,
	) -> Result<ImportMarkdownResponse, Error> {
		let InputPath = std::path::Path::new(&Payload.InputPath);

		if !InputPath.exists() {
			return Err(Error::WithMessage(
				crate::HTTP::Protocol::Response::ErrorCode::BadRequest,
				&format!("Input path does not exist: {}", Payload.InputPath),
			));
		}

		let MergeStrategy = Payload.MergeStrategy.unwrap_or_else(|| "merge".to_string());
		let ValidateOnly = Payload.ValidateOnly.unwrap_or(false);

		// Find all markdown files
		let MarkdownFiles = Self::FindMarkdownFiles(InputPath).map_err(|e| {
			Error::WithMessage(crate::HTTP::Protocol::Response::ErrorCode::InternalError, &e.to_string())
		})?;

		let mut ItemsImported = 0;
		let mut ItemsSkipped = 0;
		let mut ItemsFailed = 0;
		let mut Errors = Vec::new();

		// Process each file
		for File in &MarkdownFiles {
			match Self::ProcessMarkdownFile(
				DbState,
				VectorStore,
				&Payload.WorkspaceId,
				File,
				&MergeStrategy,
				ValidateOnly,
			)
			.await
			{
				Ok((Imported, Skipped, Failed)) => {
					ItemsImported += Imported;
					ItemsSkipped += Skipped;
					ItemsFailed += Failed;
				},
				Err(e) => {
					Errors.push(format!("{}: {}", File.display(), e));
					ItemsFailed += 1;
				},
			}
		}

		Ok(ImportMarkdownResponse {
			Success:ItemsFailed == 0,
			ItemsImported,
			ItemsSkipped,
			ItemsFailed,
			Errors,
			Message:if ItemsFailed == 0 {
				format!("Successfully imported {} items", ItemsImported)
			} else {
				format!("Import completed with {} errors", ItemsFailed)
			},
		})
	}

	/// Find all markdown files in directory
	fn FindMarkdownFiles(Path:&std::path::Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
		let mut Files = Vec::new();

		if Path.is_dir() {
			for Entry in std::fs::read_dir(Path)? {
				let Entry = Entry?;
				let Path = Entry.path();
				if Path.is_file() && Path.extension().map_or(false, |ext| ext == "md") {
					Files.push(Path);
				} else if Path.is_dir() {
					Files.extend(Self::FindMarkdownFiles(&Path)?);
				}
			}
		} else if Path.is_file() && Path.extension().map_or(false, |ext| ext == "md") {
			Files.push(Path.to_path_buf());
		}

		Ok(Files)
	}

	/// Process a single markdown file
	async fn ProcessMarkdownFile(
		DbState:&Arc<crate::Persistence::Database::Connect::Connect>,
		VectorStore:&Arc<tokio::sync::Mutex<crate::Persistence::Vector::Usearch::Store>>,
		WorkspaceId:&str,
		FilePath:&std::path::Path,
		MergeStrategy:&str,
		ValidateOnly:bool,
	) -> Result<(usize, usize, usize), crate::Error::Kind::Kind> {
		let _ = DbState;
		let _ = VectorStore;
		let _ = WorkspaceId;
		let _ = FilePath;
		let _ = MergeStrategy;
		let _ = ValidateOnly;

		// TODO: Implement actual import logic
		tracing::debug!("Processing markdown file: {}", FilePath.display());

		Ok((0, 0, 0))
	}
}

// ============================================================================
// MCP Tool Registrations
// ============================================================================

/// Get the tool definitions for import/export handlers
pub fn GetTools() -> Vec<crate::HTTP::Protocol::Request::Tool> {
	vec![
		crate::HTTP::Protocol::Request::Tool {
			Name:"export_markdown".to_string(),
			Description:"Export ConPort data to markdown files".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"item_types": {"type": "array", "items": {"type": "string"}},
					"output_path": {"type": "string"},
					"include_metadata": {"type": "boolean"}
				},
				"required": ["workspace_id", "item_types"]
			}),
		},
		crate::HTTP::Protocol::Request::Tool {
			Name:"import_markdown".to_string(),
			Description:"Import ConPort data from markdown files".to_string(),
			InputSchema:serde_json::json!({
				"type": "object",
				"properties": {
					"workspace_id": {"type": "string"},
					"input_path": {"type": "string"},
					"merge_strategy": {"type": "string", "enum": ["merge", "replace", "skip_existing"]},
					"validate_only": {"type": "boolean"}
				},
				"required": ["workspace_id", "input_path"]
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
	fn test_export_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "item_types": ["decision", "progress_entry"],
            "output_path": "./export",
            "include_metadata": true
        }"#;
		let Request:ExportMarkdownRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.WorkspaceId, "test");
		assert_eq!(Request.ItemTypes.len(), 2);
	}

	#[test]
	fn test_import_request_parsing() {
		let Json = r#"{
            "workspace_id": "test",
            "input_path": "./import",
            "merge_strategy": "merge"
        }"#;
		let Request:ImportMarkdownRequest = serde_json::from_str(Json).unwrap();
		assert_eq!(Request.WorkspaceId, "test");
		assert_eq!(Request.MergeStrategy, Some("merge".to_string()));
	}

	#[test]
	fn test_markdown_entry_serialization() {
		let Entry = MarkdownEntry {
			ItemType:"decision".to_string(),
			ItemId:"dec-1".to_string(),
			Frontmatter:serde_json::json!({"title": "Test Decision"}),
			Content:"This is the decision content".to_string(),
		};

		let Json = serde_json::to_string(&Entry).unwrap();
		let Parsed:MarkdownEntry = serde_json::from_str(&Json).unwrap();

		assert_eq!(Parsed.ItemType, "decision");
		assert_eq!(Parsed.ItemId, "dec-1");
	}

	// ============================================================================
	// HTTP Handlers (axum-compatible)
	// ============================================================================

	/// Export ConPort data to markdown files
	pub async fn ExportMarkdown(
		State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
		Json(payload):Json<ExportMarkdownRequest>,
	) -> Result<Json<ExportMarkdownResponse>, crate::Error::Kind::Kind> {
		// TODO: Implement actual markdown export
		let output_path = payload
			.OutputPath
			.unwrap_or_else(|| format!("./conport_export_{}", payload.WorkspaceId));
		Ok(Json(ExportMarkdownResponse {
			Success:true,
			FilesCreated:vec![],
			TotalItems:0,
			OutputDirectory:output_path,
			Message:"Export completed successfully".to_string(),
		}))
	}

	/// Import ConPort data from markdown files
	pub async fn ImportMarkdown(
		State(_state):State<Arc<crate::Persistence::Database::Connect::Connect>>,
		Json(payload):Json<ImportMarkdownRequest>,
	) -> Result<Json<ImportMarkdownResponse>, crate::Error::Kind::Kind> {
		// TODO: Implement actual markdown import
		Ok(Json(ImportMarkdownResponse {
			Success:true,
			ItemsImported:0,
			ItemsSkipped:0,
			ItemsFailed:0,
			Errors:vec![],
			Message:"Import completed successfully".to_string(),
		}))
	}
}
