// Workspace Manager for connection management
// Handles per-workspace connection caching and isolation

use crate::Error::Kind::Kind;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Workspace connection wrapper
pub struct Connection {
    /// Workspace identifier
    pub WorkspaceId: String,
    /// Database path for this workspace
    pub DatabasePath: PathBuf,
    /// Whether this connection is active
    pub Active: bool,
}

impl Connection {
    /// Create a new workspace connection
    pub fn New(WorkspaceId: String, DatabasePath: PathBuf) -> Self {
        Self {
            WorkspaceId,
            DatabasePath,
            Active: true,
        }
    }

    /// Close the connection
    pub fn Close(&mut self) {
        self.Active = false;
    }
}

/// Manager for workspace connections with connection caching
pub struct Manager {
    /// Cache of active workspace connections
    Connections: RwLock<HashMap<String, Arc<RwLock<Connection>>>>,
    /// Default workspace path
    DefaultPath: PathBuf,
    /// Maximum number of cached connections
    MaxCacheSize: usize,
}

impl Manager {
    /// Create a new workspace manager
    pub fn New() -> Self {
        let DefaultPath = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("conport")
            .join("workspaces");

        Self {
            Connections: RwLock::new(HashMap::new()),
            DefaultPath,
            MaxCacheSize: 10,
        }
    }

    /// Get or create a connection for a workspace
    pub fn GetConnection(&self, WorkspaceId: &str) -> Result<Arc<RwLock<Connection>>, Kind> {
        // Check cache first
        {
            let Connections = self.Connections.read();
            if let Some(Connection) = Connections.get(WorkspaceId) {
                let Conn = Connection.read();
                if Conn.Active {
                    tracing::debug!("Returning cached connection for workspace: {}", WorkspaceId);
                    return Ok(Connection.clone());
                }
            }
        }

        // Create new connection if not in cache or inactive
        let DatabasePath = self.GetWorkspacePath(WorkspaceId);

        // Check cache size and evict if necessary
        self.EnforceCacheSize();

        let Connection = Arc::new(RwLock::new(Connection::New(
            WorkspaceId.to_string(),
            DatabasePath,
        )));

        // Add to cache
        {
            let mut Connections = self.Connections.write();
            Connections.insert(WorkspaceId.to_string(), Connection.clone());
        }

        tracing::debug!("Created new connection for workspace: {}", WorkspaceId);
        Ok(Connection)
    }

    /// Get the database path for a workspace
    pub fn GetWorkspacePath(&self, WorkspaceId: &str) -> PathBuf {
        self.DefaultPath.join(WorkspaceId).join("conport.db")
    }

    /// Remove a workspace connection from cache
    pub fn RemoveWorkspace(&self, WorkspaceId: &str) -> Result<(), Kind> {
        let mut Connections = self.Connections.write();

        if let Some(Connection) = Connections.remove(WorkspaceId) {
            let mut Conn = Connection.write();
            Conn.Close();
            tracing::info!("Removed workspace connection: {}", WorkspaceId);
        }

        Ok(())
    }

    /// Get all workspace IDs
    pub fn GetWorkspaces(&self) -> Vec<String> {
        let Connections = self.Connections.read();
        Connections.keys().cloned().collect()
    }

    /// Get the count of active connections
    pub fn ConnectionCount(&self) -> usize {
        let Connections = self.Connections.read();
        Connections
            .values()
            .filter(|c| c.read().Active)
            .count()
    }

    /// Enforce maximum cache size by evicting oldest inactive connections
    fn EnforceCacheSize(&self) {
        let mut Connections = self.Connections.write();

        while Connections.len() >= self.MaxCacheSize {
            // Find and remove an inactive connection
            if let Some(InactiveKey) = Connections
                .iter()
                .find(|(_, c)| !c.read().Active)
                .map(|(k, _)| k.clone())
            {
                Connections.remove(&InactiveKey);
                tracing::debug!("Evicted inactive connection: {}", InactiveKey);
            } else {
                // All connections are active, remove oldest
                if let Some(FirstKey) = Connections.keys().next().cloned() {
                    Connections.remove(&FirstKey);
                    tracing::debug!("Evicted oldest connection: {}", FirstKey);
                }
            }
        }
    }

    /// Clear all connections
    pub fn Clear(&self) {
        let mut Connections = self.Connections.write();
        for Connection in Connections.values() {
            Connection.write().Close();
        }
        Connections.clear();
        tracing::info!("Cleared all workspace connections");
    }

    /// Set the default workspace path
    pub fn SetDefaultPath(&mut self, Path: PathBuf) {
        self.DefaultPath = Path;
    }

    /// Set maximum cache size
    pub fn SetMaxCacheSize(&mut self, Size: usize) {
        self.MaxCacheSize = Size;
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::New()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let Manager = Manager::New();
        assert_eq!(Manager.ConnectionCount(), 0);
    }

    #[test]
    fn test_get_connection() {
        let Manager = Manager::New();
        let Connection = Manager.GetConnection("test-workspace").unwrap();
        assert_eq!(Connection.read().WorkspaceId, "test-workspace");
        assert_eq!(Manager.ConnectionCount(), 1);
    }

    #[test]
    fn test_remove_workspace() {
        let Manager = Manager::New();
        Manager.GetConnection("test-workspace").unwrap();
        Manager.RemoveWorkspace("test-workspace").unwrap();
        assert_eq!(Manager.ConnectionCount(), 0);
    }

    #[test]
    fn test_connection_caching() {
        let Manager = Manager::New();
        let Conn1 = Manager.GetConnection("test-workspace").unwrap();
        let Conn2 = Manager.GetConnection("test-workspace").unwrap();
        // Should return the same connection
        assert!(Arc::ptr_eq(&Conn1, &Conn2));
    }
}