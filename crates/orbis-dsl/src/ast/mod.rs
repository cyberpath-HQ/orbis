//! Abstract Syntax Tree for Orbis DSL
//!
//! This module provides a complete AST representation for parsed Orbis DSL files.
//! The AST is designed to be:
//!
//! - **JSON Serializable**: All types implement `serde::Serialize` and `serde::Deserialize`
//! - **Easily Traversable**: Both in-memory via the `Visitor` trait and as JSON
//! - **Context-Aware**: Important context (validators, attributes) is preserved as node attributes
//! - **Filterable**: Query methods for finding specific node types
//! - **Flat**: Prefers attributes over deep nesting for better ergonomics
//!
//! # Architecture
//!
//! The AST is organized into several modules:
//!
//! - [`node`]: Core AST node types and common traits
//! - [`expr`]: Expression types with Pratt parsing for operator precedence
//! - [`types`]: Type annotation system
//! - [`component`]: Component and fragment definitions
//! - [`state`]: State declarations with validation
//! - [`control`]: Control flow structures (if, for, when)
//! - [`visitor`]: Visitor pattern for AST traversal
//! - [`filter`]: Filtering and query utilities
//! - [`builder`]: AST construction from pest parse trees
//!
//! # Example
//!
//! ```rust,ignore
//! use orbis_dsl::ast::{AstFile, parse_to_ast};
//!
//! let source = r#"
//! page {
//!     id: "my-page"
//!     title: "My Page"
//! }
//!
//! state {
//!     count = 0
//! }
//!
//! template {
//!     <Container>
//!         <Text content="Hello" />
//!     </Container>
//! }
//! "#;
//!
//! let ast = parse_to_ast(source)?;
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&ast)?;
//!
//! // Use visitor for traversal
//! use orbis_dsl::ast::Visitor;
//! struct MyVisitor;
//! impl Visitor for MyVisitor {
//!     fn visit_component(&mut self, node: &Component) {
//!         println!("Found component: {}", node.name);
//!     }
//! }
//!
//! let mut visitor = MyVisitor;
//! ast.accept(&mut visitor);
//! ```

mod builder;
mod component;
mod control;
mod expr;
mod filter;
mod node;
mod parser;
mod state;
mod types;
mod visitor;

// Re-export core types
pub use builder::{parse_to_ast, parse_to_ast_with_path, AstBuilder, BuildError, BuildErrorKind, BuildResult};
pub use component::{
    Attribute, AttributeValue, Component, EventBinding, EventHandler, FragmentDefinition,
    FragmentParam, FragmentUsage, HandlerType, SlotContent, SlotDefinition,
};
pub use control::{ElseIfBranch, ForBinding, ForBlock, IfBlock, WhenArm, WhenBlock, WhenPattern};
pub use expr::{
    Argument, ArrayLiteral, ArrowBody, ArrowFunction, ArrowStatement, BinaryExpr, BinaryOp,
    Expression, Identifier, InterpolatedString, Literal, LiteralValue, MemberAccess, MethodCall,
    ObjectLiteral, ObjectPair, SpecialVariable, SpecialVariableKind, StateAssignment, StringPart,
    UnaryExpr, UnaryOp,
};
pub use filter::{AstFilter, NodeKind};
pub use node::{
    Action, ActionItem, ActionWithHandlers, AstFile, ControlFlow, ExportStatement, ExportableItem,
    HookEntry, HooksBlock, ImportClause, ImportSpecifier, ImportStatement, InterfaceDefinition,
    InterfaceMember, LifecycleHook, LifecycleHookKind, PageBlock, PageProperties, PageProperty,
    ResponseHandler, Span, StateBlock, StyleModifier, StylesBlock, TemplateBlock, TemplateContent,
    TopLevelElement, WatcherHook, WatcherOption, WatcherOptionValue,
};
pub use parser::{
    parse, parse_file, parse_file_only, parse_with_path, ImportGraph, OrbisParser, ParseOptions,
    ParseResult, ParseWarning,
};
pub use state::{ComputedState, RegularState, StateDeclaration, ValidatedState, Validator, ValidatorArg};
pub use types::{GenericParam, LiteralType, PrimitiveKind, PrimitiveType, TypeAnnotation};
pub use visitor::{MutVisitor, Visitor, Walkable};
