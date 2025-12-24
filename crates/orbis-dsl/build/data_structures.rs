// =============================================================================
// ORBIS DSL BUILD SYSTEM - Core Data Structures
// =============================================================================
//
// This module defines the core types used throughout the build system to
// represent components, attributes, events, and their metadata.
//
// These structures are the SOURCE OF TRUTH for:
// - Pest grammar generation (component rules, attribute/event whitelisting)
// - Documentation generation (COMPONENT_REFERENCE.md)
// - LSP features (autocomplete, hover documentation)
//
// SYNCHRONIZATION: These definitions MUST match the TypeScript schemas in
// `orbis/src/types/schema/` (components.ts, actions.ts, base.ts, page.ts).
//
// =============================================================================
//
// NOTE: This file is designed to be used with `include!` in build.rs.
// This file has NO external dependencies and should be included FIRST,
// before any other build modules.
//
// =============================================================================

/// Represents a UI component with its full schema definition.
///
/// Each ComponentDef generates:
/// - Grammar rules for the component tag (e.g., `<Button ... />`)
/// - Attribute whitelisting rules
/// - Event whitelisting rules
/// - Documentation entries
///
/// # Example
/// ```ignore
/// ComponentDef {
///     name: "Button",
///     description: "A clickable button component.",
///     attributes: vec![AttributeDef::new("label", "Button text")],
///     events: vec![EventDef::new("click", "Triggered when clicked")],
///     deprecated: None,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ComponentDef {
    /// Component name in PascalCase (e.g., "Button", "StatCard")
    pub name: &'static str,
    
    /// Human-readable description for documentation
    pub description: &'static str,
    
    /// Whitelisted attributes for this component
    pub attributes: Vec<AttributeDef>,
    
    /// Whitelisted events for this component
    pub events: Vec<EventDef>,
    
    /// Deprecation info if this component is deprecated
    pub deprecated: Option<DeprecationInfo>,
}

/// Represents an attribute that can be set on a component.
///
/// Attributes can be:
/// - **Expression-based**: Accept any valid expression (`className={state.class}`)
/// - **Strongly-typed**: Only accept specific literal values (`variant="primary"`)
/// - **Required**: Must be provided or a parse/validation error occurs
/// - **Optional**: Can be omitted (default behavior)
///
/// # Examples
/// ```ignore
/// // Optional expression-based attribute
/// AttributeDef::new("className", "CSS class name(s)")
///
/// // Required attribute
/// AttributeDef::required("label", "Button text - REQUIRED")
///
/// // Strongly-typed attribute with allowed values (optional by default)
/// AttributeDef::with_values("variant", "Visual variant", vec!["primary", "secondary"])
///
/// // Required strongly-typed attribute
/// AttributeDef::required_with_values("fieldType", "Input type", INPUT_TYPE_VALUES)
/// ```
#[derive(Debug, Clone)]
pub struct AttributeDef {
    /// Attribute name in camelCase (e.g., "className", "fieldType")
    pub name: &'static str,
    
    /// Human-readable description for documentation and LSP hover
    pub description: &'static str,
    
    /// If Some, only these literal values are allowed (generates strict validation)
    /// If None, any expression is accepted
    pub allowed_values: Option<Vec<&'static str>>,
    
    /// Whether this attribute is required (must be provided)
    pub required: bool,
    
    /// Deprecation info if this attribute is deprecated
    pub deprecated: Option<DeprecationInfo>,
}

/// Represents an event that can be handled on a component.
///
/// Events use the `@event => [actions]` syntax in the DSL:
/// ```orbis
/// <Button @click => [state.count = state.count + 1] />
/// ```
#[derive(Debug, Clone)]
pub struct EventDef {
    /// Event name in camelCase (e.g., "click", "onChange")
    pub name: &'static str,
    
    /// Human-readable description for documentation and LSP hover
    pub description: &'static str,
    
    /// Deprecation info if this event is deprecated
    pub deprecated: Option<DeprecationInfo>,
}

/// Deprecation metadata for components, attributes, or events.
///
/// Used to warn developers about deprecated features and guide migration.
#[derive(Debug, Clone)]
pub struct DeprecationInfo {
    /// Human-readable deprecation message explaining why it's deprecated
    pub message: &'static str,
    
    /// Suggested replacement (if any)
    pub alternative: Option<&'static str>,
}

// =============================================================================
// IMPLEMENTATION BLOCKS
// =============================================================================

