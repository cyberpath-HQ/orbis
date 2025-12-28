//! Semantic Tokens Tests
//!
//! Tests for verifying semantic token generation for syntax highlighting.

use orbis_lsp::semantic_tokens::get_semantic_tokens;
use orbis_lsp::analysis::Analyzer;
use orbis_lsp::document::Document;
use tower_lsp::lsp_types::Url;

/// Token types for easier reading
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

/// Helper to create a document from content
fn create_document(content: &str) -> Document {
    Document::new(
        Url::parse("file:///test.orbis").unwrap(),
        content.to_string(),
        1,
        "orbis".to_string(),
    )
}

/// Helper to get semantic tokens for content
/// Returns: Vec<(line, start, length, token_type, modifiers)>
fn get_tokens_for_content(content: &str) -> Vec<(u32, u32, u32, u32, u32)> {
    let document = create_document(content);
    let analysis = Analyzer::analyze(&document.text(), document.version);
    
    if let Some(result) = get_semantic_tokens(&analysis, &document) {
        match result {
            tower_lsp::lsp_types::SemanticTokensResult::Tokens(tokens) => {
                // Convert delta tokens back to absolute positions for easier testing
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

mod arrow_operator_tests {
    use super::*;

    #[test]
    fn test_mount_hook_arrow_highlighted() {
        let content = r#"
hooks {
    @mount => {
        console.log("mounted")
    }
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Find arrow operator token (type 10 = OPERATOR)
        let arrow_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2) // OPERATOR with length 2
            .collect();
        
        assert!(!arrow_tokens.is_empty(), "Arrow operator => should be highlighted");
    }

    #[test]
    fn test_unmount_hook_arrow_highlighted() {
        let content = r#"
hooks {
    @unmount => {
        console.log("unmounting")
    }
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let arrow_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2)
            .collect();
        
        assert!(!arrow_tokens.is_empty(), "Arrow operator => should be highlighted for @unmount");
    }

    #[test]
    fn test_watcher_arrow_highlighted() {
        let content = r#"
state {
    count = 0
}
hooks {
    @watch(state.count) => {
        console.log("count changed")
    }
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let arrow_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2)
            .collect();
        
        assert!(!arrow_tokens.is_empty(), "Arrow operator => should be highlighted for @watch");
    }

    #[test]
    fn test_event_handler_arrow_highlighted() {
        let content = r#"
template {
    <Button @click => { console.log("clicked") } />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Find arrow tokens on the same line as @click
        let arrow_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2)
            .collect();
        
        assert!(!arrow_tokens.is_empty(), "Arrow operator => should be highlighted in event handlers");
    }
}

mod component_tests {
    use super::*;

    #[test]
    fn test_component_in_template_highlighted() {
        let content = r#"
template {
    <Container>
        <Text content="Hello" />
    </Container>
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Find CLASS tokens (type 2) for components
        let class_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 2)
            .collect();
        
        assert!(class_tokens.len() >= 2, "Should have at least 2 component tokens (Container, Text)");
    }

    #[test]
    fn test_component_after_if_highlighted() {
        let content = r#"
template {
    if true {
        <Text content="inside if" />
    }
    <Button label="after if" />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let class_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 2)
            .collect();
        
        assert!(class_tokens.len() >= 2, "Components inside and after if should be highlighted");
    }

    #[test]
    fn test_closing_tag_highlighted() {
        let content = r#"
template {
    <Container>
        <Text content="Hello" />
    </Container>
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Count CLASS tokens - should include closing tag component name
        let class_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 2)
            .collect();
        
        println!("CLASS tokens found: {}", class_tokens.len());
        for (i, token) in class_tokens.iter().enumerate() {
            println!("Token {}: Line {}, Col {}, Len {}", i, token.0, token.1, token.2);
        }
        
        // Container opening + Text + Container closing = at least 3
        assert!(class_tokens.len() >= 3, "Closing tag component name should be highlighted, got {} CLASS tokens", class_tokens.len());
    }

    #[test]
    fn test_multiline_self_closing_highlighted() {
        let content = r#"
template {
    <Button 
        label="Click"
        @click => { console.log("clicked") }
    />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Find OPERATOR token for />
        let slash_gt_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2) // OPERATOR with length 2
            .collect();
        
        // Should have at least 2: one for =>, one for />
        assert!(slash_gt_tokens.len() >= 2, "Multi-line self-closing /> should be highlighted");
    }
}

mod comment_tests {
    use super::*;

    #[test]
    fn test_line_comment_highlighted() {
        let content = r#"
// This is a comment
page {
    id: "test"
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Find COMMENT tokens (type 11)
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 11)
            .collect();
        
        assert!(!comment_tokens.is_empty(), "Line comments should be highlighted");
    }

    #[test]
    fn test_block_comment_highlighted() {
        let content = r#"
/* Block comment */
page {
    id: "test"
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 11)
            .collect();
        
        assert!(!comment_tokens.is_empty(), "Block comments should be highlighted");
    }

    #[test]
    fn test_docblock_comment_highlighted() {
        let content = r#"
/**
 * Docblock comment
 */
page {
    id: "test"
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 11)
            .collect();
        
        assert!(!comment_tokens.is_empty(), "Docblock comments should be highlighted");
    }

    #[test]
    fn test_template_comment_highlighted() {
        let content = r#"
template {
    // Comment inside template
    <Container />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let comment_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 11)
            .collect();
        
        assert!(!comment_tokens.is_empty(), "Comments inside template should be highlighted");
    }
}

mod fragment_tests {
    use super::*;

    #[test]
    fn test_fragment_usage_not_error() {
        let content = r#"
fragment MyCard() {
    <Card />
}

template {
    <MyCard />
}
"#;
        let document = create_document(content);
        let analysis = Analyzer::analyze(&document.text(), document.version);
        
        // Check no "undefined component" errors for MyCard
        let undefined_errors: Vec<_> = analysis.errors.iter()
            .filter(|e| e.message.contains("Undefined") && e.message.contains("my_card"))
            .collect();
        
        assert!(undefined_errors.is_empty(), "Fragment usage should not show as undefined");
    }
}

/// Integration tests using real example files from the codebase
mod integration_tests {
    use super::*;

    /// Test that parses and generates tokens for the simple.orbis example
    #[test]
    fn test_simple_orbis_file() {
        let content = r#"
page {
    id: "greeting-page"
    title: "Quick Start"
}

state {
    name: string = "World"
    count = 0
}

hooks {
    @mount => {
        console.log("Page mounted")
    }
}

template {
    <Container>
        <Button 
            label="Click Me!"
            @click => {
                state.count = state.count + 1
            }
        />
    </Container>
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Should have multiple tokens
        assert!(tokens.len() > 10, "Should generate many tokens for a complex file");
        
        // Check for key token types
        let has_keyword = tokens.iter().any(|t| t.3 == 7); // KEYWORD
        let has_class = tokens.iter().any(|t| t.3 == 2); // CLASS for components
        let has_operator = tokens.iter().any(|t| t.3 == 10); // OPERATOR for =>
        let has_decorator = tokens.iter().any(|t| t.3 == 12); // DECORATOR for @mount
        
        assert!(has_keyword, "Should have keyword tokens");
        assert!(has_class, "Should have class tokens for components");
        assert!(has_operator, "Should have operator tokens for =>");
        assert!(has_decorator, "Should have decorator tokens for @mount");
    }

    /// Test comprehensive grammar example
    #[test]
    fn test_comprehensive_grammar() {
        let content = r#"
// Line comment
/**
 * Docblock comment
 */

import { Button, Card } from "@orbis/ui"

interface User {
    id: string
    name: string
}

page {
    id: "test-page"
    title: "Test"
}

state {
    count = 0
    name: string = "Test"
    computed: number => state.count * 2
}

hooks {
    @mount => {
        console.log("mounted")
    }
    
    @watch(state.count) => {
        console.log("count changed")
    }
}

fragment UserCard(user: User) {
    <Card>
        <Text>{user.name}</Text>
    </Card>
}

template {
    // Template comment
    <Container>
        if state.count > 0 {
            <Text content="Has items" />
        } else {
            <Text content="Empty" />
        }
        
        for item in state.items {
            <ListItem key={item.id} />
        }
        
        <Button 
            label="Click"
            @click => {state.count = state.count + 1}
        />
    </Container>
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Count different token types
        let keywords: Vec<_> = tokens.iter().filter(|t| t.3 == 7).collect();
        let classes: Vec<_> = tokens.iter().filter(|t| t.3 == 2).collect();
        let operators: Vec<_> = tokens.iter().filter(|t| t.3 == 10).collect();
        let comments: Vec<_> = tokens.iter().filter(|t| t.3 == 11).collect();
        let decorators: Vec<_> = tokens.iter().filter(|t| t.3 == 12).collect();
        let functions: Vec<_> = tokens.iter().filter(|t| t.3 == 3).collect();
        
        // Verify we have a good spread of token types
        assert!(keywords.len() >= 5, "Should have multiple keywords (page, state, hooks, template, if, else, for)");
        assert!(classes.len() >= 3, "Should have multiple class tokens for components");
        assert!(operators.len() >= 3, "Should have operator tokens for => arrows");
        assert!(comments.len() >= 2, "Should have comment tokens");
        assert!(decorators.len() >= 2, "Should have decorator tokens for @mount, @watch");
        assert!(functions.len() >= 1, "Should have function tokens for fragments");
    }

    /// Test nested control flow highlighting
    #[test]
    fn test_nested_control_flow() {
        let content = r#"
template {
    if state.showOuter {
        <Container>
            if state.showInner {
                <Text content="Inner" />
            }
        </Container>
    }
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Should have tokens for Container and Text
        let class_tokens: Vec<_> = tokens.iter().filter(|t| t.3 == 2).collect();
        assert!(class_tokens.len() >= 2, "Should highlight components inside nested control flow");
    }

    /// Test multi-line event handlers
    #[test]
    fn test_multiline_event_handler() {
        let content = r#"
template {
    <Button 
        label="Submit"
        @click => {
            state.loading = true,
            api.call("submit"),
            toast.show("Done")
        }
    />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Should have EVENT token for @click
        let event_tokens: Vec<_> = tokens.iter().filter(|t| t.3 == 13).collect();
        assert!(!event_tokens.is_empty(), "Should have event tokens for @click");
        
        // Should have OPERATOR token for =>
        let operator_tokens: Vec<_> = tokens.iter()
            .filter(|t| t.3 == 10 && t.2 == 2)
            .collect();
        assert!(!operator_tokens.is_empty(), "Should have operator token for =>");
    }
}

/// Edge case tests
mod edge_case_tests {
    use super::*;

    /// Test empty template
    #[test]
    fn test_empty_template() {
        let content = r#"
template {
}
"#;
        let tokens = get_tokens_for_content(content);
        // Should at least have the keyword token
        let keyword_tokens: Vec<_> = tokens.iter().filter(|t| t.3 == 7).collect();
        assert!(!keyword_tokens.is_empty(), "Should have template keyword");
    }

    /// Test deeply nested components
    #[test]
    fn test_deeply_nested_components() {
        let content = r#"
template {
    <Outer>
        <Middle>
            <Inner>
                <Deep />
            </Inner>
        </Middle>
    </Outer>
}
"#;
        let tokens = get_tokens_for_content(content);
        
        // Should have 4 opening + 3 closing = 7 class tokens
        let class_tokens: Vec<_> = tokens.iter().filter(|t| t.3 == 2).collect();
        assert!(class_tokens.len() >= 7, "Should highlight all nested components including closing tags, got {}", class_tokens.len());
    }

    /// Test component after multiple control blocks
    #[test]
    fn test_component_after_multiple_control_blocks() {
        let content = r#"
template {
    if state.a {
        <First />
    }
    
    if state.b {
        <Second />
    }
    
    <Third />
}
"#;
        let tokens = get_tokens_for_content(content);
        
        let class_tokens: Vec<_> = tokens.iter().filter(|t| t.3 == 2).collect();
        assert!(class_tokens.len() >= 3, "Should highlight all components, got {}", class_tokens.len());
    }
}

