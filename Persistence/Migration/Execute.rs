// Database migration executor for the ConPort MCP server
use crate::Error::Kind::Kind;
use rusqlite::Connection;

pub struct Execute;

impl Execute {
    pub fn New() -> Self {
        Execute
    }

    pub fn CreateTables(Connection: &Connection) -> Result<(), Kind> {
        Connection
            .execute_batch(
                r#"
                -- Context table
                CREATE TABLE IF NOT EXISTS Context (
                    Id TEXT PRIMARY KEY,
                    Name TEXT NOT NULL,
                    Content TEXT NOT NULL,
                    WorkspacePath TEXT NOT NULL,
                    CreatedAt TEXT NOT NULL,
                    UpdatedAt TEXT NOT NULL
                );

                -- Decision table
                CREATE TABLE IF NOT EXISTS Decision (
                    Id TEXT PRIMARY KEY,
                    ContextId TEXT NOT NULL,
                    Title TEXT NOT NULL,
                    Description TEXT NOT NULL,
                    Reasoning TEXT,
                    Outcome TEXT,
                    Confidence REAL,
                    CreatedAt TEXT NOT NULL,
                    UpdatedAt TEXT NOT NULL,
                    FOREIGN KEY (ContextId) REFERENCES Context(Id)
                );

                -- Progress table
                CREATE TABLE IF NOT EXISTS Progress (
                    Id TEXT PRIMARY KEY,
                    ContextId TEXT NOT NULL,
                    Title TEXT NOT NULL,
                    Description TEXT NOT NULL,
                    Status TEXT NOT NULL,
                    CreatedAt TEXT NOT NULL,
                    UpdatedAt TEXT NOT NULL,
                    FOREIGN KEY (ContextId) REFERENCES Context(Id)
                );

                -- Pattern table
                CREATE TABLE IF NOT EXISTS Pattern (
                    Id TEXT PRIMARY KEY,
                    Name TEXT NOT NULL,
                    Description TEXT NOT NULL,
                    ContextId TEXT NOT NULL,
                    CreatedAt TEXT NOT NULL,
                    FOREIGN KEY (ContextId) REFERENCES Context(Id)
                );

                -- CustomData table
                CREATE TABLE IF NOT EXISTS CustomData (
                    Id TEXT PRIMARY KEY,
                    Key TEXT NOT NULL,
                    Value TEXT NOT NULL,
                    ContextId TEXT NOT NULL,
                    CreatedAt TEXT NOT NULL,
                    FOREIGN KEY (ContextId) REFERENCES Context(Id),
                    UNIQUE(ContextId, Key)
                );

                -- Schema version table
                CREATE TABLE IF NOT EXISTS SchemaVersion (
                    Version INTEGER PRIMARY KEY,
                    AppliedAt TEXT NOT NULL,
                    Description TEXT NOT NULL
                );

                -- Vector embeddings table
                CREATE TABLE IF NOT EXISTS Embedding (
                    Id TEXT PRIMARY KEY,
                    EntityType TEXT NOT NULL,
                    EntityId TEXT NOT NULL,
                    Embedding BLOB NOT NULL,
                    CreatedAt TEXT NOT NULL
                );

                -- Create indexes
                CREATE INDEX IF NOT EXISTS idx_decision_context ON Decision(ContextId);
                CREATE INDEX IF NOT EXISTS idx_progress_context ON Progress(ContextId);
                CREATE INDEX IF NOT EXISTS idx_pattern_context ON Pattern(ContextId);
                CREATE INDEX IF NOT EXISTS idx_customdata_context ON CustomData(ContextId);
                CREATE INDEX IF NOT EXISTS idx_embedding_entity ON Embedding(EntityType, EntityId);
                "#,
            )
            .map_err(|e| Kind::Database(e.to_string()))?;

        Ok(())
    }

    pub fn RunMigrations(Connection: &Connection) -> Result<(), Kind> {
        // Get current schema version
        let CurrentVersion = Self::GetSchemaVersion(Connection).unwrap_or(0);

        // Apply migrations based on version
        if CurrentVersion < 1 {
            Self::MigrationV1(Connection)?;
        }

        // Future migrations can be added here
        // if CurrentVersion < 2 { Self::MigrationV2(Connection)?; }

        Ok(())
    }

    fn MigrationV1(Connection: &Connection) -> Result<(), Kind> {
        tracing::info!("Running migration v1: Initial schema");

        // Create initial tables
        Self::CreateTables(Connection)?;

        // Record the migration
        Connection
            .execute(
                "INSERT INTO SchemaVersion (Version, AppliedAt, Description) VALUES (1, datetime('now'), 'Initial schema')",
                [],
            )
            .map_err(|e| Kind::Database(e.to_string()))?;

        Ok(())
    }

    pub fn GetSchemaVersion(Connection: &Connection) -> Result<i32, Kind> {
        let Result = Connection.query_row(
            "SELECT MAX(Version) FROM SchemaVersion",
            [],
            |row| row.get::<_, Option<i32>>(0),
        );

        match Result {
            Ok(version) => Ok(version.unwrap_or(0)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(Kind::Database(e.to_string())),
        }
    }

    pub fn EnsureMigrated(Connection: &Connection) -> Result<i32, Kind> {
        let Version = Self::GetSchemaVersion(Connection)?;
        
        if Version == 0 {
            tracing::info!("Database not migrated, running initial migration");
            Self::RunMigrations(Connection)?;
            return Self::GetSchemaVersion(Connection);
        }

        tracing::info!("Database already at schema version {}", Version);
        Ok(Version)
    }
}

impl Default for Execute {
    fn default() -> Self {
        Self::New()
    }
}

// CreateTables - Standalone function
pub struct CreateTables;

impl CreateTables {
    pub fn Execute(Connection: &Connection) -> Result<(), Kind> {
        Execute::CreateTables(Connection)
    }
}

// RunMigrations - Standalone function
pub struct RunMigrations;

impl RunMigrations {
    pub fn Execute(Connection: &Connection) -> Result<(), Kind> {
        Execute::RunMigrations(Connection)
    }
}

// GetSchemaVersion - Standalone function
pub struct GetSchemaVersion;

impl GetSchemaVersion {
    pub fn Execute(Connection: &Connection) -> Result<i32, Kind> {
        Execute::GetSchemaVersion(Connection)
    }
}