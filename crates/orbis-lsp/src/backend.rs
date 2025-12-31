//! Orbis LSP Backend
//!
//! This module implements the main Language Server Protocol handler using tower-lsp.
//! It orchestrates all the provider modules to deliver a complete IDE experience.

use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::{debug, error, info, warn};

use crate::analysis::{self, AnalysisResult};
use crate::capabilities;
use crate::completion;
use crate::definition;
use crate::diagnostics;
use crate::document::{Document, DocumentContext};
use crate::hover;
use crate::references;
use crate::semantic_tokens;
use crate::symbols;

/// The main Orbis Language Server backend
pub struct OrbisBackend {
    /// LSP client for sending notifications
    client: Client,

    /// Document store - maps URI to document content and analysis
    documents: Arc<DashMap<Url, DocumentState>>,
}

/// State for a single document
struct DocumentState {
    /// The document content
    document: Document,

    /// Analysis results (parsed AST, symbols, errors)
    analysis: AnalysisResult,
}

impl OrbisBackend {
    /// Create a new Orbis backend
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Analyze a document and publish diagnostics
    async fn analyze_document(&self, uri: &Url) {
        if let Some(mut entry) = self.documents.get_mut(uri) {
            let content = entry.document.text();
            let version = entry.document.version;
            let result = analysis::Analyzer::analyze(&content, version);
            
            // Publish diagnostics
            let diags = diagnostics::to_diagnostics(&result, uri);
            self.client
                .publish_diagnostics(uri.clone(), diags, None)
                .await;

            entry.analysis = result;
        }
    }

    /// Get document context at a position
    fn get_context(&self, uri: &Url, position: Position) -> Option<DocumentContext> {
        self.documents.get(uri).map(|entry| {
            entry.document.get_context(&position)
        })
    }

    /// Get analysis result for a document
    fn get_analysis(&self, uri: &Url) -> Option<AnalysisResult> {
        self.documents.get(uri).map(|entry| entry.analysis.clone())
    }

    /// Get document
    fn get_document(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).map(|entry| entry.document.clone())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for OrbisBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("Orbis LSP initializing...");

        if let Some(root) = params.root_uri {
            info!("Workspace root: {}", root);
        }

        Ok(InitializeResult {
            capabilities: capabilities::server_capabilities(),
            server_info: Some(ServerInfo {
                name: "Orbis Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Orbis LSP initialized successfully");
        self.client
            .log_message(MessageType::INFO, "Orbis Language Server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Orbis LSP shutting down...");
        Ok(())
    }

    // ==================== Text Document Sync ====================

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = params.text_document.text;
        let version = params.text_document.version;

        debug!("Document opened: {}", uri);

        let document = Document::new(uri.clone(), content.clone(), version, "orbis".to_string());
        let analysis = analysis::Analyzer::analyze(&content, version);

        self.documents.insert(
            uri.clone(),
            DocumentState { document, analysis },
        );

        self.analyze_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        debug!("Document changed: {}", uri);

        if let Some(mut entry) = self.documents.get_mut(&uri) {
            // Apply incremental changes
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Incremental update
                    entry.document.apply_changes(vec![change.clone()], version);
                } else {
                    // Full content replacement
                    entry.document = Document::new(uri.clone(), change.text, version, "orbis".to_string());
                }
            }
        }

        self.analyze_document(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        debug!("Document saved: {}", uri);

        // Re-analyze on save to ensure fresh diagnostics
        self.analyze_document(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        debug!("Document closed: {}", uri);

        // Remove document from store
        self.documents.remove(&uri);

        // Clear diagnostics
        self.client
            .publish_diagnostics(uri, vec![], None)
            .await;
    }

    // ==================== Completion ====================

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        debug!("Completion requested at {}:{}", position.line, position.character);

        let Some(context) = self.get_context(uri, position) else {
            return Ok(None);
        };

        let analysis = self.get_analysis(uri);
        let symbols = analysis.as_ref().map(|a| &a.symbols);
        let items = completion::get_completions(&context, symbols.unwrap_or(&crate::analysis::SymbolTable::default()));

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        // Items are already fully resolved, just return as-is
        Ok(item)
    }

