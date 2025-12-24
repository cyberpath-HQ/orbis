//! Metadata grammar parser - handles metadata and configuration
//!
//! This module provides parsing for metadata-related DSL constructs with
//! case-insensitive keyword matching.

const _GRAMMAR: &str = include_str!("metadata.pest");

// Uncomment to enable the parser:
#[derive(pest_derive::Parser)]
#[grammar = "metadata.pest"]
pub struct Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_is_included() {
        assert!(_GRAMMAR.contains("@builder-insertion-start"));
        assert!(_GRAMMAR.contains("meta_fields"));
    }

    #[test]
    fn grammar_contains_all_variants() {
        assert!(_GRAMMAR.contains("\"author\""));
        assert!(_GRAMMAR.contains("\"Author\""));
        assert!(_GRAMMAR.contains("\"AUTHOR\""));
    }
}
