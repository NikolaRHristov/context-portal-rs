// WorkspaceDetector for the ConPort MCP server
// Enhanced implementation matching Python workspace_detector.py

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use tracing::{info, warn, debug};

/// Workspace indicator constants matching Python implementation
pub mod indicators {
    /// Strong indicators that are most reliable for workspace detection
    pub const STRONG_INDICATORS: &[&str] = &[
        "package.json",
        ".git",
        "pyproject.toml",
        "Cargo.toml",
        "go.mod",
        "pom.xml",
    ];

    /// All indicators that suggest a directory is a workspace root
    pub const WORKSPACE_INDICATORS: &[&str] = &[
        "package.json",
        ".git",
        "pyproject.toml",
        "Cargo.toml",
        "go.mod",
        "pom.xml",
        "composer.json",
        "Gemfile",
        "requirements.txt",
        "setup.py",
        "CMakeLists.txt",
        "Makefile",
        ".gitignore",
        "README.md",
        "README.rst",
        "LICENSE",
    ];

    /// Project-specific files that indicate a development workspace
    pub const PROJECT_FILES: &[&str] = &[
        "package.json",
        "pyproject.toml",
        "Cargo.toml",
        "go.mod",
        "pom.xml",
        "composer.json",
        "Gemfile",
    ];
}

/// Detection method for debugging/diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    StrongIndicators,
    MultipleIndicators,
    ExistingContextPortal,
    EnvironmentVariable,
    Fallback,
}

impl DetectionMethod {
    pub fn as_str(&self) -> &str {
        match self {
            DetectionMethod::StrongIndicators => "strong_indicators",
            DetectionMethod::MultipleIndicators => "multiple_indicators",
            DetectionMethod::ExistingContextPortal => "existing_context_portal",
            DetectionMethod::EnvironmentVariable => "environment_variable",
            DetectionMethod::Fallback => "fallback",
        }
    }
}

/// Detection result with detailed information
#[derive(Debug, Clone)]
pub struct DetectionInfo {
    pub start_path: PathBuf,
    pub detected_workspace: PathBuf,
    pub context_portal_path: PathBuf,
    pub detection_method: DetectionMethod,
    pub indicators_found: Vec<String>,
    pub environment_variables: HashMap<String, Option<String>>,
}

impl Default for DetectionInfo {
    fn default() -> Self {
        Self {
            start_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            detected_workspace: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            context_portal_path: PathBuf::from("."),
            detection_method: DetectionMethod::Fallback,
            indicators_found: Vec::new(),
            environment_variables: HashMap::new(),
        }
    }
}

/// WorkspaceDetector for intelligent workspace detection
pub struct Detector {
    start_path: PathBuf,
    max_depth: usize,
}

