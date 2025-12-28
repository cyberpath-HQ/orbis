//! Hover Provider
//!
//! This module provides hover documentation for the Orbis DSL including:
//! - Component documentation
//! - Attribute documentation
//! - State variable types
//! - Fragment documentation

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

use crate::analysis::{span_to_range, AnalysisResult, SymbolKind, SymbolTable};
use crate::document::Document;

/// Get hover information for a position
pub fn get_hover(
    doc: &Document,
    pos: &Position,
    result: &AnalysisResult,
) -> Option<Hover> {
    // Get the word at position
    let (word, range) = doc.word_at_position(pos)?;

    // Try page property hover
    if let Some(hover) = hover_for_page_property(&word, range.clone()) {
        return Some(hover);
    }

    // Try to find symbol info
    if let Some(hover) = hover_for_symbol(&word, &result.symbols, range.clone()) {
        return Some(hover);
    }

    // Try component hover
    if let Some(hover) = hover_for_component(&word, range.clone()) {
        return Some(hover);
    }

    // Try keyword hover
    if let Some(hover) = hover_for_keyword(&word, range.clone()) {
        return Some(hover);
    }

    // Try event hover (for @click, @submit, etc.)
    if word.starts_with('@') {
        if let Some(hover) = hover_for_event(&word[1..], range.clone()) {
            return Some(hover);
        }
    }

    // Try function call hover (namespace.method format)
    if word.contains('.') {
        if let Some(hover) = hover_for_function_call(&word, range.clone()) {
            return Some(hover);
        }
    }

    None
}

/// Hover for page properties
fn hover_for_page_property(word: &str, range: Range) -> Option<Hover> {
    let (description, type_info, example) = match word {
        "id" => (
            "Unique identifier for the page",
            "`string` (required)",
            "id: \"user-profile\"",
        ),
        "title" => (
            "Page title displayed in browser tab and navigation",
            "`string` (required)",
            "title: \"User Profile\"",
        ),
        "route" => (
            "URL route path for this page",
            "`string` (required)",
            "route: \"/users/{id}\"",
        ),
        "showInMenu" => (
            "Whether to show this page in the navigation menu",
            "`boolean` (optional, default: false)",
            "showInMenu: true",
        ),
        "menuOrder" => (
            "Order of page in navigation menu (lower numbers appear first)",
            "`number` (optional)",
            "menuOrder: 10",
        ),
        "parentRoute" => (
            "Parent route for nested routing structure",
            "`string` (optional)",
            "parentRoute: \"/users\"",
        ),
        "requiresAuth" => (
            "Whether authentication is required to access this page",
            "`boolean` (optional, default: false)",
            "requiresAuth: true",
        ),
        "permissions" => (
            "Array of required permissions for accessing this page",
            "`string[]` (optional)",
            "permissions: [\"users.read\", \"users.write\"]",
        ),
        "roles" => (
            "Array of required roles for accessing this page",
            "`string[]` (optional)",
            "roles: [\"admin\", \"manager\"]",
        ),
        "layout" => (
            "Layout template to use for this page",
            "`string` (optional)",
            "layout: \"dashboard\"",
        ),
        _ => return None,
    };

    let content = format!(
        "## Page Property: `{}`\n\n{}\n\n**Type**: {}\n\n### Example\n\n```orbis\n{}\n```",
        word, description, type_info, example
    );

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(range),
    })
}

