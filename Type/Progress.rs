// Progress type for the ConPort MCP server
use serde::{Deserialize, Serialize};

/// Main Progress entry structure representing a task/progress item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
	pub Id:String,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Status:String,
	pub Description:String,
	pub ParentId:Option<i64>,
	pub CreatedAt:String,
	pub UpdatedAt:String,
}

impl Progress {
	pub fn New(WorkspaceId:String, Description:String, Status:String, ParentId:Option<i64>) -> Self {
		let now = chrono::Utc::now().to_rfc3339();
		Self {
			Id:uuid::Uuid::new_v4().to_string(),
			WorkspaceId,
			Timestamp:now.clone(),
			Status,
			Description,
			ParentId,
			CreatedAt:now.clone(),
			UpdatedAt:now,
		}
	}

	pub fn with_id(Id:String, WorkspaceId:String, Description:String, Status:String, ParentId:Option<i64>) -> Self {
		let now = chrono::Utc::now().to_rfc3339();
		Self {
			Id,
			WorkspaceId,
			Timestamp:now.clone(),
			Status,
			Description,
			ParentId,
			CreatedAt:now.clone(),
			UpdatedAt:now,
		}
	}
}

/// HTTP response type for Progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressResponse {
	pub id:String,
	pub workspace_id:String,
	pub timestamp:String,
	pub status:String,
	pub description:String,
	pub parent_id:Option<i64>,
	pub created_at:String,
	pub updated_at:String,
}

impl From<Progress> for ProgressResponse {
	fn from(p:Progress) -> Self {
		Self {
			id:p.Id,
			workspace_id:p.WorkspaceId,
			timestamp:p.Timestamp,
			status:p.Status,
			description:p.Description,
			parent_id:p.ParentId,
			created_at:p.CreatedAt,
			updated_at:p.UpdatedAt,
		}
	}
}

// ============================================================================
// Tool Argument Types
// ============================================================================

/// Arguments for logging/creating a new progress entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogProgressArgs {
	pub workspace_id:String,
	pub status:String,
	pub description:String,
	pub parent_id:Option<i64>,
	#[serde(default)]
	pub linked_item_type:Option<String>,
	#[serde(default)]
	pub linked_item_id:Option<String>,
	#[serde(default)]
	pub link_relationship_type:Option<String>,
}

impl LogProgressArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.status.is_empty() {
			return Err("status is required".to_string());
		}
		if self.description.is_empty() {
			return Err("description is required".to_string());
		}

		// Validate linked_item_type and linked_item_id must be provided together
		match (&self.linked_item_type, &self.linked_item_id) {
			(Some(_), None) | (None, Some(_)) => {
				Err("Both linked_item_type and linked_item_id must be provided together, or neither".to_string())
			},
			_ => Ok(()),
		}
	}
}

/// Arguments for retrieving progress entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProgressArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub status_filter:Option<String>,
	#[serde(default)]
	pub parent_id_filter:Option<i64>,
	#[serde(default)]
	pub limit:Option<i64>,
}

impl GetProgressArgs {
	pub fn limit_value(&self) -> i64 { self.limit.unwrap_or(50) }
}

/// Arguments for updating a progress entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressArgs {
	pub workspace_id:String,
	pub progress_id:i64,
	#[serde(default)]
	pub status:Option<String>,
	#[serde(default)]
	pub description:Option<String>,
	#[serde(default)]
	pub parent_id:Option<i64>,
}

impl UpdateProgressArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.progress_id < 1 {
			return Err("progress_id must be greater than or equal to 1".to_string());
		}

		// At least one field must be provided for update
		if self.status.is_none() && self.description.is_none() && self.parent_id.is_none() {
			return Err(
				"At least one field ('status', 'description', or 'parent_id') must be provided for update".to_string(),
			);
		}

		Ok(())
	}
}

/// Arguments for deleting a progress entry by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProgressByIdArgs {
	pub workspace_id:String,
	pub progress_id:i64,
}

impl DeleteProgressByIdArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.progress_id < 1 {
			return Err("progress_id must be greater than or equal to 1".to_string());
		}
		Ok(())
	}
}

/// Response type for update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressResponse {
	pub status:String,
	pub message:String,
}

/// Response type for delete operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProgressResponse {
	pub status:String,
	pub message:String,
}

// ============================================================================
// Valid Status Values
// ============================================================================

pub mod status {
	pub const TODO:&str = "TODO";
	pub const IN_PROGRESS:&str = "IN_PROGRESS";
	pub const DONE:&str = "DONE";
	pub const BLOCKED:&str = "BLOCKED";
	pub const CANCELLED:&str = "CANCELLED";

	pub fn all() -> Vec<&'static str> { vec![TODO, IN_PROGRESS, DONE, BLOCKED, CANCELLED] }

	pub fn is_valid(s:&str) -> bool { all().contains(&s) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_progress_creation() {
		let progress = Progress::New(
			"workspace1".to_string(),
			"Test task".to_string(),
			status::TODO.to_string(),
			None,
		);

		assert!(!progress.Id.is_empty());
		assert_eq!(progress.WorkspaceId, "workspace1");
		assert_eq!(progress.Description, "Test task");
		assert_eq!(progress.Status, "TODO");
		assert!(progress.ParentId.is_none());
	}

	#[test]
	fn test_log_progress_args_validation() {
		let valid_args = LogProgressArgs {
			workspace_id:"ws1".to_string(),
			status:"TODO".to_string(),
			description:"Test".to_string(),
			parent_id:None,
			linked_item_type:None,
			linked_item_id:None,
			link_relationship_type:None,
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = LogProgressArgs {
			workspace_id:"ws1".to_string(),
			status:"TODO".to_string(),
			description:"Test".to_string(),
			parent_id:None,
			linked_item_type:Some("decision".to_string()),
			linked_item_id:None, // Missing - should fail
			link_relationship_type:None,
		};
		assert!(invalid_args.validate().is_err());
	}

	#[test]
	fn test_update_progress_args_validation() {
		let valid_args = UpdateProgressArgs {
			workspace_id:"ws1".to_string(),
			progress_id:1,
			status:Some("DONE".to_string()),
			description:None,
			parent_id:None,
		};
		assert!(valid_args.validate().is_ok());

		let invalid_args = UpdateProgressArgs {
			workspace_id:"ws1".to_string(),
			progress_id:1,
			status:None,
			description:None,
			parent_id:None,
		};
		assert!(invalid_args.validate().is_err());
	}

	#[test]
	fn test_status_validation() {
		assert!(status::is_valid("TODO"));
		assert!(status::is_valid("IN_PROGRESS"));
		assert!(status::is_valid("DONE"));
		assert!(!status::is_valid("INVALID"));
	}
}
