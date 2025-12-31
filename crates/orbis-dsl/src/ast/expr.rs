//! Expression types with operator precedence support
//!
//! This module defines expression types and provides Pratt parsing
//! for correct operator precedence handling.

use serde::{Deserialize, Serialize};

use super::node::Span;

/// Expression in the Orbis DSL
///
/// Expressions are used in:
/// - State values and computed state
/// - Attribute values
/// - Conditions (if, when)
/// - Action arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "expr_type", rename_all = "snake_case")]
pub enum Expression {
    /// Binary operation (a + b, a && b, etc.)
    Binary(BinaryExpr),
    /// Unary operation (!a, -a)
    Unary(UnaryExpr),
    /// Literal value
    Literal(Literal),
    /// Identifier reference
    Identifier(Identifier),
    /// Member access (a.b.c)
    MemberAccess(MemberAccess),
    /// Method call (api.call(), toast.show())
    MethodCall(MethodCall),
    /// Special variable ($response, $error, $event)
    SpecialVariable(SpecialVariable),
    /// Object literal { key: value }
    Object(ObjectLiteral),
    /// Array literal [a, b, c]
    Array(ArrayLiteral),
    /// Parenthesized expression
    Grouped {
        inner: Box<Expression>,
        span: Span,
    },
    /// Arrow function (for transforms/validators)
    ArrowFunction(ArrowFunction),
    /// Interpolated string "Hello, {name}!"
    InterpolatedString(InterpolatedString),
    /// Assignment expression (target = value)
    Assignment(Assignment),
}

impl Expression {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expression::Binary(e) => &e.span,
            Expression::Unary(e) => &e.span,
            Expression::Literal(e) => &e.span,
            Expression::Identifier(e) => &e.span,
            Expression::MemberAccess(e) => &e.span,
            Expression::MethodCall(e) => &e.span,
            Expression::SpecialVariable(e) => &e.span,
            Expression::Object(e) => &e.span,
            Expression::Array(e) => &e.span,
            Expression::Grouped { span, .. } => span,
            Expression::ArrowFunction(e) => &e.span,
            Expression::InterpolatedString(e) => &e.span,
            Expression::Assignment(e) => &e.span,
        }
    }

    /// Check if this is a simple literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal(_))
    }

    /// Check if this is a state path (state.xxx)
    pub fn is_state_path(&self) -> bool {
        match self {
            Expression::MemberAccess(m) => m.root == "state",
            _ => false,
        }
    }
}

/// Binary expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpr {
    /// Left operand
    pub left: Box<Expression>,
    /// Operator
    pub op: BinaryOp,
    /// Right operand
    pub right: Box<Expression>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %
    // Comparison
    Eq,       // ==
    NotEq,    // !=
    Lt,       // <
    LtEq,     // <=
    Gt,       // >
    GtEq,     // >=
    // Logical
    And,      // &&
    Or,       // ||
}

impl BinaryOp {
    /// Get the precedence of this operator (higher = binds tighter)
    ///
    /// Precedence levels (from lowest to highest):
    /// 1. || (logical or)
    /// 2. && (logical and)
    /// 3. ==, != (equality)
    /// 4. <, <=, >, >= (comparison)
    /// 5. +, - (additive)
    /// 6. *, /, % (multiplicative)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq | BinaryOp::NotEq => 3,
            BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => 4,
            BinaryOp::Add | BinaryOp::Sub => 5,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 6,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        true // All our binary operators are left-associative
    }

    /// Parse operator from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(BinaryOp::Add),
            "-" => Some(BinaryOp::Sub),
            "*" => Some(BinaryOp::Mul),
            "/" => Some(BinaryOp::Div),
            "%" => Some(BinaryOp::Mod),
            "==" => Some(BinaryOp::Eq),
            "!=" => Some(BinaryOp::NotEq),
            "<" => Some(BinaryOp::Lt),
            "<=" => Some(BinaryOp::LtEq),
            ">" => Some(BinaryOp::Gt),
            ">=" => Some(BinaryOp::GtEq),
            "&&" => Some(BinaryOp::And),
            "||" => Some(BinaryOp::Or),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