/// Hover for symbols (state, fragments, interfaces)
fn hover_for_symbol(word: &str, symbols: &SymbolTable, range: Range) -> Option<Hover> {
    // Check state variables
    if let Some(state) = symbols.state_vars.get(word) {
        let mut content = format!("## State Variable: `{}`\n\n", word);

        if let Some(type_ann) = &state.type_annotation {
            content.push_str(&format!("**Type**: `{}`\n\n", type_ann));
        }

        if state.is_computed {
            content.push_str("*Computed property* - Derived from other state values.\n\n");
        }

        if state.is_validated {
            content.push_str("*Validated* - Has validation rules attached.\n\n");
        }

        if let Some(doc) = &state.documentation {
            content.push_str(&format!("{}\n\n", doc));
        }

        content.push_str(&format!(
            "Defined at line {}",
            state.span.start_line
        ));

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(range),
        });
    }

    // Check fragments
    if let Some(frag) = symbols.fragments.get(word) {
        let mut content = format!("## Fragment: `{}`\n\n", word);

        if !frag.params.is_empty() {
            content.push_str("**Parameters**:\n");
            for param in &frag.params {
                let required = if param.required { " *(required)*" } else { "" };
                let type_ann = param
                    .type_annotation
                    .as_ref()
                    .map(|t| format!(": {}", t))
                    .unwrap_or_default();
                content.push_str(&format!("- `{}{}`{}\n", param.name, type_ann, required));
            }
            content.push('\n');
        }

        if frag.exported {
            content.push_str("*Exported* - Can be imported by other files.\n\n");
        }

        content.push_str(&format!(
            "Defined at line {}",
            frag.span.start_line
        ));

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(range),
        });
    }

    // Check interfaces
    if let Some(iface) = symbols.interfaces.get(word) {
        let mut content = format!("## Interface: `{}`\n\n", word);

        content.push_str("```typescript\n");
        content.push_str(&format!("interface {} {{\n", word));
        for member in &iface.members {
            let optional = if member.optional { "?" } else { "" };
            content.push_str(&format!("  {}{}: {};\n", member.name, optional, member.type_annotation));
        }
        content.push_str("}\n```\n\n");

        content.push_str(&format!(
            "Defined at line {}",
            iface.span.start_line
        ));

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(range),
        });
    }

    // Check imports
    if let Some(imp) = symbols.imports.get(word) {
        let mut content = format!("## Imported: `{}`\n\n", word);

        if let Some(alias) = &imp.alias {
            content.push_str(&format!("Originally named: `{}`\n\n", imp.name));
        }

        content.push_str(&format!("**From**: `{}`", imp.source));

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(range),
        });
    }

    None
}

/// Hover for events (@click, @submit, @change, etc.)
fn hover_for_event(event_name: &str, range: Range) -> Option<Hover> {
    let (description, event_object, example) = match event_name {
        "click" => (
            "Triggered when a component is clicked",
            "**Event Object**: `{ target: HTMLElement, button: number, x: number, y: number }`",
            "@click => { state.count = state.count + 1 }",
        ),
        "submit" => (
            "Triggered when a form is submitted",
            "**Event Object**: `{ data: FormData, target: HTMLElement }`\n\n**Modifiers**: `prevent` (prevents default submission)",
            "@submit.prevent => { api.call(\"saveForm\", data: $event.data) }",
        ),
        "change" => (
            "Triggered when an input value changes",
            "**Event Object**: `{ value: any, target: HTMLElement }`",
            "@change => { state.searchQuery = $event.value }",
        ),
        "input" => (
            "Triggered on every input change (real-time)",
            "**Event Object**: `{ value: string, target: HTMLElement }`",
            "@input => { state.email = $event.value }",
        ),
        "focus" => (
            "Triggered when an element receives focus",
            "**Event Object**: `{ target: HTMLElement }`",
            "@focus => { state.fieldFocused = true }",
        ),
        "blur" => (
            "Triggered when an element loses focus",
            "**Event Object**: `{ target: HTMLElement, value: any }`",
            "@blur => { state.validateEmail() }",
        ),
        "keydown" => (
            "Triggered when a key is pressed down",
            "**Event Object**: `{ key: string, code: string, ctrl: boolean, shift: boolean, alt: boolean }`\n\n**Modifiers**: `enter`, `escape`, `space`, `ctrl`, `shift`, `alt`",
            "@keydown.enter => { state.handleSearch() }",
        ),
        "keyup" => (
            "Triggered when a key is released",
            "**Event Object**: `{ key: string, code: string, ctrl: boolean, shift: boolean, alt: boolean }`",
            "@keyup.escape => { state.closeModal() }",
        ),
        "mouseenter" => (
            "Triggered when mouse enters an element",
            "**Event Object**: `{ target: HTMLElement, x: number, y: number }`",
            "@mouseenter => { state.hoveredCard = item.id }",
        ),
        "mouseleave" => (
            "Triggered when mouse leaves an element",
            "**Event Object**: `{ target: HTMLElement, x: number, y: number }`",
            "@mouseleave => { state.hoveredCard = null }",
        ),
        "load" => (
            "Triggered when an element finishes loading (images, iframes, etc.)",
            "**Event Object**: `{ target: HTMLElement }`",
            "@load => { state.imageLoaded = true }",
        ),
        "error" => (
            "Triggered when an error occurs during loading",
            "**Event Object**: `{ target: HTMLElement, error: Error }`",
            "@error => { state.imageError = true }",
        ),
        "scroll" => (
            "Triggered when an element is scrolled",
            "**Event Object**: `{ scrollTop: number, scrollLeft: number, target: HTMLElement }`",
            "@scroll => { state.scrollPosition = $event.scrollTop }",
        ),
        "resize" => (
            "Triggered when an element or window is resized",
            "**Event Object**: `{ width: number, height: number, target: HTMLElement }`",
            "@resize => { state.windowWidth = $event.width }",
        ),
        _ => return None,
    };

    let content = format!(
        "## Event: `@{}`\n\n{}\n\n{}\n\n### Example\n\n```orbis\n{}\n```\n\n### Common Modifiers\n\n- `stop` - Stop event propagation\n- `prevent` - Prevent default behavior\n- `once` - Fire only once\n- `self` - Only trigger if event target is the element itself",
        event_name, description, event_object, example
    );

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(range),
    })
}

