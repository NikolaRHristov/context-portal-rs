// Progress type for the ConPort MCP server
pub struct Progress {
    pub Id: String,
    pub ContextId: String,
    pub Description: String,
    pub Status: String,
    pub CreatedAt: String,
    pub UpdatedAt: String,
}

impl Progress {
    pub fn New(Id: String, ContextId: String, Description: String, Status: String) -> Self {
        Self {
            Id,
            ContextId,
            Description,
            Status,
            CreatedAt: chrono::Utc::now().to_rfc3339(),
            UpdatedAt: chrono::Utc::now().to_rfc3339(),
        }
    }
}