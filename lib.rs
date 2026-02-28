// ConPort MCP Server Library
// Rust rewrite of the Python context-portal-rs project

// Error handling
pub mod Error {
pub mod Kind;
}

// Configuration
pub mod Configuration {
pub mod CLI;
pub mod DatabasePath;
}

// Type definitions
pub mod Type {
pub mod Context;
pub mod Decision;
pub mod Progress;
pub mod Pattern;
pub mod CustomData;
pub mod SystemPattern;
pub mod History;
pub mod ContextLink;
}

// AI
pub mod AI {
pub mod Embedding {
pub mod Generate;
pub mod Model;
}
}

// FileSystem
pub mod FileSystem {
pub mod Workspace {
pub mod Detector;
}
}

// Persistence
pub mod Persistence {
pub mod Database {
pub mod Connect;
pub mod Operations;
}

pub mod Vector {
pub mod Store;
pub mod Usearch;
}

pub mod Migration {
pub mod Execute;
}

pub mod Workspace {
pub mod Manager;
}
}

// HTTP
pub mod HTTP {
pub mod Application {
pub mod Create;
}

pub mod Server {
pub mod Initialize;
}

pub mod Protocol {
pub mod Request;
pub mod Response;
}

pub mod Transport {
pub mod Stdio;
}

pub mod Handler {
pub mod Context;
pub mod Decision;
pub mod Progress;
pub mod Pattern;
pub mod CustomData;
pub mod CustomDataHandler;
pub mod SystemPattern;
pub mod Search;
pub mod History;
pub mod Link;
pub mod ImportExport;
pub mod Batch;
}
}