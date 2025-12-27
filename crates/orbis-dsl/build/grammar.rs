// =============================================================================
// ORBIS DSL BUILD SYSTEM - Grammar Generation
// =============================================================================
//
// This module handles the generation of Pest grammar rules from component
// definitions. It produces:
//
// 1. **Keyword rules**: Case-insensitive matching for all identifiers
// 2. **Category rules**: Groupings of related keywords (e.g., CommonAttributes)
// 3. **Typed value rules**: Strict matching for enum-like attributes
// 4. **Component rules**: Per-component attribute/event whitelisting
//
// The generated grammar is inserted between @builder-insertion-start and
// @builder-insertion-end markers in the .pest files. Content outside these
// markers is preserved (hand-written grammar rules).
//
// =============================================================================
//
// NOTE: This file is designed to be used with `include!` in build.rs.
// All dependencies (types, functions) must be brought into scope by the
// including file BEFORE this file is included.
//
// Required imports in build.rs before include:
// - std::collections::{HashMap, HashSet}
// - std::fs::{read_to_string, File}
// - std::io::Write
// - std::path::Path
// - heck::{ToLowerCamelCase, ToUpperCamelCase}
// - maplit::hashmap
// - ComponentDef from data_structures.rs
// - generate_all_variants, replace_section from utils.rs
//
// =============================================================================

// =============================================================================
// KEYWORD CATEGORIES
// =============================================================================

