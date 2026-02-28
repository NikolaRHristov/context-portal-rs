// WorkspaceDetector for the ConPort MCP server
use std::path::{Path, PathBuf};

pub struct Detector;

impl Detector {
    pub fn Detect() -> Result<PathBuf, crate::Error::Kind::Kind> {
        // Check for common workspace indicators
        let Indicators = vec![
            ".git",
            ".vscode",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "go.mod",
            "requirements.txt",
        ];

        let CurrentDir = std::env::current_dir()
            .map_err(|e| crate::Error::Kind::Kind::WorkspaceDetection(e.to_string()))?;

        // Walk up the directory tree looking for workspace indicators
        let mut Path = CurrentDir.as_path();
        while Path != Path::new("/") {
            for Indicator in &Indicators {
                let CheckPath = Path.join(Indicator);
                if CheckPath.exists() {
                    return Ok(PathBuf::from(Path));
                }
            }
            
            match Path.parent() {
                Some(Parent) => Path = Parent,
                None => break,
            }
        }

        // Fallback to current directory
        Ok(CurrentDir)
    }

    pub fn DetectFromPath(StartPath: &PathBuf) -> Result<PathBuf, crate::Error::Kind::Kind> {
        let Indicators = vec![
            ".git",
            ".vscode",
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "go.mod",
            "requirements.txt",
        ];

        let mut Path = StartPath.as_path();
        while Path != Path::new("/") {
            for Indicator in &Indicators {
                let CheckPath = Path.join(Indicator);
                if CheckPath.exists() {
                    return Ok(PathBuf::from(Path));
                }
            }
            
            match Path.parent() {
                Some(Parent) => Path = Parent,
                None => break,
            }
        }

        Ok(StartPath.clone())
    }
}