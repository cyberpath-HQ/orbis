// Regression tests for specific user-reported issues

use orbis_lsp::semantic_tokens::get_semantic_tokens;
use orbis_lsp::analysis::Analyzer;
use orbis_lsp::document::Document;
use tower_lsp::lsp_types::Url;

fn create_document(content: &str) -> Document {
    Document::new(
        Url::parse("file:///test.orbis").unwrap(),
        content.to_string(),
        1,
        "orbis".to_string(),
    )
}

fn get_tokens_for_content(content: &str) -> Vec<(u32, u32, u32, u32, u32)> {
    let document = create_document(content);
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
fn test_comment_last_character_highlighted() {
    let content = r#"
template {
    // This is a test comment
    <Text content="test" />
}
"#;
    let tokens = get_tokens_for_content(content);
    
    // Find the comment token
    let comment_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.3 == 11) // COMMENT token type
        .collect();
    
    println!("Comment tokens: {:?}", comment_tokens);
    assert!(!comment_tokens.is_empty(), "Comment should be highlighted");
    
    // Comment is "// This is a test comment" = 26 characters
    let comment = comment_tokens[0];
    assert!(comment.2 >= 26, "Comment length should include all characters including last one, got {}", comment.2);
}

#[test]
fn test_multiline_self_closing_tag_closing_highlighted() {
    let content = r#"
template {
    <Button 
        label="Click"
        @click => {
            console.log("test")
        }
    />
}
"#;
    let tokens = get_tokens_for_content(content);
    
    println!("\n=== ALL TOKENS ===");
    for token in &tokens {
        println!("Line {}, Col {}, Len {}, Type {}", token.0, token.1, token.2, token.3);
    }
    
    // Find /> operator token - should be somewhere in the output
    let closing_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.3 == 10 && t.2 == 2) // OPERATOR type, length 2 for =>
        .collect();
    
    println!("\n=== OPERATOR TOKENS (len=2) ===");
    for token in &closing_tokens {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    
    // Look for /> specifically - it should be on the last line
    let slash_closing: Vec<_> = closing_tokens.iter()
        .filter(|t| t.0 >= 6) // Line 7 or later
        .collect();
    
    assert!(!slash_closing.is_empty(), "Multi-line self-closing /> should be highlighted");
}

#[test]
fn test_opening_tag_after_event_handler_highlighted() {
    let content = r#"
template {
    <Form @submit => { console.log("test") }>
        <Field 
            type="text"
        />
    </Form>
}
"#;
    let tokens = get_tokens_for_content(content);
    
    // Find Field component token on line 3 (0-indexed = 2)
    let field_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.0 == 2 && t.3 == 2) // Line 3, CLASS type
        .collect();
    
    println!("Field tokens on line 3: {:?}", field_tokens);
    assert!(!field_tokens.is_empty(), "Component after event handler should be highlighted");
}

#[test]
fn test_closing_tag_correct_color() {
    let content = r#"
fragment Example() {
    <Text id="test"/>
}

template {
    <Example>
    </Example>
}
"#;
    let tokens = get_tokens_for_content(content);
    
    // Find closing tag tokens - should be CLASS type (2) for component names
    let closing_example: Vec<_> = tokens.iter()
        .filter(|t| t.0 == 6 && t.3 == 2) // Line 7, CLASS type
        .collect();
    
    println!("Closing Example tokens: {:?}", closing_example);
    assert!(!closing_example.is_empty(), "Closing tag should use CLASS token type (green), not FUNCTION");
}

#[test]
fn test_hook_arrow_with_params_highlighted() {
    let content = r#"
hooks {
    @watch(state.count) => {
        console.log(state.count)
    }
}
"#;
    let tokens = get_tokens_for_content(content);
    
    println!("\n=== ALL TOKENS ===");
    for token in &tokens {
        println!("Line {}, Col {}, Len {}, Type {}", token.0, token.1, token.2, token.3);
    }
    
    // Find => arrow token - should be on the line with @watch
    let arrow_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.3 == 10 && t.2 == 2) // OPERATOR type, length 2
        .collect();
    
    println!("\n=== ARROW TOKENS ===");
    for token in &arrow_tokens {
        println!("Line {}, Col {}, Len {}", token.0, token.1, token.2);
    }
    
    assert!(!arrow_tokens.is_empty(), "@watch(...) => arrow should be highlighted");
}

#[test]
fn test_all_template_comment_lengths() {
    let content = std::fs::read_to_string(
        "/home/ebalo/Desktop/projects/rust/orbis-assets/examples/component-whitelisting-simple.orbis"
    ).expect("Failed to read file");
    
    let tokens = get_tokens_for_content(&content);
    
    // Get all comment tokens
    let comment_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.3 == 11) // COMMENT type
        .collect();
    
    println!("\n=== ALL COMMENT TOKENS ===");
    for token in &comment_tokens {
        println!("Line {}, Col {}, Len {}", token.0 + 1, token.1, token.2);
    }
    
    // Check that at least one comment has proper length
    // Line 40: "// Container with whitelisted attributes" = 42 chars
    let line_40_comments: Vec<_> = comment_tokens.iter()
        .filter(|t| t.0 == 39) // 0-indexed
        .collect();
    
    if let Some(comment) = line_40_comments.first() {
        println!("Line 40 comment length: {}", comment.2);
        
        // Read actual file line to verify
        let file_content = std::fs::read_to_string(
            "/home/ebalo/Desktop/projects/rust/orbis-assets/examples/component-whitelisting-simple.orbis"
        ).expect("Failed to read file");
        let lines: Vec<&str> = file_content.lines().collect();
        if lines.len() > 39 {
            let line_40_text = lines[39]; // 0-indexed
            println!("Line 40 actual text: {:?}", line_40_text);
            println!("From col 4: {:?}", &line_40_text[4..]);
            println!("Length from col 4: {}", line_40_text[4..].len());
        }
        
        // Comment should be "// Container with whitelisted attributes" from source
        // That's 42 characters starting from col 4
        assert!(comment.2 >= 40, "Comment should include all characters, got {}", comment.2);
    }
}