/// Returns the keyword categories for the page grammar.
///
/// Keywords are organized into categories for easier grammar rule generation
/// and maintenance. Each category becomes a Pest rule that matches any of
/// its member keywords.
///
/// # Categories
/// - `Keywords`: Core DSL keywords (page, state, template, if, for, etc.)
/// - `LifecycleHooks`: Hook names (@mount, @unmount, @watch)
/// - `ActionNamespaces`: Action prefixes (api, toast, router, console)
/// - `PrimitiveTypes`: Type annotation keywords (string, number, bool, etc.)
/// - `ComputedKeyword`: Computed property marker (@computed)
/// - `FragmentKeyword`: Fragment definition keyword
/// - `SlotKeyword`: Slot attribute for named slot content projection
/// - `InterfaceKeyword`: Interface definition keyword
/// - `ImportExportKeywords`: Import/export for modular code organization
/// - `ValidationKeywords`: Zod-like validation decorators
/// - `CssKeywords`: CSS-in-DSL support
/// - `CommonAttributes`: ARIA and HTML attributes shared by all components
/// - `CommonEvents`: Events shared by multiple components
/// - `Hook`: The hooks block keyword
pub fn generate_page_grammar_keywords() -> HashMap<&'static str, Vec<&'static str>> {
    hashmap! {
        // Core DSL keywords for structure
        "Keywords" => vec![
            "page", "state", "template", "if", "else", "for", "in", "when", "fragment", "interface", "styles",
        ],

        // Import/Export keywords for modular code organization
        "ImportExportKeywords" => vec![
            "import", "export", "from", "as", "default", "pub", "use", "mod",
        ],

        // Lifecycle and reactive hook names (used with @prefix)
        // @mount, @unmount for lifecycle
        // @watch for reactive watchers
        "LifecycleHooks" => vec!["mount", "unmount", "watch"],

        // Action namespaces for the action system
        "ActionNamespaces" => vec!["api", "toast", "router", "console", "event"],

        // Primitive types for state declarations
        "PrimitiveTypes" => vec!["string", "number", "bool", "object", "array", "null", "any", "void", "never"],

        // Computed property marker (optional prefix for derived state)
        "ComputedKeyword" => vec!["computed"],

        // Fragment definition keyword
        "FragmentKeyword" => vec!["fragment"],

        // Slot attribute for named slot content projection (Astro-like)
        "SlotKeyword" => vec!["slot"],

        // Interface definition keyword
        "InterfaceKeyword" => vec!["interface"],

        // Watcher options
        "WatcherOptions" => vec!["debounce", "immediate", "deep"],

        // Validation keywords (Zod v4-like)
        // All are optional and case-insensitive
        "ValidationKeywords" => vec![
            // String validators
            "min", "max", "length", "email", "url", "uuid", "cuid", "cuid2", "ulid",
            "regex", "pattern", "includes", "startsWith", "endsWith", "trim", "toLowerCase",
            "toUpperCase", "datetime", "date", "time", "duration", "ip", "cidr", "base64",
            "emoji", "nanoid", "jwt", "creditCard", "iban", "bic", "postalCode",
            // Number validators
            "gt", "gte", "lt", "lte", "int", "positive", "nonnegative", "negative",
            "nonpositive", "multipleOf", "finite", "safe",
            // Array validators
            "nonempty", "unique",
            // Object validators
            "strict", "passthrough", "strip", "catchall",
            // Common validators
            "required", "optional", "nullable", "nullish", "default", "catch",
            "transform", "refine", "superRefine", "pipe", "brand", "readonly",
            // Custom messages
            "message", "errorMap",
        ],

        // CSS-in-DSL keywords
        "CssKeywords" => vec![
            "styles", "scoped", "global", "media", "keyframes", "layer", "supports",
            "container", "scope", "apply", "theme", "screen", "variants",
        ],

        // Common attributes shared by all components (ARIA + HTML)
        // Added "slot" for named slot content projection
        "CommonAttributes" => vec![
            // ARIA accessibility attributes
            "role", "ariaLabel", "ariaLabelledBy", "ariaDescribedBy", "ariaHidden",
            "ariaDisabled", "ariaExpanded", "ariaPressed", "ariaSelected", "ariaChecked",
            "ariaRequired", "ariaInvalid", "ariaErrorMessage", "ariaPlaceholder",
            "ariaLive", "ariaAtomic", "ariaBusy", "ariaRelevant", "ariaControls",
            "ariaOwns", "ariaFlowTo", "ariaCurrent", "tabIndex",
            // Standard HTML attributes
            "id", "className", "style", "visible",
            // Form-related attributes (common to form elements)
            "placeholder", "disabled", "required", "label",
            // Slot attribute for content projection into fragments
            "slot",
        ],

        // Common events shared by multiple components
        "CommonEvents" => vec!["click", "focus", "blur", "mouseEnter", "mouseLeave"],

        // The hooks block keyword
        "Hook" => vec!["hooks"],

        // Additional required keywords
        "AdditionalKeywords" => vec![
            // Control flow
            "return", "const", "type",
            // CSS styles
            "styles",
        ],

        // Validator zod types
        "ValidatorPrimitiveTypes" => vec![
            "string", "number", "boolean", "bigint", "symbol", "null", "date", "enum", "string_bool",
            "string_boolean", "optional", "nullable", "nullish", "any", "unknown", "never",
            "object", "array", "tuple", "union", "xor", "intersection", "record", "map", "set", "file"
        ],
        "ValidatorCoerceTypes" => vec![
            "coerceString", "coerceNumber", "coerceBoolean", "coerceBigint"
        ],
        "ValidatorLiteralValues" => vec![
            "literal",
        ],
        "ValidatorFormats" => vec![
            "email", "uuid", "url",
            "httpUrl", "hostname", "emoji", "base64", "base64url", "hex", "jwt", "nanoid", "cuid",
            "cuid2", "ulid", "ipv4", "ipv6", "mac", "cidrv4", "cidrv6", "hash", "iso_date", "iso_time",
            "iso_datetime", "iso_duration",
        ],
        "ValidatorStringSpecifics" => vec![
            "max", "min", "length", "regex", "pattern", "startsWith", "endsWith", "includes", "lowercase", "uppercase", "trim",
            "toLowerCase", "toUpperCase", "normalize"
        ],
        "ValidatorNumberSpecifics" => vec![
            "gt", "gte", "lt", "lte", "positive", "non_negative", "negative", "non_positive",
            "multipleOf", "nan", "int", "int32"
        ],
        "ValidatorObjectSpecifics" => vec![
            "catchall", "partial", "required"
        ],
        "ValidatorFileSpecifics" => vec![
            "min", "max", "mime"
        ],
        "ValidatorMessageSpecifiers" => vec![
            "message", "errorMap"
        ],

        // Page attributes
        "PageAttributes" => vec![
            "id", "title", "description", "icon", "route", "show_in_menu", "menu_order", "parent_route", "requires_auth",
            "permissions", "roles", "layout"
        ],
    }
}

