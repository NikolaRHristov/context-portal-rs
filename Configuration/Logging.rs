// Logging configuration with rotation
// Provides Initialize() function with tracing setup, FileAppender with daily rotation

use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging with tracing setup
/// - FileAppender with daily rotation
/// - Log level from environment (CONPORT_LOG_LEVEL)
/// - Workspace-specific log directories
pub fn Initialize() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_log_directory()?;
    
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;
    
    // Get log level from environment or default to info
    let log_level = get_log_level_from_env();
    
    // Create file appender with daily rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "conport.log",
    );
    
    // Set up the subscriber with both stdout and file output
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();
    
    tracing::info!("Logging initialized. Log directory: {:?}", log_dir);
    
    Ok(())
}

/// Initialize logging with a custom workspace ID for workspace-specific logs
pub fn InitializeWithWorkspace(workspace_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = get_workspace_log_directory(workspace_id)?;
    
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;
    
    // Get log level from environment or default to info
    let log_level = get_log_level_from_env();
    
    // Create file appender with daily rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "conport.log",
    );
    
    // Set up the subscriber with both stdout and file output
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true)
                .with_target(true),
        )
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true),
        )
        .init();
    
    tracing::info!("Logging initialized for workspace {}. Log directory: {:?}", workspace_id, log_dir);
    
    Ok(())
}

/// Get the default log directory
fn get_log_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base_dir = dirs::data_local_dir()
        .ok_or("Failed to get local data directory")?;
    
    Ok(base_dir.join("conport").join("logs"))
}

/// Get workspace-specific log directory
pub fn get_workspace_log_directory(workspace_id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base_dir = dirs::data_local_dir()
        .ok_or("Failed to get local data directory")?;
    
    // Sanitize workspace ID for use in path
    let sanitized_id = workspace_id.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    
    Ok(base_dir.join("conport").join("logs").join(sanitized_id))
}

/// Get log level from environment variable
fn get_log_level_from_env() -> String {
    std::env::var("CONPORT_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase()
}

/// Parse log level string to tracing Level
pub fn parse_log_level(level: &str) -> Level {
    match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" | "warning" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// Get the current log level from environment or return default
pub fn get_current_log_level() -> Level {
    parse_log_level(&get_log_level_from_env())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("debug"), Level::DEBUG);
        assert_eq!(parse_log_level("INFO"), Level::INFO);
        assert_eq!(parse_log_level("WARN"), Level::WARN);
        assert_eq!(parse_log_level("ERROR"), Level::ERROR);
        assert_eq!(parse_log_level("unknown"), Level::INFO); // default
    }
}