//! Control flow types for the Orbis DSL
//!
//! This module defines types for control flow structures:
//! if/else, for loops, and when (pattern matching).

use serde::{Deserialize, Serialize};

use super::expr::Expression;
use super::node::{Span, TemplateContent};

/// If block with optional else-if and else branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfBlock {
    /// Primary condition
    pub condition: Expression,

    /// Content if condition is true
    pub then_content: Vec<TemplateContent>,

    /// Else-if branches
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub else_if_branches: Vec<ElseIfBranch>,

    /// Else content (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub else_content: Option<Vec<TemplateContent>>,

    pub span: Span,
}

impl IfBlock {
    /// Check if this if block has an else branch
    pub fn has_else(&self) -> bool {
        self.else_content.is_some()
    }

    /// Check if this if block has else-if branches
    pub fn has_else_if(&self) -> bool {
        !self.else_if_branches.is_empty()
    }

    /// Get the total number of branches (including main if and else)
    pub fn branch_count(&self) -> usize {
        1 + self.else_if_branches.len() + if self.has_else() { 1 } else { 0 }
    }

    /// Iterate over all conditions (main + else-if)
    pub fn all_conditions(&self) -> impl Iterator<Item = &Expression> {
        std::iter::once(&self.condition).chain(self.else_if_branches.iter().map(|b| &b.condition))
    }
}

/// Else-if branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseIfBranch {
    /// Condition for this branch
    pub condition: Expression,
    /// Content if condition is true
    pub content: Vec<TemplateContent>,
    pub span: Span,
}

/// For loop block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForBlock {
    /// Loop binding
    pub binding: ForBinding,

    /// Collection expression
    pub collection: Expression,

    /// Loop body content
    pub body: Vec<TemplateContent>,

    pub span: Span,
}

impl ForBlock {
    /// Get the item variable name
    pub fn item_var(&self) -> &str {
        match &self.binding {
            ForBinding::Simple { item } => item,
            ForBinding::WithIndex { item, .. } => item,
        }
    }

    /// Get the index variable name if present
    pub fn index_var(&self) -> Option<&str> {
        match &self.binding {
            ForBinding::Simple { .. } => None,
            ForBinding::WithIndex { index, .. } => Some(index),
        }
    }
}

/// For loop binding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "binding_type", rename_all = "snake_case")]
pub enum ForBinding {
    /// Simple binding: for item in collection
    Simple { item: String },
    /// Tuple binding: for (index, item) in collection
    WithIndex { index: String, item: String },
}

/// When block (pattern matching)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenBlock {
    /// Expression to match against
    pub expression: Expression,

    /// Match arms
    pub arms: Vec<WhenArm>,

    pub span: Span,
}

impl WhenBlock {
    /// Check if this when block has an else arm
    pub fn has_else_arm(&self) -> bool {
        self.arms.iter().any(|arm| arm.is_else)
    }

    /// Get the else arm if present
    pub fn else_arm(&self) -> Option<&WhenArm> {
        self.arms.iter().find(|arm| arm.is_else)
    }

    /// Get all pattern arms (excluding else)
    pub fn pattern_arms(&self) -> impl Iterator<Item = &WhenArm> {
        self.arms.iter().filter(|arm| !arm.is_else)
    }
}

/// When arm (pattern match case)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenArm {
    /// Pattern to match (None for else arm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<WhenPattern>,

    /// Whether this is the else arm
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_else: bool,

    /// Content for this arm
    pub content: Vec<TemplateContent>,

    pub span: Span,
}

/// Pattern for when matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "pattern_type", rename_all = "snake_case")]
pub enum WhenPattern {
    /// String literal pattern
    String { value: String },
    /// Number literal pattern
    Number { value: f64 },
    /// Boolean literal pattern
    Boolean { value: bool },
}

impl WhenPattern {
    /// Check if this pattern matches a given expression
    pub fn matches_literal(&self, value: &str) -> bool {
        match self {
            WhenPattern::String { value: v } => v == value,
            WhenPattern::Number { value: v } => value.parse::<f64>().map(|n| n == *v).unwrap_or(false),
            WhenPattern::Boolean { value: v } => value.parse::<bool>().map(|b| b == *v).unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{Identifier, Literal, LiteralValue};

    fn make_bool_expr(value: bool) -> Expression {
        Expression::Literal(Literal {
            value: LiteralValue::Boolean(value),
            span: Span::default(),
        })
    }

    fn make_ident_expr(name: &str) -> Expression {
        Expression::Identifier(Identifier {
            name: name.to_string(),
            span: Span::default(),
        })
    }

    #[test]
    fn test_if_block_branches() {
        let if_block = IfBlock {
            condition: make_bool_expr(true),
            then_content: vec![],
            else_if_branches: vec![
                ElseIfBranch {
                    condition: make_bool_expr(false),
                    content: vec![],
                    span: Span::default(),
                },
            ],
            else_content: Some(vec![]),
            span: Span::default(),
        };

        assert!(if_block.has_else());
        assert!(if_block.has_else_if());
        assert_eq!(if_block.branch_count(), 3);
        assert_eq!(if_block.all_conditions().count(), 2);
    }

    #[test]
    fn test_for_binding() {
        let simple = ForBinding::Simple { item: "user".to_string() };
        let with_index = ForBinding::WithIndex {
            index: "i".to_string(),
            item: "user".to_string(),
        };

        let for_simple = ForBlock {
            binding: simple,
            collection: make_ident_expr("users"),
            body: vec![],
            span: Span::default(),
        };

        let for_indexed = ForBlock {
            binding: with_index,
            collection: make_ident_expr("users"),
            body: vec![],
            span: Span::default(),
        };

        assert_eq!(for_simple.item_var(), "user");
        assert!(for_simple.index_var().is_none());

        assert_eq!(for_indexed.item_var(), "user");
        assert_eq!(for_indexed.index_var(), Some("i"));
    }

    #[test]
    fn test_when_pattern() {
        let pattern = WhenPattern::String { value: "loading".to_string() };
        assert!(pattern.matches_literal("loading"));
        assert!(!pattern.matches_literal("success"));

        let num_pattern = WhenPattern::Number { value: 42.0 };
        assert!(num_pattern.matches_literal("42"));
        assert!(!num_pattern.matches_literal("43"));
    }
}
