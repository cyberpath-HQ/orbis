use std::collections::HashMap;
use std::env;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;
use heck::{
    ToSnakeCase, ToShoutySnakeCase, ToLowerCamelCase, ToUpperCamelCase,
    ToKebabCase, ToShoutyKebabCase, ToTrainCase
};
use maplit::hashmap;

/// Build script for generating multiple pest grammar files with case variations.
///
/// This script generates multiple pest grammar files (e.g., page.pest, metadata.pest)
/// with automatic case-insensitive keyword matching. Each grammar is independent and
/// can have its own set of categories and keywords.
///
/// ## Generated Grammar Structure
///
/// For each grammar and its keywords, the script generates:
/// 1. **Category rules**: Match any keyword in that category
/// 2. **Individual keyword rules**: Match all case variations of a specific keyword
///
/// ## Adding New Grammars
///
/// Simply add a new entry to the `grammars` HashMap in the main function:
/// ```rust
/// grammars.insert("my_grammar", grammar_config);
/// ```
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Define multiple grammars with their keyword categories
    let grammars: HashMap<&str, HashMap<&str, Vec<&str>>> = hashmap!{
        // Page grammar - for page definitions and UI elements
        "page" => hashmap!{
            "identifiers" => vec!["page", "longString", "userId", "apiVersion"],
            "types" => vec!["string", "number", "boolean"],
        },
        // Metadata grammar - for metadata and configuration
        "metadata" => hashmap!{
            "meta_fields" => vec!["author", "version", "description", "license"],
            "config_types" => vec!["required", "optional", "deprecated"],
        },
    };
    
    // Generate each grammar file
    for (grammar_name, keyword_categories) in &grammars {
        let dest_filename = format!("{}.pest", grammar_name);
        generate_grammar_file(&out_dir, &dest_filename, keyword_categories);
        
        // Tell cargo to rerun if this grammar file changes
        println!("cargo:rerun-if-changed=src/{}", dest_filename);
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}

/// Generate a single grammar file with all its keyword variations
fn generate_grammar_file(
    out_dir: &str,
    filename: &str,
    keyword_categories: &HashMap<&str, Vec<&str>>,
) {
    let dest_path = Path::new(out_dir).join(filename);
    let builder_insertion_start = "// @builder-insertion-start";
    let builder_insertion_end = "// @builder-insertion-end";

    // Generate the grammar rules
    let mut generated_content = String::new();
    generated_content.push_str(&format!(
        "{}\n// This section of the file is managed by the Orbis build system.\n// Any changes made directly to this section will be overwritten as part of the build process.\n// To customize the contents of this file, modify the build script located at `crates/orbis-dsl/build.rs`.\n\n",
        builder_insertion_start,
    ));

    // Generate category rules
    for (category, keywords) in keyword_categories {
        let category_variants: Vec<String> = keywords.iter()
            .map(|k| k.to_upper_camel_case())
            .collect();
        generated_content.push_str(&format!(
            "{} = @{{ {} }}\n",
            category,
            category_variants.join(" | ")
        ));
    }
    generated_content.push_str("\n");

    // Generate individual keyword rules with all variations
    for keywords in keyword_categories.values() {
        for keyword in keywords {
            let variants = generate_all_variants(keyword);
            let rule_name = keyword.to_upper_camel_case();
            
            let quoted_variants: Vec<String> = variants.iter()
                .map(|v| format!("\"{}\"", v))
                .collect();
            
            generated_content.push_str(&format!(
                "{} = @{{ {} }}\n",
                rule_name,
                quoted_variants.join(" | ")
            ));
        }
    }

    generated_content.push_str(&format!("\n{}\n", builder_insertion_end));

    // Read existing file if it exists, or create template
    let existing_content = read_to_string(&dest_path).unwrap_or_else(|_| {
        format!("{}\n{}\n", builder_insertion_start, builder_insertion_end)
    });

    // Replace content between markers
    let final_content = replace_section(
        &existing_content,
        builder_insertion_start,
        builder_insertion_end,
        &generated_content,
    );

    // Write to OUT_DIR (used during build)
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", final_content).unwrap();

    // Copy to src directory for version control and easy access
    let src_dir = Path::new("src");
    let src_dest_path = src_dir.join(filename);
    let mut src_f = File::create(&src_dest_path).unwrap();
    write!(src_f, "{}", final_content).unwrap();
    
    println!("cargo:rustc-env=PEST_{}_PATH={}", 
        filename.to_uppercase().replace('.', "_"),
        dest_path.display()
    );
}

/// Generate all heck variations for a keyword (excluding Title Case)
fn generate_all_variants(keyword: &str) -> Vec<String> {
    let mut variants = Vec::new();
    
    // snake_case
    variants.push(keyword.to_snake_case());
    
    // SCREAMING_SNAKE_CASE
    variants.push(keyword.to_shouty_snake_case());
    
    // camelCase
    variants.push(keyword.to_lower_camel_case());
    
    // PascalCase
    variants.push(keyword.to_upper_camel_case());
    
    // kebab-case
    variants.push(keyword.to_kebab_case());
    
    // SCREAMING-KEBAB-CASE
    variants.push(keyword.to_shouty_kebab_case());

    // Train-Case
    variants.push(keyword.to_train_case());
    
    // Deduplicate variants (some might be identical)
    variants.sort();
    variants.dedup();
    
    variants
}

/// Replace content between two markers in a string
fn replace_section(
    content: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> String {
    if let Some(start_pos) = content.find(start_marker) {
        if let Some(end_pos) = content.find(end_marker) {
            let before = &content[..start_pos];
            let after = &content[end_pos + end_marker.len()..];
            return format!("{}{}{}", before, replacement, after);
        }
    }
    
    // If markers not found, append the section
    format!("{}\n{}", content, replacement)
}
