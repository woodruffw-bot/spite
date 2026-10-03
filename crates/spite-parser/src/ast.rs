//! The supported syntax tree.

use spite_core::{JsString, Span};

/// A parsed and validated Script. Constructed only by the parser.
#[derive(Clone, Debug, PartialEq)]
pub struct Script {
    pub(crate) statements: Vec<Statement>,
    pub(crate) strict: bool,
}

impl Script {
    /// Returns top-level statements in source order.
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
    /// Returns whether a Use Strict Directive enables strict mode.
    pub fn is_strict(&self) -> bool {
        self.strict
    }
}

/// A statement with its source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    /// Statement form.
    pub kind: StatementKind,
    /// Source range.
    pub span: Span,
}

/// Supported statements and declarations.
#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    /// An empty statement.
    Empty,
    /// An expression statement.
    Expression(Expr),
    /// A lexical declaration.
    Lexical {
        /// Whether bindings may be reassigned.
        mutable: bool,
        /// Bindings in source order.
        bindings: Vec<Binding>,
    },
    /// A block with its own lexical environment.
    Block(Vec<Statement>),
    /// A conditional statement.
    If {
        /// Condition.
        test: Expr,
        /// Selected when the condition is truthy.
        consequent: Box<Statement>,
        /// Selected when the condition is falsy.
        alternate: Option<Box<Statement>>,
    },
    /// A loop that tests its condition before each iteration.
    While {
        /// Condition.
        test: Expr,
        /// Repeated statement.
        body: Box<Statement>,
    },
    /// A loop that tests its condition after each iteration.
    DoWhile {
        /// Repeated statement, evaluated at least once.
        body: Box<Statement>,
        /// Condition.
        test: Expr,
    },
    /// Exit the nearest enclosing loop (unlabelled).
    Break,
    /// Continue the nearest enclosing loop (unlabelled).
    Continue,
    /// Throw a language value.
    Throw(Expr),
}

/// A lexical binding and optional initializer.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    /// Binding identifier.
    pub name: String,
    /// Identifier source range.
    pub span: Span,
    /// Initial value expression.
    pub initializer: Option<Expr>,
}

/// An expression with its source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub(crate) depth: usize,
    /// Expression form.
    pub kind: ExprKind,
    /// Source range.
    pub span: Span,
}

/// Supported expression forms.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// A primitive literal.
    Literal(Literal),
    /// An identifier reference.
    Identifier(String),
    /// A parenthesized expression. Retained for grammar restrictions.
    Parenthesized(Box<Expr>),
    /// A prefix unary expression.
    Unary(UnaryOp, Box<Expr>),
    /// A binary expression, including short-circuit operators.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// A simple assignment to an identifier.
    Assign(String, Box<Expr>),
    /// A conditional expression.
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// Primitive literal values.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// The null value.
    Null,
    /// A Boolean value.
    Boolean(bool),
    /// An IEEE 754 binary64 value.
    Number(f64),
    /// UTF-16 code units, including lone surrogates.
    String(JsString),
}

/// Prefix operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    /// Numeric conversion.
    Plus,
    /// Numeric negation.
    Minus,
    /// Boolean negation.
    Not,
    /// Bitwise complement.
    BitNot,
    /// The void operator.
    Void,
    /// The typeof operator.
    Typeof,
}

/// Binary operators, ordered independently of precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    /// Addition or string concatenation.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
    /// Remainder.
    Remainder,
    /// Exponentiation.
    Exponentiate,
    /// Abstract equality.
    Equal,
    /// Negated abstract equality.
    NotEqual,
    /// Strict equality.
    StrictEqual,
    /// Negated strict equality.
    StrictNotEqual,
    /// Less than.
    Less,
    /// Less than or equal.
    LessEqual,
    /// Greater than.
    Greater,
    /// Greater than or equal.
    GreaterEqual,
    /// Logical conjunction.
    And,
    /// Logical disjunction.
    Or,
    /// Nullish coalescing.
    Nullish,
    /// Bitwise conjunction.
    BitAnd,
    /// Bitwise disjunction.
    BitOr,
    /// Bitwise exclusive disjunction.
    BitXor,
    /// Signed left shift.
    LeftShift,
    /// Signed right shift.
    RightShift,
    /// Unsigned right shift.
    UnsignedRightShift,
    /// Evaluate both operands and return the second.
    Comma,
}
