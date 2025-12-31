//! Document Symbols Provider
//!
//! This module provides document symbols for the outline view, showing:
//! - Page block
//! - State variables
//! - Hooks
//! - Template structure
//! - Fragments
//! - Interfaces
//! - Styles

use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolResponse, Range, Position, SymbolKind,
};

use crate::analysis::{span_to_range, AnalysisResult};
use orbis_dsl::ast::{
    AstFile, TopLevelElement, TemplateContent, ControlFlow, HookEntry,
    ImportStatement, StateDeclaration, Span,
};

/// Get document symbols for outline view
pub fn get_document_symbols(result: &AnalysisResult) -> Option<DocumentSymbolResponse> {
    let ast = result.ast.as_ref()?;
    let symbols = build_document_symbols(ast);
    Some(DocumentSymbolResponse::Nested(symbols))
}

/// Build document symbols from AST
fn build_document_symbols(ast: &AstFile) -> Vec<DocumentSymbol> {
    let mut doc_symbols = Vec::new();

    for element in &ast.elements {
        match element {
            TopLevelElement::Page(page) => {
                let range = span_to_range(&page.span);
                let mut children = Vec::new();

                // Add page properties as children
                if let Some(id) = &page.properties.id {
                    children.push(create_symbol(
                        format!("id: \"{}\"", id),
                        SymbolKind::PROPERTY,
                        range.clone(),
                        range.clone(),
                        None,
                    ));
                }
                if let Some(title) = &page.properties.title {
                    children.push(create_symbol(
                        format!("title: \"{}\"", title),
                        SymbolKind::PROPERTY,
                        range.clone(),
                        range.clone(),
                        None,
                    ));
                }
                if let Some(route) = &page.properties.route {
                    children.push(create_symbol(
                        format!("route: \"{}\"", route),
                        SymbolKind::PROPERTY,
                        range.clone(),
                        range.clone(),
                        None,
                    ));
                }

                doc_symbols.push(create_symbol(
                    "page".to_string(),
                    SymbolKind::MODULE,
                    range.clone(),
                    range,
                    Some(children),
                ));
            }

            TopLevelElement::State(state) => {
                let range = span_to_range(&state.span);
                let children: Vec<_> = state
                    .declarations
                    .iter()
                    .map(|decl| {
                        let (name, kind, detail) = match decl {
                            StateDeclaration::Regular(r) => {
                                let detail = r.type_annotation.as_ref().map(|_| "regular".to_string());
                                (r.name.clone(), SymbolKind::VARIABLE, detail)
                            }
                            StateDeclaration::Computed(c) => {
                                (c.name.clone(), SymbolKind::FUNCTION, Some("computed".to_string()))
                            }
                            StateDeclaration::Validated(v) => {
                                (v.name.clone(), SymbolKind::VARIABLE, Some("validated".to_string()))
                            }
                        };
                        let decl_range = span_to_range(state_decl_span(decl));
                        create_symbol_with_detail(name, kind, decl_range.clone(), decl_range, detail, None)
                    })
                    .collect();

                doc_symbols.push(create_symbol(
                    "state".to_string(),
                    SymbolKind::NAMESPACE,
                    range.clone(),
                    range,
                    Some(children),
                ));
            }

            TopLevelElement::Hooks(hooks) => {
                let range = span_to_range(&hooks.span);
                let mut children = Vec::new();

                for entry in &hooks.entries {
                    match entry {
                        HookEntry::Lifecycle(hook) => {
                            let hook_name = match hook.kind {
                                orbis_dsl::ast::LifecycleHookKind::Mount => "@mount",
                                orbis_dsl::ast::LifecycleHookKind::Unmount => "@unmount",
                            };
                            let hook_range = span_to_range(&hook.span);
                            children.push(create_symbol(
                                hook_name.to_string(),
                                SymbolKind::EVENT,
                                hook_range.clone(),
                                hook_range,
                                None,
                            ));
                        }
                        HookEntry::Watcher(watcher) => {
                            let watch_target = watcher
                                .targets
                                .iter()
                                .map(|e| format_expression(e))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let watcher_range = span_to_range(&watcher.span);
                            children.push(create_symbol(
                                format!("@watch({})", watch_target),
                                SymbolKind::EVENT,
                                watcher_range.clone(),
                                watcher_range,
                                None,
                            ));
                        }
                    }
                }

                doc_symbols.push(create_symbol(
                    "hooks".to_string(),
                    SymbolKind::NAMESPACE,
                    range.clone(),
                    range,
                    Some(children),
                ));
            }

            TopLevelElement::Template(template) => {
                let range = span_to_range(&template.span);
                let children: Vec<_> = template
                    .content
                    .iter()
                    .filter_map(|content| content_to_symbol(content))
                    .collect();

                doc_symbols.push(create_symbol(
                    "template".to_string(),
                    SymbolKind::NAMESPACE,
                    range.clone(),
                    range,
                    Some(children),
                ));
            }

            TopLevelElement::Fragment(frag) => {
                let range = span_to_range(&frag.span);
                let params = frag
                    .params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");

                let children: Vec<_> = frag
                    .body
                    .iter()
                    .filter_map(|content| content_to_symbol(content))
                    .collect();

                doc_symbols.push(create_symbol_with_detail(
                    frag.name.clone(),
                    SymbolKind::FUNCTION,
                    range.clone(),
                    range,
                    Some(format!("fragment({})", params)),
                    Some(children),
                ));
            }

            TopLevelElement::Interface(iface) => {
                let range = span_to_range(&iface.span);
                let children: Vec<_> = iface
                    .members
                    .iter()
                    .map(|m| {
                        let member_range = span_to_range(&m.span);
                        let optional = if m.optional { "?" } else { "" };
                        create_symbol(
                            format!("{}{}", m.name, optional),
                            SymbolKind::FIELD,
                            member_range.clone(),
                            member_range,
                            None,
                        )
                    })
                    .collect();

                doc_symbols.push(create_symbol(
                    iface.name.clone(),
                    SymbolKind::INTERFACE,
                    range.clone(),
                    range,
                    Some(children),
                ));
            }

            TopLevelElement::Styles(styles) => {
                let range = span_to_range(&styles.span);
                doc_symbols.push(create_symbol(
                    "styles".to_string(),
                    SymbolKind::NAMESPACE,
                    range.clone(),
                    range,
                    None,
                ));
            }

            TopLevelElement::Export(export) => {
                let range = span_to_range(&export.span);
                let name = match &export.item {
                    orbis_dsl::ast::ExportableItem::Fragment(f) => f.name.clone(),
                    orbis_dsl::ast::ExportableItem::Interface(i) => i.name.clone(),
                    orbis_dsl::ast::ExportableItem::Const { name, .. } => name.clone(),
                };
                doc_symbols.push(create_symbol_with_detail(
                    name,
                    SymbolKind::MODULE,
                    range.clone(),
                    range,
                    Some("export".to_string()),
                    None,
                ));
            }

            TopLevelElement::Comment { .. } => {
                // Skip comments in outline
            }
        }
    }

    // Add imports section if there are imports
    if !ast.imports.is_empty() {
        let first_span = import_span(&ast.imports[0]);
        let last_span = import_span(ast.imports.last().unwrap());

        let range = Range {
            start: Position {
                line: (first_span.start_line.saturating_sub(1)) as u32,
                character: 0,
            },
            end: Position {
                line: (last_span.end_line.saturating_sub(1)) as u32,
                character: 0,
            },
        };

        let children: Vec<_> = ast
            .imports
            .iter()
            .map(|imp| {
                let (name, source) = match imp {
                    ImportStatement::TypeScript { clause, path, .. } => {
                        let name = match clause {
                            orbis_dsl::ast::ImportClause::Named { specifiers } => {
                                specifiers
                                    .iter()
                                    .map(|s| s.name.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                            orbis_dsl::ast::ImportClause::Default { name } => name.clone(),
                            orbis_dsl::ast::ImportClause::Namespace { name } => format!("* as {}", name),
                        };
                        (name, path.clone())
                    }
                    ImportStatement::Rust { path, alias, .. } => {
                        let name = alias.clone().unwrap_or_else(|| path.last().cloned().unwrap_or_default());
                        (name, path.join("::"))
                    }
                };
                let imp_range = span_to_range(import_span(imp));
                create_symbol_with_detail(
                    name,
                    SymbolKind::MODULE,
                    imp_range.clone(),
                    imp_range,
                    Some(format!("from {}", source)),
                    None,
                )
            })
            .collect();

        doc_symbols.insert(
            0,
            create_symbol(
                "imports".to_string(),
                SymbolKind::NAMESPACE,
                range.clone(),
                range,
                Some(children),
            ),
        );
    }

    doc_symbols
}

/// Convert template content to document symbol
fn content_to_symbol(content: &TemplateContent) -> Option<DocumentSymbol> {
    match content {
        TemplateContent::Component(comp) => {
            let range = span_to_range(&comp.span);
            let children: Vec<_> = comp
                .children
                .iter()
                .filter_map(|c| content_to_symbol(c))
                .collect();

            Some(create_symbol(
                format!("<{}>", comp.name),
                SymbolKind::CLASS,
                range.clone(),
                range,
                if children.is_empty() { None } else { Some(children) },
            ))
        }

        TemplateContent::FragmentUsage(frag) => {
            let range = span_to_range(&frag.span);
            Some(create_symbol(
                format!("<{}>", frag.name),
                SymbolKind::FUNCTION,
                range.clone(),
                range,
                None,
            ))
        }

        TemplateContent::ControlFlow(cf) => match cf {
            ControlFlow::If(if_block) => {
                let range = span_to_range(&if_block.span);
                let mut children: Vec<_> = if_block
                    .then_branch
                    .iter()
                    .filter_map(|c| content_to_symbol(c))
                    .collect();

                if let Some(else_branch) = &if_block.else_branch {
                    let else_children: Vec<_> = else_branch
                        .iter()
                        .filter_map(|c| content_to_symbol(c))
                        .collect();
                    children.push(create_symbol(
                        "else".to_string(),
                        SymbolKind::KEY,
                        range.clone(),
                        range.clone(),
                        Some(else_children),
                    ));
                }

                Some(create_symbol(
                    format!("if {}", format_expression(&if_block.condition)),
                    SymbolKind::KEY,
                    range.clone(),
                    range,
                    Some(children),
                ))
            }

            ControlFlow::For(for_block) => {
                let range = span_to_range(&for_block.span);
                let children: Vec<_> = for_block
                    .body
                    .iter()
                    .filter_map(|c| content_to_symbol(c))
                    .collect();

                let binding = if let Some(idx) = &for_block.binding.index {
                    format!("{}, {}", for_block.binding.item.name, idx.name)
                } else {
                    for_block.binding.item.name.clone()
                };

                Some(create_symbol(
                    format!("for {} in ...", binding),
                    SymbolKind::KEY,
                    range.clone(),
                    range,
                    Some(children),
                ))
            }

            ControlFlow::When(when_block) => {
                let range = span_to_range(&when_block.span);
                let children: Vec<_> = when_block
                    .arms
                    .iter()
                    .map(|arm| {
                        let arm_range = span_to_range(&arm.span);
                        let pattern = format_pattern(&arm.pattern);
                        create_symbol(
                            pattern,
                            SymbolKind::ENUM_MEMBER,
                            arm_range.clone(),
                            arm_range,
                            None,
                        )
                    })
                    .collect();

                Some(create_symbol(
                    format!("when {}", format_expression(&when_block.subject)),
                    SymbolKind::KEY,
                    range.clone(),
                    range,
                    Some(children),
                ))
            }
        },

        TemplateContent::SlotDefinition(slot) => {
            let range = span_to_range(&slot.span);
            let name = slot.name.clone().unwrap_or_else(|| "default".to_string());
            Some(create_symbol(
                format!("<slot:{}>", name),
                SymbolKind::KEY,
                range.clone(),
                range,
                None,
            ))
        }

        TemplateContent::Text { .. } => None, // Skip text nodes
        TemplateContent::Expression { .. } => None, // Skip expression nodes
        TemplateContent::Comment { .. } => None, // Skip comments
    }
}

/// Format an expression for display
fn format_expression(expr: &orbis_dsl::ast::Expression) -> String {
    match expr {
        orbis_dsl::ast::Expression::Identifier(id) => id.name.clone(),
        orbis_dsl::ast::Expression::MemberAccess(ma) => {
            format!("{}.{}", ma.root, ma.path.join("."))
        }
        orbis_dsl::ast::Expression::Literal(lit) => format!("{:?}", lit.value),
        orbis_dsl::ast::Expression::Binary(bin) => {
            format!(
                "{} {} {}",
                format_expression(&bin.left),
                bin.op.as_str(),
                format_expression(&bin.right)
            )
        }
        _ => "...".to_string(),
    }
}

/// Format a when pattern for display
fn format_pattern(pattern: &orbis_dsl::ast::WhenPattern) -> String {
    match pattern {
        orbis_dsl::ast::WhenPattern::Literal { value, .. } => format_expression(value),
        orbis_dsl::ast::WhenPattern::Binding { name, .. } => name.clone(),
        orbis_dsl::ast::WhenPattern::Range { start, end, .. } => {
            format!("{}..{}", format_expression(start), format_expression(end))
        }
        orbis_dsl::ast::WhenPattern::Or { patterns, .. } => {
            patterns.iter().map(format_pattern).collect::<Vec<_>>().join(" | ")
        }
        orbis_dsl::ast::WhenPattern::Wildcard { .. } => "_".to_string(),
    }
}

/// Create a document symbol
fn create_symbol(
    name: String,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail: None,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}

/// Create a document symbol with detail
fn create_symbol_with_detail(
    name: String,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    detail: Option<String>,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}

/// Get span from ImportStatement
fn import_span(import: &ImportStatement) -> &Span {
    match import {
        ImportStatement::TypeScript { span, .. } => span,
        ImportStatement::Rust { span, .. } => span,
    }
}

/// Get span from StateDeclaration
fn state_decl_span(decl: &StateDeclaration) -> &Span {
    match decl {
        StateDeclaration::Regular(r) => &r.span,
        StateDeclaration::Computed(c) => &c.span,
        StateDeclaration::Validated(v) => &v.span,
    }
}