impl AttributeDef {
    /// Create an optional attribute that accepts any expression value.
    ///
    /// Use this for attributes like `className`, `id`, `visible` that can
    /// accept dynamic expressions like `{state.value}` or string literals.
    ///
    /// # Example
    /// ```ignore
    /// AttributeDef::new("className", "CSS class name(s) for styling")
    /// ```
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            allowed_values: None,
            required: false,
            deprecated: None,
        }
    }

    /// Create a **required** attribute that accepts any expression value.
    ///
    /// Use this for attributes that MUST be provided, like `label` on Button.
    /// The grammar will enforce these at parse time.
    ///
    /// # Example
    /// ```ignore
    /// AttributeDef::required("label", "Button text (REQUIRED)")
    /// ```
    pub fn required(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            allowed_values: None,
            required: true,
            deprecated: None,
        }
    }

    /// Create an optional attribute with a whitelist of allowed literal values.
    ///
    /// Use this for enum-like attributes where only specific values are valid.
    /// The grammar will enforce these at parse time.
    ///
    /// # Example
    /// ```ignore
    /// AttributeDef::with_values(
    ///     "variant",
    ///     "Visual variant",
    ///     &["primary", "secondary", "outline", "ghost"]
    /// )
    /// ```
    pub fn with_values(name: &'static str, description: &'static str, values: &[&'static str]) -> Self {
        Self {
            name,
            description,
            allowed_values: Some(values.to_vec()),
            required: false,
            deprecated: None,
        }
    }

    /// Create a **required** attribute with a whitelist of allowed literal values.
    ///
    /// Use this for required enum-like attributes.
    ///
    /// # Example
    /// ```ignore
    /// AttributeDef::required_with_values("fieldType", "Input type (REQUIRED)", INPUT_TYPE_VALUES)
    /// ```
    pub fn required_with_values(name: &'static str, description: &'static str, values: &[&'static str]) -> Self {
        Self {
            name,
            description,
            allowed_values: Some(values.to_vec()),
            required: true,
            deprecated: None,
        }
    }

    /// Mark this attribute as deprecated with a message and optional alternative.
    ///
    /// # Example
    /// ```ignore
    /// AttributeDef::new("merge", "Merge with existing state")
    ///     .deprecated("Use mode='merge' instead", Some("mode"))
    /// ```
    #[allow(dead_code)]
    pub fn deprecated(mut self, message: &'static str, alternative: Option<&'static str>) -> Self {
        self.deprecated = Some(DeprecationInfo { message, alternative });
        self
    }
}

impl EventDef {
    /// Create a new event definition.
    ///
    /// # Example
    /// ```ignore
    /// EventDef::new("click", "Triggered when the element is clicked")
    /// ```
    pub fn new(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            deprecated: None,
        }
    }

    /// Mark this event as deprecated with a message and optional alternative.
    ///
    /// # Example
    /// ```ignore
    /// EventDef::new("onPress", "Legacy press event")
    ///     .deprecated("Use @click instead", Some("click"))
    /// ```
    #[allow(dead_code)]
    pub fn deprecated(mut self, message: &'static str, alternative: Option<&'static str>) -> Self {
        self.deprecated = Some(DeprecationInfo { message, alternative });
        self
    }
}

// =============================================================================
// FRAGMENT DEFINITIONS
// =============================================================================
// Fragments are reusable UI compositions that can be parameterized with
// properties (like function components in React). They support:
// - Properties (both required and optional)
// - Events that can be passed through to inner components
// - Named and unnamed slots for content projection
// =============================================================================

/// Represents a reusable fragment (component-like) definition.
///
/// Fragments allow defining reusable UI patterns within .orbis files:
/// ```orbis
/// fragment UserCard(user: User, @onClick) {
///     <Card className="user-card">
///         <Heading level="2" content={user.name} />
///         <slot />
///         <slot name="actions" />
///     </Card>
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FragmentDef {
    /// Fragment name in PascalCase (e.g., "UserCard", "Modal")
    pub name: &'static str,
    
    /// Human-readable description for documentation
    pub description: &'static str,
    
    /// Fragment properties (similar to component attributes)
    pub properties: Vec<FragmentPropertyDef>,
    
    /// Events that can be passed through (e.g., @onClick, @onSubmit)
    pub events: Vec<EventDef>,
    
    /// Named slots this fragment expects
    pub slots: Vec<SlotDef>,
}

