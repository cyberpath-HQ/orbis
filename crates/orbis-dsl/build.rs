use std::{
    collections::{HashMap, HashSet},
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

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Component definition with whitelisted attributes, events, and deprecation info
#[derive(Debug, Clone)]
struct ComponentDef {
    name:        &'static str,
    description: &'static str,
    attributes:  Vec<AttributeDef>,
    events:      Vec<EventDef>,
    deprecated:  Option<DeprecationInfo>,
}

/// Attribute definition with expression support, allowed values, and deprecation
#[derive(Debug, Clone)]
struct AttributeDef {
    name:           &'static str,
    description:    &'static str,
    /// If Some, only these literal values are allowed (e.g., "info" | "error" | "warning")
    allowed_values: Option<Vec<&'static str>>,
    deprecated:     Option<DeprecationInfo>,
}

/// Event definition with deprecation support
#[derive(Debug, Clone)]
struct EventDef {
    name:        &'static str,
    description: &'static str,
    deprecated:  Option<DeprecationInfo>,
}

/// Deprecation information with migration guidance
#[derive(Debug, Clone)]
struct DeprecationInfo {
    message:     &'static str,
    alternative: Option<&'static str>,
}

impl AttributeDef {
    /// Create an attribute that accepts any expression value
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            allowed_values: None,
            deprecated: None,
        }
    }

    /// Create an attribute with whitelisted values only
    fn with_values(name: &'static str, description: &'static str, values: Vec<&'static str>) -> Self {
        Self {
            name,
            description,
            allowed_values: Some(values),
            deprecated: None,
        }
    }

    /// Mark this attribute as deprecated
    #[allow(dead_code)]
    fn deprecated(mut self, message: &'static str, alternative: Option<&'static str>) -> Self {
        self.deprecated = Some(DeprecationInfo { message, alternative });
        self
    }
}

impl EventDef {
    fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            deprecated: None,
        }
    }

    #[allow(dead_code)]
    fn deprecated(mut self, message: &'static str, alternative: Option<&'static str>) -> Self {
        self.deprecated = Some(DeprecationInfo { message, alternative });
        self
    }
}

// ============================================================================
// MAIN BUILD SCRIPT
// ============================================================================

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

    // Generate component documentation
    generate_component_documentation(&components);

    println!("cargo:rerun-if-changed=build.rs");
}

// ============================================================================
// COMPONENT DEFINITIONS
// ============================================================================

