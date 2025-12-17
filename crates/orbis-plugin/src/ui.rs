//! Enhanced UI component and page definitions for JSON-described GUI.
//!
//! This module provides comprehensive types for defining dynamic UIs from JSON schemas,
//! supporting state management, event handling, and complex component compositions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// State Definition Types
// =============================================================================

/// State field type enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateFieldType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// A single state field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFieldDefinition {
    /// The type of the state field.
    #[serde(rename = "type")]
    pub field_type: StateFieldType,

    /// Default value for the field.
    #[serde(default)]
    pub default: Option<serde_json::Value>,

    /// Whether the field is nullable.
    #[serde(default)]
    pub nullable: bool,

    /// Description for documentation.
    #[serde(default)]
    pub description: Option<String>,
}

// =============================================================================
// Action Types
// =============================================================================

/// Action that can be executed in response to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Update state at a path.
    UpdateState {
        path: String,
        #[serde(default)]
        value: Option<serde_json::Value>,
        #[serde(default)]
        from: Option<String>,
        #[serde(default)]
        merge: bool,
    },

    /// Call a backend API.
    CallApi {
        #[serde(default)]
        name: Option<String>,
        api: String,
        #[serde(default)]
        method: Option<String>,
        #[serde(default)]
        args_from_state: Vec<String>,
        #[serde(default)]
        map_args: Vec<ArgMapping>,
        #[serde(default)]
        body: Option<serde_json::Value>,
        #[serde(default)]
        on_success: Vec<Action>,
        #[serde(default)]
        on_error: Vec<Action>,
        #[serde(default)]
        on_finally: Vec<Action>,
    },

    /// Navigate to a route.
    Navigate {
        to: String,
        #[serde(default)]
        replace: bool,
        #[serde(default)]
        params: HashMap<String, String>,
    },

    /// Show a toast notification.
    ShowToast {
        level: ToastLevel,
        message: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        duration: Option<u32>,
    },

    /// Show a dialog.
    ShowDialog {
        dialog_id: String,
        #[serde(default)]
        data: HashMap<String, String>,
    },

    /// Close a dialog.
    CloseDialog {
        #[serde(default)]
        dialog_id: Option<String>,
    },

    /// Debounced action execution.
    DebouncedAction {
        delay_ms: u32,
        action: Box<Action>,
        #[serde(default)]
        key: Option<String>,
    },

    /// Set loading state.
    SetLoading {
        loading: bool,
        #[serde(default)]
        target: Option<String>,
    },

    /// Conditional action.
    Conditional {
        condition: String,
        then: Vec<Action>,
        #[serde(default)]
        else_actions: Vec<Action>,
    },

    /// Sequence of actions.
    Sequence {
        actions: Vec<Action>,
        #[serde(default)]
        stop_on_error: bool,
    },

    /// Copy text to clipboard.
    Copy {
        text: String,
        #[serde(default)]
        show_notification: bool,
    },

    /// Open external URL.
    OpenUrl {
        url: String,
        #[serde(default)]
        new_tab: bool,
    },

    /// Emit custom event.
    Emit {
        event: String,
        #[serde(default)]
        payload: HashMap<String, serde_json::Value>,
    },
}

/// Argument mapping for API calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgMapping {
    pub from: String,
    pub to: String,
}

/// Toast notification level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

// =============================================================================
// Event Handler Types
// =============================================================================

/// Event handlers that can be attached to components.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EventHandlers {
    #[serde(default)]
    pub on_click: Vec<Action>,
    #[serde(default)]
    pub on_change: Vec<Action>,
    #[serde(default)]
    pub on_submit: Vec<Action>,
    #[serde(default)]
    pub on_focus: Vec<Action>,
    #[serde(default)]
    pub on_blur: Vec<Action>,
    #[serde(default)]
    pub on_row_click: Vec<Action>,
    #[serde(default)]
    pub on_select: Vec<Action>,
    #[serde(default)]
    pub on_page_change: Vec<Action>,
    #[serde(default)]
    pub on_sort_change: Vec<Action>,
    #[serde(default)]
    pub on_close: Vec<Action>,
    #[serde(default)]
    pub on_open: Vec<Action>,
}

// =============================================================================
// Component Schema Types
// =============================================================================

