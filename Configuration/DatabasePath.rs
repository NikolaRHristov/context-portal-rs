// DatabasePath configuration for the ConPort MCP server
use std::path::PathBuf;

pub struct DatabasePath {
    pub Path: PathBuf,
}

impl DatabasePath {
    pub fn New(Path: PathBuf) -> Self {
        Self { Path }
    }

    pub fn Default() -> Self {
        let mut Path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        Path.push("conport");
        Path.push("context.db");
        Self { Path }
    }

    pub fn EnsureDirectory(&self) -> Result<(), crate::Error::Kind> {
        if let Some(Parent) = self.Path.parent() {
            std::fs::create_dir_all(Parent)
                .map_err(|e| crate::Error::Kind::Configuration(e.to_string()))
        } else {
            Ok(())
        }
    }
}