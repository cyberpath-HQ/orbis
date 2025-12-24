// =============================================================================
// ORBIS DSL BUILD SYSTEM - Component Definitions
// =============================================================================
//
// This module contains the authoritative definitions for all UI components
// supported by the Orbis DSL. These definitions MUST be kept in sync with
// the TypeScript schemas in `orbis/src/types/schema/components.ts`.
//
// Each component definition specifies:
// - Whitelisted attributes (what properties can be set)
// - Whitelisted events (what handlers can be attached)
// - Strongly-typed attribute values (enums like variant, size, etc.)
//
// ADDING A NEW COMPONENT:
// 1. Add the TypeScript interface in `components.ts`
// 2. Add the ComponentDef here with matching attributes/events
// 3. Rebuild to regenerate grammar and documentation
// 4. Add tests in `tests/attribute_whitelisting.rs`
//
// =============================================================================
//
// NOTE: This file is designed to be used with `include!` in build.rs.
// All dependencies (types) must be brought into scope by including
// data_structures.rs BEFORE this file is included.
//
// Required in scope before include:
// - AttributeDef, ComponentDef, EventDef from data_structures.rs
//
// =============================================================================

// =============================================================================
// TYPE VALUE DEFINITIONS
// =============================================================================
// These constants define the allowed values for strongly-typed attributes.
// They MUST match the TypeScript type definitions in `base.ts`.

/// Valid size values (from base.ts: Size)
pub const SIZE_VALUES: &[&str] = &["xs", "sm", "md", "lg", "xl"];

/// Valid button variants (from base.ts: ButtonVariant)
pub const BUTTON_VARIANT_VALUES: &[&str] = &[
    "default", "destructive", "outline", "secondary", "ghost", "link"
];

/// Valid alert variants (from base.ts: AlertVariant)
pub const ALERT_VARIANT_VALUES: &[&str] = &["default", "destructive"];

/// Valid badge variants (from base.ts: BadgeVariant)
pub const BADGE_VARIANT_VALUES: &[&str] = &[
    "default", "secondary", "destructive", "outline"
];

/// Valid text variants (from base.ts: TextVariant)
pub const TEXT_VARIANT_VALUES: &[&str] = &["body", "caption", "label", "code", "muted"];

/// Valid heading levels (from base.ts: HeadingLevel)
pub const HEADING_LEVEL_VALUES: &[&str] = &["1", "2", "3", "4", "5", "6"];

/// Valid input types (from base.ts: InputType)
pub const INPUT_TYPE_VALUES: &[&str] = &[
    "text", "password", "email", "number", "tel", "url",
    "date", "time", "datetime-local", "textarea", "checkbox",
    "radio", "select", "file", "hidden", "switch"
];

/// Valid flex directions (from base.ts: FlexDirection)
pub const FLEX_DIRECTION_VALUES: &[&str] = &[
    "row", "column", "row-reverse", "column-reverse"
];

/// Valid flex justify values (from base.ts: FlexJustify)
pub const FLEX_JUSTIFY_VALUES: &[&str] = &[
    "start", "end", "center", "between", "around", "evenly"
];

/// Valid flex align values (from base.ts: FlexAlign)
pub const FLEX_ALIGN_VALUES: &[&str] = &[
    "start", "end", "center", "stretch", "baseline"
];

/// Valid image fit values (from ImageSchema)
pub const IMAGE_FIT_VALUES: &[&str] = &[
    "contain", "cover", "fill", "none", "scale-down"
];

/// Valid image loading values
pub const IMAGE_LOADING_VALUES: &[&str] = &["lazy", "eager"];

/// Valid tab orientation values
pub const ORIENTATION_VALUES: &[&str] = &["horizontal", "vertical"];

/// Valid accordion type values
pub const ACCORDION_TYPE_VALUES: &[&str] = &["single", "multiple"];

/// Valid modal size values
pub const MODAL_SIZE_VALUES: &[&str] = &["sm", "md", "lg", "xl", "full"];

/// Valid dropdown align values
pub const DROPDOWN_ALIGN_VALUES: &[&str] = &["start", "center", "end"];

/// Valid tooltip side values
pub const TOOLTIP_SIDE_VALUES: &[&str] = &["top", "bottom", "left", "right"];

