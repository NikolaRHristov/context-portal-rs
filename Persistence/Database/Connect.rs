// DatabaseConnect for the ConPort MCP server
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Connect {
    pub Connection: Mutex<Connection>,
}

impl Connect {
    pub fn New(Path: &PathBuf) -> Result<Self, crate::Error::Kind> {
        let Connection = Connection::open(Path)
            .map_err(|e| crate::Error::Kind::Database(e.to_string()))?;
        
        let Conn = Self {
            Connection: Mutex::new(Connection),
        };
        
        Conn.InitializeSchema()?;
        
        Ok(Conn)
    }

    fn InitializeSchema(&self) -> Result<(), crate::Error::Kind> {
        let Conn = self.Connection.lock()
            .map_err(|e| crate::Error::Kind::Database(e.to_string()))?;
        
        Conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS Contexts (
                Id TEXT PRIMARY KEY,
                Name TEXT NOT NULL,
                Content TEXT NOT NULL,
                WorkspacePath TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                UpdatedAt TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS Decisions (
                Id TEXT PRIMARY KEY,
                ContextId TEXT NOT NULL,
                Description TEXT NOT NULL,
                Rationale TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                FOREIGN KEY (ContextId) REFERENCES Contexts(Id)
            );

            CREATE TABLE IF NOT EXISTS ProgressItems (
                Id TEXT PRIMARY KEY,
                ContextId TEXT NOT NULL,
                Description TEXT NOT NULL,
                Status TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                UpdatedAt TEXT NOT NULL,
                FOREIGN KEY (ContextId) REFERENCES Contexts(Id)
            );

            CREATE TABLE IF NOT EXISTS Patterns (
                Id TEXT PRIMARY KEY,
                Name TEXT NOT NULL,
                Description TEXT NOT NULL,
                ContextId TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                FOREIGN KEY (ContextId) REFERENCES Contexts(Id)
            );

            CREATE TABLE IF NOT EXISTS CustomData (
                Id TEXT PRIMARY KEY,
                Key TEXT NOT NULL,
                Value TEXT NOT NULL,
                ContextId TEXT NOT NULL,
                CreatedAt TEXT NOT NULL,
                FOREIGN KEY (ContextId) REFERENCES Contexts(Id)
            );
            "
        ).map_err(|e: rusqlite::Error| crate::Error::Kind::Database(e.to_string()))?;

        Ok(())
    }
}