impl Detector {
    /// Create a new Detector with the given start path
    pub fn New(start_path: Option<PathBuf>, max_depth: usize) -> Self {
        Self {
            start_path: start_path
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from(".")),
            max_depth,
        }
    }

    /// Create a detector with default settings (current directory, max depth 10)
    pub fn NewDefault() -> Self {
        Self::New(None, 10)
    }

    /// Find the workspace root using multiple detection strategies
    pub fn FindWorkspaceRoot(&self) -> PathBuf {
        // Strategy 1: Look for strong indicators with validation
        if let Some(workspace) = self.detect_by_strong_indicators() {
            tracing::info!("Workspace detected by strong indicators: {}", workspace.display());
            return workspace;
        }

        // Strategy 2: Look for any workspace indicators (minimum 2)
        if let Some(workspace) = self.detect_by_any_indicators() {
            tracing::info!("Workspace detected by general indicators: {}", workspace.display());
            return workspace;
        }

        // Strategy 3: Look for context_portal directory (existing ConPort workspace)
        if let Some(workspace) = self.detect_by_context_portal() {
            tracing::info!("Workspace detected by existing context_portal: {}", workspace.display());
            return workspace;
        }

        // Fallback: Use start path
        tracing::warn!("No workspace indicators found, using start path: {}", self.start_path.display());
        self.start_path.clone()
    }

    /// Detect workspace using strong indicators with validation
    fn detect_by_strong_indicators(&self) -> Option<PathBuf> {
        let mut path = self.start_path.clone();
        let mut depth = 0;

        while path != path.parent().unwrap_or(&path) && depth < self.max_depth {
            let indicators = self.find_indicators_at_path(&path, indicators::STRONG_INDICATORS);
            
            if !indicators.is_empty() {
                if self.validate_workspace(&path, &indicators) {
                    return Some(path);
                }
            }

            match path.parent() {
                Some(parent) => path = parent.clone(),
                None => break,
            }
            depth += 1;
        }

        None
    }

    /// Detect workspace using any workspace indicators (minimum 2)
    fn detect_by_any_indicators(&self) -> Option<PathBuf> {
        let mut path = self.start_path.clone();
        let mut depth = 0;

        while path != path.parent().unwrap_or(&path) && depth < self.max_depth {
            let indicators = self.find_indicators_at_path(&path, indicators::WORKSPACE_INDICATORS);
            
            // If we find multiple indicators, it's likely a workspace
            if indicators.len() >= 2 {
                tracing::debug!("Found {} indicators at {}: {:?}", indicators.len(), path.display(), indicators);
                return Some(path);
            }

            match path.parent() {
                Some(parent) => path = parent.clone(),
                None => break,
            }
            depth += 1;
        }

        None
    }

    /// Detect workspace by looking for existing context_portal directory
    fn detect_by_context_portal(&self) -> Option<PathBuf> {
        let mut path = self.start_path.clone();
        let mut depth = 0;

        while path != path.parent().unwrap_or(&path) && depth < self.max_depth {
            let context_portal = path.join("context_portal");
            if context_portal.exists() && context_portal.is_dir() {
                tracing::debug!("Found existing context_portal at: {}", path.display());
                return Some(path);
            }

            match path.parent() {
                Some(parent) => path = parent.clone(),
                None => break,
            }
            depth += 1;
        }

        None
    }

    /// Validate that the detected path is actually a workspace root
    fn validate_workspace(&self, path: &Path, found_indicators: &[String]) -> bool {
        tracing::debug!("Validating workspace at {} with indicators: {:?}", path.display(), found_indicators);

        // Check package.json for project-specific content
        if found_indicators.contains(&"package.json".to_string()) {
            if self.validate_package_json(path.join("package.json")) {
                return true;
            }
        }

        // Check pyproject.toml for Python projects
        if found_indicators.contains(&"pyproject.toml".to_string()) {
            if self.validate_pyproject_toml(path.join("pyproject.toml")) {
                return true;
            }
        }

        // Check for other project files
        for project_file in indicators::PROJECT_FILES {
            if found_indicators.contains(&project_file.to_string()) {
                tracing::debug!("Found project file {}, considering valid workspace", project_file);
                return true;
            }
        }

        // If .git exists, it's likely a workspace root
        if found_indicators.contains(&".git".to_string()) {
            tracing::debug!("Found .git directory, considering valid workspace");
            return true;
        }

        false
    }

    /// Validate package.json contains project-specific content
    fn validate_package_json(&self, path: PathBuf) -> bool {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(data) => {
                        // Look for project-specific indicators
                        let has_name = data.get("name").is_some();
                        let has_scripts = data.get("scripts")
                            .and_then(|s| s.as_object())
                            .map(|obj| {
                                obj.contains_key("dev") || 
                                obj.contains_key("start") || 
                                obj.contains_key("build")
                            })
                            .unwrap_or(false);
                        let has_dependencies = data.get("dependencies").is_some() || 
                                               data.get("devDependencies").is_some();
                        let is_module = data.get("type").and_then(|t| t.as_str()) == Some("module");

                        let is_valid = has_name && (has_scripts || has_dependencies || is_module);
                        tracing::debug!("package.json validation: name={}, scripts={}, deps={}, module={}, valid={}", 
                            has_name, has_scripts, has_dependencies, is_module, is_valid);
                        is_valid
                    }
                    Err(e) => {
                        tracing::debug!("Failed to parse package.json: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                tracing::debug!("Failed to read package.json: {}", e);
                false
            }
        }
    }

    /// Validate pyproject.toml contains project-specific content
    fn validate_pyproject_toml(&self, path: PathBuf) -> bool {
        match fs::read_to_string(&path) {
            Ok(content) => {
                // Look for common project sections
                let has_project = content.contains("[project]") || content.contains("[tool.");
                tracing::debug!("pyproject.toml validation: has_project_sections={}", has_project);
                has_project
            }
            Err(e) => {
                tracing::debug!("Failed to read pyproject.toml: {}", e);
                false
            }
        }
    }

    /// Find which indicators exist at a given path
    fn find_indicators_at_path(&self, path: &Path, indicator_list: &[&str]) -> Vec<String> {
        let mut found = Vec::new();
        for indicator in indicator_list {
            let indicator_path = path.join(indicator);
            if indicator_path.exists() {
                found.push(indicator.to_string());
            }
        }
        found
    }

    /// Get the context_portal path for the workspace
    pub fn GetContextPortalPath(&self, workspace_root: &Path) -> PathBuf {
        workspace_root.join("context_portal")
    }

    /// Detect workspace from MCP client environment variables
    pub fn DetectFromMcpContext(&self) -> Option<String> {
        // Check for VSCode workspace folder
        if let Ok(vscode_workspace) = std::env::var("VSCODE_WORKSPACE_FOLDER") {
            if !vscode_workspace.is_empty() && Path::new(&vscode_workspace).is_dir() {
                tracing::debug!("Detected workspace from VSCODE_WORKSPACE_FOLDER: {}", vscode_workspace);
                return Some(vscode_workspace);
            }
        }

        // Check for ConPort workspace
        if let Ok(conport_workspace) = std::env::var("CONPORT_WORKSPACE") {
            if !conport_workspace.is_empty() && Path::new(&conport_workspace).is_dir() {
                tracing::debug!("Detected workspace from CONPORT_WORKSPACE: {}", conport_workspace);
                return Some(conport_workspace);
            }
        }

        None
    }

    /// Get detailed information about the workspace detection process
    pub fn GetDetectionInfo(&self) -> DetectionInfo {
        let workspace_root = self.FindWorkspaceRoot();
        let mut info = DetectionInfo {
            start_path: self.start_path.clone(),
            detected_workspace: workspace_root.clone(),
            context_portal_path: self.GetContextPortalPath(&workspace_root),
            detection_method: DetectionMethod::Fallback,
            indicators_found: Vec::new(),
            environment_variables: HashMap::new(),
        };

        // Check what indicators exist at the detected workspace
        for indicator in indicators::WORKSPACE_INDICATORS {
            let indicator_path = workspace_root.join(indicator);
            if indicator_path.exists() {
                info.indicators_found.push(indicator.to_string());
            }
        }

        // Determine detection method
        if info.indicators_found.iter().any(|i| indicators::STRONG_INDICATORS.contains(&i.as_str())) {
            info.detection_method = DetectionMethod::StrongIndicators;
        } else if info.indicators_found.len() >= 2 {
            info.detection_method = DetectionMethod::MultipleIndicators;
        } else if workspace_root.join("context_portal").exists() {
            info.detection_method = DetectionMethod::ExistingContextPortal;
        }

        // Store environment variables for debugging
        info.environment_variables.insert(
            "VSCODE_WORKSPACE_FOLDER".to_string(),
            std::env::var("VSCODE_WORKSPACE_FOLDER").ok(),
        );
        info.environment_variables.insert(
            "CONPORT_WORKSPACE".to_string(),
            std::env::var("CONPORT_WORKSPACE").ok(),
        );
        info.environment_variables.insert(
            "PWD".to_string(),
            std::env::var("PWD").ok(),
        );
        if let Ok(cwd) = std::env::current_dir() {
            info.environment_variables.insert("CWD".to_string(), Some(cwd.to_string_lossy().to_string()));
        }

        info
    }
}

