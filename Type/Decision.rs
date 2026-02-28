// Decision type for the ConPort MCP server
pub struct Decision {
    pub Id: String,
    pub ContextId: String,
    pub Description: String,
    pub Rationale: String,
    pub CreatedAt: String,
}

impl Decision {
    pub fn New(Id: String, ContextId: String, Description: String, Rationale: String) -> Self {
        Self {
            Id,
            ContextId,
            Description,
            Rationale,
            CreatedAt: chrono::Utc::now().to_rfc3339(),
        }
    }
}