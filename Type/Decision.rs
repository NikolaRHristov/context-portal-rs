// Decision type for the ConPort MCP server
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
	pub Id:i64,
	pub WorkspaceId:String,
	pub Timestamp:String,
	pub Summary:String,
	pub Rationale:Option<String>,
	pub ImplementationDetails:Option<String>,
	pub Tags:Option<String>,
	pub CreatedAt:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
	pub id:String,
	pub workspace_id:String,
	pub timestamp:String,
	pub summary:String,
	pub rationale:Option<String>,
	pub implementation_details:Option<String>,
	pub tags:Option<Vec<String>>,
	pub created_at:String,
}

impl From<Decision> for DecisionResponse {
	fn from(d:Decision) -> Self {
		let tags:Option<Vec<String>> = d.Tags.as_ref().and_then(|t| serde_json::from_str(t).ok());
		Self {
			id:d.Id.to_string(),
			workspace_id:d.WorkspaceId,
			timestamp:d.Timestamp,
			summary:d.Summary,
			rationale:d.Rationale,
			implementation_details:d.ImplementationDetails,
			tags,
			created_at:d.CreatedAt,
		}
	}
}

/// Arguments for logging a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDecisionArgs {
	pub workspace_id:String,
	pub summary:String,
	pub rationale:Option<String>,
	pub implementation_details:Option<String>,
	pub tags:Option<Vec<String>>,
}

impl LogDecisionArgs {
	pub fn validate(&self) -> Result<(), String> {
		if self.summary.trim().is_empty() {
			return Err("Summary is required".to_string());
		}
		Ok(())
	}
}

/// Arguments for getting decisions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetDecisionsArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub limit:Option<i64>,
}

impl GetDecisionsArgs {
	pub fn validate(&self) -> Result<(), String> { Ok(()) }
}

/// Arguments for updating a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDecisionArgs {
	pub workspace_id:String,
	pub decision_id:i64,
	pub summary:Option<String>,
	pub rationale:Option<String>,
	pub implementation_details:Option<String>,
	pub tags:Option<Vec<String>>,
}

impl UpdateDecisionArgs {
	pub fn validate(&self) -> Result<(), String> { Ok(()) }
}

/// Arguments for deleting a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDecisionArgs {
	pub workspace_id:String,
	pub decision_id:i64,
}

impl DeleteDecisionArgs {
	pub fn validate(&self) -> Result<(), String> { Ok(()) }
}

/// Arguments for searching decisions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchDecisionsArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub query:Option<String>,
	#[serde(default)]
	pub tags_filter_include_any:Option<Vec<String>>,
	#[serde(default)]
	pub tags_filter_include_all:Option<Vec<String>>,
	#[serde(default)]
	pub limit:Option<i64>,
	#[serde(default)]
	pub offset:Option<i64>,
}

impl SearchDecisionsArgs {
	pub fn validate(&self) -> Result<(), String> {
		let has_include_all = self.tags_filter_include_all.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
		let has_include_any = self.tags_filter_include_any.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

		if has_include_all && has_include_any {
			return Err("Cannot use both tags_filter_include_all and tags_filter_include_any".to_string());
		}

		Ok(())
	}
}

impl Decision {
	pub fn New(
		workspace_id:String,
		summary:String,
		rationale:Option<String>,
		implementation_details:Option<String>,
		tags:Option<Vec<String>>,
	) -> Self {
		let timestamp = chrono::Utc::now().to_rfc3339();
		let tags_json = tags
			.as_ref()
			.map(|t| serde_json::to_string(t).unwrap_or_default())
			.unwrap_or_default();

		Self {
			Id:0, // Will be set by database
			WorkspaceId:workspace_id,
			Timestamp:timestamp.clone(),
			Summary:summary,
			Rationale:rationale,
			ImplementationDetails:implementation_details,
			Tags:Some(tags_json),
			CreatedAt:timestamp,
		}
	}
}
