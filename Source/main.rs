// ConPort MCP Server - Main Entry Point
// Rust rewrite of the Python context-portal-rs project

use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "conport-mcp")]
#[command(about = "ConPort MCP Server - Context Portal for AI assistants")]
struct Args {
	/// Host to bind to
	#[arg(long, default_value = "127.0.0.1")]
	Host:String,

	/// Port to bind to
	#[arg(short, long, default_value = "3000")]
	Port:u16,

	/// Database path
	#[arg(long)]
	Database:Option<String>,

	/// Workspace ID (for workspace-specific data)
	#[arg(long)]
	WorkspaceId:Option<String>,

	/// Server mode: http or stdio
	#[arg(long, default_value = "http")]
	Mode:String,

	/// Log file path (relative to workspace)
	#[arg(long, default_value = "logs/conport.log")]
	LogFile:Option<String>,

	/// Base path for storing workspace data
	#[arg(long)]
	BasePath:Option<String>,

	/// Database filename
	#[arg(long, default_value = "context.db")]
	DbFilename:Option<String>,

	/// Enable verbose logging
	#[arg(short, long, action = clap::ArgAction::Count)]
	Verbose:u8,

	/// Disable automatic workspace detection
	#[arg(long)]
	NoAutoDetect:bool,

	/// Starting directory for workspace detection
	#[arg(long)]
	WorkspaceSearchStart:Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Parse command line arguments
	let Args {
		Host,
		Port,
		Database,
		WorkspaceId,
		Mode,
		LogFile,
		BasePath,
		DbFilename,
		Verbose,
		NoAutoDetect,
		WorkspaceSearchStart,
	} = Args::parse();

	// Initialize logging
	let LogLevel = match Verbose {
		0 => tracing::Level::INFO,
		1 => tracing::Level::DEBUG,
		_ => tracing::Level::TRACE,
	};

	// Configure logging with file output if specified
	if let Some(ref log_file) = LogFile {
		setup_logging_with_file(log_file, LogLevel)?;
	} else {
		tracing_subscriber::registry()
			.with(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
			.with(tracing_subscriber::fmt::layer())
			.init();
	}

	tracing::info!("Starting ConPort MCP Server...");

	// Handle workspace detection
	let effective_workspace_id = resolve_workspace_id(WorkspaceId, !NoAutoDetect, WorkspaceSearchStart).await?;

	tracing::info!("Effective workspace ID: {:?}", effective_workspace_id);

	// Determine database path
	let DbPath = if let Some(Path) = Database {
		std::path::PathBuf::from(Path)
	} else {
		let mut path = if let Some(ref base) = BasePath {
			std::path::PathBuf::from(base)
		} else if let Some(ref ws_id) = effective_workspace_id {
			std::path::PathBuf::from(ws_id)
		} else {
			dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
		};

		path.push("context_portal");
		if let Some(ref db_filename) = DbFilename {
			path.push(db_filename);
		} else {
			path.push("context.db");
		}
		path
	};

	tracing::info!("Using database: {:?}", DbPath);

	// Ensure database directory exists
	if let Some(Parent) = DbPath.parent() {
		std::fs::create_dir_all(Parent)?;
	}

	// Initialize database connection
	let Db = context_portal_rs::Persistence::Database::Connect::Connect::New(&DbPath).map_err(|e| {
		tracing::error!("Failed to initialize database: {}", e);
		e
	})?;

	let Db = Arc::new(Db);

	// Run based on mode
	match Mode.as_str() {
		"stdio" => {
			tracing::info!("Starting ConPort in STDIO mode...");
			run_stdio_mode(Db, effective_workspace_id).await?;
		},
		"http" | _ => {
			tracing::info!("Starting ConPort in HTTP mode...");
			run_http_mode(Db, Host, Port).await?;
		},
	}

	Ok(())
}

async fn run_http_mode(
	Db:Arc<context_portal_rs::Persistence::Database::Connect::Connect>,
	Host:String,
	Port:u16,
) -> Result<(), Box<dyn std::error::Error>> {
	// Create HTTP application with database state
	let App = context_portal_rs::HTTP::Application::Create::Create(Db).await.map_err(|e| {
		tracing::error!("Failed to create HTTP application: {}", e);
		e
	})?;

	// Start server
	let Addr = format!("{}:{}", Host, Port).parse::<SocketAddr>()?;

	tracing::info!("Listening on http://{}", Addr);

	let Listener = tokio::net::TcpListener::bind(Addr).await?;

	axum::serve(Listener, App).await.map_err(|e| {
		tracing::error!("Server error: {}", e);
		e
	})?;

	Ok(())
}

async fn run_stdio_mode(
	Db:Arc<context_portal_rs::Persistence::Database::Connect::Connect>,
	_WorkspaceId:Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
	// Pre-warm database connection
	tracing::info!("Pre-warming database connection...");

	// Initialize STDIO transport
	let _stdio_transport = context_portal_rs::HTTP::Transport::Stdio::StdioTransport::New();

	// For now, just run a simple loop waiting for stdin
	// In a full implementation, this would handle MCP protocol over stdio
	tracing::info!("STDIO mode ready. Waiting for MCP client...");

	// Read from stdin and write to stdout
	use std::io::{Read, Write};
	let mut stdin = std::io::stdin();
	let mut stdout = std::io::stdout();
	let mut buffer = String::new();

	// Simple echo loop for demonstration
	// Real implementation would parse JSON-RPC messages
	loop {
		buffer.clear();
		match stdin.read_to_string(&mut buffer) {
			Ok(0) => break, // EOF
			Ok(_) => {
				tracing::debug!("Received: {}", &buffer[..buffer.len().min(100)]);
				// For now, just acknowledge - full MCP implementation would go here
				let response = r#"{"jsonrpc": "2.0", "id": null, "result": {"capabilities": {}}}"#;
				writeln!(stdout, "{}", response).ok();
				stdout.flush().ok();
			},
			Err(e) => {
				tracing::error!("Error reading stdin: {}", e);
				break;
			},
		}
	}

	Ok(())
}

fn setup_logging_with_file(log_file:&str, level:tracing::Level) -> Result<(), Box<dyn std::error::Error>> {
	use tracing_appender::rolling::{RollingFileAppender, Rotation};
	use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

	// Create log directory
	let log_path = std::path::Path::new(log_file);
	let log_dir = log_path.parent().unwrap_or(std::path::Path::new("."));
	std::fs::create_dir_all(log_dir)?;

	// Create file appender
	let file_appender =
		RollingFileAppender::new(Rotation::DAILY, log_dir, log_file.split('/').last().unwrap_or("conport.log"));

	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

	// Keep the guard alive for the duration of the program
	std::mem::forget(_guard);

	tracing_subscriber::registry()
		.with(EnvFilter::from_default_env().add_directive(level.into()))
		.with(fmt::layer().with_writer(non_blocking))
		.with(fmt::layer().with_writer(std::io::stderr))
		.init();

	Ok(())
}

async fn resolve_workspace_id(
	provided_id:Option<String>,
	auto_detect:bool,
	start_path:Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
	// If explicitly provided, use it
	if let Some(id) = provided_id {
		return Ok(Some(id));
	}

	// Try auto-detection
	if auto_detect {
		let detector = context_portal_rs::FileSystem::Workspace::Detector::Detector::New(
			start_path.map(std::path::PathBuf::from),
			10,
		);
		let root = detector.FindWorkspaceRoot();
		if root.exists() {
			return Ok(Some(root.to_string_lossy().to_string()));
		}
	}

	Ok(None)
}
