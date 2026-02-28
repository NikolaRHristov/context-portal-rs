// CustomData type for the ConPort MCP server
pub struct CustomData {
    pub Id: String,
    pub Key: String,
    pub Value: String,
    pub ContextId: String,
    pub CreatedAt: String,
}

impl CustomData {
    pub fn New(Id: String, Key: String, Value: String, ContextId: String) -> Self {
        Self {
            Id,
            Key,
            Value,
            ContextId,
            CreatedAt: chrono::Utc::now().to_rfc3339(),
        }
    }
}