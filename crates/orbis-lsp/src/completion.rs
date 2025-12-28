//! Completion Provider
//!
//! This module provides intelligent autocompletion for the Orbis DSL including:
//! - Component names
//! - Component attributes
//! - Event names
//! - State variables
//! - Type annotations
//! - Keywords

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit,
    Documentation, InsertTextFormat, MarkupContent, MarkupKind, Position, Range, TextEdit,
};

use crate::analysis::SymbolTable;
use crate::document::{ContextType, DocumentContext};

/// Get completions for a given context
pub fn get_completions(context: &DocumentContext, symbols: &SymbolTable) -> Vec<CompletionItem> {
    match &context.context_type {
        ContextType::TopLevel => top_level_completions(),
        ContextType::PageBlock => page_attribute_completions(),
        ContextType::StateBlock => state_completions(symbols),
        ContextType::HooksBlock => hooks_completions(),
        ContextType::Template => template_completions(symbols, &context.trigger_word),
        ContextType::ComponentAttribute { component } => {
            component_attribute_completions(component, &context.trigger_word)
        }
        ContextType::ComponentEvent { component } => {
            component_event_completions(component)
        }
        ContextType::Expression => expression_completions(symbols, &context.trigger_word),
        ContextType::ActionBody => action_completions(),
        ContextType::Import => import_completions(),
        ContextType::TypeAnnotation => type_completions(),
        ContextType::Styles => styles_completions(),
        ContextType::Unknown => vec![],
    }
}