/// Hover for function calls (console.log, toast.show, etc.)
fn hover_for_function_call(call: &str, range: Range) -> Option<Hover> {
    let (namespace, method) = call.split_once('.')?;
    
    let (description, signature, example) = match (namespace, method) {
        ("console", "log") => (
            "Logs a message to the browser console",
            "`console.log(message: any, ...args: any[])`",
            "console.log(\"User logged in:\", state.user)",
        ),
        ("console", "error") => (
            "Logs an error message to the browser console",
            "`console.error(message: any, ...args: any[])`",
            "console.error(\"Failed to load data:\", error)",
        ),
        ("console", "warn") => (
            "Logs a warning message to the browser console",
            "`console.warn(message: any, ...args: any[])`",
            "console.warn(\"API rate limit approaching\")",
        ),
        ("console", "info") => (
            "Logs an informational message to the browser console",
            "`console.info(message: any, ...args: any[])`",
            "console.info(\"Application initialized\")",
        ),
        ("toast", "show") => (
            "Displays a toast notification to the user",
            "`toast.show(message: string, options?: { variant?: \"default\" | \"success\" | \"error\" | \"warning\", duration?: number })`",
            "toast.show(\"Changes saved!\", variant: \"success\")",
        ),
        ("toast", "error") => (
            "Displays an error toast notification",
            "`toast.error(message: string, options?: { duration?: number })`",
            "toast.error(\"Failed to save changes\")",
        ),
        ("toast", "success") => (
            "Displays a success toast notification",
            "`toast.success(message: string, options?: { duration?: number })`",
            "toast.success(\"User created successfully\")",
        ),
        ("toast", "warning") => (
            "Displays a warning toast notification",
            "`toast.warning(message: string, options?: { duration?: number })`",
            "toast.warning(\"Unsaved changes will be lost\")",
        ),
        ("router", "navigate") => (
            "Navigates to a different route",
            "`router.navigate(path: string, options?: { replace?: boolean, state?: any })`",
            "router.navigate(\"/users/123\")",
        ),
        ("router", "back") => (
            "Navigates back in browser history",
            "`router.back()`",
            "router.back()",
        ),
        ("router", "forward") => (
            "Navigates forward in browser history",
            "`router.forward()`",
            "router.forward()",
        ),
        ("localStorage", "set") => (
            "Stores a value in browser local storage",
            "`localStorage.set(key: string, value: any)`",
            "localStorage.set(\"theme\", state.theme)",
        ),
        ("localStorage", "get") => (
            "Retrieves a value from browser local storage",
            "`localStorage.get(key: string): any | null`",
            "state.theme = localStorage.get(\"theme\") || \"light\"",
        ),
        ("localStorage", "remove") => (
            "Removes a value from browser local storage",
            "`localStorage.remove(key: string)`",
            "localStorage.remove(\"auth_token\")",
        ),
        ("localStorage", "clear") => (
            "Clears all values from browser local storage",
            "`localStorage.clear()`",
            "localStorage.clear()",
        ),
        ("sessionStorage", "set") => (
            "Stores a value in browser session storage (cleared when tab closes)",
            "`sessionStorage.set(key: string, value: any)`",
            "sessionStorage.set(\"temp_data\", state.formData)",
        ),
        ("sessionStorage", "get") => (
            "Retrieves a value from browser session storage",
            "`sessionStorage.get(key: string): any | null`",
            "state.formData = sessionStorage.get(\"temp_data\")",
        ),
        ("sessionStorage", "remove") => (
            "Removes a value from browser session storage",
            "`sessionStorage.remove(key: string)`",
            "sessionStorage.remove(\"temp_data\")",
        ),
        ("sessionStorage", "clear") => (
            "Clears all values from browser session storage",
            "`sessionStorage.clear()`",
            "sessionStorage.clear()",
        ),
        ("Math", "random") => (
            "Returns a random number between 0 (inclusive) and 1 (exclusive)",
            "`Math.random(): number`",
            "state.randomValue = Math.random()",
        ),
        ("Math", "floor") => (
            "Returns the largest integer less than or equal to a number",
            "`Math.floor(x: number): number`",
            "state.intValue = Math.floor(state.floatValue)",
        ),
        ("Math", "ceil") => (
            "Returns the smallest integer greater than or equal to a number",
            "`Math.ceil(x: number): number`",
            "state.roundedUp = Math.ceil(state.value)",
        ),
        ("Math", "round") => (
            "Returns the value of a number rounded to the nearest integer",
            "`Math.round(x: number): number`",
            "state.rounded = Math.round(state.price)",
        ),
        ("Math", "abs") => (
            "Returns the absolute value of a number",
            "`Math.abs(x: number): number`",
            "state.distance = Math.abs(state.x - state.y)",
        ),
        ("Math", "min") => (
            "Returns the smallest of the given numbers",
            "`Math.min(...values: number[]): number`",
            "state.minValue = Math.min(state.a, state.b, state.c)",
        ),
        ("Math", "max") => (
            "Returns the largest of the given numbers",
            "`Math.max(...values: number[]): number`",
            "state.maxValue = Math.max(state.a, state.b, state.c)",
        ),
        ("JSON", "parse") => (
            "Parses a JSON string into a JavaScript object",
            "`JSON.parse(text: string): any`",
            "state.data = JSON.parse(state.jsonString)",
        ),
        ("JSON", "stringify") => (
            "Converts a JavaScript value to a JSON string",
            "`JSON.stringify(value: any, options?: { space?: number }): string`",
            "state.jsonString = JSON.stringify(state.data, space: 2)",
        ),
        ("Date", "now") => (
            "Returns the current timestamp in milliseconds since Unix epoch",
            "`Date.now(): number`",
            "state.timestamp = Date.now()",
        ),
        ("window", "alert") => (
            "Displays a modal alert dialog",
            "`window.alert(message: string)`",
            "window.alert(\"Important message!\")",
        ),
        ("window", "confirm") => (
            "Displays a modal dialog with OK and Cancel buttons",
            "`window.confirm(message: string): boolean`",
            "if window.confirm(\"Delete this item?\") { state.deleteItem() }",
        ),
        _ => return None,
    };

    let content = format!(
        "## Function: `{}`\n\n{}\n\n### Signature\n\n```typescript\n{}\n```\n\n### Example\n\n```orbis\n{}\n```",
        call, description, signature, example
    );

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(range),
    })
}