    // ==================== Hover ====================

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        debug!("Hover requested at {}:{}", position.line, position.character);

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        Ok(hover::get_hover(&document, &position, &analysis))
    }

    // ==================== Go to Definition ====================

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        debug!("Go to definition at {}:{}", position.line, position.character);

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        Ok(definition::get_definition(&document, &position, &analysis))
    }

    // ==================== Find References ====================

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        debug!("Find references at {}:{}", position.line, position.character);

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let locations = references::find_references(&document, &position, &analysis, include_declaration);

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    // ==================== Document Symbols ====================

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        debug!("Document symbols requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        Ok(symbols::get_document_symbols(&analysis))
    }

    // ==================== Workspace Symbols ====================

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = &params.query.to_lowercase();

        debug!("Workspace symbols query: {}", query);

        let mut results = Vec::new();

        // Search all documents
        for entry in self.documents.iter() {
            let uri = entry.key().clone();
            let analysis = &entry.value().analysis;

            // Add matching state symbols
            for (name, symbol) in &analysis.symbols.state_vars {
                if name.to_lowercase().contains(query) {
                    #[allow(deprecated)]
                    results.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: analysis::span_to_range(&symbol.span),
                        },
                        container_name: Some("state".to_string()),
                    });
                }
            }

            // Add matching fragments
            for (name, symbol) in &analysis.symbols.fragments {
                if name.to_lowercase().contains(query) {
                    #[allow(deprecated)]
                    results.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: analysis::span_to_range(&symbol.span),
                        },
                        container_name: None,
                    });
                }
            }

            // Add matching interfaces
            for (name, symbol) in &analysis.symbols.interfaces {
                if name.to_lowercase().contains(query) {
                    #[allow(deprecated)]
                    results.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::INTERFACE,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: analysis::span_to_range(&symbol.span),
                        },
                        container_name: None,
                    });
                }
            }
        }

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results))
        }
    }

    // ==================== Semantic Tokens ====================

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        debug!("Full semantic tokens requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        Ok(semantic_tokens::get_semantic_tokens(&analysis, &document))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        // For now, return full tokens - incremental optimization can come later
        let uri = &params.text_document.uri;

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        if let Some(result) = semantic_tokens::get_semantic_tokens(&analysis, &document) {
            match result {
                SemanticTokensResult::Tokens(tokens) => {
                    Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
                }
                SemanticTokensResult::Partial(partial) => {
                    Ok(Some(SemanticTokensRangeResult::Partial(partial)))
                }
            }
        } else {
            Ok(None)
        }
    }

    // ==================== Rename ====================

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        debug!("Prepare rename at {}:{}", position.line, position.character);

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let range = references::prepare_rename(&document, &position, &analysis);
        Ok(range.map(PrepareRenameResponse::Range))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        debug!("Rename to '{}' at {}:{}", new_name, position.line, position.character);

        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        // Find all references (including declaration)
        let locations = references::find_references(&document, &position, &analysis, true);

        if locations.is_empty() {
            return Ok(None);
        }

        // Create text edits for each location
        let mut changes: std::collections::HashMap<Url, Vec<TextEdit>> = std::collections::HashMap::new();

        for location in locations {
            let edit = TextEdit {
                range: location.range,
                new_text: new_name.clone(),
            };

            changes
                .entry(location.uri)
                .or_insert_with(Vec::new)
                .push(edit);
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    // ==================== Formatting ====================

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        debug!("Document formatting requested for {}", uri);

        // TODO: Implement formatting using orbis-dsl formatter when available
        // For now, return None to indicate formatting is not yet implemented
        // This will be connected to the CLI formatter in Phase 4

        warn!("Formatting not yet implemented - will be added with CLI tools");
        Ok(None)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        debug!(
            "Range formatting requested for {} ({}:{}-{}:{})",
            uri,
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character
        );

        // TODO: Implement range formatting
        warn!("Range formatting not yet implemented");
        Ok(None)
    }

    // ==================== Code Actions ====================

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = params.range;
        let diagnostics = &params.context.diagnostics;

        debug!("Code actions requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let mut actions = Vec::new();

        // Generate quick fixes for diagnostics
        for diag in diagnostics {
            if let Some(code) = &diag.code {
                match code {
                    NumberOrString::String(code_str) => {
                        // Check for "did you mean" suggestions
                        if let Some(data) = &diag.data {
                            if let Some(suggestion) = data.get("suggestion").and_then(|s| s.as_str()) {
                                let title = format!("Change to '{}'", suggestion);
                                
                                let edit = TextEdit {
                                    range: diag.range,
                                    new_text: suggestion.to_string(),
                                };

                                let mut changes = std::collections::HashMap::new();
                                changes.insert(uri.clone(), vec![edit]);

                                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                    title,
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: Some(vec![diag.clone()]),
                                    edit: Some(WorkspaceEdit {
                                        changes: Some(changes),
                                        document_changes: None,
                                        change_annotations: None,
                                    }),
                                    command: None,
                                    is_preferred: Some(true),
                                    disabled: None,
                                    data: None,
                                }));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Add refactoring actions if in appropriate context
        // TODO: Add extract fragment, inline fragment, etc.

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    // ==================== Folding Ranges ====================

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;

        debug!("Folding ranges requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let Some(ast) = &analysis.ast else {
            return Ok(None);
        };

        let mut ranges = Vec::new();

        // Add folding ranges for each top-level element
        for element in &ast.elements {
            let span = match element {
                orbis_dsl::ast::TopLevelElement::Page(p) => &p.span,
                orbis_dsl::ast::TopLevelElement::State(s) => &s.span,
                orbis_dsl::ast::TopLevelElement::Hooks(h) => &h.span,
                orbis_dsl::ast::TopLevelElement::Template(t) => &t.span,
                orbis_dsl::ast::TopLevelElement::Fragment(f) => &f.span,
                orbis_dsl::ast::TopLevelElement::Interface(i) => &i.span,
                orbis_dsl::ast::TopLevelElement::Styles(s) => &s.span,
                orbis_dsl::ast::TopLevelElement::Export(e) => &e.span,
                orbis_dsl::ast::TopLevelElement::Comment { span, .. } => span,
            };

            // Only add folding if it spans multiple lines
            if span.end_line > span.start_line {
                ranges.push(FoldingRange {
                    start_line: (span.start_line.saturating_sub(1)) as u32,
                    start_character: None,
                    end_line: (span.end_line.saturating_sub(1)) as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }

        // Add folding for imports if there are multiple
        if ast.imports.len() > 1 {
            if let (Some(first), Some(last)) = (ast.imports.first(), ast.imports.last()) {
                let first_line = match first {
                    orbis_dsl::ast::ImportStatement::TypeScript { span, .. } => span.start_line,
                    orbis_dsl::ast::ImportStatement::Rust { span, .. } => span.start_line,
                };
                let last_line = match last {
                    orbis_dsl::ast::ImportStatement::TypeScript { span, .. } => span.end_line,
                    orbis_dsl::ast::ImportStatement::Rust { span, .. } => span.end_line,
                };

                if last_line > first_line {
                    ranges.push(FoldingRange {
                        start_line: (first_line.saturating_sub(1)) as u32,
                        start_character: None,
                        end_line: (last_line.saturating_sub(1)) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Imports),
                        collapsed_text: None,
                    });
                }
            }
        }

        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
    }

    // ==================== Selection Ranges ====================

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let uri = &params.text_document.uri;

        debug!("Selection ranges requested for {}", uri);

        // For each position, return a hierarchy of selection ranges
        // This is a simplified implementation - a full one would use the AST
        let Some(document) = self.get_document(uri) else {
            return Ok(None);
        };

        let ranges: Vec<_> = params
            .positions
            .iter()
            .map(|pos| {
                // Get word at position, fall back to a minimal range at cursor
                let word_range = document
                    .word_at_position(pos)
                    .map(|(_, range)| range)
                    .unwrap_or(Range {
                        start: *pos,
                        end: *pos,
                    });
                
                // Create a simple hierarchy: word -> line -> document
                let line_range = Range {
                    start: Position {
                        line: pos.line,
                        character: 0,
                    },
                    end: Position {
                        line: pos.line + 1,
                        character: 0,
                    },
                };

                let doc_range = Range {
                    start: Position { line: 0, character: 0 },
                    end: Position {
                        line: document.line_count() as u32,
                        character: 0,
                    },
                };

                SelectionRange {
                    range: word_range,
                    parent: Some(Box::new(SelectionRange {
                        range: line_range,
                        parent: Some(Box::new(SelectionRange {
                            range: doc_range,
                            parent: None,
                        })),
                    })),
                }
            })
            .collect();

        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
    }

    // ==================== Document Links ====================

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = &params.text_document.uri;

        debug!("Document links requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let Some(ast) = &analysis.ast else {
            return Ok(None);
        };

        let mut links = Vec::new();

        // Add links for imports
        for import in &ast.imports {
            match import {
                orbis_dsl::ast::ImportStatement::TypeScript { path, span, .. } => {
                    // Could resolve relative paths to actual files
                    // For now, just mark as a potential link
                    if !path.starts_with("http") {
                        // Could be resolved to a local file
                        // TODO: Resolve import path to actual file
                    }
                }
                orbis_dsl::ast::ImportStatement::Rust { .. } => {
                    // Rust imports would need crate resolution
                }
            }
        }

        // Document links could include route references in the future
        // when page/route linking is implemented

        if links.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links))
        }
    }

    // ==================== Inlay Hints ====================

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;
        let range = params.range;

        debug!("Inlay hints requested for {}", uri);

        let Some(analysis) = self.get_analysis(uri) else {
            return Ok(None);
        };

        let mut hints = Vec::new();

        // Add type hints for inferred state variables
        for (name, state) in &analysis.symbols.state_vars {
            let state_range = analysis::span_to_range(&state.span);
            
            // Only include if in requested range
            if state_range.start.line >= range.start.line
                && state_range.end.line <= range.end.line
            {
                // If type is explicitly annotated, show it as a hint
                if let Some(type_annotation) = &state.type_annotation {
                    hints.push(InlayHint {
                        position: Position {
                            line: state_range.start.line,
                            character: state_range.end.character,
                        },
                        label: InlayHintLabel::String(format!("({})", type_annotation)),
                        kind: Some(InlayHintKind::TYPE),
                        text_edits: None,
                        tooltip: None,
                        padding_left: Some(true),
                        padding_right: Some(false),
                        data: None,
                    });
                }
            }
        }

        // Add parameter hints for fragment calls
        // TODO: When we have fragment call spans, add parameter name hints

        if hints.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hints))
        }
    }

    // ==================== Document Color ====================

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let uri = &params.text_document.uri;

        debug!("Document colors requested for {}", uri);

        // TODO: Parse color values from styles block
        // For now, return empty - can be implemented when styles are fully parsed
        Ok(vec![])
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let color = params.color;

        // Convert color to various formats
        let r = (color.red * 255.0) as u8;
        let g = (color.green * 255.0) as u8;
        let b = (color.blue * 255.0) as u8;
        let a = color.alpha;

        let mut presentations = vec![
            ColorPresentation {
                label: format!("#{:02x}{:02x}{:02x}", r, g, b),
                text_edit: None,
                additional_text_edits: None,
            },
            ColorPresentation {
                label: format!("rgb({}, {}, {})", r, g, b),
                text_edit: None,
                additional_text_edits: None,
            },
        ];

        if (a - 1.0).abs() > f32::EPSILON {
            presentations.push(ColorPresentation {
                label: format!("rgba({}, {}, {}, {:.2})", r, g, b, a),
                text_edit: None,
                additional_text_edits: None,
            });
        }

        Ok(presentations)
    }
}
