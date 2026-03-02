// Common filter types for ConPort MCP server
use serde::{Deserialize, Serialize};

/// Tag filter for including ALL specified tags (AND logic)
/// Items must have ALL tags in the list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilterIncludeAll {
	pub tags:Vec<String>,
}

impl TagFilterIncludeAll {
	pub fn new(tags:Vec<String>) -> Self { Self { tags } }

	pub fn validate(&self) -> Result<(), String> {
		if self.tags.is_empty() {
			return Err("TagFilterIncludeAll requires at least one tag".to_string());
		}
		Ok(())
	}
}

/// Tag filter for including ANY specified tags (OR logic)
/// Items must have AT LEAST ONE tag in the list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilterIncludeAny {
	pub tags:Vec<String>,
}

impl TagFilterIncludeAny {
	pub fn new(tags:Vec<String>) -> Self { Self { tags } }

	pub fn validate(&self) -> Result<(), String> {
		if self.tags.is_empty() {
			return Err("TagFilterIncludeAny requires at least one tag".to_string());
		}
		Ok(())
	}
}

/// Mutually exclusive tag filter validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagFilter {
	IncludeAll(TagFilterIncludeAll),
	IncludeAny(TagFilterIncludeAny),
}

impl TagFilter {
	pub fn validate(&self) -> Result<(), String> {
		match self {
			TagFilter::IncludeAll(filter) => filter.validate(),
			TagFilter::IncludeAny(filter) => filter.validate(),
		}
	}

	pub fn tags(&self) -> &[String] {
		match self {
			TagFilter::IncludeAll(filter) => &filter.tags,
			TagFilter::IncludeAny(filter) => &filter.tags,
		}
	}
}

// ============================================================================
// Pagination
// ============================================================================

/// Pagination parameters for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
	pub offset:Option<usize>,
	pub limit:Option<usize>,
}

impl Default for Pagination {
	fn default() -> Self { Self { offset:Some(0), limit:Some(50) } }
}

impl Pagination {
	pub fn new(offset:Option<usize>, limit:Option<usize>) -> Self { Self { offset, limit } }

	pub fn with_limit(limit:usize) -> Self { Self { offset:Some(0), limit:Some(limit) } }

	pub fn offset(&self) -> usize { self.offset.unwrap_or(0) }

	pub fn limit(&self) -> usize { self.limit.unwrap_or(50) }
}

// ============================================================================
// Sorting
// ============================================================================

/// Sort direction for query results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
	Asc,
	Desc,
}

impl Default for SortDirection {
	fn default() -> Self { SortDirection::Desc }
}

/// Sort field for ordering results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
	Timestamp,
	CreatedAt,
	UpdatedAt,
	Id,
	Name,
	Status,
}

impl Default for SortField {
	fn default() -> Self { SortField::Timestamp }
}

/// Sorting parameters for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sorting {
	pub field:Option<SortField>,
	pub direction:Option<SortDirection>,
}

impl Default for Sorting {
	fn default() -> Self { Self { field:Some(SortField::default()), direction:Some(SortDirection::default()) } }
}

impl Sorting {
	pub fn new(field:Option<SortField>, direction:Option<SortDirection>) -> Self { Self { field, direction } }

	pub fn field(&self) -> SortField { self.field.clone().unwrap_or_default() }

	pub fn direction(&self) -> SortDirection { self.direction.clone().unwrap_or_default() }

	pub fn to_sql(&self) -> String {
		let field_str = match self.field() {
			SortField::Timestamp => "Timestamp",
			SortField::CreatedAt => "CreatedAt",
			SortField::UpdatedAt => "UpdatedAt",
			SortField::Id => "Id",
			SortField::Name => "Name",
			SortField::Status => "Status",
		};

		let direction_str = match self.direction() {
			SortDirection::Asc => "ASC",
			SortDirection::Desc => "DESC",
		};

		format!("{} {}", field_str, direction_str)
	}
}