/// Unary expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpr {
    /// Operator
    pub op: UnaryOp,
    /// Operand
    pub operand: Box<Expression>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryOp {
    /// Logical not (!)
    Not,
    /// Negation (-)
    Neg,
}

impl UnaryOp {
    /// Parse operator from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "!" => Some(UnaryOp::Not),
            "-" => Some(UnaryOp::Neg),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
        }
    }
}

/// Literal value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Literal {
    /// The literal value
    pub value: LiteralValue,
    pub span: Span,
}

/// Literal value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum LiteralValue {
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Integer literal (for cases where integer is required)
    Integer(i64),
    /// Boolean literal
    Boolean(bool),
    /// Null literal
    Null,
}

/// Identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    /// Identifier name
    pub name: String,
    pub span: Span,
}

/// Member access expression (a.b.c)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAccess {
    /// Root object (first identifier)
    pub root: String,
    /// Path of member names
    pub path: Vec<String>,
    pub span: Span,
}

impl MemberAccess {
    /// Get the full path as a dotted string
    pub fn full_path(&self) -> String {
        let mut result = self.root.clone();
        for segment in &self.path {
            result.push('.');
            result.push_str(segment);
        }
        result
    }

    /// Check if this is a state access
    pub fn is_state(&self) -> bool {
        self.root == "state"
    }
}

/// Method call expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodCall {
    /// Namespace (api, toast, router, console, etc.)
    pub namespace: String,
    /// Method name
    pub method: String,
    /// Arguments
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub arguments: Vec<Argument>,
    pub span: Span,
}

/// Method argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    /// Argument name (for named arguments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Argument value
    pub value: Expression,
    /// Whether this is a spread argument (...expr)
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub spread: bool,
}

impl Argument {
    /// Get the expression value
    pub fn value(&self) -> &Expression {
        &self.value
    }
    
    /// Check if this is a named argument
    pub fn is_named(&self) -> bool {
        self.name.is_some()
    }
}

/// Special variable ($response, $error, $event)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialVariable {
    /// Variable kind
    pub kind: SpecialVariableKind,
    /// Optional member access path
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub path: Vec<String>,
    pub span: Span,
}

/// Special variable kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialVariableKind {
    /// $response - API response
    Response,
    /// $error - Error object
    Error,
    /// $event - Event object
    Event,
}

impl SpecialVariableKind {
    /// Get the variable prefix
    pub fn prefix(&self) -> &'static str {
        match self {
            SpecialVariableKind::Response => "$response",
            SpecialVariableKind::Error => "$error",
            SpecialVariableKind::Event => "$event",
        }
    }
}

/// Object literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLiteral {
    /// Key-value pairs
    pub pairs: Vec<ObjectPair>,
    pub span: Span,
}

/// Object key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPair {
    /// Key (identifier or string)
    pub key: String,
    /// Value
    pub value: Expression,
    pub span: Span,
}

/// Array literal expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayLiteral {
    /// Elements
    pub elements: Vec<Expression>,
    pub span: Span,
}

/// Arrow function expression (for validators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowFunction {
    /// Parameters
    pub params: Vec<String>,
    /// Function body
    pub body: ArrowBody,
    pub span: Span,
}

/// Arrow function body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "body_type", content = "content", rename_all = "snake_case")]
pub enum ArrowBody {
    /// Expression body (implicit return)
    Expression(Box<Expression>),
    /// Block body with statements
    Block(Vec<ArrowStatement>),
}

/// Arrow function statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stmt_type", rename_all = "snake_case")]
pub enum ArrowStatement {
    /// Return statement
    Return(Option<Expression>),
    /// Expression statement
    Expression(Expression),
    /// Variable declaration
    Let { name: String, value: Expression },
    /// Const declaration
    Const { name: String, value: Expression },
}

/// Interpolated string with embedded expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedString {
    /// String parts (alternating between literal and expression)
    pub parts: Vec<StringPart>,
    pub span: Span,
}

/// Part of an interpolated string
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "part_type", rename_all = "snake_case")]
pub enum StringPart {
    /// Literal text
    Literal { value: String },
    /// Interpolated expression
    Expression { value: Expression },
}