/// Hover for components
fn hover_for_component(word: &str, range: Range) -> Option<Hover> {
    let component_info = get_component_info(word)?;

    let mut content = format!("## Component: `<{}>`\n\n", word);
    content.push_str(&format!("{}\n\n", component_info.description));

    if !component_info.attributes.is_empty() {
        content.push_str("### Attributes\n\n");
        content.push_str("| Name | Description | Values |\n");
        content.push_str("|------|-------------|--------|\n");
        for (name, desc, values) in &component_info.attributes {
            let values_str = values
                .as_ref()
                .map(|v| format!("`{}`", v.join("`, `")))
                .unwrap_or_else(|| "any".to_string());
            content.push_str(&format!("| `{}` | {} | {} |\n", name, desc, values_str));
        }
        content.push('\n');
    }

    if !component_info.events.is_empty() {
        content.push_str("### Events\n\n");
        for (name, desc) in &component_info.events {
            content.push_str(&format!("- `@{}` - {}\n", name, desc));
        }
        content.push('\n');
    }

    content.push_str(&format!("**Category**: {}", component_info.category));

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(range),
    })
}

/// Hover for keywords
fn hover_for_keyword(word: &str, range: Range) -> Option<Hover> {
    let (title, description, example) = match word.to_lowercase().as_str() {
        "page" => (
            "Page Block",
            "Defines metadata for the page including id, title, route, and access control.",
            "```orbis\npage {\n    id: \"my-page\"\n    title: \"My Page\"\n    route: \"/my-page\"\n    requiresAuth: true\n}\n```",
        ),
        "state" => (
            "State Block",
            "Declares reactive state variables. State changes automatically trigger re-renders.",
            "```orbis\nstate {\n    count = 0\n    name: string = \"\"\n    @computed total: number => count * 2\n}\n```",
        ),
        "hooks" => (
            "Hooks Block",
            "Contains lifecycle hooks (@mount, @unmount) and watchers (@watch).",
            "```orbis\nhooks {\n    @mount => {\n        api.call(\"loadData\")\n    }\n    @watch(state.count, debounce: 300) => {\n        localStorage.set(\"count\", state.count)\n    }\n}\n```",
        ),
        "template" => (
            "Template Block",
            "Defines the UI using JSX-like component syntax.",
            "```orbis\ntemplate {\n    <Container>\n        <Text content={state.message} />\n        <Button label=\"Click\" @click => { state.count = state.count + 1 } />\n    </Container>\n}\n```",
        ),
        "fragment" => (
            "Fragment Definition",
            "Defines a reusable component composition with optional parameters and slots.",
            "```orbis\nfragment UserCard(user: User) {\n    <Card>\n        <Text content={user.name} />\n        <slot />\n    </Card>\n}\n```",
        ),
        "interface" => (
            "Interface Definition",
            "Defines a type interface for structured data.",
            "```orbis\ninterface User {\n    id: number\n    name: string\n    email?: string\n    role: \"admin\" | \"user\"\n}\n```",
        ),
        "styles" => (
            "Styles Block",
            "Defines scoped CSS styles for the page. Supports @apply and @screen for Tailwind.",
            "```orbis\nstyles {\n    .card {\n        @apply rounded-lg shadow-md;\n        padding: 1rem;\n    }\n    @screen md {\n        .card { padding: 2rem; }\n    }\n}\n```",
        ),
        "if" => (
            "Conditional Rendering",
            "Renders content conditionally based on an expression.",
            "```orbis\nif state.isLoggedIn {\n    <Text content=\"Welcome!\" />\n} else {\n    <Button label=\"Login\" @click => { router.navigate(\"/login\") } />\n}\n```",
        ),
        "for" => (
            "Loop Rendering",
            "Iterates over an array to render content for each item.",
            "```orbis\nfor item in state.items {\n    <Card title={item.name}>\n        <Text content={item.description} />\n    </Card>\n}\n```",
        ),
        "when" => (
            "Pattern Matching",
            "Matches an expression against multiple patterns.",
            "```orbis\nwhen state.status {\n    \"loading\" => { <LoadingOverlay visible={true} /> }\n    \"error\" => { <Alert variant=\"destructive\" title=\"Error\" /> }\n    \"success\" => { <Text content=\"Done!\" /> }\n}\n```",
        ),
        "import" => (
            "Import Statement",
            "Imports fragments, interfaces, or constants from other files.",
            "```orbis\nimport { UserCard, PostCard } from \"./fragments/cards.orbis\"\nimport { API_URL } from \"./constants.orbis\"\n```",
        ),
        "export" => (
            "Export Statement",
            "Exports fragments, interfaces, or constants for use in other files.",
            "```orbis\nexport fragment MyCard(title: string) {\n    <Card title={title}>\n        <slot />\n    </Card>\n}\n```",
        ),
        "@mount" => (
            "Mount Lifecycle Hook",
            "Called when the page is mounted. Use for initialization and data loading.",
            "```orbis\n@mount => {\n    api.call(\"loadUser\", id: state.userId) {\n        success => { state.user = $response }\n    }\n}\n```",
        ),
        "@unmount" => (
            "Unmount Lifecycle Hook",
            "Called when the page is unmounted. Use for cleanup.",
            "```orbis\n@unmount => {\n    console.log(\"Cleaning up\")\n}\n```",
        ),
        "@watch" => (
            "State Watcher",
            "Watches for changes to state variables and runs actions when they change.",
            "```orbis\n@watch(state.searchQuery, debounce: 300) => {\n    api.call(\"search\", query: state.searchQuery)\n}\n```",
        ),
        "@computed" => (
            "Computed Property",
            "Declares a derived state value that updates when dependencies change.",
            "```orbis\n@computed fullName: string => state.firstName + \" \" + state.lastName\n@computed isEven: boolean => state.count % 2 == 0\n```",
        ),
        "@validate" => (
            "Validation Rule",
            "Attaches validation rules to a state variable (Zod v4-compatible).",
            "```orbis\nemail: string = \"\" {\n    @validate email\n    @validate required\n    @validate message: \"Please enter a valid email\"\n}\n```",
        ),
        _ => return None,
    };

    let content = format!("## {}\n\n{}\n\n### Example\n\n{}", title, description, example);

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(range),
    })
}

