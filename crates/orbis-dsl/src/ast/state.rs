//! State declaration types with validation support
//!
//! This module defines types for state declarations, including
//! regular state, computed state, and validated state with Zod-compatible validators.

use serde::{Deserialize, Serialize};

use super::expr::{ArrowFunction, Expression};
use super::node::Span;
use super::types::TypeAnnotation;

/// State declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "declaration_type", rename_all = "snake_case")]
pub enum StateDeclaration {
    /// Regular state: name: Type = value
    Regular(RegularState),
    /// Computed state: name => expression
    Computed(ComputedState),
    /// Validated state: name: Type = value @validators
    Validated(ValidatedState),
}

impl StateDeclaration {
    /// Get the name of this declaration
    pub fn name(&self) -> &str {
        match self {
            StateDeclaration::Regular(s) => &s.name,
            StateDeclaration::Computed(s) => &s.name,
            StateDeclaration::Validated(s) => &s.name,
        }
    }

    /// Get the span of this declaration
    pub fn span(&self) -> &Span {
        match self {
            StateDeclaration::Regular(s) => &s.span,
            StateDeclaration::Computed(s) => &s.span,
            StateDeclaration::Validated(s) => &s.span,
        }
    }

    /// Check if this state is optional
    pub fn is_optional(&self) -> bool {
        match self {
            StateDeclaration::Regular(s) => s.optional,
            StateDeclaration::Computed(_) => false,
            StateDeclaration::Validated(s) => s.optional,
        }
    }

    /// Get type annotation if present
    pub fn type_annotation(&self) -> Option<&TypeAnnotation> {
        match self {
            StateDeclaration::Regular(s) => s.type_annotation.as_ref(),
            StateDeclaration::Computed(s) => s.type_annotation.as_ref(),
            StateDeclaration::Validated(s) => s.type_annotation.as_ref(),
        }
    }
}

/// Regular state declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularState {
    /// State property name
    pub name: String,

    /// Whether this is optional (name?)
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub optional: bool,

    /// Type annotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<TypeAnnotation>,

    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Expression>,

    /// Documentation comment from preceding /** */ or // comment
    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    pub doc_comment: Option<String>,

    pub span: Span,
}

/// Computed state declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedState {
    /// State property name
    pub name: String,

    /// Whether the @computed prefix was used
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub explicit_computed: bool,

    /// Type annotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<TypeAnnotation>,

    /// Computed expression
    pub expression: Expression,

    /// Documentation comment from preceding /** */ or // comment
    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    pub doc_comment: Option<String>,

    pub span: Span,
}

/// Validated state declaration (state with Zod-compatible validators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedState {
    /// State property name
    pub name: String,

    /// Whether this is optional (name?)
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub optional: bool,

    /// Type annotation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_annotation: Option<TypeAnnotation>,

    /// Initial value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Expression>,

    /// Validation chain (flattened as attributes)
    pub validators: Vec<Validator>,

    /// Documentation comment from preceding /** */ or // comment
    #[serde(skip_serializing_if = "Option::is_none", skip_deserializing)]
    pub doc_comment: Option<String>,

    pub span: Span,
}

impl ValidatedState {
    /// Get a validator by name
    pub fn get_validator(&self, name: &str) -> Option<&Validator> {
        self.validators.iter().find(|v| v.name.eq_ignore_ascii_case(name))
    }

    /// Check if this state has a specific validator
    pub fn has_validator(&self, name: &str) -> bool {
        self.get_validator(name).is_some()
    }

    /// Get the custom error message if present
    pub fn error_message(&self) -> Option<&str> {
        self.get_validator("message").and_then(|v| {
            v.args.first().and_then(|arg| match arg {
                ValidatorArg::String(s) => Some(s.as_str()),
                _ => None,
            })
        })
    }
}

/// Validator definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator name (e.g., "email", "min", "max", "regex")
    pub name: String,

    /// Validator category (derived from name, for filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ValidatorCategory>,

    /// Validator arguments
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub args: Vec<ValidatorArg>,

    pub span: Span,
}

impl Validator {
    /// Create a validator without arguments
    pub fn simple(name: String, span: Span) -> Self {
        Self {
            category: ValidatorCategory::from_name(&name),
            name,
            args: vec![],
            span,
        }
    }

    /// Create a validator with arguments
    pub fn with_args(name: String, args: Vec<ValidatorArg>, span: Span) -> Self {
        Self {
            category: ValidatorCategory::from_name(&name),
            name,
            args,
            span,
        }
    }

    /// Check if this is a transform validator
    pub fn is_transform(&self) -> bool {
        matches!(
            self.category,
            Some(ValidatorCategory::Transform)
        )
    }

    /// Check if this is a format validator
    pub fn is_format(&self) -> bool {
        matches!(
            self.category,
            Some(ValidatorCategory::Format)
        )
    }
}

