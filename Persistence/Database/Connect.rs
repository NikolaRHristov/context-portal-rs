// DatabaseConnect for the ConPort MCP server
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Database connection manager for the ConPort MCP server
pub struct Connect {
    pub Connection: Mutex<Connection>,
}

impl Connect {
    /// Create a new database connection and initialize the schema
    pub fn New(Path: &PathBuf) -> Result<Self, crate::Error::Kind::Kind> {
        let Connection = Connection::open(Path)
            .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

        let Conn = Self {
            Connection: Mutex::new(Connection),
        };

        Conn.InitializeSchema()?;

        Ok(Conn)
    }

    /// Initialize the database schema with all required tables
    fn InitializeSchema(&self) -> Result<(), crate::Error::Kind::Kind> {
        let conn = self.Connection.lock()
            .map_err(|e| crate::Error::Kind::Kind::Database(e.to_string()))?;

        conn.execute_batch(
            "
            -- ============================================================================
            -- Core Context Tables
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS ProductContext (
                Id INTEGER PRIMARY KEY,
                Content TEXT NOT NULL DEFAULT '{}'
            );
            
            CREATE TABLE IF NOT EXISTS ActiveContext (
                Id INTEGER PRIMARY KEY,
                Content TEXT NOT NULL DEFAULT '{}'
            );
            
            -- Seed initial context rows if they don't exist
            INSERT OR IGNORE INTO ProductContext (Id, Content) VALUES (1, '{}');
            INSERT OR IGNORE INTO ActiveContext (Id, Content) VALUES (1, '{}');

            -- ============================================================================
            -- Progress Entries Table
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS ProgressEntries (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                Status TEXT NOT NULL,
                Description TEXT NOT NULL,
                ParentId INTEGER,
                CreatedAt TEXT NOT NULL,
                UpdatedAt TEXT NOT NULL,
                FOREIGN KEY (ParentId) REFERENCES ProgressEntries(Id) ON DELETE SET NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_progress_workspace ON ProgressEntries(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_progress_parent ON ProgressEntries(ParentId);
            CREATE INDEX IF NOT EXISTS idx_progress_status ON ProgressEntries(Status);

            -- ============================================================================
            -- System Patterns Table
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS SystemPatterns (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                Name TEXT NOT NULL,
                Description TEXT,
                Tags TEXT,
                CreatedAt TEXT NOT NULL,
                UNIQUE(WorkspaceId, Name)
            );
            
            CREATE INDEX IF NOT EXISTS idx_patterns_workspace ON SystemPatterns(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_patterns_name ON SystemPatterns(Name);

            -- ============================================================================
            -- Custom Data Table
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS CustomData (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                Category TEXT NOT NULL,
                Key TEXT NOT NULL,
                Value TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                UNIQUE(WorkspaceId, Category, Key)
            );
            
            CREATE INDEX IF NOT EXISTS idx_customdata_workspace ON CustomData(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_customdata_category ON CustomData(Category);
            CREATE INDEX IF NOT EXISTS idx_customdata_category_key ON CustomData(Category, Key);

            -- ============================================================================
            -- Context History Tables
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS ProductContextHistory (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                Version INTEGER NOT NULL,
                Content TEXT NOT NULL,
                ChangeSource TEXT
            );
            
            CREATE TABLE IF NOT EXISTS ActiveContextHistory (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                Version INTEGER NOT NULL,
                Content TEXT NOT NULL,
                ChangeSource TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_product_history_workspace ON ProductContextHistory(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_product_history_version ON ProductContextHistory(Version);
            CREATE INDEX IF NOT EXISTS idx_active_history_workspace ON ActiveContextHistory(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_active_history_version ON ActiveContextHistory(Version);

            -- ============================================================================
            -- Context Links Table
            -- ============================================================================
            
            CREATE TABLE IF NOT EXISTS ContextLinks (
                Id INTEGER PRIMARY KEY AUTOINCREMENT,
                WorkspaceId TEXT NOT NULL,
                Timestamp TEXT NOT NULL,
                SourceItemType TEXT NOT NULL,
                SourceItemId TEXT NOT NULL,
                TargetItemType TEXT NOT NULL,
                TargetItemId TEXT NOT NULL,
                RelationshipType TEXT NOT NULL,
                Description TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_links_workspace ON ContextLinks(WorkspaceId);
            CREATE INDEX IF NOT EXISTS idx_links_source ON ContextLinks(SourceItemType, SourceItemId);
            CREATE INDEX IF NOT EXISTS idx_links_target ON ContextLinks(TargetItemType, TargetItemId);
            CREATE INDEX IF NOT EXISTS idx_links_relationship ON ContextLinks(RelationshipType);
            "
        ).map_err(|e: rusqlite::Error| crate::Error::Kind::Kind::Database(e.to_string()))?;

        // Initialize FTS5 tables
        self.InitializeFts5(&conn)?;

        Ok(())
    }

    /// Initialize FTS5 virtual tables for full-text search
    fn InitializeFts5(&self, conn: &Connection) -> Result<(), crate::Error::Kind::Kind> {
        // Note: FTS5 requires SQLite to be compiled with FTS5 support (bundled version has it)
        
        // Create decisions FTS5 table - using content= to reference the actual table
        // This allows for external content FTS5 which stays in sync via triggers
        let _ = conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS decisions_fts USING fts5(
                summary, rationale, implementation_details, tags,
                content='decisions',
                content_rowid='Id'
            )",
            [],
        );

        // Create triggers to keep decisions_fts in sync
        let _ = conn.execute_batch(
            "
            -- Trigger for decisions INSERT
            CREATE TRIGGER IF NOT EXISTS decisions_after_insert AFTER INSERT ON decisions
            BEGIN
                INSERT INTO decisions_fts (rowid, summary, rationale, implementation_details, tags)
                VALUES (new.Id, new.Summary, new.Rationale, new.ImplementationDetails, new.Tags);
            END;

            -- Trigger for decisions DELETE
            CREATE TRIGGER IF NOT EXISTS decisions_after_delete AFTER DELETE ON decisions
            BEGIN
                INSERT INTO decisions_fts (decisions_fts, rowid, summary, rationale, implementation_details, tags)
                VALUES ('delete', old.Id, old.Summary, old.Rationale, old.ImplementationDetails, old.Tags);
            END;

            -- Trigger for decisions UPDATE
            CREATE TRIGGER IF NOT EXISTS decisions_after_update AFTER UPDATE ON decisions
            BEGIN
                INSERT INTO decisions_fts (decisions_fts, rowid, summary, rationale, implementation_details, tags)
                VALUES ('delete', old.Id, old.Summary, old.Rationale, old.ImplementationDetails, old.Tags);
                INSERT INTO decisions_fts (rowid, summary, rationale, implementation_details, tags)
                VALUES (new.Id, new.Summary, new.Rationale, new.ImplementationDetails, new.Tags);
            END;
            "
        );

        // Create custom_data FTS5 table
        let _ = conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS custom_data_fts USING fts5(
                category, key, value_text,
                content='CustomData',
                content_rowid='Id'
            )",
            [],
        );

        // Create triggers for custom_data FTS
        let _ = conn.execute_batch(
            "
            -- Trigger for custom_data INSERT
            CREATE TRIGGER IF NOT EXISTS custom_data_after_insert AFTER INSERT ON CustomData
            BEGIN
                INSERT INTO custom_data_fts (rowid, category, key, value_text)
                VALUES (new.Id, new.Category, new.Key, new.Value);
            END;

            -- Trigger for custom_data DELETE
            CREATE TRIGGER IF NOT EXISTS custom_data_after_delete AFTER DELETE ON CustomData
            BEGIN
                INSERT INTO custom_data_fts (custom_data_fts, rowid, category, key, value_text)
                VALUES ('delete', old.Id, old.Category, old.Key, old.Value);
            END;

            -- Trigger for custom_data UPDATE
            CREATE TRIGGER IF NOT EXISTS custom_data_after_update AFTER UPDATE ON CustomData
            BEGIN
                INSERT INTO custom_data_fts (custom_data_fts, rowid, category, key, value_text)
                VALUES ('delete', old.Id, old.Category, old.Key, old.Value);
                INSERT INTO custom_data_fts (rowid, category, key, value_text)
                VALUES (new.Id, new.Category, new.Key, new.Value);
            END;
            "
        );

        Ok(())
    }
}

/// Helper function to get a timestamp in ISO 8601 format
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}