/// Enhanced component schema for JSON-described UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    /// Component type (e.g., "Container", "Button", "Form").
    #[serde(rename = "type")]
    pub component_type: String,

    /// Unique identifier for the component.
    #[serde(default)]
    pub id: Option<String>,

    /// CSS class names.
    #[serde(default, rename = "className")]
    pub class_name: Option<String>,

    /// Inline CSS styles.
    #[serde(default)]
    pub style: Option<serde_json::Value>,

    /// Visibility condition (boolean or expression).
    #[serde(default)]
    pub visible: Option<serde_json::Value>,

    /// Child components.
    #[serde(default)]
    pub children: Vec<ComponentSchema>,

    /// Event handlers.
    #[serde(default)]
    pub events: Option<EventHandlers>,

    /// All other component-specific properties.
    #[serde(flatten)]
    pub props: HashMap<String, serde_json::Value>,
}

impl ComponentSchema {
    /// Create a new component schema.
    #[must_use]
    pub fn new(component_type: &str) -> Self {
        Self {
            component_type: component_type.to_string(),
            id: None,
            class_name: None,
            style: None,
            visible: None,
            children: Vec::new(),
            events: None,
            props: HashMap::new(),
        }
    }

    /// Set the component ID.
    #[must_use]
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Add a child component.
    #[must_use]
    pub fn with_child(mut self, child: ComponentSchema) -> Self {
        self.children.push(child);
        self
    }

    /// Set a property.
    #[must_use]
    pub fn with_prop(mut self, key: &str, value: serde_json::Value) -> Self {
        self.props.insert(key.to_string(), value);
        self
    }

    /// Validate the component schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        if self.component_type.is_empty() {
            return Err(orbis_core::Error::plugin("Component type is required"));
        }

        for child in &self.children {
            child.validate()?;
        }

        Ok(())
    }
}

// =============================================================================
// Page Definition Types
// =============================================================================

/// Dialog definition for modals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogDefinition {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub content: ComponentSchema,
    #[serde(default)]
    pub footer: Option<ComponentSchema>,
    #[serde(default)]
    pub size: Option<String>,
}

/// Page lifecycle hooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PageLifecycleHooks {
    #[serde(default)]
    pub on_mount: Vec<Action>,
    #[serde(default)]
    pub on_unmount: Vec<Action>,
    #[serde(default)]
    pub on_params_change: Vec<Action>,
    #[serde(default)]
    pub on_query_change: Vec<Action>,
}

/// Enhanced page definition for plugin UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDefinition {
    /// Route path for the page.
    pub route: String,

    /// Page title.
    pub title: String,

    /// Icon name (from icon library).
    #[serde(default)]
    pub icon: Option<String>,

    /// Page description.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether to show in navigation menu.
    #[serde(default = "default_true")]
    pub show_in_menu: bool,

    /// Menu order (lower = higher priority).
    #[serde(default)]
    pub menu_order: i32,

    /// Parent route (for nested pages).
    #[serde(default)]
    pub parent_route: Option<String>,

    /// Whether authentication is required.
    #[serde(default = "default_true")]
    pub requires_auth: bool,

    /// Required permissions to view page.
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Required roles to view page.
    #[serde(default)]
    pub roles: Vec<String>,

    /// Page-level state definition.
    #[serde(default)]
    pub state: HashMap<String, StateFieldDefinition>,

    /// Computed values derived from state.
    #[serde(default)]
    pub computed: HashMap<String, String>,

    /// Page sections/content.
    pub sections: Vec<ComponentSchema>,

    /// Page-level action definitions.
    #[serde(default)]
    pub actions: HashMap<String, Action>,

    /// Page lifecycle hooks.
    #[serde(default)]
    pub hooks: Option<PageLifecycleHooks>,

    /// Dialog definitions.
    #[serde(default)]
    pub dialogs: Vec<DialogDefinition>,
}

fn default_true() -> bool {
    true
}

impl PageDefinition {
    /// Validate the page definition.
    ///
    /// # Errors
    ///
    /// Returns an error if the page is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        if self.route.is_empty() {
            return Err(orbis_core::Error::plugin("Page route is required"));
        }

        if !self.route.starts_with('/') {
            return Err(orbis_core::Error::plugin("Page route must start with '/'"));
        }

        if self.title.is_empty() {
            return Err(orbis_core::Error::plugin("Page title is required"));
        }

        for section in &self.sections {
            section.validate()?;
        }

        Ok(())
    }

    /// Get the full route path with plugin prefix.
    #[must_use]
    pub fn full_route(&self, plugin_name: &str) -> String {
        format!("/plugins/{}{}", plugin_name, self.route)
    }
}

// =============================================================================
// Navigation Types
// =============================================================================

