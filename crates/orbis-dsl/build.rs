use std::{
    collections::HashMap,
    env,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

use heck::{
    ToKebabCase,
    ToLowerCamelCase,
    ToShoutyKebabCase,
    ToShoutySnakeCase,
    ToSnakeCase,
    ToTrainCase,
    ToUpperCamelCase,
};
use maplit::hashmap;

/// Component attribute definition
#[derive(Debug, Clone)]
struct ComponentDef {
    name:       &'static str,
    attributes: Vec<AttributeDef>,
    events:     Vec<&'static str>,
}

/// Attribute definition with expression support
#[derive(Debug, Clone)]
struct AttributeDef {
    name:              &'static str,
    allows_expression: bool,
}

impl AttributeDef {
    fn literal(name: &'static str) -> Self {
        Self {
            name,
            allows_expression: false,
        }
    }

    fn expr(name: &'static str) -> Self {
        Self {
            name,
            allows_expression: true,
        }
    }
}

/// Build script for generating multiple pest grammar files with case variations.
///
/// This script generates multiple pest grammar files (e.g., page.pest, metadata.pest)
/// with automatic case-insensitive keyword matching. Each grammar is independent and
/// can have its own set of categories and keywords.
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Define component schemas with whitelisted attributes and events
    let components = define_components();

    // Define multiple grammars with their keyword categories
    let grammars: HashMap<&str, HashMap<&str, Vec<&str>>> = hashmap! {
        "page" => generate_page_grammar_keywords(),
        "metadata" => hashmap!{
            "meta_fields" => vec!["author", "version", "description", "license"],
            "config_types" => vec!["required", "optional", "deprecated"],
        },
    };

    // Generate each grammar file
    for (grammar_name, keyword_categories) in &grammars {
        let dest_filename = format!("{}.pest", grammar_name);

        if *grammar_name == "page" {
            generate_page_grammar_file(&out_dir, &dest_filename, keyword_categories, &components);
        }
        else {
            generate_grammar_file(&out_dir, &dest_filename, keyword_categories);
        }

        println!("cargo:rerun-if-changed=src/{}", dest_filename);
    }

    println!("cargo:rerun-if-changed=build.rs");
}

