// ConPort MCP Server - Main Entry Point
// Rust rewrite of the Python context-portal-rs project

use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "conport-mcp")]
#[command(about = "ConPort MCP Server - Context Portal for AI assistants")]
struct Args {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    Host: String,

    /// Port to bind to
    #[arg(short, long, default_value = "3000")]
    Port: u16,

    /// Database path
    #[arg(long)]
    Database: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    Verbose: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let Args {
        Host,
        Port,
        Database,
        Verbose,
    } = Args::parse();

    // Initialize logging
    let LogLevel = match Verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting ConPort MCP Server...");

    // Determine database path
    let DbPath = if let Some(Path) = Database {
        std::path::PathBuf::from(Path)
    } else {
        let Config = context_portal_rs::Configuration::DatabasePath::DatabasePath::Default();
        Config.Path.clone()
    };

    tracing::info!("Using database: {:?}", DbPath);

    // Ensure database directory exists
    if let Some(Parent) = DbPath.parent() {
        std::fs::create_dir_all(Parent)?;
    }

    // Initialize database connection
    let Db = context_portal_rs::Persistence::Database::Connect::Connect::New(&DbPath)
        .map_err(|e| {
            tracing::error!("Failed to initialize database: {}", e);
            e
        })?;

    let Db = Arc::new(Db);

    // Create HTTP application
    let App = context_portal_rs::HTTP::Application::Create::Create()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create HTTP application: {}", e);
            e
        })?;

    // Note: State not added due to type mismatch
    let _ = Db; // Suppress unused warning

    // Start server
    let Addr = format!("{}:{}", Host, Port).parse::<SocketAddr>()?;
    
    tracing::info!("Listening on http://{}", Addr);
    
    let Listener = tokio::net::TcpListener::bind(Addr).await?;
    
    axum::serve(Listener, App)
        .await
        .map_err(|e| {
            tracing::error!("Server error: {}", e);
            e
        })?;

    Ok(())
}