/// Convenience function for automatic workspace detection
pub fn AutoDetectWorkspace(start_path: Option<PathBuf>) -> PathBuf {
    let detector = Detector::New(start_path.clone(), 10);

    // First try MCP context detection
    if let Some(mcp_workspace) = detector.DetectFromMcpContext() {
        return PathBuf::from(mcp_workspace);
    }

    // Fall back to directory-based detection
    detector.FindWorkspaceRoot()
}

/// Resolve workspace ID using provided value or auto-detection
pub fn ResolveWorkspaceId(
    provided_workspace_id: Option<String>,
    auto_detect: bool,
    start_path: Option<PathBuf>,
) -> PathBuf {
    // If explicitly provided, use it (but handle special cases)
    if let Some(workspace_id) = provided_workspace_id {
        // Handle VSCode variable that wasn't expanded
        if workspace_id == "${workspaceFolder}" {
            tracing::warn!("workspace_id was literal '${{workspaceFolder}}', falling back to auto-detection");
        } else {
            tracing::debug!("Using provided workspace_id: {}", workspace_id);
            return PathBuf::from(workspace_id);
        }
    }

    // Auto-detect if enabled
    if auto_detect {
        let detected = AutoDetectWorkspace(start_path);
        tracing::info!("Auto-detected workspace: {}", detected.display());
        return detected;
    }

    // Final fallback to current directory
    let fallback = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    tracing::warn!("No workspace detection method available, using current directory: {}", fallback.display());
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strong_indicators_defined() {
        assert!(indicators::STRONG_INDICATORS.contains(&"package.json"));
        assert!(indicators::STRONG_INDICATORS.contains(&".git"));
        assert!(indicators::STRONG_INDICATORS.contains(&"Cargo.toml"));
    }

    #[test]
    fn test_workspace_indicators_defined() {
        assert!(indicators::WORKSPACE_INDICATORS.contains(&"Makefile"));
        assert!(indicators::WORKSPACE_INDICATORS.contains(&"README.md"));
    }

    #[test]
    fn test_detection_method_strings() {
        assert_eq!(DetectionMethod::StrongIndicators.as_str(), "strong_indicators");
        assert_eq!(DetectionMethod::MultipleIndicators.as_str(), "multiple_indicators");
        assert_eq!(DetectionMethod::Fallback.as_str(), "fallback");
    }

    #[test]
    fn test_resolve_workspace_id_with_provided() {
        let result = ResolveWorkspaceId(
            Some("/custom/path".to_string()),
            true,
            None,
        );
        assert_eq!(result, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_resolve_workspace_id_fallback() {
        let result = ResolveWorkspaceId(
            None,
            false,
            None,
        );
        // Should return current directory as fallback
        assert!(result.exists());
    }
}