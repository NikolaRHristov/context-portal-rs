//! # ConPort MCP Binary Entry Point
//!
//! ## Overview
//!
//! ConPort MCP (Model Context Protocol) Server provides a standardized interface
//! for AI assistants to interact with persistent context storage. This binary
//! serves as the main entry point for running the MCP server.
//!
//! ## Architecture
//!
//! The ConPort system consists of:
//! - **ConportLibrary**: Core library providing all functionality
//! - **This Binary**: CLI entry point that initializes and runs the server
//! - **HTTP Transport**: STDIO-based transport for MCP protocol
//!
//! ## Usage
//!
//! ```bash
//! # Run as MCP server (stdio transport)
//! cargo run
//!
//! # Or install and run
//! cargo install
//! ConportMcp
//! ```
//!
//! ## CLI Arguments
//!
//! Currently supports minimal CLI for MCP protocol communication via stdio.
//! Future versions will add:
//! - Configuration file options
//! - Database path specification
//! - Log level control
//! - Health check endpoints

#![allow(non_snake_case)]

use std::path::PathBuf;

use ConportLibrary::{
    HTTP::Server::Initialize::{Server, ServerConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting ConPort MCP Server...");

    // Create server configuration
    let config = ServerConfig {
        Host: "127.0.0.1".to_string(),
        Port: 3000,
        StdioMode: true,
        WorkspaceDetectionEnabled: true,
        MaxConnections: 10,
        DatabasePath: PathBuf::from("context.db"),
    };

    tracing::info!("Database path: {:?}", config.DatabasePath);

    // Create the server (this initializes DB, vector store, workspace manager, and embedding model internally)
    let server = Server::New(config).await?;
    
    tracing::info!("ConPort MCP Server running on stdio...");
    
    // Run the server - this uses stdio transport for MCP protocol
    server.Run().await?;

    tracing::info!("ConPort MCP Server stopped");
    Ok(())
}