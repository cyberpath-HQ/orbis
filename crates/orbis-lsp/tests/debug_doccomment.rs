use orbis_dsl::ast;

#[test]
fn debug_doc_comment_parsing() {
    let content = r#"
state {
    /**
    # Hello
    ## world
    ```
    world
    ```
     */
    message: string = "Hello World"
}
"#;
    
    match ast::parse_to_ast(content) {
        Ok(ast_file) => {
            println!("\n=== TOP LEVEL ELEMENTS ===");
            for (i, elem) in ast_file.elements.iter().enumerate() {
                match elem {
                    ast::TopLevelElement::Comment { value, span } => {
                        println!("{}: Comment at line {} to {}", i, span.start_line, span.end_line);
                        println!("   Value: {:?}", value);
                    }
                    ast::TopLevelElement::State(sb) => {
                        println!("{}: State block with {} declarations", i, sb.declarations.len());
                    }
                    _ => {}
                }
            }
            
            println!("\n=== STATE DECLARATIONS ===");
            for elem in &ast_file.elements {
                if let ast::TopLevelElement::State(state_block) = elem {
                    for (i, decl) in state_block.declarations.iter().enumerate() {
                        match decl {
                            ast::StateDeclaration::Regular(reg) => {
                                println!("{}: Regular state '{}' at line {}", i, reg.name, reg.span.start_line);
                                if let Some(doc) = &reg.doc_comment {
                                    println!("   Doc comment: {:?}", doc);
                                } else {
                                    println!("   No doc comment");
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Err(e) => {
            panic!("Parse error: {:?}", e);
        }
    }
}
