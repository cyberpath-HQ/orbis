//! Component and Fragment types for the Orbis DSL
//!
//! This module defines types for components (built-in UI elements)
//! and fragments (user-defined reusable compositions).

use serde::{Deserialize, Serialize};

use super::expr::{ArrowFunction, Expression};
use super::node::{Span, TemplateContent};
use super::types::TypeAnnotation;

// ============================================================================
// COMPONENT
// ============================================================================

/// A component instance in the template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component name (e.g., "Container", "Button", "Card")
    pub name: String,

    /// Whether this is a self-closing component (<Component />)
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub self_closing: bool,

    /// Component attributes (flat structure)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attributes: Vec<Attribute>,

    /// Event bindings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<EventBinding>,

    /// Child content (for non-self-closing components)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<TemplateContent>,

    /// Slot attribute for named slot assignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,

    pub span: Span,
}

impl Component {
    /// Get an attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    /// Get an event binding by name
    pub fn get_event(&self, name: &str) -> Option<&EventBinding> {
        self.events.iter().find(|e| e.event.eq_ignore_ascii_case(name))
    }

    /// Check if this component has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the className attribute value if present
    pub fn class_name(&self) -> Option<&AttributeValue> {
        self.get_attribute("className").or_else(|| self.get_attribute("class")).map(|a| &a.value)
    }
}

/// Component attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute name (normalized to camelCase)
    pub name: String,
    /// Attribute value
    pub value: AttributeValue,
    /// Original attribute name as written
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    pub span: Span,
}

/// Attribute value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "value_type", rename_all = "snake_case")]
pub enum AttributeValue {
    /// String literal value
    String { value: String },
    /// Expression value (wrapped in {})
    Expression { value: Expression },
}

impl AttributeValue {
    /// Check if this is a static string value
    pub fn is_static(&self) -> bool {
        matches!(self, AttributeValue::String { .. })
    }

    /// Get as string if this is a static string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String { value } => Some(value),
            _ => None,
        }
    }

    /// Get as expression if this is an expression
    pub fn as_expression(&self) -> Option<&Expression> {
        match self {
            AttributeValue::Expression { value } => Some(value),
            _ => None,
        }
    }
}

/// Event binding on a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBinding {
    /// Event name (e.g., "click", "submit", "change")
    pub event: String,
    /// Event handler
    pub handler: EventHandler,
    pub span: Span,
}

/// Event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    /// Handler type (expression, identifier, or arrow function)
    pub handler_type: HandlerType,
    /// Event modifiers (e.g., stop, prevent, once)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub modifiers: Vec<String>,
}

/// Handler type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum HandlerType {
    /// Expression reference: @click={handler}
    Expression(Expression),
    /// Identifier reference: @click={handleClick}
    Identifier(String),
    /// Arrow function: @click={() => doSomething()}
    Arrow(ArrowFunction),
}

// ============================================================================
// FRAGMENT DEFINITION
// ============================================================================

/// Fragment definition (reusable component composition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentDefinition {
    /// Fragment name (PascalCase)
    pub name: String,

    /// Parameters
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub params: Vec<FragmentParam>,

    /// Fragment body (template content)
    pub body: Vec<TemplateContent>,

    /// Whether this fragment is exported
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub exported: bool,

    /// Whether this is the default export
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_default: bool,

    pub span: Span,
}

impl FragmentDefinition {
    /// Get a parameter by name
    pub fn get_param(&self, name: &str) -> Option<&FragmentParam> {
        self.params.iter().find(|p| p.name == name)
    }

    /// Get all required parameters
    pub fn required_params(&self) -> impl Iterator<Item = &FragmentParam> {
        self.params.iter().filter(|p| p.required)
    }

    /// Get all optional parameters
    pub fn optional_params(&self) -> impl Iterator<Item = &FragmentParam> {
        self.params.iter().filter(|p| !p.required)
    }
}

/// Fragment parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentParam {
    /// Parameter name
    pub name: String,
    /// Type annotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<TypeAnnotation>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Expression>,
    /// Whether this parameter is required
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub required: bool,
    pub span: Span,
}

// ============================================================================
// FRAGMENT USAGE
// ============================================================================

/// Fragment usage (instantiation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentUsage {
    /// Fragment name being used
    pub name: String,

    /// Property bindings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<Attribute>,

    /// Event bindings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<EventBinding>,

    /// Slot content (for fragments with slots)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub slot_content: Vec<SlotContent>,

    /// Whether this is self-closing
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub self_closing: bool,

    pub span: Span,
}

/// Content for a slot in fragment usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotContent {
    /// Slot name (None for default slot)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Content to project into the slot
    pub content: Vec<TemplateContent>,
    pub span: Span,
}

// ============================================================================
// SLOT DEFINITION
// ============================================================================

/// Slot definition inside a fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDefinition {
    /// Slot name (None for default slot)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fallback content
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fallback: Vec<TemplateContent>,
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_attribute_lookup() {
        let component = Component {
            name: "Button".to_string(),
            self_closing: true,
            attributes: vec![
                Attribute {
                    name: "className".to_string(),
                    value: AttributeValue::String { value: "btn".to_string() },
                    original_name: None,
                    span: Span::default(),
                },
                Attribute {
                    name: "label".to_string(),
                    value: AttributeValue::String { value: "Click".to_string() },
                    original_name: None,
                    span: Span::default(),
                },
            ],
            events: vec![],
            children: vec![],
            slot: None,
            span: Span::default(),
        };

        assert!(component.get_attribute("className").is_some());
        assert!(component.get_attribute("ClassName").is_some()); // case-insensitive
        assert!(component.get_attribute("label").is_some());
        assert!(component.get_attribute("nonexistent").is_none());
    }

    #[test]
    fn test_attribute_value_methods() {
        let string_val = AttributeValue::String { value: "test".to_string() };
        assert!(string_val.is_static());
        assert_eq!(string_val.as_string(), Some("test"));
        assert!(string_val.as_expression().is_none());
    }
}
