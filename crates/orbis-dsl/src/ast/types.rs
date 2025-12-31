//! Type annotation system for the Orbis DSL
//!
//! This module defines the type system used in state declarations,
//! interface definitions, and fragment parameters.

use serde::{Deserialize, Serialize};

use super::node::Span;

/// Type annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type_kind", rename_all = "snake_case")]
pub enum TypeAnnotation {
    /// Primitive type (string, number, bool, etc.)
    Primitive(PrimitiveType),
    /// Named type (custom interface or type reference)
    Named {
        name: String,
        span: Span,
    },
    /// Optional/nullable type (T?)
    Optional {
        inner: Box<TypeAnnotation>,
        span: Span,
    },
    /// Array type (T[])
    Array {
        element: Box<TypeAnnotation>,
        span: Span,
    },
    /// Generic type (Array<T>, Map<K, V>)
    Generic {
        name: String,
        args: Vec<TypeAnnotation>,
        span: Span,
    },
    /// Union type (A | B | C)
    Union {
        types: Vec<TypeAnnotation>,
        span: Span,
    },
    /// Literal type ("success", 123, true)
    Literal {
        value: LiteralType,
        span: Span,
    },
}

impl TypeAnnotation {
    /// Get the span of this type annotation
    pub fn span(&self) -> &Span {
        match self {
            TypeAnnotation::Primitive(p) => &p.span,
            TypeAnnotation::Named { span, .. } => span,
            TypeAnnotation::Optional { span, .. } => span,
            TypeAnnotation::Array { span, .. } => span,
            TypeAnnotation::Generic { span, .. } => span,
            TypeAnnotation::Union { span, .. } => span,
            TypeAnnotation::Literal { span, .. } => span,
        }
    }

    /// Check if this type is optional
    pub fn is_optional(&self) -> bool {
        matches!(self, TypeAnnotation::Optional { .. })
    }

    /// Check if this type is an array
    pub fn is_array(&self) -> bool {
        matches!(self, TypeAnnotation::Array { .. })
    }

    /// Get the base type name (for primitives and named types)
    pub fn base_name(&self) -> Option<&str> {
        match self {
            TypeAnnotation::Primitive(p) => Some(p.kind.as_str()),
            TypeAnnotation::Named { name, .. } => Some(name),
            TypeAnnotation::Optional { inner, .. } => inner.base_name(),
            TypeAnnotation::Array { element, .. } => element.base_name(),
            _ => None,
        }
    }
}

/// Primitive type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveType {
    /// Kind of primitive
    pub kind: PrimitiveKind,
    pub span: Span,
}

/// Primitive type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
    Any,
    Void,
    Never,
    Unknown,
    Bigint,
    Symbol,
}

impl PrimitiveKind {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "string" => Some(PrimitiveKind::String),
            "number" => Some(PrimitiveKind::Number),
            "bool" | "boolean" => Some(PrimitiveKind::Boolean),
            "object" => Some(PrimitiveKind::Object),
            "array" => Some(PrimitiveKind::Array),
            "null" => Some(PrimitiveKind::Null),
            "any" => Some(PrimitiveKind::Any),
            "void" => Some(PrimitiveKind::Void),
            "never" => Some(PrimitiveKind::Never),
            "unknown" => Some(PrimitiveKind::Unknown),
            "bigint" => Some(PrimitiveKind::Bigint),
            "symbol" => Some(PrimitiveKind::Symbol),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveKind::String => "string",
            PrimitiveKind::Number => "number",
            PrimitiveKind::Boolean => "boolean",
            PrimitiveKind::Object => "object",
            PrimitiveKind::Array => "array",
            PrimitiveKind::Null => "null",
            PrimitiveKind::Any => "any",
            PrimitiveKind::Void => "void",
            PrimitiveKind::Never => "never",
            PrimitiveKind::Unknown => "unknown",
            PrimitiveKind::Bigint => "bigint",
            PrimitiveKind::Symbol => "symbol",
        }
    }
}

/// Literal type for union members
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "literal_kind", content = "value", rename_all = "snake_case")]
pub enum LiteralType {
    String(String),
    Number(f64),
    Boolean(bool),
}

/// Generic type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    /// Parameter name
    pub name: String,
    /// Constraint (extends clause)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<Box<TypeAnnotation>>,
    pub span: Span,
}

impl GenericParam {
    /// Create a simple generic parameter without constraints
    pub fn simple(name: String, span: Span) -> Self {
        Self {
            name,
            constraint: None,
            span,
        }
    }

    /// Create a generic parameter with a constraint
    pub fn with_constraint(name: String, constraint: TypeAnnotation, span: Span) -> Self {
        Self {
            name,
            constraint: Some(Box::new(constraint)),
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_parsing() {
        assert_eq!(PrimitiveKind::from_str("string"), Some(PrimitiveKind::String));
        assert_eq!(PrimitiveKind::from_str("STRING"), Some(PrimitiveKind::String));
        assert_eq!(PrimitiveKind::from_str("bool"), Some(PrimitiveKind::Boolean));
        assert_eq!(PrimitiveKind::from_str("boolean"), Some(PrimitiveKind::Boolean));
        assert_eq!(PrimitiveKind::from_str("unknown_type"), None);
    }

    #[test]
    fn test_type_annotation_optional() {
        let inner = TypeAnnotation::Primitive(PrimitiveType {
            kind: PrimitiveKind::String,
            span: Span::default(),
        });
        let optional = TypeAnnotation::Optional {
            inner: Box::new(inner),
            span: Span::default(),
        };
        assert!(optional.is_optional());
        assert_eq!(optional.base_name(), Some("string"));
    }
}
