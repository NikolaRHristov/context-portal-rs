// Context type for the ConPort MCP server
pub struct Context {
    pub Id: String,
    pub Name: String,
    pub Content: String,
    pub WorkspacePath: String,
    pub CreatedAt: String,
    pub UpdatedAt: String,
}

impl Context {
    pub fn New(Id: String, Name: String, Content: String, WorkspacePath: String) -> Self {
        Self {
            Id,
            Name,
            Content,
            WorkspacePath,
            CreatedAt: chrono::Utc::now().to_rfc3339(),
            UpdatedAt: chrono::Utc::now().to_rfc3339(),
        }
    }
}