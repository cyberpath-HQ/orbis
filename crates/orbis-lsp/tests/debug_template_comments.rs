use orbis_dsl::ast;

#[test]
fn debug_template_comments() {
    let content = r#"
template {
    // This is a comment in template
    <Container />
}
"#;
    
    match ast::parse_to_ast(content) {
        Ok(ast_file) => {
            println!("\n=== TEMPLATE ELEMENTS ===");
            for elem in &ast_file.elements {
                if let ast::TopLevelElement::Template(tmpl) = elem {
                    println!("Template has {} content items", tmpl.content.len());
                    for (i, content) in tmpl.content.iter().enumerate() {
                        match content {
                            ast::TemplateContent::Comment { value, span } => {
                                println!("{}: Comment at line {}: {:?}", i, span.start_line, value);
                            }
                            ast::TemplateContent::Component(comp) => {
                                println!("{}: Component '{}' at line {}", i, comp.name, comp.span.start_line);
                            }
                            _ => {
                                println!("{}: Other content", i);
                            }
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
