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
    /// Returns Script-scoped var declarations in source order, including repeats.
    pub fn var_declarations(&self) -> Vec<&Binding> {
        let mut declarations = Vec::new();
        for statement in &self.statements {
            statement.collect_var_declarations(&mut declarations);
        }
        declarations
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

impl Statement {
    pub(crate) fn collect_var_declarations<'a>(&'a self, declarations: &mut Vec<&'a Binding>) {
        match &self.kind {
            StatementKind::Var(bindings) => declarations.extend(bindings),
            StatementKind::Block(body) => {
                for statement in body {
                    statement.collect_var_declarations(declarations);
                }
            }
            StatementKind::If {
                consequent,
                alternate,
                ..
            } => {
                consequent.collect_var_declarations(declarations);
                if let Some(alternate) = alternate {
                    alternate.collect_var_declarations(declarations);
                }
            }
            StatementKind::While { body, .. }
            | StatementKind::DoWhile { body, .. }
            | StatementKind::Labelled { body, .. } => body.collect_var_declarations(declarations),
            StatementKind::For {
                initializer, body, ..
            } => {
                if let Some(ForInitializer::Var(bindings)) = initializer {
                    declarations.extend(bindings);
                }
                body.collect_var_declarations(declarations);
            }
            StatementKind::Switch { clauses, .. } => {
                for statement in clauses.iter().flat_map(|c| &c.statements) {
                    statement.collect_var_declarations(declarations);
                }
            }
            StatementKind::Try {
                body,
                handler,
                finalizer,
            } => {
                body.collect_var_declarations(declarations);
                if let Some(handler) = handler {
                    handler.body.collect_var_declarations(declarations);
                }
                if let Some(finalizer) = finalizer {
                    finalizer.collect_var_declarations(declarations);
                }
            }
            StatementKind::Empty
            | StatementKind::Debugger
            | StatementKind::Expression(_)
            | StatementKind::Lexical { .. }
            | StatementKind::Break(_)
            | StatementKind::Continue(_)
            | StatementKind::Throw(_) => {}
        }
    }
}

/// Supported statements and declarations.
#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    /// An empty statement.
    Empty,
    /// A debugger statement.
    Debugger,
    /// An expression statement.
    Expression(Expr),
    /// A variable declaration in the surrounding variable environment.
    Var(Vec<Binding>),
    /// A lexical declaration.
    Lexical {
        /// Whether bindings may be reassigned.
        mutable: bool,
        /// Bindings in source order.
        bindings: Vec<Binding>,
    },
    /// A block with its own lexical environment.
    Block(Vec<Statement>),
    /// A try statement with a catch clause, a finally clause, or both.
    Try {
        /// Protected block.
        body: Box<Statement>,
        /// Catch clause, when present.
        handler: Option<Box<CatchClause>>,
        /// Block evaluated after the protected block or catch completes.
        finalizer: Option<Box<Statement>>,
    },
    /// A switch statement with a shared lexical scope for its clauses.
    Switch {
        /// Value matched against case expressions.
        discriminant: Expr,
        /// Case and default clauses in source order.
        clauses: Vec<SwitchClause>,
    },
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
    /// A three-clause for loop.
    For {
        /// Evaluated once before the first condition.
        initializer: Option<ForInitializer>,
        /// Condition, or an unconditional loop when absent.
        test: Option<Expr>,
        /// Evaluated after each completed or continued iteration.
        update: Option<Expr>,
        /// Repeated statement.
        body: Box<Statement>,
    },
    /// A statement with a control-flow label.
    Labelled {
        /// Declared label.
        label: Label,
        /// Labelled statement.
        body: Box<Statement>,
    },
    /// Exit the target label, or the nearest loop or switch when absent.
    Break(Option<Label>),
    /// Continue the target loop, or the nearest loop when absent.
    Continue(Option<Label>),
    /// Throw a language value.
    Throw(Expr),
}

/// A catch clause with an optional binding identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct CatchClause {
    /// The catch binding, whose initializer is always absent.
    pub parameter: Option<Binding>,
    /// Catch block, with its own lexical environment inside the parameter's scope.
    pub body: Box<Statement>,
    /// Source range including the catch keyword, parameter, and block.
    pub span: Span,
}

