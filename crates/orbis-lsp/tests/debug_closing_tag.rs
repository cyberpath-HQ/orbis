#[test]
fn debug_closing_tag_parsing() {
    let content = r#"
template {
    <Container>
        <Text content="Hello" />
    </Container>
}
"#;
    
    let result = orbis_dsl::ast::parse_to_ast(content);
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    
    let ast_file = result.unwrap();
    println!("\n=== PAGE ELEMENTS ===");
    println!("Elements: {}", ast_file.elements.len());
    
    for elem in &ast_file.elements {
        match elem {
            orbis_dsl::ast::TopLevelElement::Template(template) => {
                println!("\n=== TEMPLATE ===");
                println!("Template has {} content items", template.content.len());
                
                for (i, content) in template.content.iter().enumerate() {
                    match content {
                        orbis_dsl::ast::TemplateContent::Component(comp) => {
                            println!("\nComponent {}: {}", i, comp.name);
                            println!("  Span: {}:{} to {}:{}", 
                                comp.span.start_line, comp.span.start_col,
                                comp.span.end_line, comp.span.end_col);
                            println!("  Self-closing: {}", comp.self_closing);
                            println!("  Children: {}", comp.children.len());
                            
                            for (j, child) in comp.children.iter().enumerate() {
                                match child {
                                    orbis_dsl::ast::TemplateContent::Component(child_comp) => {
                                        println!("    Child {}: {} (self-closing: {})", 
                                            j, child_comp.name, child_comp.self_closing);
                                    }
                                    _ => println!("    Child {}: Other", j),
                                }
                            }
                        }
                        _ => println!("Content {}: Other", i),
                    }
                }
            }
            _ => {}
        }
    }
}