fn define_components() -> Vec<ComponentDef> {
    vec![
        // ==================== Layout Components ====================
        ComponentDef {
            name:        "Container",
            description: "A generic container element for grouping and layout purposes.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the container is clicked"),
                EventDef::new("mouseEnter", "Triggered when mouse enters the container"),
                EventDef::new("mouseLeave", "Triggered when mouse leaves the container"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Grid",
            description: "A CSS Grid-based layout component for creating grid layouts.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("cols", "Number of columns (expression)"),
                AttributeDef::new("gap", "Gap between grid items"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the grid is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Flex",
            description: "A Flexbox-based layout component for flexible layouts.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("direction", "Flex direction", vec!["row", "column", "row-reverse", "column-reverse"]),
                AttributeDef::with_values("justify", "Justify content", vec!["start", "end", "center", "between", "around", "evenly"]),
                AttributeDef::with_values("align", "Align items", vec!["start", "end", "center", "stretch", "baseline"]),
                AttributeDef::new("gap", "Gap between flex items"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the flex container is clicked"),
            ],
            deprecated:  None,
        },

        // ==================== Typography Components ====================
        ComponentDef {
            name:        "Text",
            description: "A text display component for paragraphs and inline text.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Text content to display (supports expressions)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the text is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Heading",
            description: "A heading component for titles (h1-h6).",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Heading text content"),
                AttributeDef::with_values("level", "Heading level", vec!["1", "2", "3", "4", "5", "6"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the heading is clicked"),
            ],
            deprecated:  None,
        },

        // ==================== Form Components ====================
        ComponentDef {
            name:        "Field",
            description: "A form input field component supporting various input types.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("type", "Input type", vec!["text", "password", "email", "number", "tel", "url", "date", "time", "datetime-local", "month", "week", "color", "file", "hidden", "textarea", "select", "checkbox", "radio"]),
                AttributeDef::new("fieldName", "Name of the field for form submission"),
                AttributeDef::new("label", "Label text for the field"),
                AttributeDef::new("placeholder", "Placeholder text when empty"),
                AttributeDef::new("bind", "State path to bind the value to"),
                AttributeDef::new("value", "Current value (expression)"),
                AttributeDef::new("defaultValue", "Default value on mount"),
                AttributeDef::new("disabled", "Whether the field is disabled (expression)"),
                AttributeDef::new("required", "Whether the field is required (expression)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("input", "Triggered on every input change"),
                EventDef::new("change", "Triggered when value changes and loses focus"),
                EventDef::new("focus", "Triggered when field gains focus"),
                EventDef::new("blur", "Triggered when field loses focus"),
                EventDef::new("keyDown", "Triggered on key press"),
                EventDef::new("keyUp", "Triggered on key release"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Button",
            description: "A clickable button component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("label", "Button text label"),
                AttributeDef::with_values("type", "Button type", vec!["button", "submit", "reset"]),
                AttributeDef::with_values("variant", "Visual variant", vec!["primary", "secondary", "outline", "ghost", "destructive", "link"]),
                AttributeDef::new("disabled", "Whether the button is disabled (expression)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the button is clicked"),
                EventDef::new("mouseEnter", "Triggered when mouse enters the button"),
                EventDef::new("mouseLeave", "Triggered when mouse leaves the button"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Form",
            description: "A form container that handles submission.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("submit", "Triggered when the form is submitted"),
            ],
            deprecated:  None,
        },

        // ==================== Data Display Components ====================
        ComponentDef {
            name:        "Card",
            description: "A card container for grouping related content.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Card title (expression)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the card is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Table",
            description: "A data table component for displaying tabular data.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("data", "Array of data objects to display"),
                AttributeDef::new("columns", "Column definitions"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("rowClick", "Triggered when a row is clicked"),
                EventDef::new("cellClick", "Triggered when a cell is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "List",
            description: "A list component for displaying items.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("items", "Array of items to display"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("itemClick", "Triggered when an item is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Badge",
            description: "A small status indicator or label component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Badge text content"),
                AttributeDef::with_values("variant", "Visual variant", vec!["default", "primary", "secondary", "success", "warning", "error", "info", "outline"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the badge is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "StatCard",
            description: "A statistics display card with value, label, and optional trend indicator.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Stat label/title"),
                AttributeDef::new("value", "Main statistic value"),
                AttributeDef::new("change", "Change value (e.g., '+5%')"),
                AttributeDef::new("icon", "Icon name to display"),
                AttributeDef::with_values("trend", "Trend direction", vec!["up", "down", "neutral"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the stat card is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Alert",
            description: "An alert/notification message component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("type", "Alert type/severity", vec!["info", "success", "warning", "error"]),
                AttributeDef::new("title", "Alert title"),
                AttributeDef::new("message", "Alert message content"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("close", "Triggered when the alert is dismissed"),
            ],
            deprecated:  None,
        },

        // ==================== Navigation Components ====================
        ComponentDef {
            name:        "Link",
            description: "A navigation link component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("href", "URL to navigate to"),
                AttributeDef::new("content", "Link text content"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the link is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Dropdown",
            description: "A dropdown select component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("options", "Array of options to select from"),
                AttributeDef::new("value", "Currently selected value"),
                AttributeDef::new("placeholder", "Placeholder text when no selection"),
                AttributeDef::new("disabled", "Whether the dropdown is disabled"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("change", "Triggered when selection changes"),
            ],
            deprecated:  None,
        },

        // ==================== Feedback Components ====================
        ComponentDef {
            name:        "Progress",
            description: "A progress indicator component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("value", "Current progress value"),
                AttributeDef::new("max", "Maximum progress value"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },
        ComponentDef {
            name:        "LoadingOverlay",
            description: "A loading overlay that covers its container.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("message", "Loading message to display"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Skeleton",
            description: "A skeleton loading placeholder component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("variant", "Skeleton shape variant", vec!["text", "circular", "rectangular", "rounded"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },
        ComponentDef {
            name:        "EmptyState",
            description: "An empty state placeholder with icon and message.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("icon", "Icon to display"),
                AttributeDef::new("title", "Empty state title"),
                AttributeDef::new("description", "Empty state description"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // ==================== Misc Components ====================
        ComponentDef {
            name:        "Icon",
            description: "An icon component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("name", "Icon name from the icon set"),
                AttributeDef::with_values("size", "Icon size", vec!["xs", "sm", "md", "lg", "xl", "2xl"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the icon is clicked"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Modal",
            description: "A modal dialog component.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Modal title"),
                AttributeDef::new("open", "Whether the modal is open (expression)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("close", "Triggered when the modal is closed"),
            ],
            deprecated:  None,
        },
        ComponentDef {
            name:        "Tooltip",
            description: "A tooltip component that appears on hover.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Tooltip content text"),
                AttributeDef::with_values("position", "Tooltip position", vec!["top", "bottom", "left", "right"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },
    ]
}

// ============================================================================
// GRAMMAR KEYWORDS
// ============================================================================

fn generate_page_grammar_keywords() -> HashMap<&'static str, Vec<&'static str>> {
    hashmap! {
        // Common keywords
        "Keywords" => vec![
            "page", "state", "template", "if", "else", "for", "in", "when",
        ],

        // Lifecycle hooks
        "LifecycleHooks" => vec!["mount", "unmount"],

        // Action namespaces
        "ActionNamespaces" => vec!["api", "toast", "router", "console", "event"],

        // Primitive types
        "PrimitiveTypes" => vec!["string", "number", "bool", "object", "array"],

        // Common attribute names (shared across components)
        "CommonAttributes" => vec![
            // Accessibility
            "role", "ariaLabel", "ariaLabelledBy", "ariaDescribedBy", "ariaHidden",
            "ariaDisabled", "ariaExpanded", "ariaPressed", "ariaSelected", "ariaChecked",
            "ariaRequired", "ariaInvalid", "ariaErrorMessage", "ariaPlaceholder",
            "ariaLive", "ariaAtomic", "ariaBusy", "ariaRelevant", "ariaControls",
            "ariaOwns", "ariaFlowTo", "ariaCurrent", "tabIndex",
            // Standard HTML
            "id", "className", "style", "visible",
            // Form-related
            "placeholder", "disabled", "required", "label",
        ],

        // Common event names
        "CommonEvents" => vec!["click", "focus", "blur", "mouseEnter", "mouseLeave"],

        // Hooks block keyword
        "Hook" => vec!["hooks"],
    }
}

// ============================================================================
// GRAMMAR GENERATION
// ============================================================================

fn generate_keyword_rules(keywords: &[&str], generated_content: &mut String, generated_rules: &mut HashSet<String>) {
    for keyword in keywords {
        let variants = generate_all_variants(keyword);
        let rule_name = keyword.to_upper_camel_case();

        // Skip if already generated
        if generated_rules.contains(&rule_name) {
            continue;
        }
        generated_rules.insert(rule_name.clone());

        let quoted_variants: Vec<String> = variants.iter().map(|v| format!("^\"{}\"", v)).collect();

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
    let mut generated_rules: HashSet<String> = HashSet::new();

    generated_content.push_str(&format!(
        "{}\n// This section is auto-generated by the Orbis build system.\n// DO NOT EDIT MANUALLY - changes will be overwritten.\n// To modify, edit build.rs\n\n",
        builder_insertion_start,
    ));

    // ==== STEP 1: Collect ALL keywords from all sources ====
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
    
    // Deduplicate and sort
    all_keywords.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    all_keywords.dedup();

    // ==== STEP 2: Generate ALL keyword rules first ====
    generated_content.push_str("// Keyword rules (case-insensitive variations)\n");
    generate_keyword_rules(&all_keywords, &mut generated_content, &mut generated_rules);

    // ==== STEP 3: Generate category rules ====
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

    // ==== STEP 4: Generate attribute value rules (strongly typed) ====
    generated_content.push_str("\n// Strongly typed attribute values\n");
    generate_typed_attribute_values(components, &mut generated_content);

    // ==== STEP 5: Generate component-specific rules ====
    generated_content.push_str("\n// Component-specific rules\n");
    for component in components {
        generate_component_rules(component, &mut generated_content);
    }

    // ==== STEP 6: Generate component type unions ====
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

    // Read existing file
    let existing_dir = Path::new("src").join(filename);
    let existing_content = read_to_string(&existing_dir)
        .unwrap_or_else(|_| format!("{}\n{}", builder_insertion_start, builder_insertion_end));

    // Replace content between markers
    let final_content = replace_section(
        &existing_content,
        builder_insertion_start,
        builder_insertion_end,
        &generated_content,
    );

    // Write to OUT_DIR
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", final_content).unwrap();

    // Write to src directory
    let src_dest_path = Path::new("src").join(filename);
    let mut src_f = File::create(&src_dest_path).unwrap();
    write!(src_f, "{}", final_content).unwrap();

    println!(
        "cargo:rustc-env=PEST_{}_PATH={}",
        filename.to_uppercase().replace('.', "_"),
        dest_path.display()
    );
}

/// Generate strongly typed attribute value rules
fn generate_typed_attribute_values(components: &[ComponentDef], generated_content: &mut String) {
    let mut generated_value_rules: HashSet<String> = HashSet::new();

    for component in components {
        for attr in &component.attributes {
            if let Some(values) = &attr.allowed_values {
                let rule_name = format!("{}{}Values", component.name.to_upper_camel_case(), attr.name.to_upper_camel_case());
                
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

/// Generate rules for a single component
fn generate_component_rules(component: &ComponentDef, generated_content: &mut String) {
    let comp_name = component.name.to_upper_camel_case();

    // Attribute list
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

    // Event list
    let event_names: Vec<String> = component
        .events
        .iter()
        .map(|e| e.name.to_upper_camel_case())
        .collect();
    let events_rule = if event_names.is_empty() {
        "!\"__never__\"".to_string()
    } else {
        event_names.join(" | ")
    };
    generated_content.push_str(&format!(
        "{}Events = @{{ {} }}\n",
        comp_name,
        events_rule
    ));

    // Component name variations
    let variants = generate_all_variants(component.name);
    let quoted_variants: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
    generated_content.push_str(&format!(
        "{}ComponentNames = @{{ {} }}\n",
        comp_name,
        quoted_variants.join(" | ")
    ));

    // Generate component rules
    generated_content.push_str(&format!(
        "
{comp_name}AttributeDefinition = {{
    {comp_name}Attributes ~ \"=\" ~ attribute_value
}}
{comp_name}EventsDefinition = {{
    \"@\" ~ {comp_name}Events ~ \"=>\" ~ (action_with_handlers | action_list)
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

// ============================================================================
// DOCUMENTATION GENERATION
// ============================================================================

fn generate_component_documentation(components: &[ComponentDef]) {
    let doc_path = Path::new("docs").join("COMPONENT_REFERENCE.md");
    
    // Create docs directory if it doesn't exist
    std::fs::create_dir_all("docs").ok();

    let mut doc_content = String::new();

    // Header
    doc_content.push_str("# Orbis DSL Component Reference\n\n");
    doc_content.push_str("> **Auto-generated documentation** - Do not edit manually.\n");
    doc_content.push_str("> Generated from `build.rs` component definitions.\n\n");
    doc_content.push_str("## Table of Contents\n\n");

    // Generate TOC
    for component in components {
        doc_content.push_str(&format!(
            "- [{}](#{})\n",
            component.name,
            component.name.to_lowercase()
        ));
    }
    doc_content.push_str("\n---\n\n");

    // Generate component documentation
    for component in components {
        generate_component_doc(component, &mut doc_content);
    }

    // Write documentation file
    let mut f = File::create(&doc_path).unwrap_or_else(|_| {
        eprintln!("Warning: Could not create documentation file");
        File::create("/dev/null").unwrap()
    });
    write!(f, "{}", doc_content).ok();
}

fn generate_component_doc(component: &ComponentDef, doc_content: &mut String) {
    let comp_name = &component.name;
    
    // Component header
    doc_content.push_str(&format!("## {}\n\n", comp_name));
    
    // Deprecation warning
    if let Some(deprecation) = &component.deprecated {
        doc_content.push_str(&format!(
            "> ⚠️ **DEPRECATED**: {}\n",
            deprecation.message
        ));
        if let Some(alt) = deprecation.alternative {
            doc_content.push_str(&format!("> Use `{}` instead.\n", alt));
        }
        doc_content.push_str("\n");
    }

    // Description
    doc_content.push_str(&format!("{}\n\n", component.description));

    // Basic usage
    doc_content.push_str("### Usage\n\n");
    doc_content.push_str("```orbis\n");
    doc_content.push_str(&format!("<{} />\n", comp_name));
    doc_content.push_str("\n// With children:\n");
    doc_content.push_str(&format!("<{}>\n    // content\n</{}>\n", comp_name, comp_name));
    doc_content.push_str("```\n\n");

    // Attributes
    doc_content.push_str("### Attributes\n\n");
    if component.attributes.is_empty() {
        doc_content.push_str("*No component-specific attributes.*\n\n");
    } else {
        doc_content.push_str("| Attribute | Description | Allowed Values |\n");
        doc_content.push_str("|-----------|-------------|----------------|\n");
        for attr in &component.attributes {
            let values = match &attr.allowed_values {
                Some(vals) => format!("`{}`", vals.join("`, `")),
                None => "*expression*".to_string(),
            };
            let deprecated_marker = if attr.deprecated.is_some() { " ⚠️" } else { "" };
            doc_content.push_str(&format!(
                "| `{}`{} | {} | {} |\n",
                attr.name, deprecated_marker, attr.description, values
            ));
        }
        doc_content.push_str("\n");

        // Document deprecated attributes separately
        for attr in &component.attributes {
            if let Some(deprecation) = &attr.deprecated {
                doc_content.push_str(&format!(
                    "> ⚠️ **`{}`** is deprecated: {}\n",
                    attr.name, deprecation.message
                ));
                if let Some(alt) = deprecation.alternative {
                    doc_content.push_str(&format!("> Use `{}` instead.\n", alt));
                }
            }
        }
    }

    // Events
    doc_content.push_str("### Events\n\n");
    if component.events.is_empty() {
        doc_content.push_str("*No events.*\n\n");
    } else {
        doc_content.push_str("| Event | Description |\n");
        doc_content.push_str("|-------|-------------|\n");
        for event in &component.events {
            let deprecated_marker = if event.deprecated.is_some() { " ⚠️" } else { "" };
            doc_content.push_str(&format!(
                "| `@{}`{} | {} |\n",
                event.name, deprecated_marker, event.description
            ));
        }
        doc_content.push_str("\n");

        // Document deprecated events
        for event in &component.events {
            if let Some(deprecation) = &event.deprecated {
                doc_content.push_str(&format!(
                    "> ⚠️ **`@{}`** is deprecated: {}\n",
                    event.name, deprecation.message
                ));
                if let Some(alt) = deprecation.alternative {
                    doc_content.push_str(&format!("> Use `@{}` instead.\n", alt));
                }
            }
        }
    }

    // Example
    doc_content.push_str("### Example\n\n");
    doc_content.push_str("```orbis\n");
    
    let mut example_attrs: Vec<String> = Vec::new();
    for (i, attr) in component.attributes.iter().take(3).enumerate() {
        if let Some(values) = &attr.allowed_values {
            example_attrs.push(format!("{}=\"{}\"", attr.name, values.first().unwrap_or(&"value")));
        } else if i == 0 {
            example_attrs.push(format!("{}=\"example\"", attr.name));
        } else {
            example_attrs.push(format!("{}={{state.value}}", attr.name));
        }
    }
    
    let attrs_str = example_attrs.join(" ");
    if component.events.is_empty() {
        doc_content.push_str(&format!("<{} {} />\n", comp_name, attrs_str));
    } else {
        let event = &component.events[0];
        doc_content.push_str(&format!(
            "<{} {} @{} => [state.clicked = true] />\n",
            comp_name, attrs_str, event.name
        ));
    }
    doc_content.push_str("```\n\n");

    doc_content.push_str("---\n\n");
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn generate_grammar_file(out_dir: &str, filename: &str, keyword_categories: &HashMap<&str, Vec<&str>>) {
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

/// Generate all case variations for a keyword
fn generate_all_variants(keyword: &str) -> Vec<String> {
    let mut variants = Vec::new();

    variants.push(keyword.to_snake_case());
    variants.push(keyword.to_shouty_snake_case());
    variants.push(keyword.to_lower_camel_case());
    variants.push(keyword.to_upper_camel_case());
    variants.push(keyword.to_kebab_case());
    variants.push(keyword.to_shouty_kebab_case());
    variants.push(keyword.to_train_case());

    variants.sort();
    variants.dedup();
    variants
}

/// Replace content between markers
fn replace_section(content: &str, start_marker: &str, end_marker: &str, replacement: &str) -> String {
    if let Some(start_pos) = content.find(start_marker) {
        if let Some(end_pos) = content.find(end_marker) {
            let before = &content[.. start_pos];
            let after = &content[end_pos + end_marker.len() ..];
            return format!("{}{}{}", before, replacement, after);
        }
    }
    format!("{}\n{}", content, replacement)
}