/// A switch clause and its statement list.
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchClause {
    /// Case expression, or None for the default clause.
    pub test: Option<Expr>,
    /// Statements executed when the clause is reached.
    pub statements: Vec<Statement>,
    /// Source range, including the case or default keyword.
    pub span: Span,
}

/// The initialization clause of a three-clause for loop.
#[derive(Clone, Debug, PartialEq)]
pub enum ForInitializer {
    /// An expression whose value is discarded.
    Expression(Expr),
    /// Variable declarations in the surrounding variable environment.
    Var(Vec<Binding>),
    /// A declaration in a new loop scope.
    Lexical {
        /// Whether bindings may be reassigned and are copied per iteration.
        mutable: bool,
        /// Bindings in source order.
        bindings: Vec<Binding>,
    },
}

/// A decoded label identifier and its source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// Identifier name.
    pub name: String,
    /// Identifier source range.
    pub span: Span,
}

/// A binding identifier and optional initializer.
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
    /// An ordinary object initializer, in source property order.
    Object(Vec<ObjectProperty>),
    /// An untagged template literal. There is one more element than substitution.
    Template {
        /// Cooked and raw components in source order.
        elements: Vec<TemplateElement>,
        /// Expressions interpolated between elements.
        substitutions: Vec<Expr>,
    },
    /// An identifier reference.
    Identifier(String),
    /// A parenthesized expression. Retained for grammar restrictions.
    Parenthesized(Box<Expr>),
    /// A dotted or computed property reference.
    Member(Box<Expr>, PropertyName),
    /// A call with an ordered list of non-spread arguments.
    Call {
        /// Expression whose value is called; references retain their receiver.
        callee: Box<Expr>,
        /// Argument expressions in source order.
        arguments: Vec<Expr>,
    },
    /// A prefix unary expression.
    Unary(UnaryOp, Box<Expr>),
    /// An increment or decrement of a reference.
    Update {
        /// Increment or decrement.
        op: UpdateOp,
        /// Assignment target.
        argument: Box<Expr>,
        /// Whether the operator precedes its argument.
        prefix: bool,
    },
    /// A binary expression, including short-circuit operators.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// A simple assignment to a validated reference expression.
    Assign(Box<Expr>, Box<Expr>),
    /// A compound assignment to a validated reference, including logical assignment.
    CompoundAssign(BinaryOp, Box<Expr>, Box<Expr>),
    /// A conditional expression.
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// A property definition in an object initializer.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectProperty {
    /// The literal or computed property name.
    pub name: PropertyName,
    /// The initializer, or identifier reference for shorthand syntax.
    pub value: Expr,
    /// The property definition's evaluation form.
    pub kind: PropertyKind,
    /// Source range of the entire definition.
    pub span: Span,
}

/// Implemented object-initializer property forms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyKind {
    /// A colon-separated ordinary data property.
    Data,
    /// An identifier reference used as both name and value.
    Shorthand,
    /// A non-computed `__proto__` colon definition.
    Prototype,
}

/// A property name before runtime ToPropertyKey conversion.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyName {
    /// A String, Number, or BigInt literal. Identifier names become strings.
    Literal(Literal),
    /// A bracketed assignment expression.
    Computed(Box<Expr>),
}

/// A template literal component, excluding its delimiters.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateElement {
    /// Text with escapes interpreted, or None for an invalid escape sequence.
    pub cooked: Option<JsString>,
    /// Text with escapes preserved and CR/CRLF normalized to LF.
    pub raw: JsString,
    /// Source range excluding delimiters.
    pub span: Span,
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
    /// An exact unsigned integer, converted under the evaluator's resource budget.
    BigInt {
        /// Validated digits with separators, prefix, and suffix removed.
        digits: String,
        /// The source radix: 2, 8, 10, or 16.
        radix: u32,
    },
    /// UTF-16 code units, including lone surrogates.
    String(JsString),
}

/// Increment and decrement operators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateOp {
    /// Add one to the numeric value.
    Increment,
    /// Subtract one from the numeric value.
    Decrement,
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
    /// Delete a reference, or evaluate and discard a non-reference.
    Delete,
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
    /// Whether an object has an own or inherited property.
    In,
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