// =============================================================================
// KEYWORD RULE GENERATION
// =============================================================================

/// Generates Pest rules for a list of keywords with case-insensitive matching.
///
/// Each keyword becomes a rule that matches all its case variations:
/// ```pest
/// ClassName = @{ ^"CLASS-NAME" | ^"CLASS_NAME" | ^"Class-Name" | ... }
/// ```
///
/// # Arguments
/// * `keywords` - List of keywords to generate rules for
/// * `generated_content` - String buffer to append rules to
/// * `generated_rules` - Set of already-generated rule names (for deduplication)
pub fn generate_keyword_rules(
    keywords: &[&str],
    generated_content: &mut String,
    generated_rules: &mut HashSet<String>,
) {
    for keyword in keywords {
        let variants = generate_all_variants(keyword);
        let rule_name = keyword.to_upper_camel_case();

        // Skip if already generated (prevents duplicate rules)
        if generated_rules.contains(&rule_name) {
            continue;
        }
        generated_rules.insert(rule_name.clone());

        // Generate case-insensitive matching using Pest's ^"..." syntax
        let quoted_variants: Vec<String> = variants.iter().map(|v| format!("^\"{}\"", v)).collect();

        generated_content.push_str(&format!(
            "{} = @{{ {} }}\n",
            rule_name,
            quoted_variants.join(" | ")
        ));
    }
}

/// Generates strongly-typed attribute value rules.
///
/// For attributes with `allowed_values`, generates strict matching rules:
/// ```pest
/// ButtonVariantValues = @{ ^"default" | ^"destructive" | ^"outline" | ... }
/// ```
///
/// These rules are referenced by component attribute definitions to enforce
/// type safety at parse time.
pub fn generate_typed_attribute_values(components: &[ComponentDef], generated_content: &mut String) {
    let mut generated_value_rules: HashSet<String> = HashSet::new();

    for component in components {
        for attr in &component.attributes {
            if let Some(values) = &attr.allowed_values {
                // Create a unique rule name: ComponentNameAttrNameValues
                let rule_name = format!(
                    "{}{}Values",
                    component.name.to_upper_camel_case(),
                    attr.name.to_upper_camel_case()
                );

                // Skip if already generated (same values used by multiple components)
                if generated_value_rules.contains(&rule_name) {
                    continue;
                }
                generated_value_rules.insert(rule_name.clone());

                // Generate case-insensitive value matching
                let value_alts: Vec<String> = values.iter().map(|v| format!("^\"{}\"", v)).collect();
                generated_content.push_str(&format!(
                    "{} = @{{ {} }}\n",
                    rule_name,
                    value_alts.join(" | ")
                ));
            }
        }
    }
}

// =============================================================================
// COMPONENT RULE GENERATION
// =============================================================================

