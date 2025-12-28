//! Orbis LSP Library
//!
//! This library provides the core functionality for the Orbis Language Server.

pub mod analysis;
pub mod backend;
pub mod capabilities;
pub mod completion;
pub mod definition;
pub mod diagnostics;
pub mod document;
pub mod hover;
pub mod references;
pub mod semantic_tokens;
pub mod symbols;

pub use backend::OrbisBackend;