/// Top-level block completions
fn top_level_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "page".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Page metadata block".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines page metadata like id, title, route, etc.\n\n```orbis\npage {\n    id: \"my-page\"\n    title: \"My Page\"\n    route: \"/my-page\"\n}\n```".to_string(),
            })),
            insert_text: Some("page {\n    id: \"$1\"\n    title: \"$2\"\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "state".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("State declarations block".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Declares reactive state variables.\n\n```orbis\nstate {\n    count = 0\n    name: string = \"\"\n    @computed total: number => count * 2\n}\n```".to_string(),
            })),
            insert_text: Some("state {\n    $1\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "hooks".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Lifecycle hooks block".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines lifecycle hooks and watchers.\n\n```orbis\nhooks {\n    @mount => {\n        console.log(\"Mounted\")\n    }\n    @watch(state.count) => {\n        console.log(\"Count changed\")\n    }\n}\n```".to_string(),
            })),
            insert_text: Some("hooks {\n    @mount => {\n        $1\n    }\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "template".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Template block".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines the UI template with components.\n\n```orbis\ntemplate {\n    <Container>\n        <Text content=\"Hello, world!\" />\n    </Container>\n}\n```".to_string(),
            })),
            insert_text: Some("template {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "fragment".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Fragment definition".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines a reusable fragment component.\n\n```orbis\nfragment UserCard(user: User) {\n    <Card>\n        <Text content={user.name} />\n    </Card>\n}\n```".to_string(),
            })),
            insert_text: Some("fragment $1($2) {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "interface".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Interface definition".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines a type interface.\n\n```orbis\ninterface User {\n    id: number\n    name: string\n    email?: string\n}\n```".to_string(),
            })),
            insert_text: Some("interface $1 {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "styles".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Styles block".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Defines scoped CSS styles.\n\n```orbis\nstyles {\n    .container {\n        padding: 1rem;\n    }\n}\n```".to_string(),
            })),
            insert_text: Some("styles {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "import".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Import statement".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Import fragments or types from other files.\n\n```orbis\nimport { UserCard, PostCard } from \"./fragments/cards.orbis\"\n```".to_string(),
            })),
            insert_text: Some("import { $1 } from \"$2\"".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "export".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Export statement".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Export fragments or types for use in other files.\n\n```orbis\nexport fragment MyFragment { ... }\n```".to_string(),
            })),
            insert_text: Some("export $0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Page block attribute completions
fn page_attribute_completions() -> Vec<CompletionItem> {
    let attrs = vec![
        ("id", "Unique page identifier", "id: \"$1\""),
        ("title", "Page title", "title: \"$1\""),
        ("description", "Page description", "description: \"$1\""),
        ("icon", "Page icon name", "icon: \"$1\""),
        ("route", "URL route path", "route: \"/$1\""),
        ("showInMenu", "Show in navigation menu", "showInMenu: ${1|true,false|}"),
        ("menuOrder", "Menu display order", "menuOrder: $1"),
        ("parentRoute", "Parent route for nesting", "parentRoute: \"$1\""),
        ("requiresAuth", "Requires authentication", "requiresAuth: ${1|true,false|}"),
        ("permissions", "Required permissions", "permissions: [$1]"),
        ("roles", "Required roles", "roles: [$1]"),
        ("layout", "Layout template", "layout: \"$1\""),
    ];

    attrs
        .into_iter()
        .map(|(name, desc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(desc.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

/// State block completions
fn state_completions(symbols: &SymbolTable) -> Vec<CompletionItem> {
    let mut items = vec![
        CompletionItem {
            label: "@computed".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Computed property".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Declares a computed (derived) property.\n\n```orbis\n@computed fullName: string => firstName + \" \" + lastName\n```".to_string(),
            })),
            insert_text: Some("@computed $1: $2 => $0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@validate".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Validation rule".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Adds validation to a state variable.\n\n```orbis\nemail: string = \"\" {\n    @validate email\n    @validate required\n}\n```".to_string(),
            })),
            insert_text: Some("@validate $0".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ];

    // Add type completions
    items.extend(type_completions());

    items
}

/// Hooks block completions
fn hooks_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "@mount".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Mount lifecycle hook".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Called when the page mounts.\n\n```orbis\n@mount => {\n    api.call(\"loadData\")\n}\n```".to_string(),
            })),
            insert_text: Some("@mount => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@unmount".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Unmount lifecycle hook".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Called when the page unmounts.\n\n```orbis\n@unmount => {\n    console.log(\"Cleanup\")\n}\n```".to_string(),
            })),
            insert_text: Some("@unmount => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@watch".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("State watcher".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Watches for state changes.\n\n```orbis\n@watch(state.count, debounce: 300) => {\n    localStorage.set(\"count\", state.count)\n}\n```".to_string(),
            })),
            insert_text: Some("@watch(state.$1) => {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Template block completions
fn template_completions(symbols: &SymbolTable, trigger: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add component completions
    items.extend(component_completions());

    // Add fragment completions from symbols
    for (name, frag) in &symbols.fragments {
        let params_str = frag
            .params
            .iter()
            .map(|p| {
                if p.required {
                    format!("{}={{}}", p.name)
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        items.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Fragment".to_string()),
            label_details: Some(CompletionItemLabelDetails {
                description: Some(format!("({} params)", frag.params.len())),
                detail: None,
            }),
            insert_text: Some(format!("<{} {} />", name, params_str)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    // Add imported fragments
    for (name, imp) in &symbols.imports {
        items.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("Imported from {}", imp.source)),
            insert_text: Some(format!("<{} $0 />", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    // Add control flow
    items.extend(control_flow_completions());

    items
}

/// Component completions with all built-in components
fn component_completions() -> Vec<CompletionItem> {
    // Component definitions matching orbis-dsl/build/components.rs
    let components = vec![
        // Layout
        ("Container", "A generic container element for grouping and layout", "<Container>\n    $0\n</Container>"),
        ("Grid", "CSS Grid-based layout for responsive grids", "<Grid columns={$1}>\n    $0\n</Grid>"),
        ("Flex", "Flexbox-based layout for flexible layouts", "<Flex direction=\"$1\">\n    $0\n</Flex>"),
        ("Spacer", "Invisible spacer for consistent spacing", "<Spacer size=\"$1\" />"),
        ("Divider", "Visual separator line", "<Divider orientation=\"horizontal\" />"),
        
        // Typography
        ("Text", "Text display for paragraphs and inline text", "<Text content=\"$1\" />"),
        ("Heading", "Heading component for titles (h1-h6)", "<Heading level=\"1\" text=\"$1\" />"),
        
        // Forms
        ("Field", "Form input field with various types", "<Field name=\"$1\" fieldType=\"text\" label=\"$2\" />"),
        ("Form", "Form container for field grouping", "<Form id=\"$1\">\n    $0\n</Form>"),
        ("Button", "Clickable button with variants", "<Button label=\"$1\" @click => { $0 } />"),
        ("Dropdown", "Dropdown menu component", "<Dropdown trigger={<Button label=\"$1\" />}>\n    $0\n</Dropdown>"),
        
        // Data Display
        ("Card", "Card container for related content", "<Card title=\"$1\">\n    $0\n</Card>"),
        ("Table", "Data table with sorting/pagination", "<Table columns={$1} dataSource={state.$2} />"),
        ("List", "List for displaying arrays of items", "<List dataSource={state.$1}>\n    $0\n</List>"),
        ("Badge", "Small status indicator or label", "<Badge text=\"$1\" variant=\"default\" />"),
        ("StatCard", "Statistics display card for dashboards", "<StatCard title=\"$1\" value={$2} />"),
        ("DataDisplay", "Key-value data display", "<DataDisplay data={$1} />"),
        
        // Navigation
        ("Link", "Navigation link component", "<Link href=\"$1\" label=\"$2\" />"),
        ("Tabs", "Tabbed interface container", "<Tabs defaultTab=\"$1\">\n    $0\n</Tabs>"),
        ("Accordion", "Collapsible content panels", "<Accordion>\n    $0\n</Accordion>"),
        ("Breadcrumb", "Navigation breadcrumb trail", "<Breadcrumb items={$1} />"),
        
        // Feedback
        ("Alert", "Alert message component", "<Alert variant=\"default\">\n    $0\n</Alert>"),
        ("Progress", "Progress indicator", "<Progress value={$1} />"),
        ("LoadingOverlay", "Full-screen loading indicator", "<LoadingOverlay visible={$1} />"),
        ("Skeleton", "Loading placeholder skeleton", "<Skeleton variant=\"text\" />"),
        ("EmptyState", "Empty state placeholder", "<EmptyState title=\"$1\" />"),
        
        // Overlays
        ("Modal", "Modal dialog overlay", "<Modal open={$1} title=\"$2\">\n    $0\n</Modal>"),
        ("Tooltip", "Tooltip on hover", "<Tooltip content=\"$1\">\n    $0\n</Tooltip>"),
        
        // Media
        ("Image", "Image display component", "<Image src=\"$1\" alt=\"$2\" />"),
        ("Icon", "Icon component", "<Icon name=\"$1\" />"),
        ("Avatar", "User avatar display", "<Avatar src=\"$1\" />"),
        ("Chart", "Data visualization chart", "<Chart chartType=\"line\" dataSource={$1} />"),
        
        // Utility
        ("Section", "Page section with optional title", "<Section title=\"$1\">\n    $0\n</Section>"),
        ("PageHeader", "Page header with title/breadcrumbs", "<PageHeader title=\"$1\" />"),
    ];

    components
        .into_iter()
        .map(|(name, desc, snippet)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(desc.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("{}\n\n```orbis\n{}\n```", desc, snippet),
            })),
            // Insert just the component name, let the user add attributes
            insert_text: Some(name.to_string()),
            insert_text_format: None,
            ..Default::default()
        })
        .collect()
}

/// Control flow completions
fn control_flow_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Conditional rendering".to_string()),
            insert_text: Some("if ${1:condition} {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "for".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Loop over items".to_string()),
            insert_text: Some("for ${1:item} in ${2:state.items} {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "when".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Pattern matching".to_string()),
            insert_text: Some("when ${1:expression} {\n    ${2:pattern} => {\n        $0\n    }\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Component attribute completions
fn component_attribute_completions(component: &str, trigger: &str) -> Vec<CompletionItem> {
    let attrs = get_component_attributes(component);
    
    attrs
        .into_iter()
        .filter(|(name, _, _)| trigger.is_empty() || name.to_lowercase().starts_with(&trigger.to_lowercase()))
        .map(|(name, desc, values)| {
            let insert_text = if let Some(vals) = values {
                format!("{}=\"${{1|{}|}}\"", name, vals.join(","))
            } else {
                format!("{}=${{1}}", name)
            };

            CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(desc.to_string()),
                insert_text: Some(insert_text),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            }
        })
        .collect()
}

/// Get attributes for a component
fn get_component_attributes(component: &str) -> Vec<(&'static str, &'static str, Option<Vec<&'static str>>)> {
    match component.to_lowercase().as_str() {
        "container" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("style", "Inline styles", None),
            ("visible", "Visibility expression", None),
        ],
        "button" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("label", "Button text (required)", None),
            ("variant", "Visual variant", Some(vec!["default", "destructive", "outline", "secondary", "ghost", "link"])),
            ("size", "Button size", Some(vec!["xs", "sm", "md", "lg", "xl"])),
            ("disabled", "Disabled state", None),
            ("loading", "Loading state", None),
            ("icon", "Icon name", None),
            ("iconPosition", "Icon position", Some(vec!["left", "right"])),
            ("visible", "Visibility expression", None),
        ],
        "text" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("content", "Text content", None),
            ("variant", "Text style", Some(vec!["body", "caption", "label", "code", "muted"])),
            ("visible", "Visibility expression", None),
        ],
        "heading" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("text", "Heading text", None),
            ("level", "Heading level", Some(vec!["1", "2", "3", "4", "5", "6"])),
            ("visible", "Visibility expression", None),
        ],
        "field" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("name", "Field name (required)", None),
            ("fieldType", "Input type", Some(vec!["text", "password", "email", "number", "tel", "url", "date", "time", "datetime-local", "textarea", "checkbox", "radio", "select", "file", "hidden", "switch"])),
            ("label", "Label text", None),
            ("placeholder", "Placeholder text", None),
            ("description", "Help text", None),
            ("defaultValue", "Default value", None),
            ("bindTo", "Two-way binding path", None),
            ("required", "Required field", None),
            ("disabled", "Disabled state", None),
            ("readOnly", "Read-only state", None),
            ("options", "Options for select/radio", None),
            ("validation", "Validation rules", None),
            ("visible", "Visibility expression", None),
        ],
        "card" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("title", "Card title", None),
            ("subtitle", "Card subtitle", None),
            ("hoverable", "Hover effect", None),
            ("visible", "Visibility expression", None),
        ],
        "table" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("columns", "Column definitions (required)", None),
            ("dataSource", "Data array path (required)", None),
            ("rowKey", "Unique row key property", None),
            ("pagination", "Pagination config", None),
            ("selectable", "Selection mode", None),
            ("sortable", "Enable sorting", None),
            ("searchable", "Enable search", None),
            ("emptyText", "Empty state text", None),
            ("loading", "Loading state", None),
            ("visible", "Visibility expression", None),
        ],
        "modal" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("open", "Open state", None),
            ("title", "Modal title", None),
            ("size", "Modal size", Some(vec!["sm", "md", "lg", "xl", "full"])),
            ("closable", "Show close button", None),
            ("visible", "Visibility expression", None),
        ],
        "flex" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("direction", "Flex direction", Some(vec!["row", "column", "row-reverse", "column-reverse"])),
            ("justify", "Justify content", Some(vec!["start", "end", "center", "between", "around", "evenly"])),
            ("align", "Align items", Some(vec!["start", "end", "center", "stretch", "baseline"])),
            ("gap", "Gap between items", None),
            ("wrap", "Wrap items", None),
            ("visible", "Visibility expression", None),
        ],
        "grid" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("columns", "Number of columns", None),
            ("gap", "Gap between items", None),
            ("visible", "Visibility expression", None),
        ],
        "image" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("src", "Image source URL", None),
            ("alt", "Alt text", None),
            ("width", "Width", None),
            ("height", "Height", None),
            ("fit", "Object fit", Some(vec!["contain", "cover", "fill", "none", "scale-down"])),
            ("loading", "Loading strategy", Some(vec!["lazy", "eager"])),
            ("visible", "Visibility expression", None),
        ],
        "alert" => vec![
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("variant", "Alert variant", Some(vec!["default", "destructive"])),
            ("title", "Alert title", None),
            ("dismissible", "Can be dismissed", None),
            ("visible", "Visibility expression", None),
        ],
        _ => vec![
            // Common attributes for unknown components
            ("id", "Unique identifier", None),
            ("className", "CSS class name(s)", None),
            ("style", "Inline styles", None),
            ("visible", "Visibility expression", None),
        ],
    }
}