/// Generates all Pest grammar rules for a single component.
///
/// For each component, generates:
/// 1. `{Component}Attributes` - Union of allowed attribute keywords
/// 2. `{Component}Events` - Union of allowed event keywords
/// 3. `{Component}ComponentNames` - Case variations of component name
/// 4. `{Component}AttributeDefinition` - Attribute assignment rule
/// 5. `{Component}EventsDefinition` - Event handler rule
/// 6. `{Component}SelfClosing` - Self-closing tag: `<Component ... />`
/// 7. `{Component}Opening` - Opening tag: `<Component ...>`
///
/// # Example Output
/// ```pest
/// ButtonAttributes = @{ Id | ClassName | Label | Variant | ... }
/// ButtonEvents = @{ Click }
/// ButtonComponentNames = @{ "BUTTON" | "Button" | "button" }
/// ButtonAttributeDefinition = { ButtonAttributes ~ "=" ~ attribute_value }
/// ButtonEventsDefinition = { "@" ~ ButtonEvents ~ "=>" ~ (action_with_handlers | action_list) }
/// ButtonSelfClosing = { "<" ~ ButtonComponentNames ~ (ButtonAttributeDefinition | ButtonEventsDefinition)* ~ "/>" }
/// ButtonOpening = { "<" ~ ButtonComponentNames ~ (ButtonAttributeDefinition | ButtonEventsDefinition)* ~ ">" }
/// ```
pub fn generate_component_rules(component: &ComponentDef, generated_content: &mut String) {
    let comp_name = component.name.to_upper_camel_case();

    // =========================================================================
    // Attribute list rule
    // =========================================================================
    // Combines component-specific attributes with CommonAttributes
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

    // =========================================================================
    // Event list rule
    // =========================================================================
    // If no events, use a never-matching pattern to avoid empty alternation
    let event_names: Vec<String> = component
        .events
        .iter()
        .map(|e| e.name.to_upper_camel_case())
        .collect();
    let events_rule = if event_names.is_empty() {
        "!\"__never__\"".to_string() // Pest pattern that never matches
    }
    else {
        event_names.join(" | ")
    };
    generated_content.push_str(&format!("{}Events = @{{ {} }}\n", comp_name, events_rule));

    // =========================================================================
    // Component name variations
    // =========================================================================
    // Allows <Button>, <button>, <BUTTON>, etc.
    let variants = generate_all_variants(component.name);
    let quoted_variants: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
    generated_content.push_str(&format!(
        "{}ComponentNames = @{{ {} }}\n",
        comp_name,
        quoted_variants.join(" | ")
    ));

    // =========================================================================
    // Attribute and event definition rules
    // =========================================================================
    generated_content.push_str(&format!(
        "
{comp_name}AttributeDefinition = {{
    {comp_name}Attributes ~ \"=\" ~ attribute_value
}}
{comp_name}EventsDefinition = {{
    \"@\" ~ {comp_name}Events ~ \"=>\" ~ action_body
}}

// Self-closing variant: <{lower} ... />
{comp_name}SelfClosing = {{ 
    \"<\" ~ 
    {comp_name}ComponentNames ~ 
    ({comp_name}AttributeDefinition | {comp_name}EventsDefinition)* ~ 
    \"/>\"
}}

// Opening tag variant: <{lower} ... >
{comp_name}Opening = {{ 
    \"<\" ~ 
    {comp_name}ComponentNames ~ 
    ({comp_name}AttributeDefinition | {comp_name}EventsDefinition)* ~ 
    \">\"
}}
",
        lower = component.name.to_lower_camel_case()
    ));
}

// =============================================================================
// GRAMMAR FILE GENERATION
// =============================================================================

