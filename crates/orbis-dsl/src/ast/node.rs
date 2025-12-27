//! Core AST node types and common structures
//!
//! This module defines the foundational types for the AST, including
//! source location tracking, file structure, and block definitions.

use serde::{Deserialize, Serialize};

use super::component::{Component, FragmentDefinition, FragmentUsage, SlotDefinition};
use super::control::{ForBlock, IfBlock, WhenBlock};
use super::expr::{Expression, MethodCall, StateAssignment};
use super::state::StateDeclaration;
use super::types::TypeAnnotation;

/// Source location span for AST nodes
///
/// Tracks the start and end positions in the source file for error reporting
/// and LSP features like go-to-definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Span {
    /// Starting byte offset (0-indexed)
    pub start: usize,
    /// Ending byte offset (exclusive)
    pub end: usize,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Starting column number (1-indexed)
    pub start_col: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Ending column number (1-indexed)
    pub end_col: usize,
}

impl Span {
    /// Create a new span from positions
    pub fn new(
        start: usize,
        end: usize,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Create a span that covers both self and other
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            start_line: if self.start < other.start {
                self.start_line
            } else {
                other.start_line
            },
            start_col: if self.start < other.start {
                self.start_col
            } else {
                other.start_col
            },
            end_line: if self.end > other.end {
                self.end_line
            } else {
                other.end_line
            },
            end_col: if self.end > other.end {
                self.end_col
            } else {
                other.end_col
            },
        }
    }
}

/// Root AST node representing a complete Orbis DSL file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AstFile {
    /// Source file path (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Import statements (must appear first in file)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub imports: Vec<ImportStatement>,

    /// Top-level elements in order of appearance
    pub elements: Vec<TopLevelElement>,

    /// Full span of the file
    pub span: Span,
}

/// Top-level elements that can appear in an Orbis file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TopLevelElement {
    /// Export statement wrapping other items
    Export(ExportStatement),
    /// Page definition block
    Page(PageBlock),
    /// State declarations block
    State(StateBlock),
    /// Hooks block (lifecycle + watchers)
    Hooks(HooksBlock),
    /// Template block with components
    Template(TemplateBlock),
    /// Fragment definition
    Fragment(FragmentDefinition),
    /// Interface type definition
    Interface(InterfaceDefinition),
    /// Styles block (CSS)
    Styles(StylesBlock),
}

impl TopLevelElement {
    /// Get the span of this element
    pub fn span(&self) -> &Span {
        match self {
            TopLevelElement::Export(e) => &e.span,
            TopLevelElement::Page(p) => &p.span,
            TopLevelElement::State(s) => &s.span,
            TopLevelElement::Hooks(h) => &h.span,
            TopLevelElement::Template(t) => &t.span,
            TopLevelElement::Fragment(f) => &f.span,
            TopLevelElement::Interface(i) => &i.span,
            TopLevelElement::Styles(s) => &s.span,
        }
    }
}

// ============================================================================
// IMPORT/EXPORT
// ============================================================================

/// Import statement (TypeScript or Rust style)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "style", rename_all = "snake_case")]
pub enum ImportStatement {
    /// TypeScript-style: import { X, Y } from "path"
    TypeScript {
        clause: ImportClause,
        path: String,
        span: Span,
    },
    /// Rust-style: use path::to::Item
    Rust {
        path: Vec<String>,
        alias: Option<String>,
        span: Span,
    },
}

/// Import clause variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImportClause {
    /// Named imports: { X, Y, Z as Alias }
    Named { specifiers: Vec<ImportSpecifier> },
    /// Default import: import X from "path"
    Default { name: String },
    /// Namespace import: import * as X from "path"
    Namespace { name: String },
}

/// Individual import specifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSpecifier {
    /// Original name
    pub name: String,
    /// Alias if renamed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// Whether this is a type-only import
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_type: bool,
    pub span: Span,
}

/// Export statement wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatement {
    /// Whether this is a default export
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_default: bool,
    /// Whether this uses Rust-style `pub` visibility
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_pub: bool,
    /// The exported item
    pub item: ExportableItem,
    pub span: Span,
}

/// Items that can be exported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportableItem {
    /// Exported fragment
    Fragment(FragmentDefinition),
    /// Exported interface
    Interface(InterfaceDefinition),
    /// Exported constant
    Const {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Expression,
        span: Span,
    },
}

// ============================================================================
// PAGE BLOCK
// ============================================================================

/// Page definition block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBlock {
    /// Page properties as flat attributes
    #[serde(flatten)]
    pub properties: PageProperties,
    pub span: Span,
}

