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
    /// Comment (line or block)
    Comment { value: String, span: Span },
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
            TopLevelElement::Comment { span, .. } => span,
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
    String(String),
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
    /// Expression (interpolated in text)
    Expression { expr: Box<Expression>, span: Span },
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
    /// Fetch action (HTTP request)
    Fetch(FetchAction),
    /// Submit action (form submission)
    Submit(SubmitAction),
    /// Call action (API call)
    Call(CallAction),
    /// Custom action
    Custom(CustomAction),
}

/// Fetch action (HTTP request)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAction {
    /// URL expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Expression>,
    /// HTTP method (GET, POST, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Expression>,
    /// Request headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Expression>,
    pub span: Span,
}

/// Submit action (form submission)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAction {
    /// Target (form name or URL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Expression>,
    /// Form data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Expression>,
    pub span: Span,
}

/// Call action (API call)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallAction {
    /// API name
    pub api: String,
    /// Arguments
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<Expression>,
    pub span: Span,
}

/// Custom action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAction {
    /// Action name
    pub name: String,
    /// Parameters
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub params: Vec<Expression>,
    pub span: Span,
}

/// Action with response handlers (success, error, finally)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionWithHandlers {
    /// The action being called
    pub action: Action,
    /// Response handlers
    pub handlers: Vec<ResponseHandler>,
    pub span: Span,
}

/// Response handler (success, error, finally)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseHandler {
    /// Handler type
    pub handler_type: ResponseHandlerType,
    /// Actions to execute
    pub actions: Vec<ActionItem>,
    pub span: Span,
}

/// Response handler type (success, error, finally)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseHandlerType {
    Success,
    Error,
    Finally,
}

impl Action {
    /// Convert an action to an expression (for use in arrow function bodies)
    pub fn to_expression(&self) -> Expression {
        match self {
            Action::StateAssignment(sa) => {
                // Represent state assignment as an assignment expression
                Expression::Assignment(super::expr::Assignment {
                    target: Box::new(Expression::MemberAccess(sa.target.clone())),
                    value: Box::new(sa.value.clone()),
                    span: sa.span.clone(),
                })
            }
            Action::MethodCall(mc) => Expression::MethodCall(mc.clone()),
            Action::Fetch(fa) => Expression::MethodCall(super::expr::MethodCall {
                namespace: "fetch".to_string(),
                method: "request".to_string(),
                arguments: vec![],
                span: fa.span.clone(),
            }),
            Action::Submit(sa) => Expression::MethodCall(super::expr::MethodCall {
                namespace: "form".to_string(),
                method: "submit".to_string(),
                arguments: vec![],
                span: sa.span.clone(),
            }),
            Action::Call(ca) => Expression::MethodCall(super::expr::MethodCall {
                namespace: "api".to_string(),
                method: ca.api.clone(),
                arguments: ca
                    .args
                    .iter()
                    .map(|e| super::expr::Argument {
                        name: None,
                        value: e.clone(),
                        spread: false,
                    })
                    .collect(),
                span: ca.span.clone(),
            }),
            Action::Custom(ca) => Expression::MethodCall(super::expr::MethodCall {
                namespace: "custom".to_string(),
                method: ca.name.clone(),
                arguments: ca
                    .params
                    .iter()
                    .map(|e| super::expr::Argument {
                        name: None,
                        value: e.clone(),
                        spread: false,
                    })
                    .collect(),
                span: ca.span.clone(),
            }),
        }
    }
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
    /// Whether styles are scoped
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub scoped: bool,
    /// Style modifiers (scoped, global, module, etc.)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub modifiers: Vec<StyleModifier>,
    /// Parsed CSS rules and at-rules
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rules: Vec<StyleContent>,
    /// Raw CSS content (preserved as-is for frontend processing)
    /// Note: CSS is kept as raw string to allow frontend frameworks to handle
    /// CSS-in-JS, scoped styles, or other processing as needed.
    pub raw_css: String,
    pub span: Span,
}

/// Style modifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleModifier {
    Scoped,
    Global,
    Module,
    Critical,
    Inline,
    Custom(String),
}
/// Parsed CSS content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "content_type", rename_all = "snake_case")]
pub enum StyleContent {
    /// CSS rule with selector and declarations
    Rule(StyleRule),
    /// CSS at-rule (@media, @keyframes, etc.)
    AtRule(StyleAtRule),
    /// Comment
    Comment { value: String, span: Span },
}

/// CSS rule: selector { declarations }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    /// CSS selectors
    pub selectors: Vec<String>,
    /// CSS property declarations
    pub declarations: Vec<StyleDeclaration>,
    /// Nested style rules (for CSS nesting support)
    pub nested_rules: Vec<StyleRule>,
    pub span: Span,
}

/// CSS property declaration: property: value;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDeclaration {
    /// CSS property name
    pub property: String,
    /// CSS value (may contain interpolations)
    pub value: CssValue,
    pub span: Span,
}

/// CSS value with optional interpolation support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "value_type", rename_all = "snake_case")]
pub enum CssValue {
    /// Plain CSS value
    Plain { value: String },
    /// Value with interpolated expressions
    Interpolated { parts: Vec<CssValuePart> },
}

/// Part of an interpolated CSS value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "part_type", rename_all = "snake_case")]
pub enum CssValuePart {
    /// Plain text
    Text { value: String },
    /// Expression interpolation (e.g., {state.color})
    Expression { expr: Expression },
}

/// CSS at-rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "at_rule_type", rename_all = "snake_case")]
pub enum StyleAtRule {
    /// @media rule
    Media {
        query: String,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @keyframes rule
    Keyframes {
        name: String,
        blocks: Vec<KeyframeBlock>,
        span: Span,
    },
    /// @layer rule
    Layer {
        names: Vec<String>,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @supports rule
    Supports {
        condition: String,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @container rule
    Container {
        query: String,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @scope rule
    Scope {
        selector: String,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @apply rule (Tailwind)
    Apply { classes: String, span: Span },
    /// @screen rule (Tailwind)
    Screen {
        name: String,
        content: Vec<StyleContent>,
        span: Span,
    },
    /// @theme rule
    Theme {
        variables: Vec<ThemeVariable>,
        span: Span,
    },
    /// @variants rule
    Variants {
        names: Vec<String>,
        content: Vec<StyleContent>,
        span: Span,
    },
}

/// Keyframe block (from/to/percentage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeBlock {
    /// Keyframe selector (from, to, or percentage)
    pub selector: String,
    /// Declarations
    pub declarations: Vec<StyleDeclaration>,
    pub span: Span,
}

/// Theme variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeVariable {
    /// CSS variable name (e.g., --primary)
    pub name: String,
    /// Value
    pub value: String,
    pub span: Span,
}