/// Valid skeleton variants
pub const SKELETON_VARIANT_VALUES: &[&str] = &["text", "circular", "rectangular"];

/// Valid divider orientations
pub const DIVIDER_ORIENTATION_VALUES: &[&str] = &["horizontal", "vertical"];

/// Valid stat card change types
pub const STAT_CHANGE_TYPE_VALUES: &[&str] = &["increase", "decrease", "neutral"];

/// Valid chart types
pub const CHART_TYPE_VALUES: &[&str] = &[
    "line", "bar", "pie", "doughnut", "area", "scatter"
];

/// Valid form layout values
pub const FORM_LAYOUT_VALUES: &[&str] = &["vertical", "horizontal", "inline"];

/// Valid table column align values
pub const TABLE_ALIGN_VALUES: &[&str] = &["left", "center", "right"];

// =============================================================================
// COMPONENT DEFINITIONS
// =============================================================================

/// Returns all component definitions for the Orbis DSL.
///
/// This is the single source of truth for component schemas. The returned
/// definitions are used to:
/// - Generate Pest grammar rules
/// - Generate component documentation
/// - Power LSP autocomplete and validation
///
/// Components are organized by category:
/// - Layout: Container, Grid, Flex, Spacer, Divider
/// - Typography: Text, Heading
/// - Forms: Field, Form, Button, Dropdown
/// - Data Display: Card, Table, List, Badge, StatCard, DataDisplay
/// - Navigation: Link, Tabs, Accordion, Breadcrumb
/// - Feedback: Alert, Progress, LoadingOverlay, Skeleton, EmptyState
/// - Overlays: Modal, Tooltip, Dropdown
/// - Media: Image, Icon, Avatar, Chart
/// - Utility: Conditional, Loop, Slot, Fragment, Custom, Section, PageHeader
pub fn define_components() -> Vec<ComponentDef> {
    vec![
        // =====================================================================
        // LAYOUT COMPONENTS
        // =====================================================================
        
        ComponentDef {
            name:        "Container",
            description: "A generic container element for grouping and layout purposes. \
                         Renders as a <div> with optional styling.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("style", "Inline styles as key-value pairs"),
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
            description: "A CSS Grid-based layout component for creating responsive grid layouts. \
                         Supports responsive column counts via breakpoint object.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("columns", "Number of columns (number or responsive object {sm, md, lg, xl})"),
                AttributeDef::new("gap", "Gap between grid items (CSS value)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Flex",
            description: "A Flexbox-based layout component for flexible, responsive layouts. \
                         Provides full control over flex container properties.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("direction", "Flex direction", FLEX_DIRECTION_VALUES),
                AttributeDef::with_values("justify", "Justify content alignment", FLEX_JUSTIFY_VALUES),
                AttributeDef::with_values("align", "Align items", FLEX_ALIGN_VALUES),
                AttributeDef::new("gap", "Gap between flex items (CSS value)"),
                AttributeDef::new("wrap", "Whether to wrap items (boolean)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Spacer",
            description: "An invisible spacer component for adding consistent spacing between elements.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("size", "Spacing size", SIZE_VALUES),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Divider",
            description: "A visual separator line, optionally with a label in the middle.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("orientation", "Divider orientation", DIVIDER_ORIENTATION_VALUES),
                AttributeDef::new("label", "Optional label to show in the middle"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // TYPOGRAPHY COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Text",
            description: "A text display component for paragraphs and inline text. \
                         Supports interpolated expressions in content.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Text content (supports {expression} interpolation)"),
                AttributeDef::with_values("variant", "Text style variant", TEXT_VARIANT_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the text is clicked"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Heading",
            description: "A heading component for titles and section headers (h1-h6). \
                         Use level to control semantic importance.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("text", "Heading text content"),
                AttributeDef::with_values("level", "Heading level (1-6)", HEADING_LEVEL_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the heading is clicked"),
            ],
            deprecated:  None,
        },

        // =====================================================================
        // FORM COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Field",
            description: "A form input field supporting various input types. \
                         Use bindTo for two-way data binding with state.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier (required for forms)"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("name", "Field name for form submission (REQUIRED)"),
                AttributeDef::with_values("fieldType", "Input type", INPUT_TYPE_VALUES),
                AttributeDef::new("label", "Label text displayed above the field"),
                AttributeDef::new("placeholder", "Placeholder text when empty"),
                AttributeDef::new("description", "Help text displayed below the field"),
                AttributeDef::new("defaultValue", "Default value on mount"),
                AttributeDef::new("bindTo", "State path for two-way binding (e.g., state.email)"),
                AttributeDef::new("required", "Whether the field is required"),
                AttributeDef::new("disabled", "Whether the field is disabled"),
                AttributeDef::new("readOnly", "Whether the field is read-only"),
                AttributeDef::new("options", "Array of options for select/radio types"),
                AttributeDef::new("validation", "Validation rules object"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("change", "Triggered when value changes"),
                EventDef::new("focus", "Triggered when field gains focus"),
                EventDef::new("blur", "Triggered when field loses focus"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Form",
            description: "A form container that handles field grouping and submission. \
                         Use with Field components inside.",
            attributes:  vec![
                AttributeDef::new("id", "Unique form identifier (required)"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("fields", "Array of field definitions"),
                AttributeDef::with_values("layout", "Form layout style", FORM_LAYOUT_VALUES),
                AttributeDef::new("submitLabel", "Submit button text"),
                AttributeDef::new("cancelLabel", "Cancel button text"),
                AttributeDef::new("showReset", "Whether to show reset button"),
                AttributeDef::new("actions", "Array of form action definitions"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("submit", "Triggered when form is submitted"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Button",
            description: "A clickable button component with multiple variants and states.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("label", "Button text label (REQUIRED)"),
                AttributeDef::with_values("variant", "Visual variant", BUTTON_VARIANT_VALUES),
                AttributeDef::with_values("size", "Button size", SIZE_VALUES),
                AttributeDef::new("disabled", "Whether the button is disabled"),
                AttributeDef::new("loading", "Whether to show loading state"),
                AttributeDef::new("icon", "Icon name to display"),
                AttributeDef::with_values("iconPosition", "Icon position", &["left", "right"]),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the button is clicked"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Dropdown",
            description: "A dropdown menu triggered by a button or other element.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("trigger", "Component to use as trigger"),
                AttributeDef::new("items", "Array of dropdown item definitions"),
                AttributeDef::with_values("align", "Menu alignment", DROPDOWN_ALIGN_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // DATA DISPLAY COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Card",
            description: "A card container for grouping related content with optional header/footer.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Card title text"),
                AttributeDef::new("subtitle", "Card subtitle text"),
                AttributeDef::new("header", "Custom header component"),
                AttributeDef::new("content", "Main content component"),
                AttributeDef::new("footer", "Custom footer component"),
                AttributeDef::new("hoverable", "Whether to show hover effect"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the card is clicked"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Table",
            description: "A data table component for displaying tabular data with sorting, \
                         pagination, and row selection.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("columns", "Array of column definitions (REQUIRED)"),
                AttributeDef::required("dataSource", "State path to data array (REQUIRED)"),
                AttributeDef::new("rowKey", "Property to use as unique row key"),
                AttributeDef::new("pagination", "Pagination config (boolean or object)"),
                AttributeDef::new("selectable", "Selection mode (boolean, 'single', 'multiple')"),
                AttributeDef::new("sortable", "Whether columns are sortable"),
                AttributeDef::new("searchable", "Whether to show search input"),
                AttributeDef::new("emptyText", "Text to show when no data"),
                AttributeDef::new("loading", "Whether to show loading state"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("rowClick", "Triggered when a row is clicked"),
                EventDef::new("rowDoubleClick", "Triggered when a row is double-clicked"),
                EventDef::new("select", "Triggered when selection changes"),
                EventDef::new("pageChange", "Triggered when page changes"),
                EventDef::new("sortChange", "Triggered when sort changes"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "List",
            description: "A list component for displaying arrays of items with custom templates.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("dataSource", "State path to data array (REQUIRED)"),
                AttributeDef::new("itemTemplate", "Component template for each item"),
                AttributeDef::new("emptyTemplate", "Component to show when list is empty"),
                AttributeDef::new("emptyText", "Text to show when list is empty"),
                AttributeDef::new("loading", "Whether to show loading state"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("rowClick", "Triggered when an item is clicked"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Badge",
            description: "A small status indicator or label, typically used for counts or status.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("text", "Badge text content"),
                AttributeDef::with_values("variant", "Visual variant", BADGE_VARIANT_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "StatCard",
            description: "A statistics display card with value, label, and optional trend indicator. \
                         Commonly used in dashboards.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Stat label/title"),
                AttributeDef::new("value", "Main statistic value"),
                AttributeDef::new("change", "Change amount (e.g., '+5%')"),
                AttributeDef::with_values("changeType", "Type of change", STAT_CHANGE_TYPE_VALUES),
                AttributeDef::new("icon", "Icon name to display"),
                AttributeDef::new("description", "Additional description text"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "DataDisplay",
            description: "A labeled data display component for showing key-value pairs.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("label", "Label text"),
                AttributeDef::new("value", "Value to display"),
                AttributeDef::new("copyable", "Whether value can be copied"),
                AttributeDef::new("prefix", "Component to show before value"),
                AttributeDef::new("suffix", "Component to show after value"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // NAVIGATION COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Link",
            description: "A navigation link component for internal or external navigation.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("href", "URL to navigate to (REQUIRED)"),
                AttributeDef::new("text", "Link text content"),
                AttributeDef::new("external", "Whether link opens in new tab"),
                AttributeDef::new("icon", "Icon name to display"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Tabs",
            description: "A tabbed interface for switching between content panels.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("items", "Array of tab item definitions"),
                AttributeDef::new("defaultTab", "Key of initially active tab"),
                AttributeDef::with_values("orientation", "Tab orientation", ORIENTATION_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("change", "Triggered when active tab changes"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Accordion",
            description: "An accordion component for expandable/collapsible content sections.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("items", "Array of accordion item definitions"),
                AttributeDef::with_values("type", "Expansion type (single/multiple)", ACCORDION_TYPE_VALUES),
                AttributeDef::new("defaultOpen", "Array of initially open item keys"),
                AttributeDef::new("collapsible", "Whether items can be collapsed"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Breadcrumb",
            description: "A breadcrumb navigation component showing the current page hierarchy.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("items", "Array of breadcrumb items (label, href, icon)"),
                AttributeDef::new("separator", "Custom separator string"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // FEEDBACK COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Alert",
            description: "An alert/notification message component for displaying status messages.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("variant", "Alert variant/severity", ALERT_VARIANT_VALUES),
                AttributeDef::new("title", "Alert title"),
                AttributeDef::new("message", "Alert message content"),
                AttributeDef::new("dismissible", "Whether alert can be dismissed"),
                AttributeDef::new("icon", "Custom icon name"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("close", "Triggered when the alert is dismissed"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Progress",
            description: "A progress indicator component for showing completion status.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("value", "Current progress value (0-100 or 0-max)"),
                AttributeDef::new("max", "Maximum value (default 100)"),
                AttributeDef::new("showLabel", "Whether to show percentage label"),
                AttributeDef::with_values("size", "Progress bar size", SIZE_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "LoadingOverlay",
            description: "A loading overlay that covers its container with a spinner and message.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("loading", "Expression controlling loading state"),
                AttributeDef::new("text", "Loading message to display"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Skeleton",
            description: "A skeleton loading placeholder that mimics content shape while loading.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("width", "Skeleton width (CSS value)"),
                AttributeDef::new("height", "Skeleton height (CSS value)"),
                AttributeDef::with_values("variant", "Skeleton shape variant", SKELETON_VARIANT_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "EmptyState",
            description: "An empty state placeholder with icon, message, and optional action.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Empty state title"),
                AttributeDef::new("description", "Empty state description"),
                AttributeDef::new("icon", "Icon name to display"),
                AttributeDef::new("action", "Optional action button definition"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // OVERLAY COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Modal",
            description: "A modal dialog component for focused user interactions.",
            attributes:  vec![
                AttributeDef::new("id", "Unique modal identifier (required for open/close)"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Modal title"),
                AttributeDef::new("description", "Modal description/subtitle"),
                AttributeDef::new("content", "Main content component"),
                AttributeDef::new("footer", "Custom footer component"),
                AttributeDef::with_values("size", "Modal size", MODAL_SIZE_VALUES),
                AttributeDef::new("closable", "Whether modal can be closed"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("open", "Triggered when modal opens"),
                EventDef::new("close", "Triggered when modal closes"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Tooltip",
            description: "A tooltip component that appears on hover, providing additional context.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("content", "Tooltip content text"),
                AttributeDef::new("children", "Element that triggers the tooltip"),
                AttributeDef::with_values("side", "Tooltip position", TOOLTIP_SIDE_VALUES),
                AttributeDef::new("delayMs", "Delay before showing (milliseconds)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // MEDIA COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Image",
            description: "An image component with loading states and fallback support.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("src", "Image source URL (REQUIRED)"),
                AttributeDef::required("alt", "Alternative text for accessibility (REQUIRED)"),
                AttributeDef::new("width", "Image width (CSS value or number)"),
                AttributeDef::new("height", "Image height (CSS value or number)"),
                AttributeDef::with_values("fit", "Object-fit style", IMAGE_FIT_VALUES),
                AttributeDef::new("fallback", "Fallback image URL on error"),
                AttributeDef::with_values("loading", "Loading strategy", IMAGE_LOADING_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Icon",
            description: "An icon component using lucide-react icons.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::required("name", "Icon name from lucide-react (REQUIRED)"),
                AttributeDef::with_values("size", "Icon size", SIZE_VALUES),
                AttributeDef::new("color", "Icon color (CSS color value)"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![
                EventDef::new("click", "Triggered when the icon is clicked"),
            ],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Avatar",
            description: "An avatar component for displaying user profile images with fallback.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("src", "Image source URL"),
                AttributeDef::new("alt", "Alternative text for accessibility"),
                AttributeDef::new("fallback", "Fallback text/initials when no image"),
                AttributeDef::with_values("size", "Avatar size", SIZE_VALUES),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Chart",
            description: "A charting component for data visualization.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::with_values("chartType", "Type of chart", CHART_TYPE_VALUES),
                AttributeDef::new("dataSource", "State path to chart data"),
                AttributeDef::new("options", "Chart configuration options"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        // =====================================================================
        // UTILITY COMPONENTS
        // =====================================================================

        ComponentDef {
            name:        "Conditional",
            description: "A component for conditional rendering based on an expression. \
                         Prefer using `if` syntax in templates instead.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("condition", "Boolean expression to evaluate"),
                AttributeDef::new("then", "Component to render when condition is true"),
                AttributeDef::new("else", "Component to render when condition is false"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Loop",
            description: "A component for rendering a list of items from a data source. \
                         Prefer using `for` syntax in templates instead.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("dataSource", "State path to array data"),
                AttributeDef::new("itemVar", "Variable name for current item (default: 'item')"),
                AttributeDef::new("indexVar", "Variable name for current index (default: 'index')"),
                AttributeDef::new("template", "Component template for each item"),
                AttributeDef::new("emptyTemplate", "Component to render when array is empty"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Slot",
            description: "A slot component for content projection in fragment compositions.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("name", "Slot name for named slots"),
                AttributeDef::new("fallback", "Fallback content when slot is empty"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Fragment",
            description: "An invisible wrapper for grouping multiple elements without adding DOM nodes.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Custom",
            description: "A component for rendering plugin-defined custom components.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("component", "Name of the custom component to render"),
                AttributeDef::new("props", "Props to pass to the custom component"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "Section",
            description: "A semantic section component for grouping content with an optional title.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Section title"),
                AttributeDef::new("description", "Section description"),
                AttributeDef::new("collapsible", "Whether section can be collapsed"),
                AttributeDef::new("defaultCollapsed", "Whether section starts collapsed"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },

        ComponentDef {
            name:        "PageHeader",
            description: "A page header component with title, subtitle, breadcrumb, and actions.",
            attributes:  vec![
                AttributeDef::new("id", "Unique identifier for the element"),
                AttributeDef::new("className", "CSS class name(s) for styling"),
                AttributeDef::new("title", "Page title"),
                AttributeDef::new("subtitle", "Page subtitle"),
                AttributeDef::new("breadcrumb", "Array of breadcrumb items"),
                AttributeDef::new("actions", "Array of action button definitions"),
                AttributeDef::new("backLink", "URL for back navigation link"),
                AttributeDef::new("visible", "Expression controlling visibility"),
            ],
            events:      vec![],
            deprecated:  None,
        },
    ]
}
