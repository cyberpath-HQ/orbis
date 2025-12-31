#[test]
fn debug_all_template_comments() {
    let content = std::fs::read_to_string(
        "/home/ebalo/Desktop/projects/rust/orbis-assets/examples/component-whitelisting-simple.orbis"
    ).expect("Failed to read file");
    
    let result = orbis_dsl::ast::parse_to_ast(&content);
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());
    
    let ast_file = result.unwrap();
    println!("\n=== ALL TEMPLATE COMMENTS ===");
    
    for elem in &ast_file.elements {
        if let orbis_dsl::ast::TopLevelElement::Template(template) = elem {
            println!("Template has {} content items", template.content.len());
            
            fn visit_content(content: &orbis_dsl::ast::TemplateContent, depth: usize) {
                let indent = "  ".repeat(depth);
                match content {
                    orbis_dsl::ast::TemplateContent::Comment { value, span } => {
                        println!("{}Comment at line {}: {:?}", indent, span.start_line, value);
                    }
                    orbis_dsl::ast::TemplateContent::Component(comp) => {
                        println!("{}Component {} at line {}, {} children", 
                            indent, comp.name, comp.span.start_line, comp.children.len());
                        for child in &comp.children {
                            visit_content(child, depth + 1);
                        }
                    }
                    orbis_dsl::ast::TemplateContent::ControlFlow(cf) => {
                        match cf {
                            orbis_dsl::ast::ControlFlow::If(if_block) => {
                                println!("{}If block at line {}", indent, if_block.span.start_line);
                                for item in &if_block.then_branch {
                                    visit_content(item, depth + 1);
                                }
                                for elif in &if_block.else_if_branches {
                                    for item in &elif.body {
                                        visit_content(item, depth + 1);
                                    }
                                }
                                if let Some(else_body) = &if_block.else_branch {
                                    for item in else_body {
                                        visit_content(item, depth + 1);
                                    }
                                }
                            }
                            orbis_dsl::ast::ControlFlow::For(for_block) => {
                                println!("{}For block at line {}", indent, for_block.span.start_line);
                                for item in &for_block.body {
                                    visit_content(item, depth + 1);
                                }
                            }
                            orbis_dsl::ast::ControlFlow::When(when_block) => {
                                println!("{}When block at line {}", indent, when_block.span.start_line);
                                for arm in &when_block.arms {
                                    for item in &arm.body {
                                        visit_content(item, depth + 1);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        match content {
                            orbis_dsl::ast::TemplateContent::Text { value, span } => {
                                println!("{}Text at line {}: {:?}", indent, span.start_line, value);
                            }
                            orbis_dsl::ast::TemplateContent::Expression { span, .. } => {
                                println!("{}Expression at line {}", indent, span.start_line);
                            }
                            orbis_dsl::ast::TemplateContent::SlotDefinition(slot) => {
                                println!("{}Slot at line {}", indent, slot.span.start_line);
                            }
                            orbis_dsl::ast::TemplateContent::FragmentUsage(frag) => {
                                println!("{}Fragment at line {}", indent, frag.span.start_line);
                            }
                            _ => {
                                println!("{}Other content (unknown type)", indent);
                            }
                        }
                    }
                }
            }
            
            for content in &template.content {
                visit_content(content, 0);
            }
        }
    }
}