/// Represents a property that can be passed to a fragment.
///
/// Properties can be:
/// - **Required**: Must be provided when using the fragment
/// - **Optional**: Can be omitted, may have a default value
/// - **Typed**: With type annotations and optional constraints
///
/// # Examples
/// ```orbis
/// fragment Card(
///     title: string,              // Required, any string
///     subtitle?: string,          // Optional string
///     size: "sm" | "md" | "lg",   // Required, union type
///     onClick?,                   // Optional event
/// ) { ... }
/// ```
#[derive(Debug, Clone)]
pub struct FragmentPropertyDef {
    /// Property name in camelCase
    pub name: &'static str,
    
    /// Human-readable description
    pub description: &'static str,
    
    /// Whether this property is required
    pub required: bool,
    
    /// Type annotation (if any)
    pub type_annotation: Option<&'static str>,
    
    /// If Some, only these literal values are allowed (union type)
    pub allowed_values: Option<Vec<&'static str>>,
    
    /// Default value expression (if optional)
    pub default_value: Option<&'static str>,
}

/// Represents a named slot in a fragment.
///
/// Slots allow content projection into fragments:
/// ```orbis
/// fragment Modal(title: string) {
///     <Container className="modal">
///         <Heading content={title} />
///         <slot />                    // Default/unnamed slot
///         <slot name="footer" />      // Named slot
///     </Container>
/// }
///
/// // Usage:
/// <Modal title="Confirm">
///     <Text content="Are you sure?" />
///     <Fragment slot="footer">
///         <Button label="Cancel" />
///         <Button label="Confirm" />
///     </Fragment>
/// </Modal>
/// ```
#[derive(Debug, Clone)]
pub struct SlotDef {
    /// Slot name (None for default/unnamed slot)
    pub name: Option<&'static str>,
    
    /// Human-readable description
    pub description: &'static str,
    
    /// Whether this slot is required to be filled
    pub required: bool,
}

impl FragmentPropertyDef {
    /// Create a required property
    pub fn required(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            required: true,
            type_annotation: None,
            allowed_values: None,
            default_value: None,
        }
    }
    
    /// Create an optional property
    #[allow(dead_code)]
    pub fn optional(name: &'static str, description: &'static str) -> Self {
        Self {
            name,
            description,
            required: false,
            type_annotation: None,
            allowed_values: None,
            default_value: None,
        }
    }
    
    /// Add type annotation
    #[allow(dead_code)]
    pub fn with_type(mut self, type_annotation: &'static str) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }
    
    /// Add allowed values (creates a union type)
    #[allow(dead_code)]
    pub fn with_values(mut self, values: &[&'static str]) -> Self {
        self.allowed_values = Some(values.to_vec());
        self
    }
    
    /// Add default value (makes it optional)
    #[allow(dead_code)]
    pub fn with_default(mut self, default: &'static str) -> Self {
        self.default_value = Some(default);
        self.required = false;
        self
    }
}

impl SlotDef {
    /// Create the default (unnamed) slot
    #[allow(dead_code)]
    pub fn default_slot(description: &'static str) -> Self {
        Self {
            name: None,
            description,
            required: false,
        }
    }
    
    /// Create a named slot
    #[allow(dead_code)]
    pub fn named(name: &'static str, description: &'static str) -> Self {
        Self {
            name: Some(name),
            description,
            required: false,
        }
    }
    
    /// Mark slot as required
    #[allow(dead_code)]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

// =============================================================================
// WATCHER DEFINITIONS
// =============================================================================
// Watchers observe state changes and trigger side effects.
// They are defined in the hooks block alongside lifecycle hooks.
// =============================================================================

/// Represents a watcher definition for reactive side effects.
///
/// Watchers trigger actions when observed state properties change:
/// ```orbis
/// hooks {
///     @mount => [...]
///     
///     @watch(state.count) => [
///         console.log("Count changed to: {state.count}")
///     ]
///     
///     @watch(state.searchQuery, debounce: 300) => [
///         api.call("search", query: state.searchQuery)
///     ]
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WatcherDef {
    /// State paths being watched (e.g., ["state.count", "state.name"])
    pub watched_paths: Vec<&'static str>,
    
    /// Optional debounce delay in milliseconds
    pub debounce_ms: Option<u32>,
    
    /// Human-readable description
    pub description: &'static str,
}
