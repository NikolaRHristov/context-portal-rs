// Vector Store using USearch
// Provides high-performance vector similarity search with per-workspace indexes

use crate::Error::Kind::Kind;
use parking_lot::RwLock;
use std::collections::HashMap;
use usearch::{Index, IndexOptions, MetricKind};
use std::sync::Arc;

/// USearch index wrapper for a single workspace
pub struct WorkspaceIndex {
    /// USearch index
    pub Index: Index<f32>,
    /// Map of vector IDs to metadata
    pub Metadata: HashMap<usize, VectorMetadata>,
}

/// Metadata associated with a vector
#[derive(Debug, Clone)]
pub struct VectorMetadata {
    pub Id: String,
    pub WorkspaceId: String,
    pub ItemType: String,
    pub CreatedAt: String,
}

/// Vector search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub Id: String,
    pub Score: f32,
    pub ItemType: String,
    pub WorkspaceId: String,
}

/// Vector store using USearch with per-workspace indexes
pub struct Store {
    /// Per-workspace indexes
    Indexes: RwLock<HashMap<String, Arc<RwLock<WorkspaceIndex>>>>,
    /// Default embedding dimension
    Dimension: usize,
    /// Maximum results to return
    DefaultLimit: usize,
}

impl Store {
    /// Create a new vector store
    pub fn New() -> Self {
        Self::NewWithDimension(crate::AI::Embedding::Model::DEFAULT_DIMENSION)
    }

    /// Create a new vector store with custom dimension
    pub fn NewWithDimension(Dimension: usize) -> Self {
        Self {
            Indexes: RwLock::new(HashMap::new()),
            Dimension,
            DefaultLimit: 10,
        }
    }

    /// Get or create a workspace index
    fn GetOrCreateIndex(&self, WorkspaceId: &str) -> Arc<RwLock<WorkspaceIndex>> {
        // Check cache first
        {
            let Indexes = self.Indexes.read();
            if let Some(Index) = Indexes.get(WorkspaceId) {
                return Index.clone();
            }
        }

        // Create new index
        let Options = IndexOptions::new()
            .with_dimensions(self.Dimension)
            .with_metric(MetricKind::Cos)
            .with_expansion_add(16)
            .with_expansion_search(16);

        let Index = Index::new(&Options).expect("Failed to create USearch index");

        let WorkspaceIndex = Arc::new(RwLock::new(WorkspaceIndex {
            Index,
            Metadata: HashMap::new(),
        }));

        // Add to cache
        {
            let mut Indexes = self.Indexes.write();
            Indexes.insert(WorkspaceId.to_string(), WorkspaceIndex.clone());
        }

        WorkspaceIndex
    }

    /// Add a vector to the store
    pub fn Add(&self, WorkspaceId: &str, Id: &str, Embedding: &[f32], ItemType: &str) -> Result<(), Kind> {
        if Embedding.len() != self.Dimension {
            return Err(Kind::VectorStore(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.Dimension,
                Embedding.len()
            )));
        }

        let Index = self.GetOrCreateIndex(WorkspaceId);
        let mut Guard = Index.write();

        // Add vector to USearch index
        // USearch uses usize keys internally
        let Key = Guard.Metadata.len();
        Guard.Index.add(Key, Embedding).map_err(|e| {
            Kind::VectorStore(format!("Failed to add vector: {}", e))
        })?;

        // Store metadata
        Guard.Metadata.insert(
            Key,
            VectorMetadata {
                Id: Id.to_string(),
                WorkspaceId: WorkspaceId.to_string(),
                ItemType: ItemType.to_string(),
                CreatedAt: chrono::Utc::now().to_rfc3339(),
            },
        );

