// Initialize MCP server with lifespan management
// Handles server startup, tool registration, and graceful shutdown

use crate::Error::Kind::Kind;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Server state shared across the application
pub struct Server {
    /// Database connection pool
    pub DbState: Arc<crate::Persistence::Database::Connect::Connect>,
    /// Vector store for embeddings
    pub VectorStore: Arc<Mutex<crate::Persistence::Vector::Usearch::Store>>,
    /// Workspace manager
    pub WorkspaceManager: Arc<crate::Persistence::Workspace::Manager::Manager>,
    /// Embedding model (lazy loaded)
    pub EmbeddingModel: Arc<Mutex<Option<crate::AI::Embedding::Model::Model>>>,
    /// Server configuration
    pub Config: ServerConfig,
    /// Shutdown flag
    pub ShutdownRequested: Arc<Mutex<bool>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub Host: String,
    pub Port: u16,
    pub StdioMode: bool,
    pub WorkspaceDetectionEnabled: bool,
    pub MaxConnections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            Host: "127.0.0.1".to_string(),
            Port: 3000,
            StdioMode: true,
            WorkspaceDetectionEnabled: true,
            MaxConnections: 10,
        }
    }
}

impl Server {
    /// Create a new server instance
    pub async fn New(Config: ServerConfig) -> Result<Self, Kind> {
        // Initialize database connection
        let DbState = crate::Persistence::Database::Connect::Connect::New()
            .await
            .map_err(|e| Kind::Database(e.to_string()))?;

        // Initialize workspace manager
        let WorkspaceManager = crate::Persistence::Workspace::Manager::Manager::New();

        // Initialize vector store
        let VectorStore = Arc::new(Mutex::new(
            crate::Persistence::Vector::Usearch::Store::New()
        ));

        Ok(Self {
            DbState: Arc::new(DbState),
            VectorStore,
            WorkspaceManager: Arc::new(WorkspaceManager),
            EmbeddingModel: Arc::new(Mutex::new(None)),
            Config,
            ShutdownRequested: Arc::new(Mutex::new(false)),
        })
    }

    /// Run the MCP server
    pub async fn Run(&self) -> Result<(), Kind> {
        tracing::info!("Starting ConPort MCP Server...");

        if self.Config.StdioMode {
            // Run in stdio mode (MCP protocol)
            self.RunStdioMode().await
        } else {
            // Run in HTTP mode
            self.RunHttpMode().await
        }
    }

    /// Run in stdio mode for MCP protocol
    async fn RunStdioMode(&self) -> Result<(), Kind> {
        tracing::info!("Running in stdio mode...");

        // Initialize MCP transport
        let Transport = crate::HTTP::Transport::Stdio::Stdio::New();

        // Create the MCP application
        let App = crate::HTTP::Application::Create::Create::New(
            self.DbState.clone(),
            self.VectorStore.clone(),
            self.WorkspaceManager.clone(),
            self.EmbeddingModel.clone(),
        ).await?;

        // Run the transport loop
        Transport.Run(App).await;

        Ok(())
    }

    /// Run in HTTP mode
    async fn RunHttpMode(&self) -> Result<(), Kind> {
        use axum::Router;
        use axum::routing::get;

        tracing::info!("Running in HTTP mode on {}:{}", self.Config.Host, self.Config.Port);

        let App = crate::HTTP::Application::Create::Create::New(
            self.DbState.clone(),
            self.VectorStore.clone(),
            self.WorkspaceManager.clone(),
            self.EmbeddingModel.clone(),
        ).await?;

        let Router = Router::new()
            .route("/health", get(health_handler))
            .fallback_service(App);

        let Address = format!("{}:{}", self.Config.Host, self.Config.Port);
        let Listener = tokio::net::TcpListener::bind(&Address)
            .await
            .map_err(|e| Kind::Server(format!("Failed to bind to {}: {}", Address, e)))?;

        axum::serve(Listener, Router)
            .await
            .map_err(|e| Kind::Server(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Register a tool with the server
    pub async fn RegisterTool<T>(&self, Tool: T) -> Result<(), Kind>
    where
        T: crate::HTTP::Protocol::Request::Tool + Send + Sync + 'static,
    {
        tracing::info!("Registering tool: {}", Tool.name());
        // Tool registration logic would go here
        // For now, tools are registered in Application::Create
        Ok(())
    }

    /// Initialize the embedding model (lazy loading)
    pub async fn InitEmbeddingModel(&self) -> Result<(), Kind> {
        let mut ModelGuard = self.EmbeddingModel.lock().await;

        if ModelGuard.is_none() {
            tracing::info!("Loading embedding model...");
            let Model = crate::AI::Embedding::Model::Model::Load().await?;
            *ModelGuard = Some(Model);
            tracing::info!("Embedding model loaded successfully");
        }

        Ok(())
    }

    /// Request server shutdown
    pub async fn Shutdown(&self) -> Result<(), Kind> {
        tracing::info!("Initiating server shutdown...");

        let mut ShutdownFlag = self.ShutdownRequested.lock().await;
        *ShutdownFlag = true;

        // Clean up resources
        tracing::info!("Cleaning up resources...");

        // Close database connections
        // Note: The Connect::Close method would be called here if it exists

        tracing::info!("Server shutdown complete");
        Ok(())
    }

    /// Check if shutdown was requested
    pub async fn IsShutdownRequested(&self) -> bool {
        *self.ShutdownRequested.lock().await
    }
}

/// Health check handler for HTTP mode
async fn health_handler() -> &'static str {
    "OK"
}

// ============================================================================
// Lifespan management
// ============================================================================

/// Server lifespan handler for managing startup and shutdown
pub struct Lifespan;

impl Lifespan {
    /// Create a new lifespan handler
    pub fn New() -> Self {
        Self
    }

    /// Startup logic
    pub async fn OnStartup(Config: ServerConfig) -> Result<Server, Kind> {
        tracing::info!("Starting ConPort MCP Server...");
        let Server = Server::New(Config).await?;
        Ok(Server)
    }

    /// Shutdown logic
    pub async fn OnShutdown(Server: &Server) -> Result<(), Kind> {
        tracing::info!("Shutting down ConPort MCP Server...");
        Server.Shutdown().await
    }
}

impl Default for Lifespan {
    fn default() -> Self {
        Self::New()
    }
}