/// Navigation menu item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub external: bool,
    #[serde(default)]
    pub children: Vec<NavigationItem>,
    #[serde(default)]
    pub badge: Option<String>,
    #[serde(default)]
    pub badge_variant: Option<String>,
    #[serde(default)]
    pub visible: Option<serde_json::Value>,
    #[serde(default)]
    pub disabled: Option<serde_json::Value>,
}

/// Navigation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationConfig {
    #[serde(default)]
    pub primary: Vec<NavigationItem>,
    #[serde(default)]
    pub secondary: Vec<NavigationItem>,
    #[serde(default)]
    pub user: Vec<NavigationItem>,
    #[serde(default)]
    pub footer: Vec<NavigationItem>,
}

// =============================================================================
// Helper Types for Common Patterns
// =============================================================================

/// Table column definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub width: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub render: Option<ComponentSchema>,
}

/// Form field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub name: String,
    pub field_type: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    #[serde(default)]
    pub bind_to: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub disabled: Option<serde_json::Value>,
    #[serde(default)]
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub validation: Option<ValidationRule>,
    #[serde(default)]
    pub events: Option<EventHandlers>,
}

/// Select option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub disabled: bool,
}

/// Validation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    #[serde(default)]
    pub required: Option<serde_json::Value>,
    #[serde(default)]
    pub min: Option<serde_json::Value>,
    #[serde(default)]
    pub max: Option<serde_json::Value>,
    #[serde(default)]
    pub min_length: Option<serde_json::Value>,
    #[serde(default)]
    pub max_length: Option<serde_json::Value>,
    #[serde(default)]
    pub pattern: Option<serde_json::Value>,
    #[serde(default)]
    pub email: Option<serde_json::Value>,
    #[serde(default)]
    pub url: Option<serde_json::Value>,
    #[serde(default)]
    pub custom: Option<CustomValidation>,
}

/// Custom validation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidation {
    pub expression: String,
    pub message: String,
}

/// Tab item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub disabled: Option<serde_json::Value>,
    pub content: ComponentSchema,
}

/// Accordion item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordionItem {
    pub key: String,
    pub title: String,
    pub content: ComponentSchema,
    #[serde(default)]
    pub disabled: Option<serde_json::Value>,
}

/// Breadcrumb item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    pub label: String,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_definition_serialization() {
        let page = PageDefinition {
            route: "/users".to_string(),
            title: "User Management".to_string(),
            icon: Some("Users".to_string()),
            description: Some("Manage system users".to_string()),
            show_in_menu: true,
            menu_order: 1,
            parent_route: None,
            requires_auth: true,
            permissions: vec!["users.read".to_string()],
            roles: vec![],
            state: {
                let mut map = HashMap::new();
                map.insert(
                    "users".to_string(),
                    StateFieldDefinition {
                        field_type: StateFieldType::Array,
                        default: Some(serde_json::json!([])),
                        nullable: false,
                        description: None,
                    },
                );
                map.insert(
                    "loading".to_string(),
                    StateFieldDefinition {
                        field_type: StateFieldType::Boolean,
                        default: Some(serde_json::json!(false)),
                        nullable: false,
                        description: None,
                    },
                );
                map
            },
            computed: HashMap::new(),
            sections: vec![ComponentSchema::new("Container").with_id("main")],
            actions: HashMap::new(),
            hooks: None,
            dialogs: vec![],
        };

        let json = serde_json::to_string_pretty(&page).unwrap();
        println!("{}", json);

        let parsed: PageDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.route, "/users");
        assert_eq!(parsed.title, "User Management");
    }

    #[test]
    fn test_complex_page_deserialization() {
        let json = r#"{
            "route": "/users",
            "title": "User Management",
            "state": {
                "filters": { "type": "object", "default": { "search": "" } },
                "users": { "type": "array", "default": [] },
                "loading": { "type": "boolean", "default": false }
            },
            "sections": [
                {
                    "type": "Form",
                    "id": "filterForm",
                    "fields": [
                        {
                            "id": "search",
                            "label": "Search",
                            "field_type": "text",
                            "bind_to": "filters.search"
                        }
                    ]
                },
                {
                    "type": "Table",
                    "id": "userTable",
                    "columns": [
                        { "key": "id", "label": "ID" },
                        { "key": "email", "label": "Email" }
                    ],
                    "dataSource": "state:users"
                }
            ]
        }"#;

        let page: PageDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(page.route, "/users");
        assert_eq!(page.sections.len(), 2);
        assert!(page.state.contains_key("users"));
    }
}