// ============================================================================
// Combined Filter
// ============================================================================

/// Combined filter parameters for querying entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFilter {
	pub workspace_id:String,
	pub tag_filter:Option<TagFilter>,
	pub pagination:Option<Pagination>,
	pub sorting:Option<Sorting>,
	#[serde(default)]
	pub status_filter:Option<String>,
	#[serde(default)]
	pub parent_id_filter:Option<i64>,
}

impl EntityFilter {
	pub fn new(workspace_id:String) -> Self {
		Self {
			workspace_id,
			tag_filter:None,
			pagination:None,
			sorting:None,
			status_filter:None,
			parent_id_filter:None,
		}
	}

	pub fn with_tags(mut self, tag_filter:TagFilter) -> Self {
		self.tag_filter = Some(tag_filter);
		self
	}

	pub fn with_pagination(mut self, pagination:Pagination) -> Self {
		self.pagination = Some(pagination);
		self
	}

	pub fn with_sorting(mut self, sorting:Sorting) -> Self {
		self.sorting = Some(sorting);
		self
	}

	pub fn with_status(mut self, status:String) -> Self {
		self.status_filter = Some(status);
		self
	}

	pub fn with_parent_id(mut self, parent_id:i64) -> Self {
		self.parent_id_filter = Some(parent_id);
		self
	}
}

// ============================================================================
// Search/Filter Arguments
// ============================================================================

/// Arguments for searching/decisions filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDecisionsArgs {
	pub workspace_id:String,
	pub query:Option<String>,
	#[serde(default)]
	pub tags_filter_include_any:Option<Vec<String>>,
	#[serde(default)]
	pub tags_filter_include_all:Option<Vec<String>>,
	#[serde(default)]
	pub limit:Option<usize>,
	#[serde(default)]
	pub offset:Option<usize>,
}

impl SearchDecisionsArgs {
	pub fn validate(&self) -> Result<(), String> {
		// Check mutual exclusivity of tag filters
		let has_include_all = self.tags_filter_include_all.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
		let has_include_any = self.tags_filter_include_any.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

		if has_include_all && has_include_any {
			return Err("Cannot use both tags_filter_include_all and tags_filter_include_any".to_string());
		}

		Ok(())
	}
}

/// Arguments for filtering system patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSystemPatternsArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub tags_filter_include_any:Option<Vec<String>>,
	#[serde(default)]
	pub tags_filter_include_all:Option<Vec<String>>,
	#[serde(default)]
	pub limit:Option<usize>,
	#[serde(default)]
	pub offset:Option<usize>,
	#[serde(default)]
	pub name_filter:Option<String>,
}

impl FilterSystemPatternsArgs {
	pub fn validate(&self) -> Result<(), String> {
		// Check mutual exclusivity of tag filters
		let has_include_all = self.tags_filter_include_all.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
		let has_include_any = self.tags_filter_include_any.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

		if has_include_all && has_include_any {
			return Err("Cannot use both tags_filter_include_all and tags_filter_include_any".to_string());
		}

		Ok(())
	}
}

/// Arguments for filtering custom data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCustomDataArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub category:Option<String>,
	#[serde(default)]
	pub key_filter:Option<String>,
	#[serde(default)]
	pub limit:Option<usize>,
	#[serde(default)]
	pub offset:Option<usize>,
}

/// Arguments for filtering progress entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterProgressArgs {
	pub workspace_id:String,
	#[serde(default)]
	pub status_filter:Option<String>,
	#[serde(default)]
	pub parent_id_filter:Option<i64>,
	#[serde(default)]
	pub limit:Option<usize>,
	#[serde(default)]
	pub offset:Option<usize>,
}

impl FilterProgressArgs {
	pub fn new(workspace_id:String) -> Self {
		Self { workspace_id, status_filter:None, parent_id_filter:None, limit:None, offset:None }
	}
}
