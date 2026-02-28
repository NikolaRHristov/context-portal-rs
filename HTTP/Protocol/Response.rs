// MCP response types for the ConPort MCP server
use crate::HTTP::Protocol::Request::RequestId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    
    // Custom error codes
    DatabaseError = -32000,
    NotFound = -32001,
    Unauthorized = -32002,
    InvalidInput = -32003,
}

impl ErrorCode {
    pub fn Code(self) -> i32 {
        self as i32
    }

    pub fn Message(&self) -> &str {
        match self {
            ErrorCode::ParseError => "Parse error",
            ErrorCode::InvalidRequest => "Invalid Request",
            ErrorCode::MethodNotFound => "Method not found",
            ErrorCode::InvalidParams => "Invalid params",
            ErrorCode::InternalError => "Internal error",
            ErrorCode::DatabaseError => "Database error",
            ErrorCode::NotFound => "Not found",
            ErrorCode::Unauthorized => "Unauthorized",
            ErrorCode::InvalidInput => "Invalid input",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn New(Code: ErrorCode) -> Self {
        Self {
            code: Code as i32,
            message: Code.Message().to_string(),
            data: None,
        }
    }

    pub fn WithMessage(Code: ErrorCode, Message: &str) -> Self {
        Self {
            code: Code as i32,
            message: Message.to_string(),
            data: None,
        }
    }

    pub fn WithData(Code: ErrorCode, Message: &str, Data: serde_json::Value) -> Self {
        Self {
            code: Code as i32,
            message: Message.to_string(),
            data: Some(Data),
        }
    }

    pub fn ParseError(Message: &str) -> Self {
        Self::WithMessage(ErrorCode::ParseError, Message)
    }

    pub fn InvalidRequest(Message: &str) -> Self {
        Self::WithMessage(ErrorCode::InvalidRequest, Message)
    }

    pub fn MethodNotFound(Method: &str) -> Self {
        Self::WithMessage(ErrorCode::MethodNotFound, &format!("Method not found: {}", Method))
    }

    pub fn InvalidParams(Message: &str) -> Self {
        Self::WithMessage(ErrorCode::InvalidParams, Message)
    }

    pub fn InternalError(Message: &str) -> Self {
        Self::WithMessage(ErrorCode::InternalError, Message)
    }

    pub fn DatabaseError(Message: &str) -> Self {
        Self::WithMessage(ErrorCode::DatabaseError, Message)
    }

    pub fn NotFound(Resource: &str) -> Self {
        Self::WithMessage(ErrorCode::NotFound, &format!("Not found: {}", Resource))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl ResponseResult {
    pub fn Success(Result: serde_json::Value) -> Self {
        Self {
            result: Some(Result),
            error: None,
        }
    }

    pub fn Error(Error: Error) -> Self {
        Self {
            result: None,
            error: Some(Error),
        }
    }

    pub fn IsSuccess(&self) -> bool {
        self.error.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub enum Response {
    #[serde(rename = "2.0")]
    JsonRpc {
        id: RequestId,
        #[serde(flatten)]
        result: ResponseResult,
    },
}

impl Response {
    pub fn Success(Id: RequestId, Result: serde_json::Value) -> Self {
        Response::JsonRpc {
            id: Id,
            result: ResponseResult::Success(Result),
        }
    }

    pub fn Error(Id: RequestId, Error: Error) -> Self {
        Response::JsonRpc {
            id: Id,
            result: ResponseResult::Error(Error),
        }
    }

    pub fn ParseError(Id: RequestId, Message: &str) -> Self {
        Self::Error(Id, Error::ParseError(Message))
    }

    pub fn InvalidRequest(Id: RequestId, Message: &str) -> Self {
        Self::Error(Id, Error::InvalidRequest(Message))
    }

    pub fn MethodNotFound(Id: RequestId, Method: &str) -> Self {
        Self::Error(Id, Error::MethodNotFound(Method))
    }

    pub fn InvalidParams(Id: RequestId, Message: &str) -> Self {
        Self::Error(Id, Error::InvalidParams(Message))
    }

    pub fn InternalError(Id: RequestId, Message: &str) -> Self {
        Self::Error(Id, Error::InternalError(Message))
    }
}