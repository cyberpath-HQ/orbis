//! # Orbis Core
//!
//! Core types, errors, and utilities shared across all Orbis crates.

pub mod error;
pub mod mode;
pub mod profile;
pub mod types;

pub use error::{Error, Result};
pub use mode::{AppMode, RunMode};
pub use profile::Profile;