// ============================================================================
// ASSIGNMENT EXPRESSION
// ============================================================================

/// Assignment expression (target = value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// Target of the assignment (member access or identifier)
    pub target: Box<Expression>,
    /// Value being assigned
    pub value: Box<Expression>,
    pub span: Span,
}

// ============================================================================
// STATE ASSIGNMENT
// ============================================================================

/// State assignment action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAssignment {
    /// Target state path
    pub target: MemberAccess,
    /// Value to assign
    pub value: Expression,
    pub span: Span,
}

// ============================================================================
// PRATT PARSER SUPPORT
// ============================================================================

/// Token for Pratt parsing
#[derive(Debug, Clone)]
pub enum ExprToken<'a> {
    /// Term (identifier, literal, etc.)
    Term(Expression),
    /// Binary operator
    BinaryOp(&'a str),
    /// Unary operator
    UnaryOp(&'a str),
    /// End of expression
    End,
}

/// Pratt parser for expressions
///
/// This implements the Pratt parsing algorithm for correct operator
/// precedence handling. The parser uses the following precedence levels
/// (from lowest to highest):
///
/// 1. `||` (logical or)
/// 2. `&&` (logical and)
/// 3. `==`, `!=` (equality)
/// 4. `<`, `<=`, `>`, `>=` (comparison)
/// 5. `+`, `-` (additive)
/// 6. `*`, `/`, `%` (multiplicative)
/// 7. `!`, `-` (unary prefix operators)
pub struct PrattParser;

impl PrattParser {
    /// Parse a sequence of terms and operators into a properly precedenced expression
    pub fn parse(terms: Vec<Expression>, operators: Vec<BinaryOp>, span: Span) -> Expression {
        if terms.len() == 1 && operators.is_empty() {
            return terms.into_iter().next().unwrap();
        }

        Self::parse_expr(&terms, &operators, 0, terms.len() - 1, &span)
    }

    fn parse_expr(
        terms: &[Expression],
        operators: &[BinaryOp],
        start: usize,
        end: usize,
        span: &Span,
    ) -> Expression {
        if start == end {
            return terms[start].clone();
        }

        // Find the operator with lowest precedence (to split on)
        let mut min_prec = u8::MAX;
        let mut split_idx = None;

        for (i, op) in operators.iter().enumerate().skip(start).take(end - start) {
            let prec = op.precedence();
            // For left-associative operators, we want the rightmost lowest-precedence op
            if prec <= min_prec {
                min_prec = prec;
                split_idx = Some(i);
            }
        }

        match split_idx {
            Some(idx) => {
                let left = Self::parse_expr(terms, operators, start, idx, span);
                let right = Self::parse_expr(terms, operators, idx + 1, end, span);
                Expression::Binary(BinaryExpr {
                    left: Box::new(left),
                    op: operators[idx],
                    right: Box::new(right),
                    span: span.clone(),
                })
            }
            None => terms[start].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_num(n: f64) -> Expression {
        Expression::Literal(Literal {
            value: LiteralValue::Number(n),
            span: Span::default(),
        })
    }

    #[test]
    fn test_pratt_simple() {
        // 1 + 2
        let terms = vec![make_num(1.0), make_num(2.0)];
        let ops = vec![BinaryOp::Add];
        let result = PrattParser::parse(terms, ops, Span::default());
        
        match result {
            Expression::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_pratt_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let terms = vec![make_num(1.0), make_num(2.0), make_num(3.0)];
        let ops = vec![BinaryOp::Add, BinaryOp::Mul];
        let result = PrattParser::parse(terms, ops, Span::default());
        
        match result {
            Expression::Binary(b) => {
                assert_eq!(b.op, BinaryOp::Add);
                match b.right.as_ref() {
                    Expression::Binary(inner) => {
                        assert_eq!(inner.op, BinaryOp::Mul);
                    }
                    _ => panic!("Expected inner binary expression"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_operator_precedence_values() {
        assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
        assert!(BinaryOp::Add.precedence() > BinaryOp::Eq.precedence());
        assert!(BinaryOp::Eq.precedence() > BinaryOp::And.precedence());
        assert!(BinaryOp::And.precedence() > BinaryOp::Or.precedence());
    }
}
