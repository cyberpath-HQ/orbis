//! Test with actual .orbis files to verify real behavior

use orbis_lsp::semantic_tokens::get_semantic_tokens;
use orbis_lsp::analysis::Analyzer;
use orbis_lsp::document::Document;
use tower_lsp::lsp_types::Url;
use std::fs;

/// Token types
mod token_types {
    pub const NAMESPACE: u32 = 0;
    pub const TYPE: u32 = 1;
    pub const CLASS: u32 = 2;
    pub const FUNCTION: u32 = 3;
    pub const PARAMETER: u32 = 4;
    pub const VARIABLE: u32 = 5;
    pub const PROPERTY: u32 = 6;
    pub const KEYWORD: u32 = 7;
    pub const STRING: u32 = 8;
    pub const NUMBER: u32 = 9;
    pub const OPERATOR: u32 = 10;
    pub const COMMENT: u32 = 11;
    pub const DECORATOR: u32 = 12;
    pub const EVENT: u32 = 13;
}

fn get_tokens_for_file(path: &str) -> Vec<(u32, u32, u32, u32, u32)> {
    let content = fs::read_to_string(path).expect("Failed to read file");
    let document = Document::new(
        Url::parse("file:///test.orbis").unwrap(),
        content.clone(),
        1,
        "orbis".to_string(),
    );
    let analysis = Analyzer::analyze(&document.text(), document.version);
    
    if let Some(result) = get_semantic_tokens(&analysis, &document) {
        match result {
            tower_lsp::lsp_types::SemanticTokensResult::Tokens(tokens) => {
                let mut absolute_tokens = Vec::new();
                let mut prev_line = 0u32;
                let mut prev_start = 0u32;
                
                for token in tokens.data {
                    let line = prev_line + token.delta_line;
                    let start = if token.delta_line == 0 {
                        prev_start + token.delta_start
                    } else {
                        token.delta_start
                    };
                    
                    absolute_tokens.push((
                        line,
                        start,
                        token.length,
                        token.token_type,
                        token.token_modifiers_bitset,
                    ));
                    
                    prev_line = line;
                    prev_start = start;
                }
                
                return absolute_tokens;
            }
            _ => {}
        }
    }
    
    vec![]
}

#[test]
fn test_component_whitelisting_simple() {
    let tokens = get_tokens_for_file("../../examples/component-whitelisting-simple.orbis");
    
    println!("\n=== ALL TOKENS ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: Line {}, Col {}, Len {}, Type {} ({})", 
            i, token.0, token.1, token.2, token.3, 
            match token.3 {
                0 => "NAMESPACE",
                1 => "TYPE",
                2 => "CLASS",
                3 => "FUNCTION",
                4 => "PARAMETER",
                5 => "VARIABLE",
                6 => "PROPERTY",
                7 => "KEYWORD",
                8 => "STRING",
                9 => "NUMBER",
                10 => "OPERATOR",
                11 => "COMMENT",
                12 => "DECORATOR",
                13 => "EVENT",
                _ => "UNKNOWN"
            }
        );
    }
    
    // Check for docblock comment (lines 11-17)
    let docblock_comments: Vec<_> = tokens.iter()
        .filter(|t| t.0 >= 11 && t.0 <= 17 && t.3 == 11)
        .collect();
    println!("\n=== DOCBLOCK COMMENTS (lines 11-17) ===");
    for token in &docblock_comments {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    assert!(!docblock_comments.is_empty(), "Docblock should be highlighted");
    
    // Check for @mount arrow (line 25 in file = line 24 0-indexed)
    let mount_arrows: Vec<_> = tokens.iter()
        .filter(|t| (t.0 >= 23 && t.0 <= 25) && t.3 == 10 && t.2 == 2)
        .collect();
    println!("\n=== @mount => ARROWS (lines 24-26) ===");
    for token in &mount_arrows {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    assert!(!mount_arrows.is_empty(), "@mount => arrow should be highlighted");
    
    // Check template comments (line 40+ in file)
    // After fixing parser, ALL comments should be captured
    let template_comments: Vec<_> = tokens.iter()
        .filter(|t| t.0 >= 38 && t.0 <= 110 && t.3 == 11)
        .collect();
    println!("\n=== TEMPLATE COMMENTS (lines 39-111) ===");
    for token in &template_comments {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    // Should have comments on lines: 40, 45, 47, 49, 53, 72, 83, 89, 92, 95, 104, 109 = 12 total
    assert!(template_comments.len() >= 12, "All template comments should be highlighted, found {}", template_comments.len());
    
    // Check Button with @click handler (lines 76-82)
    // Check Button component (line 73 in file = line 72 0-indexed)
    let button_components: Vec<_> = tokens.iter()
        .filter(|t| t.0 == 72 && t.3 == 2) // CLASS token for "Button"
        .collect();
    println!("\n=== BUTTON COMPONENTS (line 73) ===");
    for token in &button_components {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    assert!(!button_components.is_empty(), "Button component should be highlighted");
    
    // Check self-closing /> for Button (line 81 in file = line 80 0-indexed)
    let button_closings: Vec<_> = tokens.iter()
        .filter(|t| t.0 == 80 && t.3 == 10 && t.2 == 2) // /> operator
        .collect();
    println!("\n=== BUTTON /> CLOSINGS (lines 76-82) ===");
    for token in &button_closings {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    
    // Check components inside if block (lines 98-102)
    let if_block_components: Vec<_> = tokens.iter()
        .filter(|t| t.0 >= 97 && t.0 <= 102 && t.3 == 2)
        .collect();
    println!("\n=== COMPONENTS IN IF BLOCK (lines 98-102) ===");
    for token in &if_block_components {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    assert!(!if_block_components.is_empty(), "Components in if block should be highlighted");
    
    // Component after if block (line 102 - Skeleton)
    let after_if_components: Vec<_> = tokens.iter()
        .filter(|t| t.0 == 101 && t.3 == 2) // CLASS token
        .collect();
    println!("\n=== COMPONENTS AFTER IF BLOCK (line 102) ===");
    for token in &after_if_components {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    assert!(!after_if_components.is_empty(), "Components after if block should be highlighted");
}
