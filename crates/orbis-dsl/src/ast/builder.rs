//! AST Builder - Constructs AST from pest parse trees
//!
//! This module provides the `AstBuilder` which converts pest `Pairs` into
//! the structured AST representation.

use pest::iterators::{Pair, Pairs};
use std::fmt;

use crate::page::Rule;

use super::component::{
    Attribute, AttributeValue, Component, EventBinding, EventHandler, FragmentDefinition,
    FragmentParam, FragmentUsage, HandlerType, SlotContent, SlotDefinition,
};
use super::control::{ElseIfBranch, ForBinding, ForBlock, IfBlock, WhenArm, WhenBlock, WhenPattern};
use super::expr::{
    Argument, ArrowFunction, BinaryExpr, BinaryOp, Expression, Identifier, Literal, MemberAccess,
    MethodCall, StateAssignment, UnaryExpr, UnaryOp,
};
use super::node::{
    Action, ActionItem, ActionWithHandlers, AstFile, ControlFlow, CustomAction,
    FetchAction, HookEntry, HooksBlock, ImportClause, ImportSpecifier, ImportStatement,
    InterfaceDefinition, InterfaceMember, LifecycleHook, LifecycleHookKind, PageBlock,
    ResponseHandler, ResponseHandlerType, Span, StateBlock, StylesBlock, SubmitAction,
    TemplateBlock, TemplateContent, TopLevelElement, WatcherHook, WatcherOption,
    WatcherOptionValue,
};
use super::state::{
    ComputedState, RegularState, StateDeclaration, ValidatedState, Validator, ValidatorArg,
};
use super::types::{GenericParam, TypeAnnotation};

// ============================================================================
// HELPER TYPES
// ============================================================================

/// Helper enum for tracking state while parsing if blocks
#[allow(dead_code)]
#[derive(PartialEq)]
enum CollectingFor {
    Main,
    ElseIf,
    Else,
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Error that can occur during AST building
#[derive(Debug, Clone)]
pub struct BuildError {
    /// Error message
    pub message: String,
    /// Location where error occurred
    pub span: Option<Span>,
    /// Error kind
    pub kind: BuildErrorKind,
}

impl BuildError {
    pub fn new(message: impl Into<String>, span: Option<Span>, kind: BuildErrorKind) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }

    pub fn unexpected_rule(expected: &str, got: Rule, span: Span) -> Self {
        Self::new(
            format!("Expected {}, got {:?}", expected, got),
            Some(span),
            BuildErrorKind::UnexpectedRule,
        )
    }

    pub fn missing_required(field: &str, span: Span) -> Self {
        Self::new(
            format!("Missing required field: {}", field),
            Some(span),
            BuildErrorKind::MissingRequired,
        )
    }

    pub fn invalid_value(message: impl Into<String>, span: Span) -> Self {
        Self::new(message, Some(span), BuildErrorKind::InvalidValue)
    }

    #[allow(dead_code)]
    pub fn unsupported(what: &str, span: Span) -> Self {
        Self::new(
            format!("Unsupported: {}", what),
            Some(span),
            BuildErrorKind::Unsupported,
        )
    }

    #[allow(dead_code)]
    pub fn missing(what: &str) -> Self {
        Self::new(
            format!("Missing: {}", what),
            None,
            BuildErrorKind::MissingRequired,
        )
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = &self.span {
            write!(
                f,
                "{}:{}: {}",
                span.start_line, span.start_col, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for BuildError {}

/// Kind of build error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildErrorKind {
    /// Unexpected rule in parse tree
    UnexpectedRule,
    /// Missing required field
    MissingRequired,
    /// Invalid value
    InvalidValue,
    /// Parse error from pest
    ParseError,
    /// Internal error
    Internal,
    /// Unsupported construct
    Unsupported,
}

/// Result type for AST building
pub type BuildResult<T> = Result<T, BuildError>;

// ============================================================================
// SPAN UTILITIES
// ============================================================================

/// Create span from pest Pair
pub fn span_from_pair(pair: &Pair<'_, Rule>) -> Span {
    let span = pair.as_span();
    let (start_line, start_col) = span.start_pos().line_col();
    let (end_line, end_col) = span.end_pos().line_col();

    Span {
        start: span.start(),
        end: span.end(),
        start_line,
        start_col,
        end_line,
        end_col,
    }
}

/// Merge two spans into one covering both
fn merge_spans(a: &Span, b: &Span) -> Span {
    Span {
        start: a.start.min(b.start),
        end: a.end.max(b.end),
        start_line: a.start_line.min(b.start_line),
        start_col: if a.start_line <= b.start_line {
            a.start_col
        } else {
            b.start_col
        },
        end_line: a.end_line.max(b.end_line),
        end_col: if a.end_line >= b.end_line {
            a.end_col
        } else {
            b.end_col
        },
    }
}

// ============================================================================
// AST BUILDER
// ============================================================================

/// Builder for constructing AST from pest parse trees
#[derive(Debug, Default)]
pub struct AstBuilder {
    /// Source file path (for error messages)
    source_path: Option<String>,
}

impl AstBuilder {
    /// Create a new AST builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder with source path for better error messages
    pub fn with_source_path(source_path: impl Into<String>) -> Self {
        Self {
            source_path: Some(source_path.into()),
        }
    }

    /// Build AST from parsed pairs
    pub fn build(&self, pairs: Pairs<'_, Rule>) -> BuildResult<AstFile> {
        let mut imports = Vec::new();
        let mut elements = Vec::new();
        let mut file_span: Option<Span> = None;

        for pair in pairs {
            let pair_span = span_from_pair(&pair);
            file_span = Some(match file_span {
                Some(s) => merge_spans(&s, &pair_span),
                None => pair_span.clone(),
            });

            match pair.as_rule() {
                Rule::file => {
                    // Process the file's inner pairs
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::import_section => {
                                imports.extend(self.build_import_section(inner)?);
                            }
                            Rule::top_level_element => {
                                if let Some(elem) = self.build_top_level_element(inner)? {
                                    elements.push(elem);
                                }
                            }
                            Rule::EOI => {}
                            _ => {}
                        }
                    }
                }
                Rule::EOI => {}
                _ => {}
            }
        }

        Ok(AstFile {
            path: self.source_path.clone(),
            imports,
            elements,
            span: file_span.unwrap_or_default(),
        })
    }

    // ========================================================================
    // IMPORTS
    // ========================================================================

    fn build_import_section(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<ImportStatement>> {
        let mut imports = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::import_statement {
                imports.push(self.build_import_statement(inner)?);
            }
        }
        Ok(imports)
    }

    fn build_import_statement(&self, pair: Pair<'_, Rule>) -> BuildResult<ImportStatement> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::ts_import => self.build_ts_import(inner),
            Rule::rust_use => self.build_rust_use(inner),
            _ => Err(BuildError::unexpected_rule(
                "import statement",
                inner.as_rule(),
                span,
            )),
        }
    }

