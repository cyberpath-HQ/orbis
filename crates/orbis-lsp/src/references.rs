//! Find References Provider
//!
//! This module provides find-all-references functionality for:
//! - State variables
//! - Fragments
//! - Interfaces

use tower_lsp::lsp_types::{Location, Position, Url};

use crate::analysis::{span_to_range, AnalysisResult, SymbolKind};
use crate::document::Document;

/// Find all references to a symbol at the given position
pub fn find_references(
    doc: &Document,
    pos: &Position,
    result: &AnalysisResult,
    include_declaration: bool,
) -> Vec<Location> {
    let mut locations = Vec::new();

    // Get the word at position
    let (word, _range) = match doc.word_at_position(pos) {
        Some(w) => w,
        None => return locations,
    };

    // Handle state.variable syntax
    let search_word = if word.starts_with("state.") {
        word.strip_prefix("state.").unwrap_or(&word).to_string()
    } else {
        word.clone()
    };

    // Determine what kind of symbol this is
    let symbol_kind = if result.symbols.state_vars.contains_key(&search_word) {
        Some(SymbolKind::StateVariable)
    } else if result.symbols.fragments.contains_key(&search_word) {
        Some(SymbolKind::Fragment)
    } else if result.symbols.interfaces.contains_key(&search_word) {
        Some(SymbolKind::Interface)
    } else {
        None
    };

    let symbol_kind = match symbol_kind {
        Some(k) => k,
        None => return locations,
    };

    // Find all references
    for reference in &result.symbols.references {
        if reference.name == search_word && reference.kind == symbol_kind {
            // Skip declaration if not including it
            if !include_declaration && reference.is_definition {
                continue;
            }

            locations.push(Location {
                uri: doc.uri.clone(),
                range: span_to_range(&reference.span),
            });
        }
    }

    locations
}

/// Prepare rename operation (check if symbol can be renamed)
pub fn prepare_rename(
    doc: &Document,
    pos: &Position,
    result: &AnalysisResult,
) -> Option<tower_lsp::lsp_types::Range> {
    let (word, range) = doc.word_at_position(pos)?;

    // Can only rename local symbols (state, fragments, interfaces)
    if result.symbols.state_vars.contains_key(&word)
        || result.symbols.fragments.contains_key(&word)
        || result.symbols.interfaces.contains_key(&word)
    {
        Some(range)
    } else {
        None
    }
}
