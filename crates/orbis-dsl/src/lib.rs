//! Orbis DSL - Domain Specific Language parsers for Orbis
//!
//! This crate provides multiple pest-based parsers for different aspects of the Orbis DSL,
//! each with automatic case-insensitive keyword matching. Keywords can be written in any
//! case format (snake_case, camelCase, PascalCase, kebab-case, etc.) and will be recognized
//! automatically.
//!
//! ## Available Grammars
//!
//! - **page**: Page definitions and UI elements
//! - **metadata**: Metadata and configuration fields
//!
//! ## Usage
//!
//! ```rust,ignore
//! use orbis_dsl::page::{Parser, Rule};
//! use pest::Parser as PestParser;
//!
//! let input = "longString: String"; // or long_string, LONG_STRING, etc.
//! let pairs = Parser::parse(Rule::field_declaration, input)?;
//! ```

pub mod page;
pub mod metadata;