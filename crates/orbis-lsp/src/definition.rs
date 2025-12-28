//! Go to Definition Provider
//!
//! This module provides go-to-definition functionality for:
//! - State variables
//! - Fragments
//! - Interfaces
//! - Imports

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

use crate::analysis::{span_to_range, AnalysisResult, SymbolKind, SymbolTable};
use crate::document::Document;

/// Get definition location for a symbol at the given position
pub fn get_definition(
    doc: &Document,
    pos: &Position,
    result: &AnalysisResult,
) -> Option<GotoDefinitionResponse> {
    // Get the word at position
    let (word, _range) = doc.word_at_position(pos)?;

    // Check if it's a state.xxx reference
    let prefix = doc.get_line(pos.line as usize)?;
    let col = pos.character as usize;
    let before_word = if col > word.len() {
        &prefix[..col - word.len()]
    } else {
        ""
    };

    // Handle state.variable
    if before_word.ends_with("state.") || word.starts_with("state.") {
        let var_name = if word.starts_with("state.") {
            word.strip_prefix("state.").unwrap_or(&word)
        } else {
            &word
        };

        if let Some(sym) = result.symbols.state_vars.get(var_name) {
            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: doc.uri.clone(),
                range: span_to_range(&sym.span),
            }));
        }
    }

    // Check state variables
    if let Some(sym) = result.symbols.state_vars.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: doc.uri.clone(),
            range: span_to_range(&sym.span),
        }));
    }

    // Check fragments
    if let Some(frag) = result.symbols.fragments.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: doc.uri.clone(),
            range: span_to_range(&frag.span),
        }));
    }

    // Check interfaces
    if let Some(iface) = result.symbols.interfaces.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: doc.uri.clone(),
            range: span_to_range(&iface.span),
        }));
    }

    // Check imports (would navigate to the import statement)
    if let Some(imp) = result.symbols.imports.get(&word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: doc.uri.clone(),
            range: span_to_range(&imp.span),
        }));
    }

    None
}

/// Get type definition for a symbol (navigates to type declaration)
pub fn get_type_definition(
    doc: &Document,
    pos: &Position,
    result: &AnalysisResult,
) -> Option<GotoDefinitionResponse> {
    let (word, _range) = doc.word_at_position(pos)?;

    // If it's a state variable, try to find its type
    if let Some(sym) = result.symbols.state_vars.get(&word) {
        if let Some(type_name) = &sym.type_annotation {
            // Try to find an interface with this name
            if let Some(iface) = result.symbols.interfaces.get(type_name) {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: doc.uri.clone(),
                    range: span_to_range(&iface.span),
                }));
            }
        }
    }

    None
}
