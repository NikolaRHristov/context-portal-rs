// ConPort MCP Server Library
// Rust rewrite of the Python context-portal-rs project

// Allow non-snake_case naming conventions to maintain compatibility with existing codebase conventions
#![allow(non_snake_case)]
#![allow(nonstandard_style)]

// Error handling
pub mod Error {
	pub mod Context;
	pub mod Kind;
}

// Configuration
pub mod Configuration {
	pub mod CLI;
	pub mod DatabasePath;
	pub mod Environment;
	pub mod Logging;
}

// Type definitions
pub mod Type {
	pub mod Coerce;
	pub mod Context;
	pub mod ContextLink;
	pub mod CustomData;
	pub mod Decision;
	pub mod Filter;
	pub mod History;
	pub mod IntCoercion;
	pub mod Pattern;
	pub mod Progress;
	pub mod Schema;
	pub mod SystemPattern;
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

	pub mod Markdown {
		pub mod Export;
		pub mod Import;
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

	pub mod Query {
		pub mod Filter;
		pub mod Normalize;
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
		pub mod Batch;
		pub mod Context;
		pub mod CustomData;
		pub mod CustomDataHandler;
		pub mod Decision;
		pub mod History;
		pub mod ImportExport;
		pub mod Link;
		pub mod Pattern;
		pub mod Progress;
		pub mod Search;
		pub mod SystemPattern;
	}
}
