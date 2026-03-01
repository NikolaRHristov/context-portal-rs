// Pattern type for the ConPort MCP server
pub struct Pattern {
	pub Id:String,
	pub Name:String,
	pub Description:String,
	pub ContextId:String,
	pub CreatedAt:String,
}

impl Pattern {
	pub fn New(Id:String, Name:String, Description:String, ContextId:String) -> Self {
		Self { Id, Name, Description, ContextId, CreatedAt:chrono::Utc::now().to_rfc3339() }
	}
}