/// Validator argument
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "arg_type", content = "value", rename_all = "snake_case")]
pub enum ValidatorArg {
    /// String argument
    String(String),
    /// Number argument
    Number(f64),
    /// Boolean argument
    Boolean(bool),
    /// Regex pattern
    Regex { pattern: String, flags: Option<String> },
    /// Arrow function (for transform/refine)
    Function(ArrowFunction),
    /// Object literal (for error maps, etc.)
    Object(Vec<(String, ValidatorArg)>),
    /// Identifier reference
    Identifier(String),
}

impl ValidatorArg {
    /// Get as number if this is a number argument
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ValidatorArg::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string if this is a string argument
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ValidatorArg::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as regex if this is a regex argument
    pub fn as_regex(&self) -> Option<(&str, Option<&str>)> {
        match self {
            ValidatorArg::Regex { pattern, flags } => Some((pattern, flags.as_deref())),
            _ => None,
        }
    }
}

/// Validator category for filtering and grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorCategory {
    /// Primitive type validators (string, number, boolean, etc.)
    Type,
    /// Format validators (email, url, uuid, etc.)
    Format,
    /// String-specific validators (min, max, length, etc.)
    String,
    /// Number-specific validators (gt, gte, lt, lte, int, etc.)
    Number,
    /// Array-specific validators (nonempty, unique, etc.)
    Array,
    /// Object-specific validators (strict, passthrough, etc.)
    Object,
    /// Transform validators (trim, toLowerCase, etc.)
    Transform,
    /// Refinement validators (refine, superRefine)
    Refinement,
    /// Meta validators (message, errorMap, optional, etc.)
    Meta,
    /// Coercion validators (coerceString, coerceNumber, etc.)
    Coercion,
}

impl ValidatorCategory {
    /// Determine category from validator name
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        match lower.as_str() {
            // Types
            "string" | "number" | "boolean" | "bigint" | "symbol" | "null" | "date" | "enum"
            | "stringbool" | "stringboolean" | "any" | "unknown" | "never" | "object" | "array"
            | "tuple" | "union" | "xor" | "intersection" | "record" | "map" | "set" | "file" => {
                Some(ValidatorCategory::Type)
            }

            // Formats
            "email" | "uuid" | "url" | "httpurl" | "hostname" | "emoji" | "base64" | "base64url"
            | "hex" | "jwt" | "nanoid" | "cuid" | "cuid2" | "ulid" | "ipv4" | "ipv6" | "mac"
            | "cidrv4" | "cidrv6" | "hash" | "isodate" | "isotime" | "isodatetime"
            | "isoduration" | "creditcard" | "iban" | "bic" | "postalcode" | "ip" | "cidr" => {
                Some(ValidatorCategory::Format)
            }

            // String specifics
            "length" | "startswith" | "endswith" | "includes" | "regex" | "pattern" => {
                Some(ValidatorCategory::String)
            }

            // Number specifics
            "gt" | "gte" | "lt" | "lte" | "positive" | "nonnegative" | "negative" | "nonpositive"
            | "multipleof" | "nan" | "int" | "int32" | "finite" | "safe" => {
                Some(ValidatorCategory::Number)
            }

            // Array specifics
            "nonempty" | "unique" => Some(ValidatorCategory::Array),

            // Object specifics
            "strict" | "passthrough" | "strip" | "catchall" | "partial" => {
                Some(ValidatorCategory::Object)
            }

            // Transforms
            "trim" | "tolowercase" | "touppercase" | "lowercase" | "uppercase" | "normalize"
            | "transform" => Some(ValidatorCategory::Transform),

            // Refinements
            "refine" | "superrefine" | "pipe" | "brand" => Some(ValidatorCategory::Refinement),

            // Meta
            "required" | "optional" | "nullable" | "nullish" | "default" | "catch" | "readonly"
            | "message" | "errormap" => Some(ValidatorCategory::Meta),

            // Coercion
            "coercestring" | "coercenumber" | "coerceboolean" | "coercebigint" => {
                Some(ValidatorCategory::Coercion)
            }

            // Common to string/array (context-dependent)
            "min" | "max" => None, // Category depends on context

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_category() {
        assert_eq!(ValidatorCategory::from_name("email"), Some(ValidatorCategory::Format));
        assert_eq!(ValidatorCategory::from_name("EMAIL"), Some(ValidatorCategory::Format));
        assert_eq!(ValidatorCategory::from_name("trim"), Some(ValidatorCategory::Transform));
        assert_eq!(ValidatorCategory::from_name("int"), Some(ValidatorCategory::Number));
        assert_eq!(ValidatorCategory::from_name("min"), None); // Context-dependent
    }

    #[test]
    fn test_validated_state() {
        let state = ValidatedState {
            name: "email".to_string(),
            optional: false,
            type_annotation: None,
            value: None,
            validators: vec![
                Validator::simple("email".to_string(), Span::default()),
                Validator::with_args(
                    "message".to_string(),
                    vec![ValidatorArg::String("Invalid email".to_string())],
                    Span::default(),
                ),
            ],
            span: Span::default(),
        };

        assert!(state.has_validator("email"));
        assert!(state.has_validator("EMAIL")); // Case-insensitive
        assert_eq!(state.error_message(), Some("Invalid email"));
    }
}
