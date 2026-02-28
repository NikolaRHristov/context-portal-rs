// ConPort MCP Server Library
// Rust rewrite of the Python context-portal-rs project

// Error handling
pub mod Error {
    pub mod Kind;
    pub use Kind::Kind;
}

// Configuration
pub mod Configuration {
    pub mod DatabasePath;
    pub use DatabasePath::DatabasePath;
}

// Type definitions
pub mod Type {
    pub mod Context;
    pub mod Decision;
    pub mod Progress;
    pub mod Pattern;
    pub mod CustomData;

    pub use Context::Context;
    pub use Decision::Decision;
    pub use Progress::Progress;
    pub use Pattern::Pattern;
    pub use CustomData::CustomData;
}

// AI
pub mod AI {
    pub mod Embedding {
        pub mod Generate;
        pub use Generate::Generate;
    }
}

// FileSystem
pub mod FileSystem {
    pub mod Workspace {
        pub mod Detector;
        pub use Detector::Detector;
    }
}

// Persistence
pub mod Persistence {
    pub mod Database {
        pub mod Connect;
        pub use Connect::Connect;
    }

    pub mod Vector {
        pub mod Store;
        pub use Store::Store;
    }
}

// HTTP
pub mod HTTP {
    pub mod Application {
        pub mod Create;
        pub use Create::Create;
    }

    pub mod Handler {
        pub mod Context;
        pub mod Decision;
        pub mod Progress;

        pub use Context::*;
        pub use Decision::*;
        pub use Progress::*;
    }
}

// Re-export Error type for convenience
pub type Error = Kind;