/// Generates the complete page.pest grammar file.
///
/// This is the main entry point for grammar generation. It:
/// 1. Collects all keywords from categories and component definitions
/// 2. Generates keyword rules with case variations
/// 3. Generates category rules (unions of related keywords)
/// 4. Generates typed attribute value rules
/// 5. Generates component-specific rules
/// 6. Generates component type unions (SelfClosingComponents, OpeningComponents)
///
/// The generated content is inserted between `@builder-insertion-start` and
/// `@builder-insertion-end` markers, preserving hand-written grammar rules
/// outside those markers.
///
/// # Arguments
/// * `out_dir` - Cargo's OUT_DIR for build artifacts
/// * `filename` - Name of the .pest file (e.g., "page.pest")
/// * `keyword_categories` - Keyword groupings for category rules
/// * `components` - Component definitions for component rules
pub fn generate_page_grammar_file(
    out_dir: &str,
    filename: &str,
    keyword_categories: &HashMap<&str, Vec<&str>>,
    components: &[ComponentDef],
) {
    let dest_path = Path::new(out_dir).join(filename);
    let builder_insertion_start = "// @builder-insertion-start";
    let builder_insertion_end = "// @builder-insertion-end";

    let mut generated_content = String::new();
    let mut generated_rules: HashSet<String> = HashSet::new();

    // =========================================================================
    // Header comment
    // =========================================================================
    generated_content.push_str(&format!(
        "{}\n// This section is auto-generated by the Orbis build system.\n// DO NOT EDIT MANUALLY - changes will be \
         overwritten.\n// To modify, edit build/components.rs or build/grammar.rs\n\n",
        builder_insertion_start,
    ));

    // =========================================================================
    // STEP 1: Collect ALL keywords from all sources
    // =========================================================================
    // This ensures every identifier used in the grammar has a case-insensitive
    // matching rule generated for it.
    let mut all_keywords: Vec<&str> = Vec::new();

    // From keyword categories
    for keywords in keyword_categories.values() {
        all_keywords.extend(keywords.iter().cloned());
    }

    // From component attributes
    for component in components {
        for attr in &component.attributes {
            all_keywords.push(attr.name);
        }
        for event in &component.events {
            all_keywords.push(event.name);
        }
    }

    // Deduplicate and sort for consistent output
    all_keywords.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    all_keywords.dedup();

    // =========================================================================
    // STEP 2: Generate ALL keyword rules first
    // =========================================================================
    generated_content.push_str("// Keyword rules (case-insensitive variations)\n");
    generate_keyword_rules(&all_keywords, &mut generated_content, &mut generated_rules);

    // =========================================================================
    // STEP 3: Generate category rules
    // =========================================================================
    generated_content.push_str("\n// Category rules\n");
    let mut sorted_categories: Vec<(&&str, &Vec<&str>)> = keyword_categories.iter().collect();
    sorted_categories.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    for (category, keywords) in sorted_categories {
        let category_variants: Vec<String> = keywords.iter().map(|k| k.to_upper_camel_case()).collect();
        generated_content.push_str(&format!(
            "{} = @{{ {} }}\n",
            category,
            category_variants.join(" | ")
        ));
    }

    // =========================================================================
    // STEP 4: Generate attribute value rules (strongly typed)
    // =========================================================================
    generated_content.push_str("\n// Strongly typed attribute values\n");
    generate_typed_attribute_values(components, &mut generated_content);

    // =========================================================================
    // STEP 5: Generate component-specific rules
    // =========================================================================
    generated_content.push_str("\n// Component-specific rules\n");
    for component in components {
        generate_component_rules(component, &mut generated_content);
    }

    // =========================================================================
    // STEP 6: Generate component type unions
    // =========================================================================
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

    generated_content.push_str(&format!("\n{}", builder_insertion_end));

    // =========================================================================
    // Read existing file and merge
    // =========================================================================
    let existing_dir = Path::new("src").join(filename);
    let existing_content = read_to_string(&existing_dir)
        .unwrap_or_else(|_| format!("{}\n{}", builder_insertion_start, builder_insertion_end));

    let final_content = replace_section(
        &existing_content,
        builder_insertion_start,
        builder_insertion_end,
        &generated_content,
    );

    // =========================================================================
    // Write output files
    // =========================================================================
    // Write to OUT_DIR (required by Cargo)
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", final_content).unwrap();

    // Write to src directory (for version control and IDE support)
    let src_dest_path = Path::new("src").join(filename);
    let mut src_f = File::create(&src_dest_path).unwrap();
    write!(src_f, "{}", final_content).unwrap();

    // Set environment variable for Pest to find the grammar
    println!(
        "cargo:rustc-env=PEST_{}_PATH={}",
        filename.to_uppercase().replace('.', "_"),
        dest_path.display()
    );
}

/// Generates a generic grammar file (for non-page grammars like metadata.pest).
///
/// This is a simpler version of `generate_page_grammar_file` that only handles
/// keyword categories without component-specific rules.
pub fn generate_grammar_file(out_dir: &str, filename: &str, keyword_categories: &HashMap<&str, Vec<&str>>) {
    let dest_path = Path::new(out_dir).join(filename);
    let builder_insertion_start = "// @builder-insertion-start";
    let builder_insertion_end = "// @builder-insertion-end";

    let mut generated_content = String::new();
    let mut generated_rules: HashSet<String> = HashSet::new();

    generated_content.push_str(&format!(
        "{}\n// Auto-generated by Orbis build system. Do not edit manually.\n\n",
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

    // Generate keyword rules
    for keywords in keyword_categories.values() {
        generate_keyword_rules(keywords, &mut generated_content, &mut generated_rules);
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