/// Component event completions
fn component_event_completions(component: &str) -> Vec<CompletionItem> {
    let events = get_component_events(component);

    events
        .into_iter()
        .map(|(name, desc)| CompletionItem {
            label: format!("@{}", name),
            kind: Some(CompletionItemKind::EVENT),
            detail: Some(desc.to_string()),
            insert_text: Some(format!("@{} => {{\n    $0\n}}", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

/// Get events for a component
fn get_component_events(component: &str) -> Vec<(&'static str, &'static str)> {
    match component.to_lowercase().as_str() {
        "container" => vec![
            ("click", "Triggered when clicked"),
            ("mouseEnter", "Triggered on mouse enter"),
            ("mouseLeave", "Triggered on mouse leave"),
        ],
        "button" => vec![("click", "Triggered when clicked")],
        "text" => vec![("click", "Triggered when clicked")],
        "heading" => vec![("click", "Triggered when clicked")],
        "field" => vec![
            ("change", "Triggered when value changes"),
            ("focus", "Triggered on focus"),
            ("blur", "Triggered on blur"),
        ],
        "form" => vec![("submit", "Triggered on form submit")],
        "card" => vec![("click", "Triggered when clicked")],
        "table" => vec![
            ("rowClick", "Triggered when row clicked"),
            ("rowDoubleClick", "Triggered on row double-click"),
            ("select", "Triggered on selection change"),
            ("pageChange", "Triggered on page change"),
            ("sortChange", "Triggered on sort change"),
        ],
        "list" => vec![("rowClick", "Triggered when item clicked")],
        "modal" => vec![("close", "Triggered when modal closes")],
        _ => vec![
            ("click", "Triggered when clicked"),
        ],
    }
}

/// Expression completions
fn expression_completions(symbols: &SymbolTable, trigger: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add state prefix
    items.push(CompletionItem {
        label: "state".to_string(),
        kind: Some(CompletionItemKind::MODULE),
        detail: Some("Access state variables".to_string()),
        insert_text: Some("state.".to_string()),
        ..Default::default()
    });

    // Add state variables if trigger starts with "state."
    if trigger.starts_with("state.") {
        for (name, sym) in &symbols.state_vars {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: sym.type_annotation.clone(),
                documentation: sym.documentation.clone().map(|d| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: d,
                    })
                }),
                ..Default::default()
            });
        }
    }

    // Add special variables
    items.extend(vec![
        CompletionItem {
            label: "$response".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("API response data".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$error".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Error object".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$event".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Event object".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$oldValue".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Previous value (in watchers)".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$newValue".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("New value (in watchers)".to_string()),
            ..Default::default()
        },
    ]);

    items
}

/// Action completions
fn action_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "api.call".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Make an API call".to_string()),
            insert_text: Some("api.call(\"$1\", $2) {\n    success => { $0 }\n    error => { }\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "toast.show".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Show a toast notification".to_string()),
            insert_text: Some("toast.show(\"$1\", level: ${2|info,success,warning,error|})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "router.navigate".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Navigate to a route".to_string()),
            insert_text: Some("router.navigate(\"$1\")".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "router.back".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Navigate back".to_string()),
            insert_text: Some("router.back()".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "console.log".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("Log to console".to_string()),
            insert_text: Some("console.log($1)".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "state assignment".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("Assign to state variable".to_string()),
            insert_text: Some("state.$1 = $2".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Import completions
fn import_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "from".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Import from path".to_string()),
            insert_text: Some("from \"$1\"".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Type annotation completions
fn type_completions() -> Vec<CompletionItem> {
    let types = vec![
        ("string", "String type"),
        ("number", "Number type"),
        ("boolean", "Boolean type"),
        ("bool", "Boolean type (alias)"),
        ("object", "Object type"),
        ("array", "Array type"),
        ("null", "Null type"),
        ("any", "Any type"),
        ("void", "Void type"),
        ("never", "Never type"),
    ];

    types
        .into_iter()
        .map(|(name, desc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(desc.to_string()),
            ..Default::default()
        })
        .collect()
}

/// Styles block completions
fn styles_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "@apply".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Apply Tailwind utilities".to_string()),
            insert_text: Some("@apply $1;".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@screen".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Responsive breakpoint".to_string()),
            insert_text: Some("@screen ${1|sm,md,lg,xl,2xl|} {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@media".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Media query".to_string()),
            insert_text: Some("@media ($1) {\n    $0\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "@keyframes".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Animation keyframes".to_string()),
            insert_text: Some("@keyframes $1 {\n    from { $2 }\n    to { $0 }\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}
