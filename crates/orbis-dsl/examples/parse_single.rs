//! Parse a single Orbis file to JSON

use std::env;
use std::path::PathBuf;

use orbis_dsl::ast::parse_to_ast_with_path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let file_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        eprintln!("Usage: cargo run --example parse_single <file.orbis>");
        std::process::exit(1);
    };
    
    println!("Parsing: {}", file_path.display());
    
    match std::fs::read_to_string(&file_path) {
        Ok(source) => {
            match parse_to_ast_with_path(&source, file_path.to_string_lossy()) {
                Ok(ast) => {
                    match serde_json::to_string_pretty(&ast) {
                        Ok(json) => {
                            // Save to file
                            let json_path = file_path.with_extension("orbis.json");
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
                            }
                        }
                        Err(e) => {
                            eprintln!("  ✗ Failed to serialize: {}", e);
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
}
