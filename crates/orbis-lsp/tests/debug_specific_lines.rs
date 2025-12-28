use orbis_lsp::semantic_tokens::get_semantic_tokens;
use orbis_lsp::analysis::Analyzer;
use orbis_lsp::document::Document;
use tower_lsp::lsp_types::Url;

#[test]
fn debug_component_whitelisting_specific_lines() {
    let content = std::fs::read_to_string(
        "/home/ebalo/Desktop/projects/rust/orbis-assets/examples/component-whitelisting-simple.orbis"
    ).expect("Failed to read file");
    
    let document = Document::new(
        Url::parse("file:///test.orbis").unwrap(),
        content.clone(),
        1,
        "orbis".to_string(),
    );
    
    let analysis = Analyzer::analyze(&document.text(), document.version);
    
    let result = get_semantic_tokens(&analysis, &document);
    
    if let Some(tower_lsp::lsp_types::SemanticTokensResult::Tokens(tokens)) = result {
        let mut absolute_tokens = Vec::new();
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;
        
        for token in &tokens.data {
            let line = prev_line + token.delta_line;
            let start = if token.delta_line == 0 {
                prev_start + token.delta_start
            } else {
                token.delta_start
            };
            
            absolute_tokens.push((line, start, token.length, token.token_type, token.token_modifiers_bitset));
            
            prev_line = line;
            prev_start = start;
        }
        
        println!("\n=== HOOKS BLOCK (lines 22-32) ===");
        for token in &absolute_tokens {
            if token.0 >= 22 && token.0 <= 32 {
                let line_content = content.lines().nth(token.0 as usize).unwrap_or("");
                let token_text = &line_content.chars().skip(token.1 as usize).take(token.2 as usize).collect::<String>();
                println!("Line {}, Col {}, Len {}, Type {}, Text: {:?}", 
                    token.0 + 1, token.1, token.2, token.3, token_text);
            }
        }
        
        println!("\n=== FIRST FIELD (lines 54-62) ===");
        for token in &absolute_tokens {
            if token.0 >= 53 && token.0 <= 62 {
                let line_content = content.lines().nth(token.0 as usize).unwrap_or("");
                let token_text = &line_content.chars().skip(token.1 as usize).take(token.2 as usize).collect::<String>();
                println!("Line {}, Col {}, Len {}, Type {}, Text: {:?}", 
                    token.0 + 1, token.1, token.2, token.3, token_text);
            }
        }
        
        println!("\n=== SECOND FIELD (lines 64-69) ===");
        for token in &absolute_tokens {
            if token.0 >= 63 && token.0 <= 69 {
                let line_content = content.lines().nth(token.0 as usize).unwrap_or("");
                let token_text = &line_content.chars().skip(token.1 as usize).take(token.2 as usize).collect::<String>();
                println!("Line {}, Col {}, Len {}, Type {}, Text: {:?}", 
                    token.0 + 1, token.1, token.2, token.3, token_text);
            }
        }
        
        println!("\n=== BUTTON (lines 73-81) ===");
        for token in &absolute_tokens {
            if token.0 >= 72 && token.0 <= 81 {
                let line_content = content.lines().nth(token.0 as usize).unwrap_or("");
                let token_text = &line_content.chars().skip(token.1 as usize).take(token.2 as usize).collect::<String>();
                println!("Line {}, Col {}, Len {}, Type {}, Text: {:?}", 
                    token.0 + 1, token.1, token.2, token.3, token_text);
            }
        }
        
        // Check for => arrows in hooks
        let hooks_arrows: Vec<_> = absolute_tokens.iter()
            .filter(|t| t.0 >= 22 && t.0 <= 32 && t.3 == 10 && t.2 == 2)
            .collect();
        println!("\n=== ARROWS IN HOOKS (type 10, len 2) ===");
        println!("Found {} arrow tokens", hooks_arrows.len());
        for token in hooks_arrows {
            println!("  Line {}, Col {}", token.0 + 1, token.1);
        }
        
        // Check for /> on lines 62, 69, 81
        let closing_slashes: Vec<_> = absolute_tokens.iter()
            .filter(|t| (t.0 == 61 || t.0 == 68 || t.0 == 80) && t.3 == 10 && t.2 == 2)
            .collect();
        println!("\n=== /> ON LINES 62, 69, 81 (type 10, len 2) ===");
        println!("Found {} closing slash tokens", closing_slashes.len());
        for token in closing_slashes {
            println!("  Line {}, Col {}", token.0 + 1, token.1);
        }
        
        // Check Field on line 55
        let field_tokens: Vec<_> = absolute_tokens.iter()
            .filter(|t| t.0 == 54)
            .collect();
        println!("\n=== ALL TOKENS ON LINE 55 ===");
        for token in field_tokens {
            let line_content = content.lines().nth(54).unwrap_or("");
            let token_text = &line_content.chars().skip(token.1 as usize).take(token.2 as usize).collect::<String>();
            println!("  Col {}, Len {}, Type {}, Text: {:?}", token.1, token.2, token.3, token_text);
        }
    }
}
