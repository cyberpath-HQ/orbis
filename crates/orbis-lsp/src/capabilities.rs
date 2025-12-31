//! Server Capabilities
//!
//! This module defines the LSP capabilities that the Orbis language server supports.

use tower_lsp::lsp_types::{
    CompletionOptions, HoverProviderCapability, OneOf, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};

/// Get the server capabilities
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // Document synchronization - incremental updates
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),

        // Completion
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![
                "<".to_string(),   // Component start
                ".".to_string(),   // Member access
                "@".to_string(),   // Event binding
                "{".to_string(),   // Expression start
                "\"".to_string(),  // String
                ":".to_string(),   // Type annotation
                " ".to_string(),   // Attribute after space
                "/".to_string(),   // Import path
            ]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
            completion_item: None,
        }),

        // Hover
        hover_provider: Some(HoverProviderCapability::Simple(true)),

        // Go to definition
        definition_provider: Some(OneOf::Left(true)),

        // Find references
        references_provider: Some(OneOf::Left(true)),

        // Document symbols (outline)
        document_symbol_provider: Some(OneOf::Left(true)),

        // Semantic tokens for rich highlighting
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: semantic_tokens_legend(),
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),

        // Workspace capabilities
        workspace: None,

        // Document highlight (same symbol highlighting)
        document_highlight_provider: None,

        // Other capabilities we don't currently support
        signature_help_provider: None,
        type_definition_provider: None,
        implementation_provider: None,
        code_action_provider: None,
        code_lens_provider: None,
        document_formatting_provider: None,
        document_range_formatting_provider: None,
        document_on_type_formatting_provider: None,
        rename_provider: None,
        document_link_provider: None,
        color_provider: None,
        folding_range_provider: None,
        declaration_provider: None,
        execute_command_provider: None,
        workspace_symbol_provider: None,
        call_hierarchy_provider: None,
        moniker_provider: None,
        linked_editing_range_provider: None,
        selection_range_provider: None,
        position_encoding: None,
        inline_value_provider: None,
        inlay_hint_provider: None,
        diagnostic_provider: None,
        experimental: None,
    }
}

/// Semantic token types supported by Orbis LSP
pub const SEMANTIC_TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,     // state, page, template, hooks
    SemanticTokenType::TYPE,          // Type annotations
    SemanticTokenType::CLASS,         // Component names
    SemanticTokenType::FUNCTION,      // Fragment names, actions
    SemanticTokenType::PARAMETER,     // Fragment parameters
    SemanticTokenType::VARIABLE,      // State variables
    SemanticTokenType::PROPERTY,      // Object properties, attributes
    SemanticTokenType::KEYWORD,       // if, else, for, when, etc.
    SemanticTokenType::STRING,        // String literals
    SemanticTokenType::NUMBER,        // Number literals
    SemanticTokenType::OPERATOR,      // Operators
    SemanticTokenType::COMMENT,       // Comments
    SemanticTokenType::DECORATOR,     // @mount, @watch, @computed
    SemanticTokenType::EVENT,         // Event names like click, change
    SemanticTokenType::MACRO,         // Special variables like $response
];

/// Semantic token modifiers
pub const SEMANTIC_TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,     // Variable/fragment declaration
    SemanticTokenModifier::DEFINITION,      // Definition site
    SemanticTokenModifier::READONLY,        // Computed properties
    SemanticTokenModifier::DEPRECATED,      // Deprecated items
    SemanticTokenModifier::MODIFICATION,    // Assignment target
    SemanticTokenModifier::DOCUMENTATION,   // Doc comments
    SemanticTokenModifier::DEFAULT_LIBRARY, // Built-in components
];

/// Get the semantic tokens legend
pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: SEMANTIC_TOKEN_TYPES.to_vec(),
        token_modifiers: SEMANTIC_TOKEN_MODIFIERS.to_vec(),
    }
}

/// Get the index for a semantic token type
pub fn token_type_index(token_type: SemanticTokenType) -> u32 {
    SEMANTIC_TOKEN_TYPES
        .iter()
        .position(|t| *t == token_type)
        .unwrap_or(0) as u32
}

/// Get the modifier bitfield for a set of modifiers
pub fn token_modifiers_bitfield(modifiers: &[SemanticTokenModifier]) -> u32 {
    let mut bitfield = 0u32;
    for modifier in modifiers {
        if let Some(idx) = SEMANTIC_TOKEN_MODIFIERS.iter().position(|m| m == modifier) {
            bitfield |= 1 << idx;
        }
    }
    bitfield
}