    fn build_ts_import(&self, pair: Pair<'_, Rule>) -> BuildResult<ImportStatement> {
        let span = span_from_pair(&pair);
        let mut clause = None;
        let mut path = String::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::ts_import_clause => {
                    let mut specifiers = Vec::new();
                    for spec in inner.into_inner() {
                        if spec.as_rule() == Rule::ts_import_specifier {
                            specifiers.push(self.build_import_specifier(spec)?);
                        }
                    }
                    clause = Some(ImportClause::Named { specifiers });
                }
                Rule::ts_import_default => {
                    let name = inner.as_str().to_string();
                    clause = Some(ImportClause::Default { name });
                }
                Rule::ts_import_namespace => {
                    // Get the alias after "* as"
                    for ns_inner in inner.into_inner() {
                        if ns_inner.as_rule() == Rule::identifier {
                            clause = Some(ImportClause::Namespace {
                                name: ns_inner.as_str().to_string(),
                            });
                        }
                    }
                }
                Rule::import_path => {
                    path = self.extract_string_content(inner);
                }
                _ => {}
            }
        }

        Ok(ImportStatement::TypeScript {
            clause: clause.unwrap_or(ImportClause::Named {
                specifiers: Vec::new(),
            }),
            path,
            span,
        })
    }

    fn build_import_specifier(&self, pair: Pair<'_, Rule>) -> BuildResult<ImportSpecifier> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut alias = None;
        let mut is_type = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::Type => {
                    is_type = true;
                }
                Rule::identifier => {
                    if name.is_empty() {
                        name = inner.as_str().to_string();
                    } else {
                        alias = Some(inner.as_str().to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(ImportSpecifier {
            name,
            alias,
            is_type,
            span,
        })
    }

    fn build_rust_use(&self, pair: Pair<'_, Rule>) -> BuildResult<ImportStatement> {
        let span = span_from_pair(&pair);
        let mut path_parts = Vec::new();
        let mut alias = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::rust_path => {
                    for seg in inner.into_inner() {
                        path_parts.push(seg.as_str().to_string());
                    }
                }
                Rule::identifier => {
                    alias = Some(inner.as_str().to_string());
                }
                _ => {}
            }
        }

        Ok(ImportStatement::Rust {
            path: path_parts,
            alias,
            span,
        })
    }

    // ========================================================================
    // TOP-LEVEL ELEMENTS
    // ========================================================================

    fn build_top_level_element(
        &self,
        pair: Pair<'_, Rule>,
    ) -> BuildResult<Option<TopLevelElement>> {
        let inner = pair.into_inner().next();
        let inner = match inner {
            Some(p) => p,
            None => return Ok(None),
        };

        match inner.as_rule() {
            Rule::page_block => Ok(Some(TopLevelElement::Page(self.build_page_block(inner)?))),
            Rule::state_block => Ok(Some(TopLevelElement::State(self.build_state_block(inner)?))),
            Rule::hooks_block => Ok(Some(TopLevelElement::Hooks(self.build_hooks_block(inner)?))),
            Rule::template_block => {
                Ok(Some(TopLevelElement::Template(self.build_template_block(inner)?)))
            }
            Rule::fragment_definition => Ok(Some(TopLevelElement::Fragment(
                self.build_fragment_definition(inner)?,
            ))),
            Rule::interface_definition => Ok(Some(TopLevelElement::Interface(
                self.build_interface_definition(inner)?,
            ))),
            Rule::styles_block => {
                Ok(Some(TopLevelElement::Styles(self.build_styles_block(inner)?)))
            }
            Rule::export_statement => self.build_export_statement(inner),
            _ => Ok(None),
        }
    }

    fn build_export_statement(
        &self,
        pair: Pair<'_, Rule>,
    ) -> BuildResult<Option<TopLevelElement>> {
        // export_statement contains ts_export or rust_pub
        let inner = pair.into_inner().next().unwrap();
        let mut is_default = false;

        match inner.as_rule() {
            Rule::ts_export => {
                for exp_inner in inner.into_inner() {
                    match exp_inner.as_rule() {
                        Rule::Default => {
                            is_default = true;
                        }
                        Rule::exportable_item => {
                            return self.build_exportable_item(exp_inner, true, is_default);
                        }
                        _ => {}
                    }
                }
            }
            Rule::rust_pub => {
                for pub_inner in inner.into_inner() {
                    if pub_inner.as_rule() == Rule::exportable_item {
                        return self.build_exportable_item(pub_inner, true, false);
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn build_exportable_item(
        &self,
        pair: Pair<'_, Rule>,
        exported: bool,
        is_default: bool,
    ) -> BuildResult<Option<TopLevelElement>> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::fragment_definition => {
                let mut frag = self.build_fragment_definition(inner)?;
                frag.exported = exported;
                frag.is_default = is_default;
                Ok(Some(TopLevelElement::Fragment(frag)))
            }
            Rule::interface_definition => {
                let iface = self.build_interface_definition(inner)?;
                Ok(Some(TopLevelElement::Interface(iface)))
            }
            Rule::const_declaration => {
                // Parse: const identifier (: type_annotation)? = state_value
                let span = span_from_pair(&inner);
                let mut name: Option<String> = None;
                let mut type_annotation = None;
                let mut value_expr = None;
                
                for part in inner.into_inner() {
                    match part.as_rule() {
                        Rule::identifier => {
                            name = Some(part.as_str().to_string());
                        }
                        Rule::type_annotation => {
                            type_annotation = Some(self.build_type_annotation(part)?);
                        }
                        Rule::state_value => {
                            value_expr = Some(self.build_state_value(part)?);
                        }
                        _ => {}
                    }
                }
                
                if let (Some(name), Some(value)) = (name, value_expr) {
                    let item = super::node::ExportableItem::Const {
                        name,
                        type_annotation,
                        value,
                        span: span.clone(),
                    };
                    
                    let export_stmt = super::node::ExportStatement {
                        is_default,
                        is_pub: exported,
                        item,
                        span,
                    };
                    
                    Ok(Some(TopLevelElement::Export(export_stmt)))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    // ========================================================================
    // PAGE BLOCK
    // ========================================================================

    fn build_page_block(&self, pair: Pair<'_, Rule>) -> BuildResult<PageBlock> {
        let span = span_from_pair(&pair);
        let mut properties = std::collections::HashMap::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::page_property {
                let (key, value) = self.build_page_property(inner)?;
                properties.insert(key, value);
            }
        }

        use super::node::PageProperties;
        Ok(PageBlock {
            properties: PageProperties {
                id: properties.get("id").cloned(),
                title: properties.get("title").cloned(),
                description: properties.get("description").cloned(),
                icon: properties.get("icon").cloned(),
                route: properties.get("route").cloned(),
                show_in_menu: properties.get("showinmenu").and_then(|v| v.parse().ok()),
                menu_order: properties.get("menuorder").and_then(|v| v.parse().ok()),
                parent_route: properties.get("parentroute").cloned(),
                requires_auth: properties.get("requiresauth").and_then(|v| v.parse().ok()),
                permissions: Vec::new(),
                roles: Vec::new(),
                layout: properties.get("layout").cloned(),
            },
            span,
        })
    }

    fn build_page_property(&self, pair: Pair<'_, Rule>) -> BuildResult<(String, String)> {
        let mut key = String::new();
        let mut value = String::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::page_key => {
                    // Normalize key: lowercase and remove underscores/hyphens for consistent lookups
                    key = inner
                        .as_str()
                        .to_lowercase()
                        .replace(['-', '_'], "");
                }
                Rule::literal_value => {
                    value = self.extract_literal_as_string(inner);
                }
                _ => {}
            }
        }

        Ok((key, value))
    }

    fn extract_literal_as_string(&self, pair: Pair<'_, Rule>) -> String {
        let inner = pair.into_inner().next();
        match inner {
            Some(p) => match p.as_rule() {
                Rule::quoted_string => self.extract_string_content(p),
                Rule::number => p.as_str().to_string(),
                Rule::boolean => p.as_str().to_string(),
                Rule::null_value => "null".to_string(),
                _ => p.as_str().to_string(),
            },
            None => String::new(),
        }
    }

    fn extract_string_content(&self, pair: Pair<'_, Rule>) -> String {
        // Remove quotes and process escape sequences
        let s = pair.as_str();
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    }

    // ========================================================================
    // STATE BLOCK
    // ========================================================================

    fn build_state_block(&self, pair: Pair<'_, Rule>) -> BuildResult<StateBlock> {
        let span = span_from_pair(&pair);
        let mut declarations = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::state_declaration {
                declarations.push(self.build_state_declaration(inner)?);
            }
        }

        Ok(StateBlock { declarations, span })
    }

    fn build_state_declaration(&self, pair: Pair<'_, Rule>) -> BuildResult<StateDeclaration> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::computed_state_declaration => Ok(StateDeclaration::Computed(
                self.build_computed_state_declaration(inner)?,
            )),
            Rule::validated_state_declaration => Ok(StateDeclaration::Validated(
                self.build_validated_state_declaration(inner)?,
            )),
            Rule::regular_state_declaration => Ok(StateDeclaration::Regular(
                self.build_regular_state_declaration(inner)?,
            )),
            _ => Err(BuildError::unexpected_rule(
                "state declaration",
                inner.as_rule(),
                span_from_pair(&inner),
            )),
        }
    }

    fn build_computed_state_declaration(&self, pair: Pair<'_, Rule>) -> BuildResult<ComputedState> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut explicit_computed = false;
        let mut type_annotation = None;
        let mut expression = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::ComputedKeyword => {
                    explicit_computed = true;
                }
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::type_annotation => {
                    type_annotation = Some(self.build_type_annotation(inner)?);
                }
                Rule::expression => {
                    expression = Some(self.build_expression(inner)?);
                }
                _ => {}
            }
        }

        Ok(ComputedState {
            name,
            explicit_computed,
            type_annotation,
            expression: expression.unwrap_or_else(|| Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span: span.clone(),
            })),
            span,
        })
    }

    fn build_validated_state_declaration(
        &self,
        pair: Pair<'_, Rule>,
    ) -> BuildResult<ValidatedState> {
        let span = span_from_pair(&pair);
        let source = pair.as_str();
        let mut name = String::new();
        let mut optional = false;
        let mut type_annotation = None;
        let mut value = None;
        let mut validators = Vec::new();

        // Check for optional marker by looking at the source text
        if source.contains("?:") || (source.contains('?') && !source.contains("@")) {
            if let Some(q_pos) = source.find('?') {
                let after_q = &source[q_pos + 1..];
                if after_q.trim_start().starts_with(':') || after_q.trim_start().starts_with('@') || after_q.trim_start().starts_with('=') {
                    optional = true;
                }
            }
        }

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::type_annotation => {
                    type_annotation = Some(self.build_type_annotation(inner)?);
                }
                Rule::state_value => {
                    value = Some(self.build_state_value(inner)?);
                }
                Rule::validation_chain => {
                    validators = self.build_validation_chain(inner)?;
                }
                _ => {}
            }
        }

        Ok(ValidatedState {
            name,
            optional,
            type_annotation,
            value,
            validators,
            span,
        })
    }

    fn build_regular_state_declaration(&self, pair: Pair<'_, Rule>) -> BuildResult<RegularState> {
        let span = span_from_pair(&pair);
        let source = pair.as_str();
        let mut name = String::new();
        let mut optional = false;
        let mut type_annotation = None;
        let mut value = None;
        
        // Check for optional marker by looking at the source text
        // The pattern is: identifier ~ "?"? ~ ...
        // So we need to check if there's a ? after the identifier
        if source.contains("?:") || source.matches('?').count() > 0 {
            // Need to check if the ? comes before : or = or at end
            if let Some(q_pos) = source.find('?') {
                // Check if ? is followed by : or whitespace then :
                let after_q = &source[q_pos + 1..];
                if after_q.trim_start().starts_with(':') || after_q.is_empty() {
                    optional = true;
                }
            }
        }

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::type_annotation => {
                    type_annotation = Some(self.build_type_annotation(inner)?);
                }
                Rule::state_value => {
                    value = Some(self.build_state_value(inner)?);
                }
                _ => {}
            }
        }

        Ok(RegularState {
            name,
            optional,
            type_annotation,
            value,
            span,
        })
    }

    fn build_validation_chain(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<Validator>> {
        let mut validators = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::validator {
                validators.push(self.build_validator(inner)?);
            }
        }
        Ok(validators)
    }

    fn build_validator(&self, pair: Pair<'_, Rule>) -> BuildResult<Validator> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut args = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::validator_name => {
                    name = inner.as_str().to_lowercase();
                }
                Rule::validator_args => {
                    for arg in inner.into_inner() {
                        if arg.as_rule() == Rule::validator_arg_list {
                            for val in arg.into_inner() {
                                if val.as_rule() == Rule::validator_arg {
                                    args.push(self.build_validator_arg(val)?);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Validator::with_args(name, args, span))
    }

    fn build_validator_arg(&self, pair: Pair<'_, Rule>) -> BuildResult<ValidatorArg> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::regex_literal => {
                // Parse regex pattern and flags
                let s = inner.as_str();
                // Format is /pattern/flags
                let mut pattern = s.to_string();
                let mut flags = None;
                if s.starts_with('/') {
                    if let Some(last_slash) = s.rfind('/') {
                        if last_slash > 0 {
                            pattern = s[1..last_slash].to_string();
                            let flag_str = &s[last_slash + 1..];
                            if !flag_str.is_empty() {
                                flags = Some(flag_str.to_string());
                            }
                        }
                    }
                }
                Ok(ValidatorArg::Regex { pattern, flags })
            }
            Rule::arrow_function => {
                let arrow = self.build_arrow_function(inner)?;
                Ok(ValidatorArg::Function(arrow))
            }
            Rule::object_literal => {
                // Build object as key-value pairs
                let pairs = self.build_object_pairs(inner)?;
                Ok(ValidatorArg::Object(pairs))
            }
            Rule::literal_value => {
                let lit_inner = inner.into_inner().next().unwrap();
                match lit_inner.as_rule() {
                    Rule::quoted_string => {
                        Ok(ValidatorArg::String(self.extract_string_content(lit_inner)))
                    }
                    Rule::number => {
                        let n: f64 = lit_inner.as_str().parse().unwrap_or(0.0);
                        Ok(ValidatorArg::Number(n))
                    }
                    Rule::boolean => {
                        let b = lit_inner.as_str().to_lowercase() == "true";
                        Ok(ValidatorArg::Boolean(b))
                    }
                    _ => Ok(ValidatorArg::String(lit_inner.as_str().to_string())),
                }
            }
            Rule::identifier => Ok(ValidatorArg::Identifier(inner.as_str().to_string())),
            _ => Ok(ValidatorArg::String(inner.as_str().to_string())),
        }
    }

    fn build_object_pairs(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<(String, ValidatorArg)>> {
        let mut pairs = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::object_pair {
                let mut key = String::new();
                let mut val = None;
                for p in inner.into_inner() {
                    match p.as_rule() {
                        Rule::identifier => {
                            if key.is_empty() {
                                key = p.as_str().to_string();
                            }
                        }
                        Rule::quoted_string => {
                            if key.is_empty() {
                                key = self.extract_string_content(p);
                            }
                        }
                        Rule::literal_value
                        | Rule::object_literal
                        | Rule::array_literal
                        | Rule::member_access => {
                            // Convert to ValidatorArg
                            val = Some(self.pair_to_validator_arg(p)?);
                        }
                        _ => {}
                    }
                }
                if let Some(v) = val {
                    pairs.push((key, v));
                }
            }
        }
        Ok(pairs)
    }

    fn pair_to_validator_arg(&self, pair: Pair<'_, Rule>) -> BuildResult<ValidatorArg> {
        match pair.as_rule() {
            Rule::literal_value => {
                let lit_inner = pair.into_inner().next().unwrap();
                match lit_inner.as_rule() {
                    Rule::quoted_string => {
                        Ok(ValidatorArg::String(self.extract_string_content(lit_inner)))
                    }
                    Rule::number => {
                        let n: f64 = lit_inner.as_str().parse().unwrap_or(0.0);
                        Ok(ValidatorArg::Number(n))
                    }
                    Rule::boolean => {
                        let b = lit_inner.as_str().to_lowercase() == "true";
                        Ok(ValidatorArg::Boolean(b))
                    }
                    _ => Ok(ValidatorArg::String(lit_inner.as_str().to_string())),
                }
            }
            Rule::object_literal => {
                let pairs = self.build_object_pairs(pair)?;
                Ok(ValidatorArg::Object(pairs))
            }
            Rule::member_access | Rule::identifier => {
                Ok(ValidatorArg::Identifier(pair.as_str().to_string()))
            }
            _ => Ok(ValidatorArg::String(pair.as_str().to_string())),
        }
    }

    fn build_state_value(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::literal_value => self.build_literal_value(inner),
            Rule::object_literal => self.build_object_literal(inner),
            Rule::array_literal => self.build_array_literal(inner),
            _ => Err(BuildError::unexpected_rule(
                "state value",
                inner.as_rule(),
                span_from_pair(&inner),
            )),
        }
    }

    // ========================================================================
    // TYPE ANNOTATIONS
    // ========================================================================

    fn build_type_annotation(&self, pair: Pair<'_, Rule>) -> BuildResult<TypeAnnotation> {
        let inner = pair.into_inner().next().unwrap();
        self.build_type_annotation_inner(inner)
    }

    fn build_type_annotation_inner(&self, pair: Pair<'_, Rule>) -> BuildResult<TypeAnnotation> {
        let span = span_from_pair(&pair);

        match pair.as_rule() {
            Rule::union_type => {
                let mut types = Vec::new();
                for inner in pair.into_inner() {
                    types.push(self.build_type_annotation_inner(inner)?);
                }
                Ok(TypeAnnotation::Union { types, span })
            }
            Rule::non_union_type => {
                let inner = pair.into_inner().next().unwrap();
                self.build_type_annotation_inner(inner)
            }
            Rule::nullable_type => {
                let inner = pair.into_inner().next().unwrap();
                let inner_type = Box::new(self.build_type_annotation_inner(inner)?);
                Ok(TypeAnnotation::Optional { inner: inner_type, span })
            }
            Rule::array_type => {
                let inner = pair.into_inner().next().unwrap();
                let element = Box::new(self.build_type_annotation_inner(inner)?);
                Ok(TypeAnnotation::Array { element, span })
            }
            Rule::generic_type => {
                let mut name = String::new();
                let mut type_args = Vec::new();
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::identifier => name = inner.as_str().to_string(),
                        Rule::type_annotation => {
                            type_args.push(self.build_type_annotation(inner)?);
                        }
                        _ => {}
                    }
                }
                Ok(TypeAnnotation::Generic { name, args: type_args, span })
            }
            Rule::literal_type => {
                let inner = pair.into_inner().next().unwrap();
                let literal = match inner.as_rule() {
                    Rule::quoted_string => super::types::LiteralType::String(self.extract_string_content(inner)),
                    Rule::number => super::types::LiteralType::Number(inner.as_str().parse().unwrap_or(0.0)),
                    Rule::boolean => super::types::LiteralType::Boolean(inner.as_str().to_lowercase() == "true"),
                    _ => super::types::LiteralType::String(inner.as_str().to_string()),
                };
                Ok(TypeAnnotation::Literal { value: literal, span })
            }
            Rule::simple_type => {
                let text = pair.as_str().to_lowercase();
                use super::types::{PrimitiveKind, PrimitiveType};
                let kind = match text.as_str() {
                    "string" => Some(PrimitiveKind::String),
                    "number" => Some(PrimitiveKind::Number),
                    "bool" | "boolean" => Some(PrimitiveKind::Boolean),
                    "object" => Some(PrimitiveKind::Object),
                    "array" => Some(PrimitiveKind::Array),
                    "null" => Some(PrimitiveKind::Null),
                    "any" => Some(PrimitiveKind::Any),
                    "void" => Some(PrimitiveKind::Void),
                    "never" => Some(PrimitiveKind::Never),
                    _ => None,
                };
                if let Some(k) = kind {
                    Ok(TypeAnnotation::Primitive(PrimitiveType { kind: k, span }))
                } else {
                    Ok(TypeAnnotation::Named {
                        name: pair.as_str().to_string(),
                        span,
                    })
                }
            }
            _ => Ok(TypeAnnotation::Named {
                name: pair.as_str().to_string(),
                span,
            }),
        }
    }

    // ========================================================================
    // EXPRESSIONS
    // ========================================================================

    fn build_expression(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let mut terms = Vec::new();
        let mut operators = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::unary_expr => {
                    terms.push(self.build_unary_expr(inner)?);
                }
                Rule::bin_op => {
                    operators.push(self.parse_binary_op(inner.as_str()));
                }
                _ => {}
            }
        }

        // Build expression tree with proper precedence using Pratt parsing
        if terms.is_empty() {
            return Ok(Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span,
            }));
        }

        if terms.len() == 1 && operators.is_empty() {
            return Ok(terms.remove(0));
        }

        // Simple left-to-right precedence for now (full Pratt parsing uses priority)
        self.build_binary_tree(terms, operators, span)
    }

    fn build_binary_tree(
        &self,
        mut terms: Vec<Expression>,
        mut operators: Vec<BinaryOp>,
        span: Span,
    ) -> BuildResult<Expression> {
        if terms.len() == 1 {
            return Ok(terms.remove(0));
        }

        // Find the lowest precedence operator
        let mut min_prec = u8::MAX;
        let mut min_idx = 0;
        for (i, op) in operators.iter().enumerate() {
            let prec = op.precedence();
            if prec <= min_prec {
                min_prec = prec;
                min_idx = i;
            }
        }

        let op = operators.remove(min_idx);
        let right_terms = terms.split_off(min_idx + 1);
        let right_ops = operators.split_off(min_idx);

        let left = self.build_binary_tree(terms, operators, span.clone())?;
        let right = self.build_binary_tree(right_terms, right_ops, span.clone())?;

        Ok(Expression::Binary(BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span,
        }))
    }

    fn build_unary_expr(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let mut unary_op = None;
        let mut term = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::unary_op => {
                    unary_op = Some(self.parse_unary_op(inner.as_str()));
                }
                Rule::term => {
                    term = Some(self.build_term(inner)?);
                }
                _ => {}
            }
        }

        let expr = term.unwrap_or_else(|| {
            Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span: span.clone(),
            })
        });

        if let Some(op) = unary_op {
            Ok(Expression::Unary(UnaryExpr {
                op,
                operand: Box::new(expr),
                span,
            }))
        } else {
            Ok(expr)
        }
    }

    fn build_term(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next();

        match inner {
            Some(p) => match p.as_rule() {
                Rule::special_variable => self.build_special_variable(p),
                Rule::literal_value => self.build_literal_value(p),
                Rule::member_access => self.build_member_access_expr(p),
                Rule::identifier => Ok(Expression::Identifier(Identifier {
                    name: p.as_str().to_string(),
                    span: span_from_pair(&p),
                })),
                Rule::object_literal => self.build_object_literal(p),
                Rule::array_literal => self.build_array_literal(p),
                Rule::expression => self.build_expression(p),
                _ => Ok(Expression::Literal(Literal {
                    value: super::expr::LiteralValue::Null,
                    span,
                })),
            },
            None => Ok(Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span,
            })),
        }
    }

    fn build_special_variable(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let text = pair.as_str();
        let inner = pair.into_inner().next().unwrap();

        let (kind, path) = match inner.as_rule() {
            Rule::response_var => {
                let mut parts: Vec<&str> = text.split('.').collect();
                let _base = parts.remove(0); // "$response"
                let path: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                (super::expr::SpecialVariableKind::Response, path)
            }
            Rule::error_var => {
                let mut parts: Vec<&str> = text.split('.').collect();
                let _base = parts.remove(0);
                let path: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                (super::expr::SpecialVariableKind::Error, path)
            }
            Rule::event_var => {
                let mut parts: Vec<&str> = text.split('.').collect();
                let _base = parts.remove(0);
                let path: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                (super::expr::SpecialVariableKind::Event, path)
            }
            Rule::event_param_ref => {
                // @eventName reference - store the event name in path
                let name = text.trim_start_matches('@').to_string();
                (super::expr::SpecialVariableKind::Event, vec![name])
            }
            _ => (super::expr::SpecialVariableKind::Response, vec![]),
        };

        Ok(Expression::SpecialVariable(super::expr::SpecialVariable {
            kind,
            path,
            span,
        }))
    }

    fn build_literal_value(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        let value = match inner.as_rule() {
            Rule::quoted_string => {
                super::expr::LiteralValue::String(self.extract_string_content(inner))
            }
            Rule::number => {
                let text = inner.as_str();
                if text.contains('.') {
                    super::expr::LiteralValue::Number(text.parse().unwrap_or(0.0))
                } else {
                    super::expr::LiteralValue::Integer(text.parse().unwrap_or(0))
                }
            }
            Rule::boolean => {
                super::expr::LiteralValue::Boolean(inner.as_str().to_lowercase() == "true")
            }
            Rule::null_value => super::expr::LiteralValue::Null,
            _ => super::expr::LiteralValue::Null,
        };

        Ok(Expression::Literal(Literal { value, span }))
    }

    fn build_member_access_expr(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let mut parts = Vec::new();
        
        // Extract identifiers from the member_access pairs
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::identifier {
                parts.push(inner.as_str().to_string());
            }
        }

        if parts.is_empty() {
            return Ok(Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span,
            }));
        }

        let root = parts.remove(0);
        let path = parts;

        Ok(Expression::MemberAccess(MemberAccess { root, path, span }))
    }

    fn build_object_literal(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let mut pairs = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::object_pair {
                let mut key = String::new();
                let mut value = None;

                for p in inner.into_inner() {
                    match p.as_rule() {
                        Rule::identifier => {
                            if key.is_empty() {
                                key = p.as_str().to_string();
                            } else if value.is_none() {
                                value = Some(Expression::Identifier(Identifier {
                                    name: p.as_str().to_string(),
                                    span: span_from_pair(&p),
                                }));
                            }
                        }
                        Rule::quoted_string => {
                            if key.is_empty() {
                                key = self.extract_string_content(p);
                            }
                        }
                        Rule::literal_value => {
                            value = Some(self.build_literal_value(p)?);
                        }
                        Rule::object_literal => {
                            value = Some(self.build_object_literal(p)?);
                        }
                        Rule::array_literal => {
                            value = Some(self.build_array_literal(p)?);
                        }
                        Rule::member_access => {
                            value = Some(self.build_member_access_expr(p)?);
                        }
                        _ => {}
                    }
                }

                if let Some(v) = value {
                    pairs.push(super::expr::ObjectPair {
                        key,
                        value: v,
                        span: span.clone(),
                    });
                }
            }
        }

        Ok(Expression::Object(super::expr::ObjectLiteral {
            pairs,
            span,
        }))
    }

    fn build_array_literal(&self, pair: Pair<'_, Rule>) -> BuildResult<Expression> {
        let span = span_from_pair(&pair);
        let mut elements = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::array_element {
                let elem_inner = inner.into_inner().next();
                if let Some(p) = elem_inner {
                    let expr = match p.as_rule() {
                        Rule::literal_value => self.build_literal_value(p)?,
                        Rule::object_literal => self.build_object_literal(p)?,
                        Rule::array_literal => self.build_array_literal(p)?,
                        Rule::member_access => self.build_member_access_expr(p)?,
                        Rule::identifier => Expression::Identifier(Identifier {
                            name: p.as_str().to_string(),
                            span: span_from_pair(&p),
                        }),
                        _ => continue,
                    };
                    elements.push(expr);
                }
            }
        }

        Ok(Expression::Array(super::expr::ArrayLiteral {
            elements,
            span,
        }))
    }

    fn build_arrow_function(&self, pair: Pair<'_, Rule>) -> BuildResult<ArrowFunction> {
        let span = span_from_pair(&pair);
        let mut params: Vec<String> = Vec::new();
        let mut body = super::expr::ArrowBody::Expression(Box::new(Expression::Literal(Literal {
            value: super::expr::LiteralValue::Null,
            span: span.clone(),
        })));

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::arrow_params => {
                    for p in inner.into_inner() {
                        if p.as_rule() == Rule::identifier {
                            params.push(p.as_str().to_string());
                        }
                    }
                }
                Rule::arrow_body => {
                    body = self.build_arrow_body(inner)?;
                }
                _ => {}
            }
        }

        Ok(ArrowFunction { params, body, span })
    }

    fn build_arrow_body(&self, pair: Pair<'_, Rule>) -> BuildResult<super::expr::ArrowBody> {
        let inner = pair.into_inner().next();
        match inner {
            Some(p) => match p.as_rule() {
                Rule::expression => Ok(super::expr::ArrowBody::Expression(Box::new(
                    self.build_expression(p)?,
                ))),
                _ => {
                    // Block with statements
                    let mut statements = Vec::new();
                    for stmt in p.into_inner() {
                        if stmt.as_rule() == Rule::arrow_statement {
                            let stmt_inner = stmt.into_inner().next();
                            if let Some(s) = stmt_inner {
                                if s.as_rule() == Rule::expression {
                                    statements.push(super::expr::ArrowStatement::Return(Some(
                                        self.build_expression(s)?,
                                    )));
                                }
                            }
                        }
                    }
                    Ok(super::expr::ArrowBody::Block(statements))
                }
            },
            None => Ok(super::expr::ArrowBody::Expression(Box::new(
                Expression::Literal(Literal {
                    value: super::expr::LiteralValue::Null,
                    span: Span::default(),
                }),
            ))),
        }
    }

    fn parse_binary_op(&self, s: &str) -> BinaryOp {
        match s {
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::NotEq,
            "<" => BinaryOp::Lt,
            "<=" => BinaryOp::LtEq,
            ">" => BinaryOp::Gt,
            ">=" => BinaryOp::GtEq,
            "&&" => BinaryOp::And,
            "||" => BinaryOp::Or,
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            _ => BinaryOp::Add,
        }
    }

    fn parse_unary_op(&self, s: &str) -> UnaryOp {
        match s {
            "!" => UnaryOp::Not,
            "-" => UnaryOp::Neg,
            _ => UnaryOp::Not,
        }
    }

    // ========================================================================
    // HOOKS BLOCK BUILDING
    // ========================================================================

    fn build_hooks_block(&self, pair: Pair<'_, Rule>) -> BuildResult<HooksBlock> {
        let span = span_from_pair(&pair);
        let mut entries = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::hook_entry => {
                    entries.push(self.build_hook_entry(inner)?);
                }
                _ => {}
            }
        }

        Ok(HooksBlock { entries, span })
    }

    fn build_hook_entry(&self, pair: Pair<'_, Rule>) -> BuildResult<HookEntry> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::lifecycle_hook => Ok(HookEntry::Lifecycle(self.build_lifecycle_hook(inner)?)),
            Rule::watcher_hook => Ok(HookEntry::Watcher(self.build_watcher_hook(inner)?)),
            _ => {
                let span = span_from_pair(&inner);
                Err(BuildError::unsupported("hook entry type", span))
            }
        }
    }

    fn build_lifecycle_hook(&self, pair: Pair<'_, Rule>) -> BuildResult<LifecycleHook> {
        let span = span_from_pair(&pair);
        let mut kind = LifecycleHookKind::Mount;
        let mut actions = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::lifecycle_hook_name => {
                    let text = inner.as_str().to_lowercase();
                    kind = if text.contains("unmount") {
                        LifecycleHookKind::Unmount
                    } else {
                        LifecycleHookKind::Mount
                    };
                }
                Rule::hook_action_body => {
                    actions = self.build_action_items(inner)?;
                }
                _ => {}
            }
        }

        Ok(LifecycleHook { kind, actions, span })
    }

    fn build_watcher_hook(&self, pair: Pair<'_, Rule>) -> BuildResult<WatcherHook> {
        let span = span_from_pair(&pair);
        let mut targets = Vec::new();
        let mut options = Vec::new();
        let mut actions = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::watch_target => {
                    // watch_target contains either state_path or member_access
                    if let Some(target_inner) = inner.into_inner().next() {
                        match target_inner.as_rule() {
                            Rule::member_access => {
                                targets.push(self.build_member_access_expr(target_inner)?);
                            }
                            Rule::state_path => {
                                // state_path is: ^"state" ~ ("." ~ identifier)+
                                // We need to extract it as member_access with root="state"
                                let span = span_from_pair(&target_inner);
                                let mut path = Vec::new();
                                
                                for p in target_inner.into_inner() {
                                    if p.as_rule() == Rule::identifier {
                                        path.push(p.as_str().to_string());
                                    }
                                }
                                
                                targets.push(Expression::MemberAccess(MemberAccess {
                                    root: "state".to_string(),
                                    path,
                                    span,
                                }));
                            }
                            _ => {
                                // Fallback: try to build as expression
                                targets.push(self.build_expression(target_inner)?);
                            }
                        }
                    }
                }
                Rule::watcher_options => {
                    options = self.build_watcher_options(inner)?;
                }
                Rule::hook_action_body => {
                    actions = self.build_action_items(inner)?;
                }
                _ => {}
            }
        }

        Ok(WatcherHook {
            targets,
            options,
            actions,
            span,
        })
    }

    fn build_watcher_options(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<WatcherOption>> {
        let mut options = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::watcher_option {
                options.push(self.build_watcher_option(inner)?);
            }
        }

        Ok(options)
    }

    fn build_watcher_option(&self, pair: Pair<'_, Rule>) -> BuildResult<WatcherOption> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut value = WatcherOptionValue::Boolean(false);

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::watcher_option_name => {
                    name = inner.as_str().to_lowercase();
                }
                Rule::number => {
                    let num: f64 = inner.as_str().parse().unwrap_or(0.0);
                    value = WatcherOptionValue::Number(num);
                }
                Rule::boolean => {
                    let b = inner.as_str().to_lowercase() == "true";
                    value = WatcherOptionValue::Boolean(b);
                }
                _ => {}
            }
        }

        Ok(WatcherOption { name, value, span })
    }

    fn build_action_items(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<ActionItem>> {
        let mut items = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::action_item => {
                    items.push(self.build_action_item(inner)?);
                }
                Rule::action_with_handlers => {
                    items.push(ActionItem::WithHandlers(self.build_action_with_handlers(inner)?));
                }
                Rule::action_body => {
                    // Nested action body
                    items.extend(self.build_action_items(inner)?);
                }
                Rule::action => {
                    items.push(ActionItem::Simple(self.build_action(inner)?));
                }
                _ => {}
            }
        }

        Ok(items)
    }

    fn build_action_item(&self, pair: Pair<'_, Rule>) -> BuildResult<ActionItem> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::action_with_handlers => {
                Ok(ActionItem::WithHandlers(self.build_action_with_handlers(inner)?))
            }
            Rule::action => Ok(ActionItem::Simple(self.build_action(inner)?)),
            _ => {
                let span = span_from_pair(&inner);
                Err(BuildError::unsupported("action item type", span))
            }
        }
    }

    fn build_action(&self, pair: Pair<'_, Rule>) -> BuildResult<Action> {
        let span = span_from_pair(&pair);
        let text = pair.as_str().trim();

        // Check for state assignment: state.path = value
        if text.contains('=') && !text.contains("=>") && !text.contains("==") {
            return self.build_state_assignment_action(pair);
        }

        // Check for method call patterns
        let inner = pair.into_inner().next();
        if let Some(p) = inner {
            match p.as_rule() {
                Rule::method_call | Rule::member_access => {
                    return self.build_method_call_action(p);
                }
                _ => {
                    // Try to build as custom action
                    let name = p.as_str().to_string();
                    return Ok(Action::Custom(CustomAction {
                        name,
                        params: vec![],
                        span,
                    }));
                }
            }
        }

        // Fallback to custom action
        Ok(Action::Custom(CustomAction {
            name: text.to_string(),
            params: vec![],
            span,
        }))
    }

    fn build_state_assignment_action(&self, pair: Pair<'_, Rule>) -> BuildResult<Action> {
        let span = span_from_pair(&pair);
        
        // The pair might be an `action` rule containing a `state_assignment` rule
        // or it might be the `state_assignment` rule itself
        let assignment_pair = if pair.as_rule() == Rule::state_assignment {
            pair
        } else {
            // Look for state_assignment in inner rules
            pair.into_inner()
                .find(|p| p.as_rule() == Rule::state_assignment)
                .ok_or_else(|| {
                    BuildError::invalid_value("Expected state_assignment rule", span.clone())
                })?
        };
        
        let mut target: Option<MemberAccess> = None;
        let mut value: Option<Expression> = None;

        for inner in assignment_pair.into_inner() {
            match inner.as_rule() {
                Rule::state_path => {
                    // state_path is: ^"state" ~ ("." ~ identifier)+
                    // Extract it as member_access with root="state"
                    let inner_span = span_from_pair(&inner);
                    let mut path = Vec::new();
                    
                    for p in inner.into_inner() {
                        if p.as_rule() == Rule::identifier {
                            path.push(p.as_str().to_string());
                        }
                    }
                    
                    target = Some(MemberAccess {
                        root: "state".to_string(),
                        path,
                        span: inner_span,
                    });
                }
                Rule::expression => {
                    value = Some(self.build_expression(inner)?);
                }
                _ => {}
            }
        }

        let target = target.ok_or_else(|| {
            BuildError::missing_required("state_path in state_assignment", span.clone())
        })?;
        
        let value = value.ok_or_else(|| {
            BuildError::missing_required("expression in state_assignment", span.clone())
        })?;

        Ok(Action::StateAssignment(StateAssignment {
            target,
            value,
            span,
        }))
    }

    fn parse_simple_value(&self, text: &str, span: Span) -> Expression {
        let trimmed = text.trim();

        // Boolean
        if trimmed.eq_ignore_ascii_case("true") {
            return Expression::Literal(Literal {
                value: super::expr::LiteralValue::Boolean(true),
                span,
            });
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Expression::Literal(Literal {
                value: super::expr::LiteralValue::Boolean(false),
                span,
            });
        }

        // Null
        if trimmed.eq_ignore_ascii_case("null") {
            return Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span,
            });
        }

        // Number
        if let Ok(n) = trimmed.parse::<f64>() {
            return Expression::Literal(Literal {
                value: super::expr::LiteralValue::Number(n),
                span,
            });
        }

        // String
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            let s = &trimmed[1..trimmed.len() - 1];
            return Expression::Literal(Literal {
                value: super::expr::LiteralValue::String(s.to_string()),
                span,
            });
        }

        // Member access
        if trimmed.contains('.') {
            let parts: Vec<&str> = trimmed.split('.').collect();
            return Expression::MemberAccess(MemberAccess {
                root: parts[0].to_string(),
                path: parts[1..].iter().map(|s| s.to_string()).collect(),
                span,
            });
        }

        // Identifier
        Expression::Identifier(Identifier {
            name: trimmed.to_string(),
            span,
        })
    }

    fn build_method_call_action(&self, pair: Pair<'_, Rule>) -> BuildResult<Action> {
        let span = span_from_pair(&pair);
        let mut namespace = String::new();
        let mut method = String::new();
        let mut arguments: Vec<Argument> = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::namespace => {
                    namespace = inner.as_str().to_lowercase();
                }
                Rule::method_name => {
                    method = inner.as_str().to_string();
                }
                Rule::argument_list => {
                    arguments = self.build_argument_list(inner)?;
                }
                _ => {}
            }
        }

        // Convert Vec<Argument> to Vec<Expression> for actions that need it
        let args_as_expressions: Vec<Expression> = arguments.iter().map(|a| a.value.clone()).collect();

        // Handle special namespaces
        match namespace.as_str() {
            "fetch" => {
                return Ok(Action::Fetch(FetchAction {
                    url: args_as_expressions.first().cloned(),
                    method: None,
                    body: None,
                    headers: None,
                    span,
                }));
            }
            "submit" => {
                return Ok(Action::Submit(SubmitAction {
                    target: args_as_expressions.first().cloned(),
                    data: None,
                    span,
                }));
            }
            _ => {}
        }

        Ok(Action::MethodCall(MethodCall {
            namespace,
            method,
            arguments,
            span,
        }))
    }

    /// Build argument list from grammar pairs
    fn build_argument_list(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<Argument>> {
        let mut args = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::argument => {
                    args.push(self.build_argument_value(inner)?);
                }
                Rule::named_argument => {
                    args.push(self.build_named_argument_value(inner)?);
                }
                Rule::expression => {
                    args.push(Argument {
                        name: None,
                        value: self.build_expression(inner)?,
                        spread: false,
                    });
                }
                _ => {}
            }
        }

        Ok(args)
    }

    /// Build a single argument
    fn build_argument_value(&self, pair: Pair<'_, Rule>) -> BuildResult<Argument> {
        let inner = pair.into_inner().next();
        match inner {
            Some(p) => match p.as_rule() {
                Rule::named_argument => self.build_named_argument_value(p),
                Rule::expression => Ok(Argument {
                    name: None,
                    value: self.build_expression(p)?,
                    spread: false,
                }),
                _ => Ok(Argument {
                    name: None,
                    value: self.parse_simple_value(p.as_str(), span_from_pair(&p)),
                    spread: false,
                }),
            },
            None => Err(BuildError::missing_required(
                "argument value",
                Span::default(),
            )),
        }
    }

    /// Build a named argument (name: value)
    fn build_named_argument_value(&self, pair: Pair<'_, Rule>) -> BuildResult<Argument> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut value: Option<Expression> = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::expression => {
                    value = Some(self.build_expression(inner)?);
                }
                _ => {}
            }
        }

        Ok(Argument {
            name: Some(name),
            value: value.unwrap_or(Expression::Literal(Literal {
                value: super::expr::LiteralValue::Null,
                span,
            })),
            spread: false,
        })
    }

    /// Helper to extract string from expression if possible
    #[allow(dead_code)]
    fn expr_to_string(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Literal(lit) => match &lit.value {
                super::expr::LiteralValue::String(s) => Some(s.clone()),
                _ => None,
            },
            Expression::Identifier(id) => Some(id.name.clone()),
            _ => None,
        }
    }

    fn build_action_with_handlers(&self, pair: Pair<'_, Rule>) -> BuildResult<ActionWithHandlers> {
        let span = span_from_pair(&pair);
        let mut action = None;
        let mut handlers = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::method_call => {
                    action = Some(self.build_method_call_action(inner)?);
                }
                Rule::response_handler => {
                    handlers.push(self.build_response_handler(inner)?);
                }
                _ => {}
            }
        }

        let action = action.unwrap_or(Action::Custom(CustomAction {
            name: "unknown".to_string(),
            params: vec![],
            span: span.clone(),
        }));

        Ok(ActionWithHandlers {
            action,
            handlers,
            span,
        })
    }

    fn build_response_handler(&self, pair: Pair<'_, Rule>) -> BuildResult<ResponseHandler> {
        let span = span_from_pair(&pair);
        let mut handler_type = ResponseHandlerType::Success;
        let mut actions = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::handler_type => {
                    let text = inner.as_str().to_lowercase();
                    handler_type = match text.as_str() {
                        "error" => ResponseHandlerType::Error,
                        "finally" => ResponseHandlerType::Finally,
                        _ => ResponseHandlerType::Success,
                    };
                }
                Rule::action_block => {
                    actions = self.build_action_items(inner)?;
                }
                _ => {}
            }
        }

        Ok(ResponseHandler {
            handler_type,
            actions,
            span,
        })
    }

    // ========================================================================
    // TEMPLATE BLOCK BUILDING
    // ========================================================================

    fn build_template_block(&self, pair: Pair<'_, Rule>) -> BuildResult<TemplateBlock> {
        let span = span_from_pair(&pair);
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::component => {
                    let comp = self.build_component(inner)?;
                    // Convert Slot components to SlotDefinition
                    if comp.name == "Slot" {
                        let name = comp.attributes.iter()
                            .find(|a| a.name == "name")
                            .and_then(|a| {
                                if let crate::ast::component::AttributeValue::String { value } = &a.value {
                                    Some(value.clone())
                                } else {
                                    None
                                }
                            });
                        
                        // If no name provided, set it to "default"
                        let name = Some(name.unwrap_or_else(|| "default".to_string()));
                        
                        content.push(TemplateContent::SlotDefinition(SlotDefinition {
                            name,
                            fallback: comp.children.clone(),
                            span: comp.span,
                        }));
                    } else {
                        content.push(TemplateContent::Component(comp));
                    }
                }
                Rule::control_flow => {
                    content.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                Rule::fragment_usage => {
                    content.push(TemplateContent::FragmentUsage(self.build_fragment_usage(inner)?));
                }
                Rule::line_comment => {
                    let comment_span = span_from_pair(&inner);
                    let value = inner.as_str().trim_start_matches("//").trim().to_string();
                    content.push(TemplateContent::Comment {
                        value,
                        span: comment_span,
                    });
                }
                Rule::text_content => {
                    // text_content can be plain_component_text or interpolated_text
                    if let Some(text_inner) = inner.clone().into_inner().next() {
                        match text_inner.as_rule() {
                            Rule::interpolated_text => {
                                // It's an expression inside {}, extract it
                                if let Some(expr_pair) = text_inner.into_inner().next() {
                                    if expr_pair.as_rule() == Rule::expression {
                                        let expr = self.build_expression(expr_pair)?;
                                        content.push(TemplateContent::Expression {
                                            expr: Box::new(expr),
                                            span: span_from_pair(&inner),
                                        });
                                        continue;
                                    }
                                }
                            }
                            Rule::plain_component_text => {
                                let text_span = span_from_pair(&inner);
                                let value = inner.as_str().to_string();
                                content.push(TemplateContent::Text {
                                    value,
                                    span: text_span,
                                });
                                continue;
                            }
                            _ => {}
                        }
                    }
                    // Fallback: treat as plain text
                    let text_span = span_from_pair(&inner);
                    let value = inner.as_str().to_string();
                    content.push(TemplateContent::Text {
                        value,
                        span: text_span,
                    });
                }
                Rule::line_comment => {
                    let comment_span = span_from_pair(&inner);
                    let value = inner.as_str().trim_start_matches("//").trim().to_string();
                    content.push(TemplateContent::Comment {
                        value,
                        span: comment_span,
                    });
                }
                _ => {}
            }
        }

        Ok(TemplateBlock { content, span })
    }

    fn build_component(&self, pair: Pair<'_, Rule>) -> BuildResult<Component> {
        let _span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::self_closing_component => self.build_self_closing_component(inner),
            Rule::component_with_children => self.build_component_with_children(inner),
            _ => {
                // Handle generated component rules
                self.build_generated_component(inner)
            }
        }
    }

    fn build_self_closing_component(&self, pair: Pair<'_, Rule>) -> BuildResult<Component> {
        let _span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        // This could be a generated component or fragment
        self.build_generated_component(inner)
    }

    fn build_component_with_children(&self, pair: Pair<'_, Rule>) -> BuildResult<Component> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut events = Vec::new();
        let mut children = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::component_children => {
                    children = self.build_component_children(inner)?;
                }
                Rule::component_closing_tag => {
                    // Ignore closing tag
                }
                _ => {
                    // Opening component - extract name, attributes, events, and possibly children
                    // (for fragment_with_content parsed as component)
                    let comp = self.build_generated_component(inner)?;
                    name = comp.name;
                    attributes = comp.attributes;
                    events = comp.events;
                    // If the generated component already has children (from fragment_as_component),
                    // use those
                    if !comp.children.is_empty() {
                        children = comp.children;
                    }
                }
            }
        }
        
        // Deduplicate attributes (keep last occurrence)
        attributes = self.dedupe_attributes(attributes);

        Ok(Component {
            name,
            attributes,
            events,
            children,
            self_closing: false,
            slot: None,
            span,
        })
    }

    fn build_generated_component(&self, pair: Pair<'_, Rule>) -> BuildResult<Component> {
        let span = span_from_pair(&pair);
        let text = pair.as_str();
        let rule_name = format!("{:?}", pair.as_rule());

        // Handle fragment_with_content or fragment_self_closing parsed as component
        // These are components without specific grammar rules (like CardHeader, CardContent)
        if matches!(
            pair.as_rule(),
            Rule::fragment_with_content | Rule::fragment_self_closing
        ) {
            return self.build_fragment_as_component(pair);
        }

        // Extract component name from the rule name or first identifier
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut events = Vec::new();

        // Try to get component name from rule name (e.g., "ContainerSelfClosing" -> "Container")
        // Common suffixes to strip: SelfClosing, Opening, WithChildren
        for suffix in ["SelfClosing", "Opening", "WithChildren"] {
            if let Some(stripped) = rule_name.strip_suffix(suffix) {
                name = stripped.to_string();
                break;
            }
        }

        // Fallback: get name from text (e.g., "<Button ..." -> "Button")
        if name.is_empty() {
            if let Some(stripped) = text.strip_prefix('<') {
                let name_end = stripped
                    .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                    .unwrap_or(stripped.len());
                name = stripped[..name_end].to_string();
            }
        }

        // Parse attributes and events from inner pairs
        // Use a helper function to recurse into nested rules
        self.collect_component_parts(pair, &mut name, &mut attributes, &mut events);
        
        // Deduplicate attributes (keep last occurrence)
        attributes = self.dedupe_attributes(attributes);

        Ok(Component {
            name,
            attributes,
            events,
            children: vec![],
            self_closing: true,
            slot: None,
            span,
        })
    }

    /// Deduplicate attributes, keeping the last occurrence of each attribute name
    fn dedupe_attributes(&self, attributes: Vec<Attribute>) -> Vec<Attribute> {
        use std::collections::HashMap;
        
        // Use HashMap to track last occurrence by attribute name
        let mut attr_map: HashMap<String, Attribute> = HashMap::new();
        
        for attr in attributes {
            // Store/overwrite with the last occurrence
            attr_map.insert(attr.name.clone(), attr);
        }
        
        // Convert back to Vec (order may differ but that's acceptable)
        attr_map.into_values().collect()
    }

    /// Recursively collect component name, attributes, and events from a pair tree
    fn collect_component_parts(
        &self,
        pair: Pair<'_, Rule>,
        name: &mut String,
        attributes: &mut Vec<Attribute>,
        events: &mut Vec<EventBinding>,
    ) {
        for inner in pair.into_inner() {
            let rule_str = format!("{:?}", inner.as_rule());

            // Check for *AttributeDefinition rules
            if rule_str.ends_with("AttributeDefinition") {
                if let Ok(attr) = self.build_component_attribute(inner) {
                    attributes.push(attr);
                }
            }
            // Check for *EventsDefinition rules
            else if rule_str.ends_with("EventsDefinition") {
                if let Ok(event) = self.build_component_event(inner) {
                    events.push(event);
                }
            }
            // Check for component name rules
            else if rule_str.ends_with("ComponentNames") {
                *name = inner.as_str().to_string();
            }
            // Recurse into wrapper rules (like OpeningComponents -> CardOpening)
            else if rule_str.ends_with("Opening")
                || rule_str.ends_with("SelfClosing")
                || rule_str.ends_with("Components")
            {
                // This is a wrapper/nested component rule, recurse into it
                self.collect_component_parts(inner, name, attributes, events);
            }
        }
    }

    /// Build a fragment parsed as a component (e.g., CardHeader, CardContent)
    /// These are components without specific grammar rules, parsed as fragment_with_content
    fn build_fragment_as_component(&self, pair: Pair<'_, Rule>) -> BuildResult<Component> {
        let span = span_from_pair(&pair);
        let is_self_closing = matches!(pair.as_rule(), Rule::fragment_self_closing);
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut events = Vec::new();
        let mut children = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::fragment_usage_name => {
                    name = inner.as_str().to_string();
                }
                Rule::fragment_usage_attributes => {
                    for attr in inner.into_inner() {
                        match attr.as_rule() {
                            Rule::fragment_property_binding => {
                                if let Ok(a) = self.build_attribute(attr) {
                                    attributes.push(a);
                                }
                            }
                            Rule::fragment_event_binding => {
                                if let Ok(e) = self.build_event_binding(attr) {
                                    events.push(e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Rule::fragment_slot_content => {
                    // Build fragment slot content as children
                    // Important: extend instead of replace to accumulate all children
                    let content_list = self.build_template_content_list(inner)?;
                    children.extend(content_list);
                }
                _ => {}
            }
        }
        
        // Deduplicate attributes (keep last occurrence)
        attributes = self.dedupe_attributes(attributes);

        Ok(Component {
            name,
            attributes,
            events,
            children,
            self_closing: is_self_closing,
            slot: None,
            span,
        })
    }

    /// Build a component attribute from a *AttributeDefinition rule
    fn build_component_attribute(&self, pair: Pair<'_, Rule>) -> BuildResult<Attribute> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut value = AttributeValue::String {
            value: String::new(),
        };

        for inner in pair.into_inner() {
            let rule_str = format!("{:?}", inner.as_rule());

            // The first child is the attribute name (e.g., ContainerAttributes -> "className")
            if rule_str.ends_with("Attributes") || inner.as_rule() == Rule::identifier {
                name = inner.as_str().to_lowercase();
                // Normalize attribute names
                name = self.normalize_attribute_name(&name);
            }
            // The second child is the attribute value
            else if inner.as_rule() == Rule::attribute_value {
                value = self.build_attribute_value(inner)?;
            }
        }

        Ok(Attribute {
            name,
            value,
            original_name: None,
            span,
        })
    }

    /// Normalize an attribute name (e.g., "classname" -> "className", "onclick" -> "onClick")
    fn normalize_attribute_name(&self, name: &str) -> String {
        // Handle common attribute name normalizations
        let normalized = name.to_lowercase();
        match normalized.as_str() {
            "classname" | "class-name" | "class_name" => "className".to_string(),
            "onclick" | "on-click" | "on_click" => "onClick".to_string(),
            "onchange" | "on-change" | "on_change" => "onChange".to_string(),
            "onsubmit" | "on-submit" | "on_submit" => "onSubmit".to_string(),
            "fieldtype" | "field-type" | "field_type" => "fieldType".to_string(),
            "bindto" | "bind-to" | "bind_to" => "bindTo".to_string(),
            "datasource" | "data-source" | "data_source" => "dataSource".to_string(),
            "defaultvalue" | "default-value" | "default_value" => "defaultValue".to_string(),
            _ => normalized,
        }
    }

    /// Build an attribute value from the attribute_value rule
    fn build_attribute_value(&self, pair: Pair<'_, Rule>) -> BuildResult<AttributeValue> {
        let inner = pair.into_inner().next();
        match inner {
            Some(p) => match p.as_rule() {
                Rule::quoted_string => {
                    let s = self.extract_string_content(p);
                    Ok(AttributeValue::String { value: s })
                }
                Rule::expression_value => {
                    // expression_value = { "{" ~ expression ~ "}" }
                    let expr = p
                        .into_inner()
                        .find(|inner| inner.as_rule() == Rule::expression);
                    match expr {
                        Some(e) => Ok(AttributeValue::Expression {
                            value: self.build_expression(e)?,
                        }),
                        None => Ok(AttributeValue::String {
                            value: String::new(),
                        }),
                    }
                }
                _ => {
                    // Fallback: treat as string
                    Ok(AttributeValue::String {
                        value: p.as_str().to_string(),
                    })
                }
            },
            None => Ok(AttributeValue::String {
                value: String::new(),
            }),
        }
    }

    /// Build a component event from a *EventsDefinition rule
    fn build_component_event(&self, pair: Pair<'_, Rule>) -> BuildResult<EventBinding> {
        let span = span_from_pair(&pair);
        let mut event_name = String::new();
        let mut actions = Vec::new();

        for inner in pair.into_inner() {
            let rule_str = format!("{:?}", inner.as_rule());

            // Get the event name (e.g., ContainerEvents -> "click")
            if rule_str.ends_with("Events") {
                event_name = inner.as_str().to_lowercase();
            }
            // Get the action body
            else if inner.as_rule() == Rule::action_body {
                actions = self.build_action_items(inner)?;
            }
        }

        // Convert actions to an arrow function body if we have actions
        let handler_type = if !actions.is_empty() {
            // Create an arrow function with the actions as statements
            let statements: Vec<super::expr::ArrowStatement> = actions
                .iter()
                .filter_map(|item| match item {
                    ActionItem::Simple(action) => Some(super::expr::ArrowStatement::Expression(
                        action.to_expression(),
                    )),
                    ActionItem::WithHandlers(_) => None,
                })
                .collect();

            HandlerType::Arrow(ArrowFunction {
                params: vec![],
                body: super::expr::ArrowBody::Block(statements),
                span: span.clone(),
            })
        } else {
            HandlerType::Identifier("handler".to_string())
        };

        let handler = EventHandler {
            handler_type,
            modifiers: vec![],
        };

        Ok(EventBinding {
            event: event_name,
            handler,
            span,
        })
    }

    fn build_component_children(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<TemplateContent>> {
        let mut children = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::component => {
                    let comp = self.build_component(inner)?;
                    // Convert Slot components to SlotDefinition
                    if comp.name == "Slot" {
                        let name = comp.attributes.iter()
                            .find(|a| a.name == "name")
                            .and_then(|a| {
                                if let crate::ast::component::AttributeValue::String { value } = &a.value {
                                    Some(value.clone())
                                } else {
                                    None
                                }
                            });
                        
                        // If no name provided, set it to "default"
                        let name = Some(name.unwrap_or_else(|| "default".to_string()));
                        
                        children.push(TemplateContent::SlotDefinition(SlotDefinition {
                            name,
                            fallback: comp.children.clone(),
                            span: comp.span,
                        }));
                    } else {
                        children.push(TemplateContent::Component(comp));
                    }
                }
                Rule::control_flow => {
                    children.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                Rule::fragment_usage => {
                    children
                        .push(TemplateContent::FragmentUsage(self.build_fragment_usage(inner)?));
                }
                Rule::text_content => {
                    // text_content can be plain_component_text or interpolated_text
                    if let Some(text_inner) = inner.clone().into_inner().next() {
                        match text_inner.as_rule() {
                            Rule::interpolated_text => {
                                // It's an expression inside {}, extract it
                                if let Some(expr_pair) = text_inner.into_inner().next() {
                                    if expr_pair.as_rule() == Rule::expression {
                                        // Convert to TemplateContent::Expression
                                        let expr = self.build_expression(expr_pair)?;
                                        children.push(TemplateContent::Expression {
                                            expr: Box::new(expr),
                                            span: span_from_pair(&inner),
                                        });
                                        continue;
                                    }
                                }
                            }
                            Rule::plain_component_text => {
                                // Plain text, use as is
                                let text_span = span_from_pair(&inner);
                                let value = inner.as_str().to_string();
                                children.push(TemplateContent::Text {
                                    value,
                                    span: text_span,
                                });
                                continue;
                            }
                            _ => {}
                        }
                    }
                    // Fallback: treat as plain text
                    let text_span = span_from_pair(&inner);
                    let value = inner.as_str().to_string();
                    children.push(TemplateContent::Text {
                        value,
                        span: text_span,
                    });
                }
                Rule::line_comment => {
                    let comment_span = span_from_pair(&inner);
                    let value = inner.as_str().trim_start_matches("//").trim().to_string();
                    children.push(TemplateContent::Comment {
                        value,
                        span: comment_span,
                    });
                }
                _ => {}
            }
        }

        Ok(children)
    }

    fn build_attribute(&self, pair: Pair<'_, Rule>) -> BuildResult<Attribute> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut value = AttributeValue::String {
            value: String::new(),
        };

        // Try to iterate grammar pairs first
        let mut found_parts = false;
        for inner in pair.clone().into_inner() {
            found_parts = true;
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::attribute_value => {
                    value = self.build_attribute_value(inner)?;
                }
                _ => {}
            }
        }

        // If we found grammar parts, return the attribute
        if found_parts && !name.is_empty() {
            return Ok(Attribute {
                name,
                value,
                original_name: None,
                span,
            });
        }

        // Fallback: Simple text-based attribute parsing for legacy support
        let text = pair.as_str();
        let parts: Vec<&str> = text.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(BuildError::missing_required("attribute value", span));
        }

        name = parts[0].trim().to_string();
        let value_str = parts[1].trim();

        value = if value_str.starts_with('{') && value_str.ends_with('}') {
            // Expression value
            let expr_str = &value_str[1..value_str.len() - 1];
            AttributeValue::Expression {
                value: self.parse_simple_value(expr_str, span.clone()),
            }
        } else if value_str.starts_with('"') && value_str.ends_with('"') {
            // String value
            let s = &value_str[1..value_str.len() - 1];
            AttributeValue::String { value: s.to_string() }
        } else {
            // Treat as identifier reference
            AttributeValue::Expression {
                value: Expression::Identifier(Identifier {
                    name: value_str.to_string(),
                    span: span.clone(),
                }),
            }
        };

        Ok(Attribute {
            name,
            value,
            original_name: None,
            span,
        })
    }

    fn build_event_binding(&self, pair: Pair<'_, Rule>) -> BuildResult<EventBinding> {
        let span = span_from_pair(&pair);
        let text = pair.as_str();

        // Parse @eventName={handler} or @eventName => { actions }
        let name = text
            .trim_start_matches('@')
            .split(|c| c == '=' || c == ' ')
            .next()
            .unwrap_or("click")
            .to_string();

        let handler = EventHandler {
            handler_type: HandlerType::Identifier("handler".to_string()),
            modifiers: vec![],
        };

        Ok(EventBinding {
            event: name,
            handler,
            span,
        })
    }

    // ========================================================================
    // CONTROL FLOW BUILDING
    // ========================================================================

    fn build_control_flow(&self, pair: Pair<'_, Rule>) -> BuildResult<ControlFlow> {
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::if_block => Ok(ControlFlow::If(self.build_if_block(inner)?)),
            Rule::for_block => Ok(ControlFlow::For(self.build_for_block(inner)?)),
            Rule::when_block => Ok(ControlFlow::When(self.build_when_block(inner)?)),
            _ => {
                let span = span_from_pair(&inner);
                Err(BuildError::unsupported("control flow type", span))
            }
        }
    }

    fn build_if_block(&self, pair: Pair<'_, Rule>) -> BuildResult<IfBlock> {
        let span = span_from_pair(&pair);
        let mut condition = Expression::Literal(Literal {
            value: super::expr::LiteralValue::Boolean(true),
            span: span.clone(),
        });
        let mut then_branch = Vec::new();
        let mut else_if_branches = Vec::new();
        let mut else_branch: Option<Vec<TemplateContent>> = None;

        // The if_block grammar is flat: If ~ expression ~ "{" ~ template_content* ~ "}" ~ ...
        // We need to parse the sequence of pairs to identify:
        // 1. First expression + template_content* = main if
        // 2. Additional expression + template_content* after "else if" = else-if branches
        // 3. Template_content* after final "else" = else branch
        let mut collecting_for = CollectingFor::Main;
        let mut current_expr: Option<Expression> = None;
        let mut current_content: Vec<TemplateContent> = Vec::new();
        let mut saw_else = false;
        let mut in_final_else = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::If => {
                    // If we see If after seeing Else, it's an "else if"
                    if saw_else {
                        // Finalize previous branch before starting else-if
                        if let Some(expr) = current_expr.take() {
                            match collecting_for {
                                CollectingFor::Main => {
                                    condition = expr;
                                    then_branch = std::mem::take(&mut current_content);
                                }
                                CollectingFor::ElseIf => {
                                    else_if_branches.push(ElseIfBranch {
                                        condition: expr,
                                        body: std::mem::take(&mut current_content),
                                        span: span.clone(),
                                    });
                                }
                                CollectingFor::Else => {}
                            }
                        }
                        collecting_for = CollectingFor::ElseIf;
                        saw_else = false;
                    }
                }
                Rule::Else => {
                    saw_else = true;
                }
                Rule::expression => {
                    // Seeing expression clears the saw_else flag (this is an else-if expression)
                    saw_else = false;
                    
                    // When we see a new expression, finalize the previous branch
                    if let Some(expr) = current_expr.take() {
                        match collecting_for {
                            CollectingFor::Main => {
                                condition = expr;
                                then_branch = std::mem::take(&mut current_content);
                            }
                            CollectingFor::ElseIf => {
                                else_if_branches.push(ElseIfBranch {
                                    condition: expr,
                                    body: std::mem::take(&mut current_content),
                                    span: span.clone(),
                                });
                            }
                            CollectingFor::Else => {
                                // Shouldn't happen - else doesn't have expression
                            }
                        }
                    }
                    current_expr = Some(self.build_expression(inner)?);
                }
                Rule::template_content => {
                    // If we saw "else" and haven't seen another expression, we're in the final else branch
                    if saw_else && !in_final_else {
                        // Finalize the previous branch (main if or last else-if)
                        if let Some(expr) = current_expr.take() {
                            match collecting_for {
                                CollectingFor::Main => {
                                    condition = expr;
                                    then_branch = std::mem::take(&mut current_content);
                                }
                                CollectingFor::ElseIf => {
                                    else_if_branches.push(ElseIfBranch {
                                        condition: expr,
                                        body: std::mem::take(&mut current_content),
                                        span: span.clone(),
                                    });
                                }
                                CollectingFor::Else => {}
                            }
                        }
                        collecting_for = CollectingFor::Else;
                        in_final_else = true;
                        saw_else = false;
                    }
                    
                    // template_content contains: component | control_flow | line_comment
                    if let Some(child) = inner.into_inner().next() {
                        let content = match child.as_rule() {
                            Rule::component => {
                                TemplateContent::Component(self.build_component(child)?)
                            }
                            Rule::control_flow => {
                                TemplateContent::ControlFlow(self.build_control_flow(child)?)
                            }
                            Rule::line_comment => {
                                let comment_span = span_from_pair(&child);
                                let value =
                                    child.as_str().trim_start_matches("//").trim().to_string();
                                TemplateContent::Comment {
                                    value,
                                    span: comment_span,
                                }
                            }
                            _ => continue,
                        };
                        current_content.push(content);
                    }
                }
                Rule::component => {
                    // Same logic for direct component rules
                    if saw_else && !in_final_else {
                        if let Some(expr) = current_expr.take() {
                            match collecting_for {
                                CollectingFor::Main => {
                                    condition = expr;
                                    then_branch = std::mem::take(&mut current_content);
                                }
                                CollectingFor::ElseIf => {
                                    else_if_branches.push(ElseIfBranch {
                                        condition: expr,
                                        body: std::mem::take(&mut current_content),
                                        span: span.clone(),
                                    });
                                }
                                CollectingFor::Else => {}
                            }
                        }
                        collecting_for = CollectingFor::Else;
                        in_final_else = true;
                        saw_else = false;
                    }
                    current_content
                        .push(TemplateContent::Component(self.build_component(inner)?));
                }
                Rule::control_flow => {
                    // Same logic for direct control_flow rules
                    if saw_else && !in_final_else {
                        if let Some(expr) = current_expr.take() {
                            match collecting_for {
                                CollectingFor::Main => {
                                    condition = expr;
                                    then_branch = std::mem::take(&mut current_content);
                                }
                                CollectingFor::ElseIf => {
                                    else_if_branches.push(ElseIfBranch {
                                        condition: expr,
                                        body: std::mem::take(&mut current_content),
                                        span: span.clone(),
                                    });
                                }
                                CollectingFor::Else => {}
                            }
                        }
                        collecting_for = CollectingFor::Else;
                        in_final_else = true;
                        saw_else = false;
                    }
                    current_content
                        .push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                _ => {
                    // Other tokens we don't need to handle
                }
            }
        }

        // Finalize the last branch
        if let Some(expr) = current_expr {
            match collecting_for {
                CollectingFor::Main => {
                    condition = expr;
                    then_branch = current_content;
                }
                CollectingFor::ElseIf => {
                    // This is an else-if branch
                    else_if_branches.push(ElseIfBranch {
                        condition: expr,
                        body: current_content,
                        span: span.clone(),
                    });
                }
                CollectingFor::Else => {
                    // This shouldn't happen - else doesn't have expression
                    // But if it does, just put content in else_branch
                    if !current_content.is_empty() {
                        else_branch = Some(current_content);
                    }
                }
            }
        } else if collecting_for == CollectingFor::Else && !current_content.is_empty() {
            // We're in the else branch (no expression)
            else_branch = Some(current_content);
        } else if !current_content.is_empty() {
            // Content for the then branch (main if)
            then_branch = current_content;
        }

        Ok(IfBlock {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
            span,
        })
    }

    fn build_for_block(&self, pair: Pair<'_, Rule>) -> BuildResult<ForBlock> {
        let span = span_from_pair(&pair);
        let mut binding = ForBinding {
            item: Identifier {
                name: "item".to_string(),
                span: span.clone(),
            },
            index: None,
            span: span.clone(),
        };
        let mut iterable = Expression::Identifier(Identifier {
            name: "items".to_string(),
            span: span.clone(),
        });
        let mut body = Vec::new();

        // The for_block grammar is: For ~ for_binding ~ In ~ expression ~ "{" ~ template_content* ~ "}"
        // So we get for_binding, expression, and template_content children
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::for_binding => {
                    binding = self.build_for_binding(inner)?;
                }
                Rule::expression => {
                    iterable = self.build_expression(inner)?;
                }
                Rule::template_content => {
                    // template_content contains: component | control_flow | line_comment
                    if let Some(child) = inner.into_inner().next() {
                        let content = match child.as_rule() {
                            Rule::component => {
                                TemplateContent::Component(self.build_component(child)?)
                            }
                            Rule::control_flow => {
                                TemplateContent::ControlFlow(self.build_control_flow(child)?)
                            }
                            Rule::line_comment => {
                                let comment_span = span_from_pair(&child);
                                let value =
                                    child.as_str().trim_start_matches("//").trim().to_string();
                                TemplateContent::Comment {
                                    value,
                                    span: comment_span,
                                }
                            }
                            _ => continue,
                        };
                        body.push(content);
                    }
                }
                Rule::component => {
                    body.push(TemplateContent::Component(self.build_component(inner)?));
                }
                Rule::control_flow => {
                    body.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                _ => {}
            }
        }

        Ok(ForBlock {
            binding,
            iterable,
            body,
            span,
        })
    }

    fn build_for_binding(&self, pair: Pair<'_, Rule>) -> BuildResult<ForBinding> {
        let span = span_from_pair(&pair);
        let mut item = Identifier {
            name: "item".to_string(),
            span: span.clone(),
        };
        let mut index = None;
        let mut found_item = false;

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::identifier {
                if !found_item {
                    // First identifier is the item
                    item = Identifier {
                        name: inner.as_str().to_string(),
                        span: span_from_pair(&inner),
                    };
                    found_item = true;
                } else {
                    // Second identifier is the index
                    index = Some(Identifier {
                        name: inner.as_str().to_string(),
                        span: span_from_pair(&inner),
                    });
                }
            }
        }

        Ok(ForBinding { item, index, span })
    }

    fn build_when_block(&self, pair: Pair<'_, Rule>) -> BuildResult<WhenBlock> {
        let span = span_from_pair(&pair);
        let mut subject = Expression::Identifier(Identifier {
            name: "value".to_string(),
            span: span.clone(),
        });
        let mut arms = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::expression => {
                    subject = self.build_expression(inner)?;
                }
                Rule::when_arm => {
                    arms.push(self.build_when_arm(inner)?);
                }
                _ => {}
            }
        }

        Ok(WhenBlock {
            subject,
            arms,
            span,
        })
    }

    fn build_when_arm(&self, pair: Pair<'_, Rule>) -> BuildResult<WhenArm> {
        let span = span_from_pair(&pair);
        let mut pattern = WhenPattern::Wildcard { span: span.clone() };
        let mut body = Vec::new();

        // when_arm = { (when_pattern | Else) ~ "=>" ~ ("{" ~ template_content* ~ "}" | component) }
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::when_pattern => {
                    pattern = self.build_when_pattern(inner)?;
                }
                Rule::template_content => {
                    // template_content contains: component | control_flow | line_comment
                    if let Some(child) = inner.into_inner().next() {
                        let content = match child.as_rule() {
                            Rule::component => {
                                TemplateContent::Component(self.build_component(child)?)
                            }
                            Rule::control_flow => {
                                TemplateContent::ControlFlow(self.build_control_flow(child)?)
                            }
                            Rule::line_comment => {
                                let comment_span = span_from_pair(&child);
                                let value =
                                    child.as_str().trim_start_matches("//").trim().to_string();
                                TemplateContent::Comment {
                                    value,
                                    span: comment_span,
                                }
                            }
                            _ => continue,
                        };
                        body.push(content);
                    }
                }
                Rule::component => {
                    body.push(TemplateContent::Component(self.build_component(inner)?));
                }
                Rule::control_flow => {
                    body.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                _ => {
                    // Could be Else keyword - that means wildcard pattern
                    let text = inner.as_str().to_lowercase();
                    if text == "else" {
                        pattern = WhenPattern::Wildcard { span: span.clone() };
                    }
                }
            }
        }

        Ok(WhenArm {
            pattern,
            guard: None, // Guards not in grammar
            body,
            span,
        })
    }

    fn build_when_pattern(&self, pair: Pair<'_, Rule>) -> BuildResult<WhenPattern> {
        let span = span_from_pair(&pair);
        let text = pair.as_str().trim();

        // Check for wildcard
        if text == "_" {
            return Ok(WhenPattern::Wildcard { span });
        }

        // Check for literal patterns
        if text.starts_with('"') || text.starts_with('\'') {
            let s = &text[1..text.len() - 1];
            return Ok(WhenPattern::Literal {
                value: Expression::Literal(Literal {
                    value: super::expr::LiteralValue::String(s.to_string()),
                    span: span.clone(),
                }),
                span,
            });
        }

        if text.eq_ignore_ascii_case("true") || text.eq_ignore_ascii_case("false") {
            return Ok(WhenPattern::Literal {
                value: Expression::Literal(Literal {
                    value: super::expr::LiteralValue::Boolean(
                        text.eq_ignore_ascii_case("true"),
                    ),
                    span: span.clone(),
                }),
                span,
            });
        }

        if let Ok(n) = text.parse::<f64>() {
            return Ok(WhenPattern::Literal {
                value: Expression::Literal(Literal {
                    value: super::expr::LiteralValue::Number(n),
                    span: span.clone(),
                }),
                span,
            });
        }

        // Identifier pattern (binding)
        Ok(WhenPattern::Binding {
            name: text.to_string(),
            span,
        })
    }

    fn build_template_content_list(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<TemplateContent>> {
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::component => {
                    let comp = self.build_component(inner)?;
                    // Convert Slot components to SlotDefinition
                    if comp.name == "Slot" {
                        let name = comp.attributes.iter()
                            .find(|a| a.name == "name")
                            .and_then(|a| {
                                if let crate::ast::component::AttributeValue::String { value } = &a.value {
                                    Some(value.clone())
                                } else {
                                    None
                                }
                            });
                        
                        // If no name provided, set it to "default"
                        let name = Some(name.unwrap_or_else(|| "default".to_string()));
                        
                        content.push(TemplateContent::SlotDefinition(SlotDefinition {
                            name,
                            fallback: comp.children.clone(),
                            span: comp.span,
                        }));
                    } else {
                        content.push(TemplateContent::Component(comp));
                    }
                }
                Rule::control_flow => {
                    content.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                Rule::fragment_usage => {
                    content.push(TemplateContent::FragmentUsage(self.build_fragment_usage(inner)?));
                }
                Rule::text_content => {
                    // text_content can be plain_component_text or interpolated_text
                    if let Some(text_inner) = inner.clone().into_inner().next() {
                        match text_inner.as_rule() {
                            Rule::interpolated_text => {
                                // It's an expression inside {}, extract it
                                if let Some(expr_pair) = text_inner.into_inner().next() {
                                    if expr_pair.as_rule() == Rule::expression {
                                        let expr = self.build_expression(expr_pair)?;
                                        content.push(TemplateContent::Expression {
                                            expr: Box::new(expr),
                                            span: span_from_pair(&inner),
                                        });
                                        continue;
                                    }
                                }
                            }
                            Rule::plain_component_text => {
                                let text_span = span_from_pair(&inner);
                                let value = inner.as_str().to_string();
                                content.push(TemplateContent::Text {
                                    value,
                                    span: text_span,
                                });
                                continue;
                            }
                            _ => {}
                        }
                    }
                    // Fallback: treat as plain text
                    let text_span = span_from_pair(&inner);
                    let value = inner.as_str().to_string();
                    content.push(TemplateContent::Text {
                        value,
                        span: text_span,
                    });
                }
                Rule::line_comment => {
                    let comment_span = span_from_pair(&inner);
                    let value = inner.as_str().trim_start_matches("//").trim().to_string();
                    content.push(TemplateContent::Comment {
                        value,
                        span: comment_span,
                    });
                }
                _ => {}
            }
        }

        Ok(content)
    }

    // ========================================================================
    // FRAGMENT BUILDING
    // ========================================================================

    fn build_fragment_definition(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentDefinition> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut params = Vec::new();
        let mut body = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::fragment_name => {
                    name = inner.as_str().to_string();
                }
                Rule::fragment_params => {
                    params = self.build_fragment_params(inner)?;
                }
                Rule::fragment_body => {
                    body = self.build_fragment_body(inner)?;
                }
                _ => {}
            }
        }

        Ok(FragmentDefinition {
            name,
            params,
            body,
            exported: false,
            is_default: false,
            span,
        })
    }

    fn build_fragment_params(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<FragmentParam>> {
        let mut params = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::fragment_param {
                params.push(self.build_fragment_param(inner)?);
            }
        }

        Ok(params)
    }

    fn build_fragment_param(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentParam> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::fragment_property_param => self.build_fragment_property_param(inner),
            Rule::fragment_event_param => self.build_fragment_event_param(inner),
            _ => {
                Err(BuildError::unsupported("fragment param type", span))
            }
        }
    }

    fn build_fragment_property_param(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentParam> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut type_annotation = None;
        let mut default = None;
        let mut required = true;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::type_annotation => {
                    type_annotation = Some(self.build_type_annotation(inner)?);
                }
                Rule::state_value => {
                    default = Some(self.build_state_value(inner)?);
                    required = false;
                }
                _ => {
                    // Check for optional marker "?"
                    if inner.as_str() == "?" {
                        required = false;
                    }
                }
            }
        }

        Ok(FragmentParam {
            name,
            type_annotation,
            default,
            required,
            span,
        })
    }

    fn build_fragment_event_param(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentParam> {
        let span = span_from_pair(&pair);
        let text = pair.as_str();
        let name = text.trim_start_matches('@').trim_end_matches('?').to_string();
        let required = !text.ends_with('?');

        Ok(FragmentParam {
            name: format!("@{}", name),
            type_annotation: None,
            default: None,
            required,
            span,
        })
    }

    fn build_fragment_body(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<TemplateContent>> {
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::component => {
                    let comp = self.build_component(inner)?;
                    // Convert Slot components to SlotDefinition
                    if comp.name == "Slot" {
                        let name = comp.attributes.iter()
                            .find(|a| a.name == "name")
                            .and_then(|a| {
                                if let crate::ast::component::AttributeValue::String { value } = &a.value {
                                    Some(value.clone())
                                } else {
                                    None
                                }
                            });
                        
                        // If no name provided, set it to "default"
                        let name = Some(name.unwrap_or_else(|| "default".to_string()));
                        
                        content.push(TemplateContent::SlotDefinition(SlotDefinition {
                            name,
                            fallback: comp.children.clone(),
                            span: comp.span,
                        }));
                    } else {
                        content.push(TemplateContent::Component(comp));
                    }
                }
                Rule::control_flow => {
                    content.push(TemplateContent::ControlFlow(self.build_control_flow(inner)?));
                }
                Rule::slot_definition => {
                    content.push(TemplateContent::SlotDefinition(self.build_slot_definition(inner)?));
                }
                Rule::fragment_usage => {
                    content.push(TemplateContent::FragmentUsage(self.build_fragment_usage(inner)?));
                }
                Rule::line_comment => {
                    let comment_span = span_from_pair(&inner);
                    let value = inner.as_str().trim_start_matches("//").trim().to_string();
                    content.push(TemplateContent::Comment {
                        value,
                        span: comment_span,
                    });
                }
                _ => {}
            }
        }

        Ok(content)
    }

    fn build_slot_definition(&self, pair: Pair<'_, Rule>) -> BuildResult<SlotDefinition> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::slot_self_closing => {
                let mut name = None;
                for p in inner.into_inner() {
                    if p.as_rule() == Rule::slot_name_attr {
                        name = Some(self.extract_slot_name(p));
                    }
                }
                Ok(SlotDefinition {
                    name,
                    fallback: vec![],
                    span,
                })
            }
            Rule::slot_with_fallback => {
                let mut name = None;
                let mut fallback = Vec::new();
                for p in inner.into_inner() {
                    match p.as_rule() {
                        Rule::slot_name_attr => {
                            name = Some(self.extract_slot_name(p));
                        }
                        Rule::fragment_body => {
                            fallback = self.build_fragment_body(p)?;
                        }
                        _ => {}
                    }
                }
                Ok(SlotDefinition {
                    name,
                    fallback,
                    span,
                })
            }
            _ => {
                Ok(SlotDefinition {
                    name: None,
                    fallback: vec![],
                    span,
                })
            }
        }
    }

    fn extract_slot_name(&self, pair: Pair<'_, Rule>) -> String {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::quoted_string {
                return self.extract_string_content(inner);
            }
        }
        String::new()
    }

    fn build_fragment_usage(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentUsage> {
        let span = span_from_pair(&pair);
        let inner = pair.into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::fragment_self_closing => self.build_fragment_self_closing(inner),
            Rule::fragment_with_content => self.build_fragment_with_content(inner),
            _ => {
                Ok(FragmentUsage {
                    name: "Unknown".to_string(),
                    properties: vec![],
                    events: vec![],
                    slot_content: vec![],
                    self_closing: true,
                    span,
                })
            }
        }
    }

    fn build_fragment_self_closing(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentUsage> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut properties = Vec::new();
        let mut events = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::fragment_usage_name => {
                    name = inner.as_str().to_string();
                }
                Rule::fragment_usage_attributes => {
                    for attr in inner.into_inner() {
                        match attr.as_rule() {
                            Rule::fragment_property_binding => {
                                if let Ok(a) = self.build_attribute(attr) {
                                    properties.push(a);
                                }
                            }
                            Rule::fragment_event_binding => {
                                if let Ok(e) = self.build_event_binding(attr) {
                                    events.push(e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(FragmentUsage {
            name,
            properties,
            events,
            slot_content: vec![],
            self_closing: true,
            span,
        })
    }

    fn build_fragment_with_content(&self, pair: Pair<'_, Rule>) -> BuildResult<FragmentUsage> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut properties = Vec::new();
        let mut events = Vec::new();
        let mut slot_content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::fragment_usage_name => {
                    name = inner.as_str().to_string();
                }
                Rule::fragment_usage_attributes => {
                    for attr in inner.into_inner() {
                        match attr.as_rule() {
                            Rule::fragment_property_binding => {
                                if let Ok(a) = self.build_attribute(attr) {
                                    properties.push(a);
                                }
                            }
                            Rule::fragment_event_binding => {
                                if let Ok(e) = self.build_event_binding(attr) {
                                    events.push(e);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Rule::fragment_slot_content => {
                    let content = self.build_template_content_list(inner)?;
                    if !content.is_empty() {
                        slot_content.push(SlotContent {
                            name: None, // Default slot
                            content,
                            span: span.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(FragmentUsage {
            name,
            properties,
            events,
            slot_content,
            self_closing: false,
            span,
        })
    }

    // ========================================================================
    // INTERFACE DEFINITION BUILDING
    // ========================================================================

    fn build_interface_definition(&self, pair: Pair<'_, Rule>) -> BuildResult<InterfaceDefinition> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut generics = Vec::new();
        let mut members = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::generic_params => {
                    generics = self.build_generic_params(inner)?;
                }
                Rule::interface_member => {
                    members.push(self.build_interface_member(inner)?);
                }
                _ => {}
            }
        }

        Ok(InterfaceDefinition {
            name,
            generics,
            members,
            span,
        })
    }

    fn build_generic_params(&self, pair: Pair<'_, Rule>) -> BuildResult<Vec<GenericParam>> {
        let mut params = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::generic_param {
                params.push(self.build_generic_param(inner)?);
            }
        }

        Ok(params)
    }

    fn build_generic_param(&self, pair: Pair<'_, Rule>) -> BuildResult<GenericParam> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut constraint = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    if name.is_empty() {
                        name = inner.as_str().to_string();
                    }
                }
                Rule::type_annotation => {
                    constraint = Some(self.build_type_annotation(inner)?);
                }
                _ => {}
            }
        }

        Ok(GenericParam {
            name,
            constraint: constraint.map(Box::new),
            span,
        })
    }

    fn build_interface_member(&self, pair: Pair<'_, Rule>) -> BuildResult<InterfaceMember> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut optional = false;
        let mut type_annotation = TypeAnnotation::Named {
            name: "unknown".to_string(),
            span: span.clone(),
        };

        let text = pair.as_str();
        if text.contains('?') {
            optional = true;
        }

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().to_string();
                }
                Rule::type_annotation => {
                    type_annotation = self.build_type_annotation(inner)?;
                }
                _ => {}
            }
        }

        Ok(InterfaceMember {
            name,
            optional,
            type_annotation,
            span,
        })
    }

    // ========================================================================
    // STYLES BLOCK BUILDING
    // ========================================================================

    fn build_styles_block(&self, pair: Pair<'_, Rule>) -> BuildResult<StylesBlock> {
        let span = span_from_pair(&pair);
        let mut scoped = true;
        let mut modifiers = Vec::new();
        let mut raw_css = String::new();
        let mut rules = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::style_modifier => {
                    let mod_text = inner.as_str().to_lowercase();
                    if mod_text == "global" {
                        scoped = false;
                        modifiers.push(super::node::StyleModifier::Global);
                    } else if mod_text == "scoped" {
                        modifiers.push(super::node::StyleModifier::Scoped);
                    }
                }
                Rule::style_content => {
                    // Collect raw CSS content
                    raw_css.push_str(inner.as_str());
                    raw_css.push('\n');
                    // Also parse the content
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(content) = self.build_style_content(child) {
                            rules.push(content);
                        }
                    }
                }
                Rule::style_rule => {
                    raw_css.push_str(inner.as_str());
                    raw_css.push('\n');
                    if let Ok(rule) = self.build_style_rule(inner) {
                        rules.push(super::node::StyleContent::Rule(rule));
                    }
                }
                Rule::style_at_rule => {
                    raw_css.push_str(inner.as_str());
                    raw_css.push('\n');
                    if let Ok(at_rule) = self.build_style_at_rule(inner) {
                        rules.push(super::node::StyleContent::AtRule(at_rule));
                    }
                }
                _ => {
                    // Collect any other content as raw CSS
                    let content = inner.as_str().trim();
                    if !content.is_empty() {
                        raw_css.push_str(content);
                        raw_css.push('\n');
                    }
                }
            }
        }

        Ok(StylesBlock {
            scoped,
            modifiers,
            rules,
            raw_css: raw_css.trim().to_string(),
            span,
        })
    }

    fn build_style_content(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleContent> {
        match pair.as_rule() {
            Rule::style_rule => Ok(super::node::StyleContent::Rule(
                self.build_style_rule(pair)?,
            )),
            Rule::style_at_rule => Ok(super::node::StyleContent::AtRule(
                self.build_style_at_rule(pair)?,
            )),
            Rule::line_comment => {
                let span = span_from_pair(&pair);
                let value = pair.as_str().trim_start_matches("//").trim().to_string();
                Ok(super::node::StyleContent::Comment { value, span })
            }
            _ => {
                let span = span_from_pair(&pair);
                Err(BuildError::unsupported("style content type", span))
            }
        }
    }

    fn build_style_rule(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleRule> {
        let span = span_from_pair(&pair);
        let mut selectors = Vec::new();
        let mut declarations = Vec::new();
        let mut nested_rules = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::style_selector_list => {
                    for selector in inner.into_inner() {
                        if selector.as_rule() == Rule::style_selector {
                            selectors.push(selector.as_str().trim().to_string());
                        }
                    }
                }
                Rule::style_rule_content => {
                    // Style rule content can be either a declaration or a nested rule
                    for content_inner in inner.into_inner() {
                        match content_inner.as_rule() {
                            Rule::style_declaration => {
                                if let Ok(decl) = self.build_style_declaration(content_inner) {
                                    declarations.push(decl);
                                }
                            }
                            Rule::style_rule => {
                                if let Ok(nested_rule) = self.build_style_rule(content_inner) {
                                    nested_rules.push(nested_rule);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Rule::style_declaration => {
                    // Legacy support for direct style_declaration (if grammar still produces it)
                    if let Ok(decl) = self.build_style_declaration(inner) {
                        declarations.push(decl);
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleRule {
            selectors,
            declarations,
            nested_rules,
            span,
        })
    }

    fn build_style_declaration(
        &self,
        pair: Pair<'_, Rule>,
    ) -> BuildResult<super::node::StyleDeclaration> {
        let span = span_from_pair(&pair);
        let mut property = String::new();
        let mut value = super::node::CssValue::Plain {
            value: String::new(),
        };

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::css_property => {
                    property = inner.as_str().trim().to_string();
                }
                Rule::css_value => {
                    value = self.build_css_value(inner)?;
                }
                Rule::at_apply => {
                    // @apply is handled as a declaration with special property
                    property = "@apply".to_string();
                    for apply_inner in inner.into_inner() {
                        if apply_inner.as_rule() == Rule::tailwind_classes {
                            value = super::node::CssValue::Plain {
                                value: apply_inner.as_str().trim().to_string(),
                            };
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleDeclaration {
            property,
            value,
            span,
        })
    }

    fn build_css_value(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::CssValue> {
        let mut parts = Vec::new();
        let mut has_interpolation = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::css_value_part => {
                    let mut value = inner.as_str().to_string();
                    // Strip quotes from quoted strings (e.g., content: "text" -> content: text)
                    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                        value = value[1..value.len()-1].to_string();
                    } else if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
                        value = value[1..value.len()-1].to_string();
                    }
                    parts.push(super::node::CssValuePart::Text { value });
                }
                Rule::css_interpolation => {
                    has_interpolation = true;
                    // Get the expression inside the interpolation
                    if let Some(expr_pair) = inner.into_inner().next() {
                        if let Ok(expr) = self.build_expression(expr_pair) {
                            parts.push(super::node::CssValuePart::Expression { expr });
                        }
                    }
                }
                _ => {}
            }
        }

        if has_interpolation {
            Ok(super::node::CssValue::Interpolated { parts })
        } else {
            // Combine all text parts into a single plain value and trim whitespace
            let combined: String = parts
                .into_iter()
                .filter_map(|p| {
                    if let super::node::CssValuePart::Text { value } = p {
                        Some(value)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string();
            Ok(super::node::CssValue::Plain { value: combined })
        }
    }

    fn build_style_at_rule(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        // Get the first child which determines the at-rule type
        let inner = pair
            .into_inner()
            .next()
            .ok_or_else(|| BuildError::missing("at-rule type"))?;

        match inner.as_rule() {
            Rule::at_media => self.build_at_media(inner),
            Rule::at_keyframes => self.build_at_keyframes(inner),
            Rule::at_layer => self.build_at_layer(inner),
            Rule::at_supports => self.build_at_supports(inner),
            Rule::at_container => self.build_at_container(inner),
            Rule::at_scope => self.build_at_scope(inner),
            Rule::at_apply => self.build_at_apply(inner),
            Rule::at_screen => self.build_at_screen(inner),
            Rule::at_theme => self.build_at_theme(inner),
            Rule::at_variants => self.build_at_variants(inner),
            _ => {
                let span = span_from_pair(&inner);
                Err(BuildError::unsupported("at-rule type", span))
            }
        }
    }

    fn build_at_media(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut query = String::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::media_query => {
                    query = inner.as_str().trim().to_string();
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Media {
            query,
            content,
            span,
        })
    }

    fn build_at_keyframes(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut blocks = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => {
                    name = inner.as_str().trim().to_string();
                }
                Rule::keyframe_block => {
                    if let Ok(block) = self.build_keyframe_block(inner) {
                        blocks.push(block);
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Keyframes { name, blocks, span })
    }

    fn build_keyframe_block(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::KeyframeBlock> {
        let span = span_from_pair(&pair);
        let mut selector = String::new();
        let mut declarations = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::keyframe_selector => {
                    selector = inner.as_str().trim().to_string();
                }
                Rule::style_declaration => {
                    if let Ok(decl) = self.build_style_declaration(inner) {
                        declarations.push(decl);
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::KeyframeBlock {
            selector,
            declarations,
            span,
        })
    }

    fn build_at_layer(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut names = Vec::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::layer_names => {
                    for name in inner.into_inner() {
                        if name.as_rule() == Rule::identifier {
                            names.push(name.as_str().trim().to_string());
                        }
                    }
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Layer {
            names,
            content,
            span,
        })
    }

    fn build_at_supports(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut condition = String::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::supports_condition => {
                    condition = inner.as_str().trim().to_string();
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Supports {
            condition,
            content,
            span,
        })
    }

    fn build_at_container(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut query = String::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::container_query => {
                    query = inner.as_str().trim().to_string();
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Container {
            query,
            content,
            span,
        })
    }

    fn build_at_scope(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut selector = String::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::scope_selector => {
                    selector = inner.as_str().trim().to_string();
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Scope {
            selector,
            content,
            span,
        })
    }

    fn build_at_apply(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut classes = String::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::tailwind_classes {
                classes = inner.as_str().trim().to_string();
            }
        }

        Ok(super::node::StyleAtRule::Apply { classes, span })
    }

    fn build_at_screen(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::screen_name => {
                    name = inner.as_str().trim().to_string();
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Screen {
            name,
            content,
            span,
        })
    }

    fn build_at_theme(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut variables = Vec::new();

        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::theme_variable {
                if let Ok(var) = self.build_theme_variable(inner) {
                    variables.push(var);
                }
            }
        }

        Ok(super::node::StyleAtRule::Theme { variables, span })
    }

    fn build_theme_variable(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::ThemeVariable> {
        let span = span_from_pair(&pair);
        let mut name = String::new();
        let mut value = String::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::css_variable => {
                    name = inner.as_str().trim().to_string();
                }
                Rule::css_value => {
                    // Get the raw value as string
                    value = inner.as_str().trim().to_string();
                }
                _ => {}
            }
        }

        Ok(super::node::ThemeVariable { name, value, span })
    }

    fn build_at_variants(&self, pair: Pair<'_, Rule>) -> BuildResult<super::node::StyleAtRule> {
        let span = span_from_pair(&pair);
        let mut names = Vec::new();
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::variant_names => {
                    for name in inner.into_inner() {
                        if name.as_rule() == Rule::identifier {
                            names.push(name.as_str().trim().to_string());
                        }
                    }
                }
                Rule::style_content => {
                    if let Some(child) = inner.into_inner().next() {
                        if let Ok(c) = self.build_style_content(child) {
                            content.push(c);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(super::node::StyleAtRule::Variants {
            names,
            content,
            span,
        })
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Parse source code to AST
///
/// This is the main entry point for parsing Orbis DSL source code into an AST.
///
/// # Example
///
/// ```rust,ignore
/// use orbis_dsl::ast::parse_to_ast;
///
/// let source = r#"
/// page {
///     id: "my-page"
/// }
/// "#;
///
/// let ast = parse_to_ast(source)?;
/// ```
pub fn parse_to_ast(source: &str) -> BuildResult<AstFile> {
    use pest::Parser;

    let pairs = crate::page::Parser::parse(Rule::file, source)
        .map_err(|e| BuildError::new(e.to_string(), None, BuildErrorKind::ParseError))?;

    let builder = AstBuilder::new();
    builder.build(pairs)
}

/// Parse source code to AST with source path for error messages
pub fn parse_to_ast_with_path(source: &str, path: impl Into<String>) -> BuildResult<AstFile> {
    use pest::Parser;

    let pairs = crate::page::Parser::parse(Rule::file, source)
        .map_err(|e| BuildError::new(e.to_string(), None, BuildErrorKind::ParseError))?;

    let builder = AstBuilder::with_source_path(path);
    builder.build(pairs)
}