/// Page properties stored as direct fields for flat JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageProperties {
    /// Unique page identifier
    pub id: Option<String>,
    /// Page title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Page description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Route path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Show in navigation menu
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_in_menu: Option<bool>,
    /// Menu order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_order: Option<i64>,
    /// Parent route for nested routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_route: Option<String>,
    /// Whether authentication is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_auth: Option<bool>,
    /// Required permissions
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub permissions: Vec<String>,
    /// Required roles
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<String>,
    /// Layout to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,
}

/// Individual page property (used during parsing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageProperty {
    pub key: String,
    pub value: String,
    pub span: Span,
}

// ============================================================================
// STATE BLOCK
// ============================================================================

/// State declarations block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBlock {
    /// State declarations
    pub declarations: Vec<StateDeclaration>,
    pub span: Span,
}

// ============================================================================
// HOOKS BLOCK
// ============================================================================

/// Hooks block containing lifecycle hooks and watchers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksBlock {
    /// Hook entries (lifecycle or watcher)
    pub entries: Vec<HookEntry>,
    pub span: Span,
}

/// A hook entry (either lifecycle or watcher)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_type", rename_all = "snake_case")]
pub enum HookEntry {
    /// Lifecycle hook (@mount, @unmount)
    Lifecycle(LifecycleHook),
    /// Watcher hook (@watch)
    Watcher(WatcherHook),
}

/// Lifecycle hook kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleHookKind {
    Mount,
    Unmount,
}

/// Lifecycle hook (@mount, @unmount)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHook {
    /// Hook kind
    pub kind: LifecycleHookKind,
    /// Actions to execute
    pub actions: Vec<ActionItem>,
    pub span: Span,
}

/// Watcher hook (@watch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherHook {
    /// What to watch (state paths)
    pub targets: Vec<Expression>,
    /// Watcher options
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<WatcherOption>,
    /// Actions to execute when watched values change
    pub actions: Vec<ActionItem>,
    pub span: Span,
}

/// Watcher option (debounce, immediate, deep)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherOption {
    pub name: String,
    pub value: WatcherOptionValue,
    pub span: Span,
}

/// Watcher option values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WatcherOptionValue {
    Number(f64),
    Boolean(bool),
}

// ============================================================================
// TEMPLATE BLOCK
// ============================================================================

/// Template block with components and control flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateBlock {
    /// Template content
    pub content: Vec<TemplateContent>,
    pub span: Span,
}

/// Content that can appear in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "content_type", rename_all = "snake_case")]
pub enum TemplateContent {
    /// Component usage
    Component(Component),
    /// Control flow (if, for, when)
    ControlFlow(ControlFlow),
    /// Fragment usage
    FragmentUsage(FragmentUsage),
    /// Slot definition (inside fragments)
    SlotDefinition(SlotDefinition),
    /// Text content
    Text { value: String, span: Span },
    /// Comment
    Comment { value: String, span: Span },
}

/// Control flow wrapper for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "flow_type", rename_all = "snake_case")]
pub enum ControlFlow {
    If(IfBlock),
    For(ForBlock),
    When(WhenBlock),
}

// ============================================================================
// ACTIONS
// ============================================================================

/// Action item (simple action or action with handlers)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type", rename_all = "snake_case")]
pub enum ActionItem {
    /// Simple action
    Simple(Action),
    /// Action with response handlers
    WithHandlers(ActionWithHandlers),
}

/// Simple action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// State assignment
    StateAssignment(StateAssignment),
    /// Method call
    MethodCall(MethodCall),
}

/// Action with response handlers (success, error, finally)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionWithHandlers {
    /// The action being called
    pub call: MethodCall,
    /// Response handlers
    pub handlers: Vec<ResponseHandler>,
    pub span: Span,
}

/// Response handler (success, error, finally)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseHandler {
    /// Handler type
    pub handler_type: HandlerType,
    /// Actions to execute
    pub actions: Vec<ActionItem>,
    pub span: Span,
}

/// Handler type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerType {
    Success,
    Error,
    Finally,
}

// ============================================================================
// INTERFACE DEFINITION
// ============================================================================

/// Interface type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    /// Interface name
    pub name: String,
    /// Generic type parameters
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub generics: Vec<super::types::GenericParam>,
    /// Interface members
    pub members: Vec<InterfaceMember>,
    pub span: Span,
}

/// Interface member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMember {
    /// Field name
    pub name: String,
    /// Whether the field is optional
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub optional: bool,
    /// Field type
    pub type_annotation: TypeAnnotation,
    pub span: Span,
}

// ============================================================================
// STYLES BLOCK
// ============================================================================

/// Styles block (CSS-in-DSL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylesBlock {
    /// Style modifier (scoped/global)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<StyleModifier>,
    /// Raw CSS content (preserved as-is for now)
    /// TODO: Parse CSS into structured form if needed
    pub content: String,
    pub span: Span,
}

/// Style modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleModifier {
    Scoped,
    Global,
}
