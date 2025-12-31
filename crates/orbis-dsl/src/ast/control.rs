//! Control flow types for the Orbis DSL
//!
//! This module defines types for control flow structures:
//! if/else, for loops, and when (pattern matching).

use serde::{Deserialize, Serialize};

use super::expr::{Expression, Identifier};
use super::node::{Span, TemplateContent};

/// If block with optional else-if and else branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfBlock {
    /// Primary condition
    pub condition: Expression,

    /// Content if condition is true
    pub then_branch: Vec<TemplateContent>,

    /// Else-if branches
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub else_if_branches: Vec<ElseIfBranch>,

    /// Else content (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub else_branch: Option<Vec<TemplateContent>>,

    pub span: Span,
}

impl IfBlock {
    /// Check if this if block has an else branch
    pub fn has_else(&self) -> bool {
        self.else_branch.is_some()
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
    pub body: Vec<TemplateContent>,
    pub span: Span,
}

/// For loop block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForBlock {
    /// Loop binding
    pub binding: ForBinding,

    /// Collection expression
    pub iterable: Expression,

    /// Loop body content
    pub body: Vec<TemplateContent>,

    pub span: Span,
}

impl ForBlock {
    /// Get the item variable name
    pub fn item_var(&self) -> &str {
        &self.binding.item.name
    }

    /// Get the index variable name if present
    pub fn index_var(&self) -> Option<&str> {
        self.binding.index.as_ref().map(|i| i.name.as_str())
    }
}

/// For loop binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForBinding {
    /// Item variable
    pub item: Identifier,
    /// Optional index variable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<Identifier>,
    pub span: Span,
}

/// When block (pattern matching)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenBlock {
    /// Expression to match against
    pub subject: Expression,

    /// Match arms
    pub arms: Vec<WhenArm>,

    pub span: Span,
}

impl WhenBlock {
    /// Check if this when block has a wildcard arm
    pub fn has_wildcard_arm(&self) -> bool {
        self.arms.iter().any(|arm| matches!(&arm.pattern, WhenPattern::Wildcard { .. }))
    }

    /// Get the wildcard arm if present
    pub fn wildcard_arm(&self) -> Option<&WhenArm> {
        self.arms.iter().find(|arm| matches!(&arm.pattern, WhenPattern::Wildcard { .. }))
    }

    /// Get all non-wildcard pattern arms
    pub fn pattern_arms(&self) -> impl Iterator<Item = &WhenArm> {
        self.arms.iter().filter(|arm| !matches!(&arm.pattern, WhenPattern::Wildcard { .. }))
    }
}

/// When arm (pattern match case)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenArm {
    /// Pattern to match
    pub pattern: WhenPattern,

    /// Optional guard condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<Expression>,

    /// Content for this arm
    pub body: Vec<TemplateContent>,

    pub span: Span,
}

/// Pattern for when matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "pattern_type", rename_all = "snake_case")]
pub enum WhenPattern {
    /// Literal pattern (string, number, boolean)
    Literal { value: Expression, span: Span },
    /// Binding pattern (captures value into variable)
    Binding { name: String, span: Span },
    /// Range pattern (start..end)
    Range { start: Box<Expression>, end: Box<Expression>, span: Span },
    /// Or pattern (pattern1 | pattern2)
    Or { patterns: Vec<WhenPattern>, span: Span },
    /// Wildcard/else pattern (_)
    Wildcard { span: Span },
}

impl WhenPattern {
    /// Check if this is a wildcard pattern
    pub fn is_wildcard(&self) -> bool {
        matches!(self, WhenPattern::Wildcard { .. })
    }
    
    /// Get the span of this pattern
    pub fn span(&self) -> &Span {
        match self {
            WhenPattern::Literal { span, .. } => span,
            WhenPattern::Binding { span, .. } => span,
            WhenPattern::Range { span, .. } => span,
            WhenPattern::Or { span, .. } => span,
            WhenPattern::Wildcard { span } => span,
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
            then_branch: vec![],
            else_if_branches: vec![
                ElseIfBranch {
                    condition: make_bool_expr(false),
                    body: vec![],
                    span: Span::default(),
                },
            ],
            else_branch: Some(vec![]),
            span: Span::default(),
        };

        assert!(if_block.has_else());
        assert!(if_block.has_else_if());
        assert_eq!(if_block.branch_count(), 3);
        assert_eq!(if_block.all_conditions().count(), 2);
    }

    #[test]
    fn test_for_binding() {
        let simple = ForBinding {
            item: Identifier { name: "user".to_string(), span: Span::default() },
            index: None,
            span: Span::default(),
        };
        let with_index = ForBinding {
            item: Identifier { name: "user".to_string(), span: Span::default() },
            index: Some(Identifier { name: "i".to_string(), span: Span::default() }),
            span: Span::default(),
        };

        let for_simple = ForBlock {
            binding: simple,
            iterable: make_ident_expr("users"),
            body: vec![],
            span: Span::default(),
        };

        let for_indexed = ForBlock {
            binding: with_index,
            iterable: make_ident_expr("users"),
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
        let pattern = WhenPattern::Literal { 
            value: Expression::Literal(Literal {
                value: LiteralValue::String("loading".to_string()),
                span: Span::default(),
            }),
            span: Span::default(),
        };
        assert!(!pattern.is_wildcard());

        let wildcard = WhenPattern::Wildcard { span: Span::default() };
        assert!(wildcard.is_wildcard());
    }
}
