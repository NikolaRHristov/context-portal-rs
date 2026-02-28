// CustomData HTTP handler for the ConPort MCP server
use crate::HTTP::Protocol::Response::Error;
use crate::Type::CustomData::CustomData as CustomDataType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct CustomDataResponse {
    pub Id: String,
    pub Key: String,
    pub Value: String,
    pub ContextId: String,
    pub CreatedAt: String,
}

impl From<CustomDataType> for CustomDataResponse {
    fn from(data: CustomDataType) -> Self {
        Self {
            Id: data.Id,
            Key: data.Key,
            Value: data.Value,
            ContextId: data.ContextId,
            CreatedAt: data.CreatedAt,
        }
    }
}

#[derive(Deserialize)]
pub struct SetCustomDataRequest {
    pub Key: String,
    pub Value: String,
    pub ContextId: String,
}

#[derive(Deserialize)]
pub struct GetCustomDataRequest {
    pub Key: String,
    pub ContextId: String,
}

// HandleSetCustomData - Standalone handler function for MCP protocol
pub struct HandleSetCustomData;

impl HandleSetCustomData {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        Payload: SetCustomDataRequest,
    ) -> Result<CustomDataResponse, Error> {
        let _ = DbState; // Suppress unused warning
        let CustomData = CustomDataType::New(
            uuid::Uuid::new_v4().to_string(),
            Payload.Key,
            Payload.Value,
            Payload.ContextId,
        );
        
        // TODO: Persist to database
        
        Ok(CustomDataResponse::from(CustomData))
    }
}

// HandleGetCustomData - Standalone handler function for MCP protocol
pub struct HandleGetCustomData;

impl HandleGetCustomData {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        ContextId: String,
        Key: String,
    ) -> Result<CustomDataResponse, Error> {
        let _ = DbState; // Suppress unused warning
        // TODO: Query from database
        Err(Error::NotFound(&format!("CustomData {} not found in context {}", Key, ContextId)))
    }
}

// HandleDeleteCustomData - Standalone handler function for MCP protocol
pub struct HandleDeleteCustomData;

impl HandleDeleteCustomData {
    pub async fn Execute(
        DbState: &Arc<crate::Persistence::Database::Connect::Connect>,
        ContextId: String,
        Key: String,
    ) -> Result<(), Error> {
        let _ = DbState; // Suppress unused warning
        // TODO: Delete from database
        Err(Error::NotFound(&format!("CustomData {} not found in context {}", Key, ContextId)))
    }
}