/// Define all components with their allowed attributes and events
fn define_components() -> Vec<ComponentDef> {
    vec![
        // Layout Components
        ComponentDef {
            name:       "Container",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click", "mouse_enter", "mouse_leave"],
        },
        ComponentDef {
            name:       "Grid",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("cols"),
                AttributeDef::expr("gap"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Flex",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("direction"),
                AttributeDef::literal("justify"),
                AttributeDef::literal("align"),
                AttributeDef::expr("gap"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        // Typography Components
        ComponentDef {
            name:       "Text",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("content"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Heading",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("content"),
                AttributeDef::expr("level"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        // Form Components
        ComponentDef {
            name:       "Field",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("type"),
                AttributeDef::literal("fieldName"), // Changed from name
                AttributeDef::literal("placeholder"),
                AttributeDef::literal("bind"),
                AttributeDef::expr("value"),
                AttributeDef::expr("defaultValue"),
                AttributeDef::expr("disabled"),
                AttributeDef::expr("required"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["input", "change", "focus", "blur", "key_down", "key_up"],
        },
        ComponentDef {
            name:       "Button",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("label"),
                AttributeDef::literal("type"),
                AttributeDef::expr("disabled"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click", "mouse_enter", "mouse_leave"],
        },
        ComponentDef {
            name:       "Form",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["submit"],
        },
        // Data Display Components
        ComponentDef {
            name:       "Card",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("title"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Table",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("data"),
                AttributeDef::expr("columns"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["rowClick", "cellClick"],
        },
        ComponentDef {
            name:       "List",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("items"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["itemClick"],
        },
        ComponentDef {
            name:       "Badge",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("content"),
                AttributeDef::literal("variant"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "StatCard",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("title"),
                AttributeDef::expr("value"),
                AttributeDef::expr("change"),
                AttributeDef::expr("icon"),
                AttributeDef::expr("trend"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Alert",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("type"),
                AttributeDef::expr("title"),
                AttributeDef::expr("message"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["close"],
        },
        // Navigation Components
        ComponentDef {
            name:       "Link",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("href"),
                AttributeDef::expr("content"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Dropdown",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("options"),
                AttributeDef::expr("value"),
                AttributeDef::expr("placeholder"),
                AttributeDef::expr("disabled"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["change"],
        },
        // Feedback Components
        ComponentDef {
            name:       "Progress",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("value"),
                AttributeDef::expr("max"),
                AttributeDef::expr("visible"),
            ],
            events:     vec![],
        },
        ComponentDef {
            name:       "LoadingOverlay",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("message"),
                AttributeDef::expr("visible"),
            ],
            events:     vec![],
        },
        ComponentDef {
            name:       "Skeleton",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("variant"),
                AttributeDef::expr("visible"),
            ],
            events:     vec![],
        },
        ComponentDef {
            name:       "EmptyState",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("icon"),
                AttributeDef::expr("title"),
                AttributeDef::expr("description"),
                AttributeDef::expr("visible"),
            ],
            events:     vec![],
        },
        // Misc Components
        ComponentDef {
            name:       "Icon",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::literal("iconName"), // Changed from name
                AttributeDef::expr("size"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["click"],
        },
        ComponentDef {
            name:       "Modal",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("title"),
                AttributeDef::expr("open"),
                AttributeDef::expr("visible"),
            ],
            events:     vec!["close"],
        },
        ComponentDef {
            name:       "Tooltip",
            attributes: vec![
                AttributeDef::literal("id"),
                AttributeDef::literal("className"),
                AttributeDef::expr("content"),
                AttributeDef::literal("position"),
                AttributeDef::expr("visible"),
            ],
            events:     vec![],
        },
    ]
}

fn generate_page_grammar_keywords() -> HashMap<&'static str, Vec<&'static str>> {
    let keywords = hashmap! {
        // Common keywords (avoid duplicates with ActionNamespaces)
        "Keywords" => vec![
            "page", "state", "template", "if", "else", "for", "in", "when",
        ],

        // Lifecycle hooks
        "LifecycleHooks" => vec!["mount", "unmount"],

        // Action namespaces (keep state here, not in Keywords)
        "ActionNamespaces" => vec!["api", "toast", "router", "console", "event"],

        // Primitive types
        "PrimitiveTypes" => vec!["string", "number", "bool", "object", "array"],

        // Common attribute names
        "CommonAttributes" => vec![
            "role",
            "ariaLabel",
            "ariaLabelledBy",
            "ariaDescribedBy",
            "ariaHidden",
            "ariaDisabled",
            "ariaExpanded",
            "ariaPressed",
            "ariaSelected",
            "ariaChecked",
            "ariaRequired",
            "ariaInvalid",
            "ariaErrorMessage",
            "ariaPlaceholder",
            "ariaLive",
            "ariaAtomic",
            "ariaBusy",
            "ariaRelevant",
            "ariaControls",
            "ariaOwns",
            "ariaFlowTo",
            "ariaCurrent",
            "tabIndex",
            "id",
            "className",
            "style",
            "visible",
            // HTML form attributes
            "placeholder",
            "disabled",
            "required",
            "label",
        ],

        // Common event names
        "CommonEvents" => vec![
            "click",
            "focus",
            "blur",
            "mouse_enter",
            "mouse_leave",
        ],

        "Hook" => vec!["hooks"],
    };

    keywords
}

fn generate_keyword_rules(keywords: &[&str], generated_content: &mut String) {
    for keyword in keywords {
        let variants = generate_all_variants(keyword);
        let rule_name = keyword.to_upper_camel_case();

        // Skip if already generated (avoid duplicates)
        if generated_content.contains(&format!("{} = @{{", rule_name)) {
            continue;
        }

        let quoted_variants: Vec<String> = variants.iter().map(|v| format!("^\"{}\"", v)).collect();

        // Generate the rule
        generated_content.push_str(&format!(
            "{} = @{{ {} }}\n",
            rule_name,
            quoted_variants.join(" | ")
        ));
    }
}

fn generate_page_grammar_file(
    out_dir: &str,
    filename: &str,
    keyword_categories: &HashMap<&str, Vec<&str>>,
    components: &[ComponentDef],
) {
    let dest_path = Path::new(out_dir).join(filename);
    let builder_insertion_start = "// @builder-insertion-start";
    let builder_insertion_end = "// @builder-insertion-end";

    let mut generated_content = String::new();
    generated_content.push_str(&format!(
        "{}\n// This section of the file is managed by the Orbis build system.\n\n",
        builder_insertion_start,
    ));

    // Generate individual keyword rules
    let mut keyword_categories_values = keyword_categories.values().flatten().cloned().collect::<Vec<&str>>();
    keyword_categories_values.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    generate_keyword_rules(&keyword_categories_values, &mut generated_content);

    generated_content.push_str("\n// Category rules\n");

    // Generate category rules AFTER individual keyword rules
    let mut keyword_categories: Vec<(&&str, &Vec<&str>)> = keyword_categories.iter().collect();
    keyword_categories.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    for (category, keywords) in keyword_categories {
        let category_variants: Vec<String> = keywords.iter().map(|k| k.to_upper_camel_case()).collect();

        generated_content.push_str(&format!(
            "{} = @{{ {} }}\n",
            category,
            category_variants.join(" | ")
        ));
    }
    generated_content.push_str("\n// Component-specific rules\n");

    // Generate component-specific attribute rules
    for component in components {
        let comp_name = component.name.to_upper_camel_case();

        // Generate attribute list
        let mut attr_names: Vec<String> = component
            .attributes
            .iter()
            .map(|a| a.name.to_upper_camel_case())
            .collect();
        attr_names.push("CommonAttributes".to_owned());

        generated_content.push_str(&format!(
            "{}Attributes = @{{ {} }}\n",
            comp_name,
            attr_names.join(" | ")
        ));

        // Generate event list (handle empty events with placeholder)
        let event_names: Vec<String> = component
            .events
            .iter()
            .map(|e| e.to_upper_camel_case())
            .collect();
        let events_rule = if event_names.is_empty() {
            // Use a never-matching rule for components with no events
            "!\"__never__\"".to_string()
        } else {
            event_names.join(" | ")
        };
        generated_content.push_str(&format!(
            "{}Events = @{{ {} }}\n",
            comp_name,
            events_rule
        ));

        // generate component name variations
        let variants = generate_all_variants(component.name);
        let quoted_variants: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
        generated_content.push_str(&format!(
            "{}ComponentNames = @{{ {} }}\n",
            comp_name,
            quoted_variants.join(" | ")
        ));

        // Generate component rules - separate self-closing and opening
        generated_content.push_str(&format!(
            "
{comp_name}AttributeDefinition = {{
    {comp_name}Attributes ~ \"=\" ~ attribute_value
}}
{comp_name}EventsDefinition = {{
    \"@\" ~ {comp_name}Events ~ \"=>\" ~ (action_with_handlers | action_list)
}}

// Self-closing variant: <{} ... />
{comp_name}SelfClosing = {{ 
    \"<\" ~ 
    {comp_name}ComponentNames ~ 
    ({comp_name}AttributeDefinition | {comp_name}EventsDefinition)* ~ 
    \"/>\"
}}

// Opening tag variant: <{} ... >
{comp_name}Opening = {{ 
    \"<\" ~ 
    {comp_name}ComponentNames ~ 
    ({comp_name}AttributeDefinition | {comp_name}EventsDefinition)* ~ 
    \">\"
}}
", component.name.to_lower_camel_case(), component.name.to_lower_camel_case()
        ));
    }

    // Generate ComponentTypes rules (union of self-closing and opening variants)
    generated_content.push_str("\n// Component types (union of all whitelisted components)\n");
    let self_closing_rules: Vec<String> = components
        .iter()
        .map(|c| format!("{}SelfClosing", c.name.to_upper_camel_case()))
        .collect();
    let opening_rules: Vec<String> = components
        .iter()
        .map(|c| format!("{}Opening", c.name.to_upper_camel_case()))
        .collect();
    
    generated_content.push_str(&format!(
        "SelfClosingComponents = {{ {} }}\n",
        self_closing_rules.join(" | ")
    ));
    generated_content.push_str(&format!(
        "OpeningComponents = {{ {} }}\n",
        opening_rules.join(" | ")
    ));

    generated_content.push_str("\n// Event rules\n");
    // Generate event rules
    for events in components.iter().map(|c| &c.events) {
        generate_keyword_rules(events, &mut generated_content);
    }

    generated_content.push_str("\n// Attribute rules\n");
    // Generate attribute rules
    for attribute in components.iter().flat_map(|c| &c.attributes) {
        generate_keyword_rules(&[attribute.name], &mut generated_content);
    }
    
    // Explicitly generate common HTML attribute rules that might be missing
    let html_attrs = vec!["placeholder", "disabled", "required", "label"];
    for attr in &html_attrs {
        generate_keyword_rules(&[attr], &mut generated_content);
    }

    generated_content.push_str(&format!("\n{}", builder_insertion_end));

    // Read existing file
    let existing_dir = Path::new("src").join(filename);
    let existing_content = read_to_string(&existing_dir)
        .unwrap_or_else(|_| format!("{}\n{}\n", builder_insertion_start, builder_insertion_end));

    let final_content = replace_section(
        &existing_content,
        builder_insertion_start,
        builder_insertion_end,
        &generated_content,
    );

    // Write files
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", final_content).unwrap();

    let src_dest_path = Path::new("src").join(filename);
    let mut src_f = File::create(&src_dest_path).unwrap();
    write!(src_f, "{}", final_content).unwrap();

    println!(
        "cargo:rustc-env=PEST_{}_PATH={}",
        filename.to_uppercase().replace('.', "_"),
        dest_path.display()
    );
}

/// Generate a single grammar file with all its keyword variations
fn generate_grammar_file(out_dir: &str, filename: &str, keyword_categories: &HashMap<&str, Vec<&str>>) {
    let dest_path = Path::new(out_dir).join(filename);
    let builder_insertion_start = "// @builder-insertion-start";
    let builder_insertion_end = "// @builder-insertion-end";

    // Generate the grammar rules
    let mut generated_content = String::new();
    generated_content.push_str(&format!(
        "{}\n// This section of the file is managed by the Orbis build system.\n// Any changes made directly to this \
         section will be overwritten as part of the build process.\n// To customize the contents of this file, modify \
         the build script located at `crates/orbis-dsl/build.rs`.\n\n",
        builder_insertion_start,
    ));

    // Generate category rules
    for (category, keywords) in keyword_categories {
        let category_variants: Vec<String> = keywords.iter().map(|k| k.to_upper_camel_case()).collect();
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

            let quoted_variants: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();

            generated_content.push_str(&format!(
                "{} = @{{ {} }}\n",
                rule_name,
                quoted_variants.join(" | ")
            ));
        }
    }

    generated_content.push_str(&format!("\n{}", builder_insertion_end));

    // Read existing file if it exists, or create template
    let existing_dir = Path::new("src").join(filename);
    let existing_content = read_to_string(&existing_dir)
        .unwrap_or_else(|_| format!("{}\n{}\n", builder_insertion_start, builder_insertion_end));

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

    println!(
        "cargo:rustc-env=PEST_{}_PATH={}",
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
fn replace_section(content: &str, start_marker: &str, end_marker: &str, replacement: &str) -> String {
    if let Some(start_pos) = content.find(start_marker) {
        if let Some(end_pos) = content.find(end_marker) {
            let before = &content[.. start_pos];
            let after = &content[end_pos + end_marker.len() ..];
            return format!("{}{}{}", before, replacement, after);
        }
    }

    // If markers not found, append the section
    format!("{}\n{}", content, replacement)
}