        tracing::debug!("Added vector {} to workspace {}", Id, WorkspaceId);
        Ok(())
    }

    /// Search for similar vectors
    pub fn Search(&self, WorkspaceId: &str, Query: &[f32], Limit: usize) -> Result<Vec<SearchResult>, Kind> {
        if Query.len() != self.Dimension {
            return Err(Kind::VectorStore(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.Dimension,
                Query.len()
            )));
        }

        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Guard = Index.read();

        if Guard.Index.is_empty() {
            return Ok(vec![]);
        }

        // Search in USearch
        let Results = Guard.Index.search(Query, Limit);

        let SearchResults: Vec<SearchResult> = Results
            .into_iter()
            .filter_map(|(Key, Score)| {
                Guard.Metadata.get(&Key).map(|Metadata| SearchResult {
                    Id: Metadata.Id.clone(),
                    Score,
                    ItemType: Metadata.ItemType.clone(),
                    WorkspaceId: Metadata.WorkspaceId.clone(),
                })
            })
            .collect();

        Ok(SearchResults)
    }

    /// Delete a vector from the store
    pub fn Delete(&self, WorkspaceId: &str, Id: &str) -> Result<(), Kind> {
        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Mutex = Index.write();

        // Find the key with matching ID
        let Key = Mutex.Metadata
            .iter()
            .find(|(_, m)| m.Id == Id)
            .map(|(k, _)| *k);

        if let Some(Key) = Key {
            Mutex.Index.remove(Key).map_err(|e| {
                Kind::VectorStore(format!("Failed to remove vector: {}", e))
            })?;
            Mutex.Metadata.remove(&Key);
            tracing::debug!("Deleted vector {} from workspace {}", Id, WorkspaceId);
        }

        Ok(())
    }

    /// Get a vector by ID
    pub fn Get(&self, WorkspaceId: &str, Id: &str) -> Result<Vec<f32>, Kind> {
        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Guard = Index.read();

        // Find the key with matching ID
        let Key = Guard.Metadata
            .iter()
            .find(|(_, m)| m.Id == Id)
            .map(|(k, _)| *k)
            .ok_or_else(|| Kind::NotFound(format!("Vector {} not found in workspace {}", Id, WorkspaceId)))?;

        // Get vector from index
        let Vector = Guard.Index.get_vector(Key)
            .map_err(|e| Kind::VectorStore(format!("Failed to get vector: {}", e)))?
            .to_vec();

        Ok(Vector)
    }

    /// Get all vector IDs for a workspace
    pub fn GetAllIds(&self, WorkspaceId: &str) -> Vec<String> {
        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Guard = Index.read();

        Guard.Metadata.values().map(|m| m.Id.clone()).collect()
    }

    /// Get vector count for a workspace
    pub fn Count(&self, WorkspaceId: &str) -> usize {
        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Guard = Index.read();
        Guard.Index.size()
    }

    /// Clear all vectors for a workspace
    pub fn Clear(&self, WorkspaceId: &str) -> Result<(), Kind> {
        let Index = self.GetOrCreateIndex(WorkspaceId);
        let Mutex = Index.write();

        Mutex.Index.clear();
        Mutex.Metadata.clear();

        tracing::info!("Cleared all vectors for workspace {}", WorkspaceId);
        Ok(())
    }

    /// Clear all workspaces
    pub fn ClearAll(&self) {
        let mut Indexes = self.Indexes.write();
        for (_, Index) in Indexes.iter_mut() {
            let Mutex = Index.write();
            Mutex.Index.clear();
            Mutex.Metadata.clear();
        }
        Indexes.clear();
        tracing::info!("Cleared all vector indexes");
    }

    /// Get the embedding dimension
    pub fn Dimension(&self) -> usize {
        self.Dimension
    }

    /// Get all workspace IDs
    pub fn GetWorkspaces(&self) -> Vec<String> {
        let Indexes = self.Indexes.read();
        Indexes.keys().cloned().collect()
    }

    /// Remove a workspace index
    pub fn RemoveWorkspace(&self, WorkspaceId: &str) -> Result<(), Kind> {
        let mut Indexes = self.Indexes.write();
        if let Some(Index) = Indexes.remove(WorkspaceId) {
            let Mutex = Index.write();
            Mutex.Index.clear();
            Mutex.Metadata.clear();
            tracing::info!("Removed workspace index: {}", WorkspaceId);
        }
        Ok(())
    }
}

impl Default for Store {
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
    fn test_store_creation() {
        let Store = Store::New();
        assert_eq!(Store.Dimension(), 384);
    }

    #[test]
    fn test_add_and_search() {
        let Store = Store::New();
        let Embedding = vec![0.1; 384];

        Store.Add("workspace1", "vec1", &Embedding, "decision").unwrap();

        let Results = Store.Search("workspace1", &Embedding, 10).unwrap();
        assert_eq!(Results.len(), 1);
        assert_eq!(Results[0].Id, "vec1");
    }

    #[test]
    fn test_delete() {
        let Store = Store::New();
        let Embedding = vec![0.1; 384];

        Store.Add("workspace1", "vec1", &Embedding, "decision").unwrap();
        assert_eq!(Store.Count("workspace1"), 1);

        Store.Delete("workspace1", "vec1").unwrap();
        assert_eq!(Store.Count("workspace1"), 0);
    }

    #[test]
    fn test_workspace_isolation() {
        let Store = Store::New();
        let Embedding1 = vec![0.1; 384];
        let Embedding2 = vec![0.9; 384];

        Store.Add("workspace1", "vec1", &Embedding1, "decision").unwrap();
        Store.Add("workspace2", "vec2", &Embedding2, "decision").unwrap();

        let Results1 = Store.Search("workspace1", &Embedding1, 10).unwrap();
        let Results2 = Store.Search("workspace2", &Embedding2, 10).unwrap();

        assert_eq!(Results1.len(), 1);
        assert_eq!(Results2.len(), 1);
        assert_ne!(Results1[0].Id, Results2[0].Id);
    }

    #[test]
    fn test_clear_workspace() {
        let Store = Store::New();
        let Embedding = vec![0.1; 384];

        Store.Add("workspace1", "vec1", &Embedding, "decision").unwrap();
        assert_eq!(Store.Count("workspace1"), 1);

        Store.Clear("workspace1").unwrap();
        assert_eq!(Store.Count("workspace1"), 0);
    }
}