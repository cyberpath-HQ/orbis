//! Example: Parse Orbis DSL files to JSON AST
//!
//! This example demonstrates parsing Orbis DSL page files and outputting
//! their AST as JSON.
//!
//! Run with:
//! ```bash
//! cargo run -p orbis-dsl --example parse_to_json
//! ```

use std::path::PathBuf;

use orbis_dsl::ast::{parse_to_ast_with_path, OrbisParser, ParseOptions};

fn main() {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/pages");

    let example_files = ["dashboard.orbis", "registration-form.orbis", "product-catalog.orbis"];

    println!("{}", "=".repeat(80));
    println!("Orbis DSL to JSON AST Parser");
    println!("{}", "=".repeat(80));
    println!();

    for filename in &example_files {
        let file_path = examples_dir.join(filename);

        println!("Parsing: {}", filename);
        println!("{}", "-".repeat(80));

        // Read and parse the file
        match std::fs::read_to_string(&file_path) {
            Ok(source) => {
                match parse_to_ast_with_path(&source, file_path.to_string_lossy()) {
                    Ok(ast) => {
                        // Output as JSON
                        match serde_json::to_string_pretty(&ast) {
                            Ok(json) => {
                                // Save to file
                                let json_path = examples_dir.join(format!("{}.json", filename));
                                if let Err(e) = std::fs::write(&json_path, &json) {
                                    eprintln!("  ✗ Failed to write JSON: {}", e);
                                } else {
                                    println!("  ✓ Parsed successfully!");
                                    println!("  ✓ JSON saved to: {}", json_path.display());

                                    // Print summary
                                    println!();
                                    println!("  AST Summary:");
                                    println!("    - Path: {:?}", ast.path);
                                    println!("    - Imports: {}", ast.imports.len());
                                    println!("    - Top-level elements: {}", ast.elements.len());

                                    // Count element types
                                    let mut pages = 0;
                                    let mut states = 0;
                                    let mut templates = 0;
                                    let mut interfaces = 0;
                                    let mut hooks = 0;
                                    let mut fragments = 0;
                                    let mut styles = 0;
                                    let mut exports = 0;

                                    for element in &ast.elements {
                                        match element {
                                            orbis_dsl::ast::TopLevelElement::Page(_) => pages += 1,
                                            orbis_dsl::ast::TopLevelElement::State(_) => states += 1,
                                            orbis_dsl::ast::TopLevelElement::Template(_) => {
                                                templates += 1
                                            }
                                            orbis_dsl::ast::TopLevelElement::Interface(_) => {
                                                interfaces += 1
                                            }
                                            orbis_dsl::ast::TopLevelElement::Hooks(_) => hooks += 1,
                                            orbis_dsl::ast::TopLevelElement::Fragment(_) => {
                                                fragments += 1
                                            }
                                            orbis_dsl::ast::TopLevelElement::Styles(_) => {
                                                styles += 1
                                            }
                                            orbis_dsl::ast::TopLevelElement::Export(_) => {
                                                exports += 1
                                            }
                                        }
                                    }

                                    println!("      Pages: {}", pages);
                                    println!("      Interfaces: {}", interfaces);
                                    println!("      States: {}", states);
                                    println!("      Hooks: {}", hooks);
                                    println!("      Fragments: {}", fragments);
                                    println!("      Templates: {}", templates);
                                    println!("      Styles: {}", styles);
                                    println!("      Exports: {}", exports);
                                }
                            }
                            Err(e) => {
                                eprintln!("  ✗ Failed to serialize to JSON: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  ✗ Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to read file: {}", e);
            }
        }

        println!();
    }

    println!("{}", "=".repeat(80));
    println!("Testing OrbisParser with single-file mode");
    println!("{}", "=".repeat(80));
    println!();

    // Demonstrate the new OrbisParser API
    let mut parser = OrbisParser::with_options(ParseOptions::single_file());

    let simple_source = r#"
page {
    id: "test-page"
    title: "Test Page"
}

state {
    counter = 0
    message = "Hello World"
}

template {
    <Container>
        <Text content={state.message} />
        <Button 
            label="Click me"
            @click => { state.counter = state.counter + 1 }
        />
        <Text content={state.counter} />
    </Container>
}
"#;

    match parser.parse_source(simple_source) {
        Ok(ast) => {
            println!("✓ OrbisParser.parse_source() works!");
            println!("  Elements: {}", ast.elements.len());

            // Pretty print JSON
            if let Ok(json) = serde_json::to_string_pretty(&ast) {
                println!();
                println!("JSON Output:");
                println!("{}", json);
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
        }
    }
}
