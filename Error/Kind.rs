// Error kinds for the ConPort MCP server
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Kind {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Workspace detection error: {0}")]
    WorkspaceDetection(String),
    
    #[error("Vector store error: {0}")]
    VectorStore(String),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("Embedding error: {0}")]
    Embedding(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}