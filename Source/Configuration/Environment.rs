// Environment variable handling for ConPort MCP server
use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Supported log levels for the application
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

impl LogLevel {
	pub fn from_str(s:&str) -> Self {
		match s.to_lowercase().as_str() {
			"trace" => LogLevel::Trace,
			"debug" => LogLevel::Debug,
			"info" => LogLevel::Info,
			"warn" | "warning" => LogLevel::Warn,
			"error" => LogLevel::Error,
			_ => LogLevel::default(),
		}
	}
}

/// Environment configuration for ConPort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
	pub workspace_id:Option<String>,
	pub log_level:LogLevel,
	pub database_path:Option<String>,
	pub port:Option<u16>,
	pub host:Option<String>,
	pub max_connections:Option<usize>,
	pub vector_store_enabled:bool,
	pub vector_store_path:Option<String>,
}

impl Default for EnvironmentConfig {
	fn default() -> Self {
		Self {
			workspace_id:None,
			log_level:LogLevel::default(),
			database_path:None,
			port:None,
			host:None,
			max_connections:None,
			vector_store_enabled:false,
			vector_store_path:None,
		}
	}
}

/// Load environment configuration from environment variables
pub fn LoadFromEnvironment() -> EnvironmentConfig {
	let mut config = EnvironmentConfig::default();

	// CONPORT_WORKSPACE - Workspace identifier
	if let Ok(workspace) = env::var("CONPORT_WORKSPACE") {
		if !workspace.is_empty() {
			config.workspace_id = Some(workspace);
		}
	}

	// CONPORT_LOG_LEVEL - Logging level
	if let Ok(log_level) = env::var("CONPORT_LOG_LEVEL") {
		config.log_level = LogLevel::from_str(&log_level);
	}

	// CONPORT_DATABASE_PATH - Custom database path
	if let Ok(db_path) = env::var("CONPORT_DATABASE_PATH") {
		if !db_path.is_empty() {
			config.database_path = Some(db_path);
		}
	}

	// CONPORT_PORT - Server port
	if let Ok(port) = env::var("CONPORT_PORT") {
		if let Ok(port_num) = port.parse::<u16>() {
			config.port = Some(port_num);
		}
	}

	// CONPORT_HOST - Server host
	if let Ok(host) = env::var("CONPORT_HOST") {
		if !host.is_empty() {
			config.host = Some(host);
		}
	}

	// CONPORT_MAX_CONNECTIONS - Maximum database connections
	if let Ok(max_conn) = env::var("CONPORT_MAX_CONNECTIONS") {
		if let Ok(num) = max_conn.parse::<usize>() {
			config.max_connections = Some(num);
		}
	}

	// CONPORT_VECTOR_STORE_ENABLED - Enable vector store
	if let Ok(enabled) = env::var("CONPORT_VECTOR_STORE_ENABLED") {
		config.vector_store_enabled = enabled.to_lowercase() == "true" || enabled == "1";
	}

	// CONPORT_VECTOR_STORE_PATH - Vector store path
	if let Ok(vs_path) = env::var("CONPORT_VECTOR_STORE_PATH") {
		if !vs_path.is_empty() {
			config.vector_store_path = Some(vs_path);
		}
	}

	config
}

/// Get the workspace ID from environment, with fallback to current directory
pub fn get_workspace_from_environment() -> String {
	env::var("CONPORT_WORKSPACE").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| {
		std::env::current_dir()
			.map(|p| p.to_string_lossy().to_string())
			.unwrap_or_else(|_| String::from("."))
	})
}

/// Get log level from environment with default
pub fn get_log_level_from_environment() -> LogLevel {
	env::var("CONPORT_LOG_LEVEL")
		.ok()
		.map(|s| LogLevel::from_str(&s))
		.unwrap_or_default()
}

/// Check if vector store is enabled via environment
pub fn is_vector_store_enabled() -> bool {
	env::var("CONPORT_VECTOR_STORE_ENABLED")
		.ok()
		.map(|s| s.to_lowercase() == "true" || s == "1")
		.unwrap_or(false)
}

/// Get database path from environment or use default
pub fn get_database_path_from_environment() -> Option<PathBuf> {
	env::var("CONPORT_DATABASE_PATH")
		.ok()
		.filter(|s| !s.is_empty())
		.map(PathBuf::from)
}

/// Environment variable names constant
pub mod env_vars {
	/// Workspace identifier
	pub const CONPORT_WORKSPACE:&str = "CONPORT_WORKSPACE";
	/// Logging level (trace, debug, info, warn, error)
	pub const CONPORT_LOG_LEVEL:&str = "CONPORT_LOG_LEVEL";
	/// Custom database path
	pub const CONPORT_DATABASE_PATH:&str = "CONPORT_DATABASE_PATH";
	/// Server port
	pub const CONPORT_PORT:&str = "CONPORT_PORT";
	/// Server host
	pub const CONPORT_HOST:&str = "CONPORT_HOST";
	/// Maximum database connections
	pub const CONPORT_MAX_CONNECTIONS:&str = "CONPORT_MAX_CONNECTIONS";
	/// Enable vector store (true/false)
	pub const CONPORT_VECTOR_STORE_ENABLED:&str = "CONPORT_VECTOR_STORE_ENABLED";
	/// Vector store path
	pub const CONPORT_VECTOR_STORE_PATH:&str = "CONPORT_VECTOR_STORE_PATH";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_log_level_parsing() {
		assert_eq!(LogLevel::from_str("debug"), LogLevel::Debug);
		assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
		assert_eq!(LogLevel::from_str("warn"), LogLevel::Warn);
		assert_eq!(LogLevel::from_str("warning"), LogLevel::Warn);
		assert_eq!(LogLevel::from_str("invalid"), LogLevel::Info);
	}

	#[test]
	fn test_default_log_level() {
		assert_eq!(LogLevel::default(), LogLevel::Info);
	}

	#[test]
	fn test_environment_config_default() {
		let config = EnvironmentConfig::default();
		assert_eq!(config.log_level, LogLevel::Info);
		assert!(!config.vector_store_enabled);
		assert!(config.workspace_id.is_none());
	}
}
