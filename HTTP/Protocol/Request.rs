// MCP request types for the ConPort MCP server
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::Number(n) => write!(f, "{}", n),
            RequestId::String(s) => write!(f, "{}", s),
        }
    }
}

impl Default for RequestId {
    fn default() -> Self {
        RequestId::Number(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    // Context methods
    ContextList,
    ContextGet,
    ContextCreate,
    ContextUpdate,
    ContextDelete,
    
    // Decision methods
    DecisionList,
    DecisionGet,
    DecisionCreate,
    DecisionUpdate,
    DecisionDelete,
    
    // Progress methods
    ProgressList,
    ProgressGet,
    ProgressCreate,
    ProgressUpdate,
    ProgressDelete,
    
    // Pattern methods
    PatternList,
    PatternGet,
    PatternCreate,
    PatternUpdate,
    PatternDelete,
    
    // Custom data methods
    CustomDataList,
    CustomDataGet,
    CustomDataSet,
    CustomDataDelete,
    
    // Search methods
    SearchDecisions,
    SearchContext,
    
    // System methods
    Initialize,
    Shutdown,
    Tools,
    Resources,
    Ping,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::ContextList => write!(f, "context/list"),
            Method::ContextGet => write!(f, "context/get"),
            Method::ContextCreate => write!(f, "context/create"),
            Method::ContextUpdate => write!(f, "context/update"),
            Method::ContextDelete => write!(f, "context/delete"),
            Method::DecisionList => write!(f, "decision/list"),
            Method::DecisionGet => write!(f, "decision/get"),
            Method::DecisionCreate => write!(f, "decision/create"),
            Method::DecisionUpdate => write!(f, "decision/update"),
            Method::DecisionDelete => write!(f, "decision/delete"),
            Method::ProgressList => write!(f, "progress/list"),
            Method::ProgressGet => write!(f, "progress/get"),
            Method::ProgressCreate => write!(f, "progress/create"),
            Method::ProgressUpdate => write!(f, "progress/update"),
            Method::ProgressDelete => write!(f, "progress/delete"),
            Method::PatternList => write!(f, "pattern/list"),
            Method::PatternGet => write!(f, "pattern/get"),
            Method::PatternCreate => write!(f, "pattern/create"),
            Method::PatternUpdate => write!(f, "pattern/update"),
            Method::PatternDelete => write!(f, "pattern/delete"),
            Method::CustomDataList => write!(f, "custom_data/list"),
            Method::CustomDataGet => write!(f, "custom_data/get"),
            Method::CustomDataSet => write!(f, "custom_data/set"),
            Method::CustomDataDelete => write!(f, "custom_data/delete"),
            Method::SearchDecisions => write!(f, "search/decisions"),
            Method::SearchContext => write!(f, "search/context"),
            Method::Initialize => write!(f, "initialize"),
            Method::Shutdown => write!(f, "shutdown"),
            Method::Tools => write!(f, "tools"),
            Method::Resources => write!(f, "resources"),
            Method::Ping => write!(f, "ping"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc", content = "params")]
pub enum Request {
    #[serde(rename = "2.0")]
    JsonRpc(RequestData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestData {
    #[serde(default)]
    pub id: RequestId,
    pub method: Method,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

impl Request {
    pub fn Id(&self) -> &RequestId {
        match self {
            Request::JsonRpc(data) => &data.id,
        }
    }

    pub fn Method(&self) -> &Method {
        match self {
            Request::JsonRpc(data) => &data.method,
        }
    }

    pub fn Params(&self) -> Option<&serde_json::Value> {
        match self {
            Request::JsonRpc(data) => data.params.as_ref(),
        }
    }
}

impl RequestData {
    pub fn New(Method: Method) -> Self {
        Self {
            id: RequestId::default(),
            method: Method,
            params: None,
        }
    }

    pub fn WithId(Method: Method, Id: RequestId) -> Self {
        Self {
            id: Id,
            method: Method,
            params: None,
        }
    }

    pub fn WithParams(Method: Method, Params: serde_json::Value) -> Self {
        Self {
            id: RequestId::default(),
            method: Method,
            params: Some(Params),
        }
    }
}