/// Component information for hover
struct ComponentInfo {
    description: String,
    category: String,
    attributes: Vec<(&'static str, &'static str, Option<Vec<&'static str>>)>,
    events: Vec<(&'static str, &'static str)>,
}

/// Get component info for hover
fn get_component_info(name: &str) -> Option<ComponentInfo> {
    match name {
        "Container" => Some(ComponentInfo {
            description: "A generic container element for grouping and layout purposes. Renders as a `<div>` with optional styling.".to_string(),
            category: "Layout".to_string(),
            attributes: vec![
                ("id", "Unique identifier", None),
                ("className", "CSS class name(s)", None),
                ("style", "Inline styles", None),
                ("visible", "Visibility expression", None),
            ],
            events: vec![
                ("click", "Triggered when clicked"),
                ("mouseEnter", "Triggered on mouse enter"),
                ("mouseLeave", "Triggered on mouse leave"),
            ],
        }),
        "Button" => Some(ComponentInfo {
            description: "A clickable button component with multiple variants and states.".to_string(),
            category: "Forms".to_string(),
            attributes: vec![
                ("label", "Button text (required)", None),
                ("variant", "Visual variant", Some(vec!["default", "destructive", "outline", "secondary", "ghost", "link"])),
                ("size", "Button size", Some(vec!["xs", "sm", "md", "lg", "xl"])),
                ("disabled", "Disabled state", None),
                ("loading", "Loading state", None),
                ("icon", "Icon name", None),
                ("iconPosition", "Icon position", Some(vec!["left", "right"])),
            ],
            events: vec![
                ("click", "Triggered when clicked"),
            ],
        }),
        "Text" => Some(ComponentInfo {
            description: "A text display component for paragraphs and inline text. Supports `{expression}` interpolation in content.".to_string(),
            category: "Typography".to_string(),
            attributes: vec![
                ("content", "Text content (supports interpolation)", None),
                ("variant", "Text style", Some(vec!["body", "caption", "label", "code", "muted"])),
                ("className", "CSS class name(s)", None),
            ],
            events: vec![
                ("click", "Triggered when clicked"),
            ],
        }),
        "Heading" => Some(ComponentInfo {
            description: "A heading component for titles and section headers (h1-h6).".to_string(),
            category: "Typography".to_string(),
            attributes: vec![
                ("text", "Heading text", None),
                ("level", "Heading level", Some(vec!["1", "2", "3", "4", "5", "6"])),
                ("className", "CSS class name(s)", None),
            ],
            events: vec![
                ("click", "Triggered when clicked"),
            ],
        }),
        "Field" => Some(ComponentInfo {
            description: "A form input field supporting various input types. Use `bindTo` for two-way data binding.".to_string(),
            category: "Forms".to_string(),
            attributes: vec![
                ("name", "Field name (required)", None),
                ("fieldType", "Input type", Some(vec!["text", "password", "email", "number", "tel", "url", "date", "textarea", "checkbox", "radio", "select", "file"])),
                ("label", "Label text", None),
                ("placeholder", "Placeholder text", None),
                ("bindTo", "Two-way binding path", None),
                ("required", "Required field", None),
                ("disabled", "Disabled state", None),
            ],
            events: vec![
                ("change", "Triggered when value changes"),
                ("focus", "Triggered on focus"),
                ("blur", "Triggered on blur"),
            ],
        }),
        "Card" => Some(ComponentInfo {
            description: "A card container for grouping related content with optional header and footer.".to_string(),
            category: "Data Display".to_string(),
            attributes: vec![
                ("title", "Card title", None),
                ("subtitle", "Card subtitle", None),
                ("hoverable", "Show hover effect", None),
                ("className", "CSS class name(s)", None),
            ],
            events: vec![
                ("click", "Triggered when clicked"),
            ],
        }),
        "Table" => Some(ComponentInfo {
            description: "A data table component for displaying tabular data with sorting, pagination, and row selection.".to_string(),
            category: "Data Display".to_string(),
            attributes: vec![
                ("columns", "Column definitions (required)", None),
                ("dataSource", "Data array path (required)", None),
                ("rowKey", "Unique row key property", None),
                ("pagination", "Pagination config", None),
                ("selectable", "Selection mode", None),
                ("sortable", "Enable sorting", None),
            ],
            events: vec![
                ("rowClick", "Triggered when row clicked"),
                ("select", "Triggered on selection change"),
                ("pageChange", "Triggered on page change"),
            ],
        }),
        "Modal" => Some(ComponentInfo {
            description: "A modal dialog overlay for displaying focused content.".to_string(),
            category: "Overlays".to_string(),
            attributes: vec![
                ("open", "Open state", None),
                ("title", "Modal title", None),
                ("size", "Modal size", Some(vec!["sm", "md", "lg", "xl", "full"])),
                ("closable", "Show close button", None),
            ],
            events: vec![
                ("close", "Triggered when closed"),
            ],
        }),
        "Flex" => Some(ComponentInfo {
            description: "A Flexbox-based layout component for flexible, responsive layouts.".to_string(),
            category: "Layout".to_string(),
            attributes: vec![
                ("direction", "Flex direction", Some(vec!["row", "column", "row-reverse", "column-reverse"])),
                ("justify", "Justify content", Some(vec!["start", "end", "center", "between", "around", "evenly"])),
                ("align", "Align items", Some(vec!["start", "end", "center", "stretch", "baseline"])),
                ("gap", "Gap between items", None),
                ("wrap", "Wrap items", None),
            ],
            events: vec![],
        }),
        "Grid" => Some(ComponentInfo {
            description: "A CSS Grid-based layout component for creating responsive grid layouts.".to_string(),
            category: "Layout".to_string(),
            attributes: vec![
                ("columns", "Number of columns", None),
                ("gap", "Gap between items", None),
            ],
            events: vec![],
        }),
        "Alert" => Some(ComponentInfo {
            description: "An alert message component for displaying important information.".to_string(),
            category: "Feedback".to_string(),
            attributes: vec![
                ("variant", "Alert variant", Some(vec!["default", "destructive"])),
                ("title", "Alert title", None),
                ("dismissible", "Can be dismissed", None),
            ],
            events: vec![],
        }),
        "Image" => Some(ComponentInfo {
            description: "An image display component with lazy loading support.".to_string(),
            category: "Media".to_string(),
            attributes: vec![
                ("src", "Image source URL", None),
                ("alt", "Alt text", None),
                ("width", "Width", None),
                ("height", "Height", None),
                ("fit", "Object fit", Some(vec!["contain", "cover", "fill", "none", "scale-down"])),
                ("loading", "Loading strategy", Some(vec!["lazy", "eager"])),
            ],
            events: vec![],
        }),
        _ => None,
    }
}
