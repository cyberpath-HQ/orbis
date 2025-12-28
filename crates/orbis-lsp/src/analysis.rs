//! Semantic Analysis and Caching
//!
//! This module provides semantic analysis of Orbis DSL documents, including:
//! - AST caching for performance
//! - Symbol table construction
//! - Type information
//! - Component metadata from orbis-dsl build definitions

use std::collections::HashMap;
use std::sync::Arc;

use orbis_dsl::ast::{
    self, AstFile, Component, Expression, FragmentDefinition, MemberAccess, Span, StateBlock,
    StateDeclaration, TemplateContent, TopLevelElement,
};
use tower_lsp::lsp_types::{Position, Range};

/// Semantic analysis result for a document
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// The parsed AST (None if parse failed)
    pub ast: Option<AstFile>,
    /// Parse errors
    pub errors: Vec<AnalysisError>,
    /// Symbol table
    pub symbols: SymbolTable,
    /// Document version this analysis is for
    pub version: i32,
}

/// An analysis error with rich information
#[derive(Debug, Clone)]
pub struct AnalysisError {
    /// Error message
    pub message: String,
    /// Error location
    pub span: Option<Span>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Suggestion for fixing
    pub suggestion: Option<String>,
    /// Related information
    pub related: Vec<RelatedInfo>,
}

/// Related information for an error
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub message: String,
    pub span: Span,
}

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Symbol table for a document
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// State variable declarations
    pub state_vars: HashMap<String, StateSymbol>,
    /// Fragment definitions
    pub fragments: HashMap<String, FragmentSymbol>,
    /// Imported symbols
    pub imports: HashMap<String, ImportSymbol>,
    /// Interface definitions
    pub interfaces: HashMap<String, InterfaceSymbol>,
    /// All symbol references (for find references)
    pub references: Vec<SymbolReference>,
}

/// A state variable symbol
#[derive(Debug, Clone)]
pub struct StateSymbol {
    pub name: String,
    pub type_annotation: Option<String>,
    pub is_computed: bool,
    pub is_validated: bool,
    pub span: Span,
    pub documentation: Option<String>,
}

/// A fragment symbol
#[derive(Debug, Clone)]
pub struct FragmentSymbol {
    pub name: String,
    pub params: Vec<FragmentParamSymbol>,
    pub exported: bool,
    pub span: Span,
}

/// Fragment parameter
#[derive(Debug, Clone)]
pub struct FragmentParamSymbol {
    pub name: String,
    pub type_annotation: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

/// An imported symbol
#[derive(Debug, Clone)]
pub struct ImportSymbol {
    pub name: String,
    pub alias: Option<String>,
    pub source: String,
    pub span: Span,
}

/// An interface symbol
#[derive(Debug, Clone)]
pub struct InterfaceSymbol {
    pub name: String,
    pub members: Vec<InterfaceMemberSymbol>,
    pub span: Span,
}

/// Interface member
#[derive(Debug, Clone)]
pub struct InterfaceMemberSymbol {
    pub name: String,
    pub type_annotation: String,
    pub optional: bool,
}

/// A reference to a symbol
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub is_definition: bool,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    StateVariable,
    Fragment,
    Import,
    Interface,
    Component,
    Parameter,
}

/// Analyzer for Orbis DSL documents
pub struct Analyzer;

impl Analyzer {
    /// Analyze a document and return the result
    pub fn analyze(source: &str, version: i32) -> AnalysisResult {
        let mut errors = Vec::new();
        let mut symbols = SymbolTable::default();

        // Try to parse the document
        let ast = match ast::parse_to_ast(source) {
            Ok(ast) => {
                // Build symbol table from AST
                Self::build_symbol_table(&ast, &mut symbols);
                // Collect references
                Self::collect_references(&ast, &mut symbols);
                // Run semantic checks
                errors.extend(Self::semantic_checks(&ast, &symbols));
                Some(ast)
            }
            Err(e) => {
                errors.push(AnalysisError {
                    message: e.message.clone(),
                    span: e.span,
                    severity: ErrorSeverity::Error,
                    suggestion: Self::suggest_fix(&e.message),
                    related: vec![],
                });
                None
            }
        };

        AnalysisResult {
            ast,
            errors,
            symbols,
            version,
        }
    }

