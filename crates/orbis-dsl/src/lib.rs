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
//! ## AST Module
//!
//! The `ast` module provides a complete Abstract Syntax Tree representation:
//!
//! ```rust,ignore
//! use orbis_dsl::ast::{parse_to_ast, AstFile, Visitor};
//!
//! let source = r#"
//! page { id: "test" title: "Test" }
//! template { <Container /> }
//! "#;
//!
//! let ast = parse_to_ast(source)?;
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&ast)?;
//!
//! // Use visitor for traversal
//! struct MyVisitor;
//! impl Visitor for MyVisitor {
//!     fn visit_component(&mut self, node: &Component) {
//!         println!("Found component: {}", node.name);
//!     }
//! }
//! ```
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

pub mod ast;
pub mod metadata;
pub mod page;