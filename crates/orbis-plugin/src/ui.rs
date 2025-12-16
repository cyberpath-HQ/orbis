//! UI component and page definitions for JSON-described GUI.

use serde::{Deserialize, Serialize};

/// Page definition for plugin UI.
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

    /// Page layout schema.
    pub layout: ComponentSchema,
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
        // Validate route
        if self.route.is_empty() {
            return Err(orbis_core::Error::plugin("Page route is required"));
        }

        if !self.route.starts_with('/') {
            return Err(orbis_core::Error::plugin("Page route must start with '/'"));
        }

        // Validate title
        if self.title.is_empty() {
            return Err(orbis_core::Error::plugin("Page title is required"));
        }

        // Validate layout
        self.layout.validate()?;

        Ok(())
    }

    /// Get the full route path with plugin prefix.
    #[must_use]
    pub fn full_route(&self, plugin_name: &str) -> String {
        format!("/plugins/{}{}", plugin_name, self.route)
    }
}

/// Component schema for JSON-described UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSchema {
    /// Component type.
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component properties.
    #[serde(default)]
    pub props: serde_json::Value,

    /// Child components.
    #[serde(default)]
    pub children: Vec<ComponentSchema>,

    /// Conditional rendering expression.
    #[serde(default)]
    pub condition: Option<String>,

    /// Data binding expression.
    #[serde(default)]
    pub bind: Option<String>,

    /// Event handlers.
    #[serde(default)]
    pub events: serde_json::Value,

    /// CSS styles.
    #[serde(default)]
    pub style: serde_json::Value,

    /// CSS class names.
    #[serde(default)]
    pub class_name: Option<String>,
}

impl ComponentSchema {
    /// Validate the component schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema is invalid.
    pub fn validate(&self) -> orbis_core::Result<()> {
        // Validate component type
        if self.component_type.is_empty() {
            return Err(orbis_core::Error::plugin("Component type is required"));
        }

        // Recursively validate children
        for child in &self.children {
            child.validate()?;
        }

        Ok(())
    }
}

/// Pre-defined UI components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum UiComponent {
    /// Container component.
    Container {
        children: Vec<ComponentSchema>,
        #[serde(default)]
        layout: ContainerLayout,
    },

    /// Card component.
    Card {
        title: Option<String>,
        children: Vec<ComponentSchema>,
    },

    /// Text display.
    Text {
        content: String,
        #[serde(default)]
        variant: TextVariant,
    },

    /// Heading.
    Heading {
        content: String,
        #[serde(default = "default_heading_level")]
        level: u8,
    },

    /// Button.
    Button {
        label: String,
        #[serde(default)]
        variant: ButtonVariant,
        #[serde(default)]
        on_click: Option<String>,
    },

    /// Text input.
    TextInput {
        name: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
    },

    /// Select dropdown.
    Select {
        name: String,
        options: Vec<SelectOption>,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        required: bool,
    },

    /// Checkbox.
    Checkbox {
        name: String,
        label: String,
        #[serde(default)]
        default_checked: bool,
    },

    /// Table.
    Table {
        columns: Vec<TableColumn>,
        data_source: String,
        #[serde(default)]
        pagination: bool,
    },

    /// Form container.
    Form {
        children: Vec<ComponentSchema>,
        on_submit: String,
        #[serde(default)]
        method: String,
    },

    /// Image.
    Image {
        src: String,
        #[serde(default)]
        alt: Option<String>,
    },

    /// Link.
    Link {
        href: String,
        label: String,
        #[serde(default)]
        external: bool,
    },

    /// Divider.
    Divider,

    /// Spacer.
    Spacer {
        #[serde(default = "default_spacer_size")]
        size: String,
    },

    /// Alert/notification.
    Alert {
        message: String,
        #[serde(default)]
        severity: AlertSeverity,
    },

    /// Loading spinner.
    Loading {
        #[serde(default)]
        text: Option<String>,
    },

    /// Tabs container.
    Tabs {
        items: Vec<TabItem>,
    },

    /// Modal dialog.
    Modal {
        title: String,
        children: Vec<ComponentSchema>,
        #[serde(default)]
        trigger: Option<String>,
    },

    /// Chart (data visualization).
    Chart {
        chart_type: ChartType,
        data_source: String,
        #[serde(default)]
        options: serde_json::Value,
    },

    /// Custom component (rendered by name).
    Custom {
        component_name: String,
        props: serde_json::Value,
    },
}

fn default_heading_level() -> u8 {
    1
}

fn default_spacer_size() -> String {
    "md".to_string()
}

/// Container layout options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerLayout {
    #[default]
    Vertical,
    Horizontal,
    Grid,
}

/// Text variant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextVariant {
    #[default]
    Body,
    Caption,
    Label,
    Code,
}

/// Button variant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

/// Select option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub disabled: bool,
}

/// Table column definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub header: String,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub width: Option<String>,
}

/// Tab item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItem {
    pub key: String,
    pub label: String,
    pub content: ComponentSchema,
}

/// Alert severity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Chart type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Doughnut,
    Area,
    Scatter,
}