    /// Build symbol table from AST
    fn build_symbol_table(ast: &AstFile, symbols: &mut SymbolTable) {
        // Process imports
        for import in &ast.imports {
            match import {
                ast::ImportStatement::TypeScript { clause, path, span } => {
                    if let ast::ImportClause::Named { specifiers } = clause {
                        for spec in specifiers {
                            symbols.imports.insert(
                                spec.alias.clone().unwrap_or_else(|| spec.name.clone()),
                                ImportSymbol {
                                    name: spec.name.clone(),
                                    alias: spec.alias.clone(),
                                    source: path.clone(),
                                    span: span.clone(),
                                },
                            );
                        }
                    }
                }
                ast::ImportStatement::Rust { path, alias, span } => {
                    let name = path.last().cloned().unwrap_or_default();
                    symbols.imports.insert(
                        alias.clone().unwrap_or_else(|| name.clone()),
                        ImportSymbol {
                            name,
                            alias: alias.clone(),
                            source: path.join("::"),
                            span: span.clone(),
                        },
                    );
                }
            }
        }

        // Process top-level elements
        for element in &ast.elements {
            match element {
                TopLevelElement::State(state_block) => {
                    Self::process_state_block(state_block, symbols);
                }
                TopLevelElement::Fragment(frag) => {
                    Self::process_fragment(frag, symbols);
                }
                TopLevelElement::Interface(iface) => {
                    symbols.interfaces.insert(
                        iface.name.clone(),
                        InterfaceSymbol {
                            name: iface.name.clone(),
                            members: iface
                                .members
                                .iter()
                                .map(|m| InterfaceMemberSymbol {
                                    name: m.name.clone(),
                                    type_annotation: format_type_annotation(&m.type_annotation),
                                    optional: m.optional,
                                })
                                .collect(),
                            span: iface.span.clone(),
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn process_state_block(block: &StateBlock, symbols: &mut SymbolTable) {
        for decl in &block.declarations {
            let (name, type_ann, is_computed, is_validated, span, doc_comment) = match decl {
                StateDeclaration::Regular(r) => {
                    (r.name.clone(), r.type_annotation.as_ref(), false, false, &r.span, r.doc_comment.clone())
                }
                StateDeclaration::Computed(c) => {
                    (c.name.clone(), c.type_annotation.as_ref(), true, false, &c.span, c.doc_comment.clone())
                }
                StateDeclaration::Validated(v) => {
                    (v.name.clone(), v.type_annotation.as_ref(), false, true, &v.span, v.doc_comment.clone())
                }
            };

            symbols.state_vars.insert(
                name.clone(),
                StateSymbol {
                    name,
                    type_annotation: type_ann.map(format_type_annotation),
                    is_computed,
                    is_validated,
                    span: span.clone(),
                    documentation: doc_comment,
                },
            );
        }
    }

    fn process_fragment(frag: &FragmentDefinition, symbols: &mut SymbolTable) {
        symbols.fragments.insert(
            frag.name.clone(),
            FragmentSymbol {
                name: frag.name.clone(),
                params: frag
                    .params
                    .iter()
                    .map(|p| FragmentParamSymbol {
                        name: p.name.clone(),
                        type_annotation: p.type_annotation.as_ref().map(format_type_annotation),
                        required: p.required,
                        default_value: p.default.as_ref().map(|_| "...".to_string()),
                    })
                    .collect(),
                exported: frag.exported,
                span: frag.span.clone(),
            },
        );
    }

    /// Collect all symbol references in the AST
    fn collect_references(ast: &AstFile, symbols: &mut SymbolTable) {
        // Add definitions as references
        for (name, sym) in &symbols.state_vars {
            symbols.references.push(SymbolReference {
                name: name.clone(),
                kind: SymbolKind::StateVariable,
                span: sym.span.clone(),
                is_definition: true,
            });
        }

        for (name, sym) in &symbols.fragments {
            symbols.references.push(SymbolReference {
                name: name.clone(),
                kind: SymbolKind::Fragment,
                span: sym.span.clone(),
                is_definition: true,
            });
        }

        // Walk the AST to find usages
        for element in &ast.elements {
            if let TopLevelElement::Template(template) = element {
                for content in &template.content {
                    Self::collect_template_references(content, symbols);
                }
            }
        }
    }

    fn collect_template_references(content: &TemplateContent, symbols: &mut SymbolTable) {
        match content {
            TemplateContent::Component(comp) => {
                // Collect component references
                symbols.references.push(SymbolReference {
                    name: comp.name.clone(),
                    kind: SymbolKind::Component,
                    span: comp.span.clone(),
                    is_definition: false,
                });

                // Check attributes for state references
                for attr in &comp.attributes {
                    if let ast::AttributeValue::Expression { value } = &attr.value {
                        Self::collect_expression_references(value, symbols);
                    }
                }

                // Check children
                for child in &comp.children {
                    Self::collect_template_references(child, symbols);
                }
            }
            TemplateContent::FragmentUsage(frag) => {
                symbols.references.push(SymbolReference {
                    name: frag.name.clone(),
                    kind: SymbolKind::Fragment,
                    span: frag.span.clone(),
                    is_definition: false,
                });
            }
            TemplateContent::ControlFlow(cf) => match cf {
                ast::ControlFlow::If(if_block) => {
                    Self::collect_expression_references(&if_block.condition, symbols);
                    for content in &if_block.then_branch {
                        Self::collect_template_references(content, symbols);
                    }
                    if let Some(else_branch) = &if_block.else_branch {
                        for content in else_branch {
                            Self::collect_template_references(content, symbols);
                        }
                    }
                }
                ast::ControlFlow::For(for_block) => {
                    Self::collect_expression_references(&for_block.iterable, symbols);
                    for content in &for_block.body {
                        Self::collect_template_references(content, symbols);
                    }
                }
                ast::ControlFlow::When(when_block) => {
                    Self::collect_expression_references(&when_block.subject, symbols);
                    for arm in &when_block.arms {
                        for content in &arm.body {
                            Self::collect_template_references(content, symbols);
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn collect_expression_references(expr: &Expression, symbols: &mut SymbolTable) {
        match expr {
            Expression::MemberAccess(ma) => {
                if ma.root == "state" {
                    // This is a state variable reference
                    if let Some(first_part) = ma.path.first() {
                        symbols.references.push(SymbolReference {
                            name: first_part.clone(),
                            kind: SymbolKind::StateVariable,
                            span: ma.span.clone(),
                            is_definition: false,
                        });
                    }
                }
            }
            Expression::Binary(bin) => {
                Self::collect_expression_references(&bin.left, symbols);
                Self::collect_expression_references(&bin.right, symbols);
            }
            Expression::Unary(un) => {
                Self::collect_expression_references(&un.operand, symbols);
            }
            Expression::Grouped { inner, .. } => {
                Self::collect_expression_references(inner, symbols);
            }
            Expression::MethodCall(mc) => {
                // Check if namespace references a state variable
                if symbols.state_vars.contains_key(&mc.namespace) {
                    symbols.references.push(SymbolReference {
                        name: mc.namespace.clone(),
                        kind: SymbolKind::StateVariable,
                        span: mc.span.clone(),
                        is_definition: false,
                    });
                }
                for arg in &mc.arguments {
                    Self::collect_expression_references(&arg.value, symbols);
                }
            }
            _ => {}
        }
    }

    /// Run semantic checks
    fn semantic_checks(ast: &AstFile, symbols: &SymbolTable) -> Vec<AnalysisError> {
        let mut errors = Vec::new();

        // Check for undefined state references
        for reference in &symbols.references {
            if reference.is_definition {
                continue;
            }

            match reference.kind {
                SymbolKind::StateVariable => {
                    if !symbols.state_vars.contains_key(&reference.name) {
                        errors.push(AnalysisError {
                            message: format!("Undefined state variable: '{}'", reference.name),
                            span: Some(reference.span.clone()),
                            severity: ErrorSeverity::Error,
                            suggestion: Self::suggest_similar_symbol(
                                &reference.name,
                                symbols.state_vars.keys(),
                            ),
                            related: vec![],
                        });
                    }
                }
                SymbolKind::Fragment => {
                    // Check if fragment is defined locally or imported
                    if !symbols.fragments.contains_key(&reference.name)
                        && !symbols.imports.contains_key(&reference.name)
                    {
                        errors.push(AnalysisError {
                            message: format!("Undefined fragment: '{}'", reference.name),
                            span: Some(reference.span.clone()),
                            severity: ErrorSeverity::Error,
                            suggestion: Self::suggest_similar_symbol(
                                &reference.name,
                                symbols.fragments.keys(),
                            ),
                            related: vec![],
                        });
                    }
                }
                SymbolKind::Component => {
                    // Check if component is a built-in component or a defined fragment
                    if !Self::is_builtin_component(&reference.name)
                        && !symbols.fragments.contains_key(&reference.name)
                        && !symbols.imports.contains_key(&reference.name)
                    {
                        errors.push(AnalysisError {
                            message: format!("Undefined component: '{}'", reference.name),
                            span: Some(reference.span.clone()),
                            severity: ErrorSeverity::Error,
                            suggestion: Some("Make sure you've defined this component as a fragment or check the spelling.".to_string()),
                            related: vec![],
                        });
                    }
                }
                _ => {}
            }
        }

        // Check for unused state variables
        let used_vars: std::collections::HashSet<_> = symbols
            .references
            .iter()
            .filter(|r| r.kind == SymbolKind::StateVariable && !r.is_definition)
            .map(|r| &r.name)
            .collect();

        for (name, sym) in &symbols.state_vars {
            if !used_vars.contains(name) {
                errors.push(AnalysisError {
                    message: format!("Unused state variable: '{}'", name),
                    span: Some(sym.span.clone()),
                    severity: ErrorSeverity::Warning,
                    suggestion: Some("Consider removing this variable if it's not needed.".to_string()),
                    related: vec![],
                });
            }
        }

        errors
    }

    fn suggest_fix(message: &str) -> Option<String> {
        // Pattern matching for common errors
        if message.contains("Expected") {
            return Some("Check your syntax. You may be missing a closing bracket or quotation mark.".to_string());
        }
        if message.contains("Unknown component") {
            return Some("Make sure you're using a valid Orbis component name.".to_string());
        }
        None
    }

    /// Check if a component name is a built-in Orbis component
    /// Component names in AST are normalized to snake_case, so we compare against those
    fn is_builtin_component(name: &str) -> bool {
        matches!(
            name,
            // Layout components
            "container" | "grid" | "flex" | "spacer" | "divider" |
            // Typography
            "text" | "heading" |
            // Forms
            "field" | "form" | "button" | "dropdown" |
            // Data Display
            "card" | "table" | "list" | "badge" | "stat_card" | "data_display" |
            // Navigation
            "link" | "tabs" | "accordion" | "breadcrumb" |
            // Feedback
            "alert" | "progress" | "loading_overlay" | "skeleton" | "empty_state" |
            // Overlays
            "modal" | "tooltip" |
            // Media
            "image" | "icon" | "avatar" | "chart" |
            // Utility
            "section" | "page_header"
        )
    }

    fn suggest_similar_symbol<'a>(
        name: &str,
        candidates: impl Iterator<Item = &'a String>,
    ) -> Option<String> {
        let candidates: Vec<_> = candidates.collect();
        if candidates.is_empty() {
            return None;
        }

        // Simple Levenshtein distance for suggestions
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for candidate in candidates {
            let distance = levenshtein_distance(name, candidate);
            if distance < best_distance && distance <= 3 {
                best_distance = distance;
                best_match = Some(candidate.clone());
            }
        }

        best_match.map(|m| format!("Did you mean '{}'?", m))
    }
}

/// Simple Levenshtein distance implementation
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Format a type annotation to string
fn format_type_annotation(type_ann: &ast::TypeAnnotation) -> String {
    match type_ann {
        ast::TypeAnnotation::Primitive(p) => p.kind.as_str().to_string(),
        ast::TypeAnnotation::Array { element, .. } => {
            format!("{}[]", format_type_annotation(element))
        }
        ast::TypeAnnotation::Union { types, .. } => {
            types.iter().map(format_type_annotation).collect::<Vec<_>>().join(" | ")
        }
        ast::TypeAnnotation::Optional { inner, .. } => {
            format!("{}?", format_type_annotation(inner))
        }
        ast::TypeAnnotation::Generic { name, args, .. } => {
            let args_str = args
                .iter()
                .map(format_type_annotation)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args_str)
        }
        ast::TypeAnnotation::Literal { value, .. } => format!("{:?}", value),
        ast::TypeAnnotation::Named { name, .. } => name.clone(),
    }
}

/// Convert AST Span to LSP Range
pub fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position {
            line: (span.start_line.saturating_sub(1)) as u32,
            character: (span.start_col.saturating_sub(1)) as u32,
        },
        end: Position {
            line: (span.end_line.saturating_sub(1)) as u32,
            character: (span.end_col.saturating_sub(1)) as u32,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_simple_document() {
        let source = r#"
page {
    id: "test"
    title: "Test Page"
}

state {
    count = 0
}

template {
    <Container>
        <Text content={state.count} />
    </Container>
}
"#;

        let result = Analyzer::analyze(source, 1);
        assert!(result.ast.is_some());
        assert!(result.symbols.state_vars.contains_key("count"));
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("count", "coutn"), 2);
    }
}
