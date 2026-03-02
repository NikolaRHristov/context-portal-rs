// CLI argument handling for the ConPort MCP server
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "conport-mcp")]
#[command(about = "ConPort MCP Server - Context Portal for AI Assistants")]
#[command(long_about = None)]
pub struct CLI {
	/// Server host address
	#[arg(long, default_value = "127.0.0.1")]
	pub Host:String,

	/// Server port number
	#[arg(long, default_value = "8080")]
	pub Port:u16,

	/// Workspace ID for context isolation
	#[arg(long)]
	pub WorkspaceId:Option<String>,

	/// Server mode (stdio or http)
	#[arg(long, default_value = "stdio")]
	pub Mode:ServerMode,

	/// Logging level (trace, debug, info, warn, error)
	#[arg(long, default_value = "info")]
	pub LogLevel:LogLevel,

	/// Log file path (optional, logs to stdout if not specified)
	#[arg(long)]
	pub LogFile:Option<String>,

	/// Database path override
	#[arg(long)]
	pub DatabasePath:Option<String>,

	/// Enable vector search functionality
	#[arg(long, default_value = "false")]
	pub VectorEnabled:bool,

	/// Embedding dimension for vector storage
	#[arg(long, default_value = "384")]
	pub EmbeddingDimension:usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServerMode {
	Stdio,
	Http,
}

impl Default for ServerMode {
	fn default() -> Self { ServerMode::Stdio }
}

impl std::str::FromStr for ServerMode {
	type Err = String;

	fn from_str(s:&str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"stdio" => Ok(ServerMode::Stdio),
			"http" => Ok(ServerMode::Http),
			_ => Err(format!("Invalid server mode: {}", s)),
		}
	}
}

impl std::fmt::Display for ServerMode {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ServerMode::Stdio => write!(f, "stdio"),
			ServerMode::Http => write!(f, "http"),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Default for LogLevel {
	fn default() -> Self { LogLevel::Info }
}

impl std::str::FromStr for LogLevel {
	type Err = String;

	fn from_str(s:&str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"trace" => Ok(LogLevel::Trace),
			"debug" => Ok(LogLevel::Debug),
			"info" => Ok(LogLevel::Info),
			"warn" => Ok(LogLevel::Warn),
			"error" => Ok(LogLevel::Error),
			_ => Err(format!("Invalid log level: {}", s)),
		}
	}
}

impl LogLevel {
	pub fn ToTracingLevel(&self) -> tracing::Level {
		match self {
			LogLevel::Trace => tracing::Level::TRACE,
			LogLevel::Debug => tracing::Level::DEBUG,
			LogLevel::Info => tracing::Level::INFO,
			LogLevel::Warn => tracing::Level::WARN,
			LogLevel::Error => tracing::Level::ERROR,
		}
	}
}

impl CLI {
	pub fn Parse() -> Self { clap::Parser::parse() }

	pub fn ServerAddress(&self) -> String { format!("{}:{}", self.Host, self.Port) }
}
