//! Orbis DSL - Domain Specific Language parsers for Orbis
//!
//! This crate provides multiple pest-based parsers for different aspects of the Orbis DSL,
//! each with automatic case-insensitive keyword matching. Keywords can be written in any
//! case format (snake_case, camelCase, PascalCase, kebab-case, etc.) and will be recognized
//! automatically.
//!
//! ## Available Grammars
//!
//! - **page**: Page definitions and UI elements
//! - **metadata**: Metadata and configuration fields
//!
//! ## Usage
//!
//! ```rust,ignore
//! use orbis_dsl::page::{Parser, Rule};
//! use pest::Parser as PestParser;
//!
//! let input = "longString: String"; // or long_string, LONG_STRING, etc.
//! let pairs = Parser::parse(Rule::field_declaration, input)?;
//! ```

pub mod page {
    //! Page grammar parser - handles page definitions and UI elements
    //!
    //! This module provides parsing for page-related DSL constructs with
    //! case-insensitive keyword matching.
     
    const _GRAMMAR: &str = include_str!("page.pest");
    
    // Uncomment to enable the parser:
    #[derive(pest_derive::Parser)]
    #[grammar = "page.pest"]
    pub struct Parser;
    
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn grammar_is_included() {
            assert!(_GRAMMAR.contains("@builder-insertion-start"));
            assert!(_GRAMMAR.contains("identifiers"));
        }
        
        #[test]
        fn grammar_contains_all_variants() {
            assert!(_GRAMMAR.contains("\"longString\""));
            assert!(_GRAMMAR.contains("\"long_string\""));
            assert!(_GRAMMAR.contains("\"LongString\""));
            assert!(_GRAMMAR.contains("\"LONG_STRING\""));
        }
    }
}

pub mod metadata {
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
}
