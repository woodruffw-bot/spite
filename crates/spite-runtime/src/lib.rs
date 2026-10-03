//! A tree-walking interpreter for the implemented ECMAScript subset.

mod value;
pub use value::Value;

use spite_core::{Diagnostic, JsString, Span};
use spite_parser::{ast::*, parse_script};
use std::{collections::BTreeMap, fmt};
use value::{exponentiate, to_uint32};

/// Built-in error categories produced by implemented runtime operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExceptionKind {
    /// Invalid declaration instantiation.
    SyntaxError,
    /// An unresolvable or uninitialized binding.
    ReferenceError,
    /// An operation forbidden by a value or binding's type.
    TypeError,
}

/// An evaluation failure. Host failures are distinct from JavaScript exceptions.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Parsing failed before execution began.
    Parse(Diagnostic),
    /// An implemented operation produced a built-in JavaScript error.
    Exception {
        /// Error category.
        kind: ExceptionKind,
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
    /// A throw statement produced an uncaught value.
    Thrown(Value),
    /// Evaluation needs functionality that has not been implemented.
    Unsupported {
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
    /// Evaluation exceeded a host resource limit.
    Limit {
        /// Associated source range.
        span: Span,
        /// Explanation.
        message: String,
    },
}

impl Error {
    fn is_language_exception(&self) -> bool {
        match self {
            Self::Thrown(_) | Self::Exception { .. } => true,
            Self::Parse(_) | Self::Unsupported { .. } | Self::Limit { .. } => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(diagnostic) => write!(f, "{diagnostic}"),
            Self::Exception {
                kind,
                span,
                message,
            } => write!(f, "{kind:?} at {}..{}: {message}", span.start, span.end),
            Self::Thrown(value) => write!(f, "uncaught {value}"),
            Self::Unsupported { span, message } => {
                write!(f, "Unsupported at {}..{}: {message}", span.start, span.end)
            }
            Self::Limit { span, message } => {
                write!(f, "Limit at {}..{}: {message}", span.start, span.end)
            }
        }
    }
}
impl std::error::Error for Error {}

/// Host resource limits. They do not alter ECMAScript exceptions.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum evaluation steps per Script, including binding and clause work.
    pub max_steps: usize,
    /// Maximum code units in any produced string.
    pub max_string_units: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_string_units: 1024 * 1024,
        }
    }
}

#[derive(Debug)]
struct BindingState {
    value: Option<Value>,
    mutable: bool,
}

// Resolve references before evaluating assignment RHS expressions. GetValue and
// PutValue are separate operations, including the unresolvable typeof case.
enum Reference<'a> {
    Lexical(usize, &'a str),
    Global(&'a str),
    Unresolvable(&'a str),
    UnsupportedGlobal(&'a str),
}

// Implemented statement completions. Throws already carry a non-empty value in
// Error::Thrown; host failures remain separate from these control transfers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompletionKind {
    Normal,
    Break,
    Continue,
}

struct Completion {
    kind: CompletionKind,
    // None is empty, distinct from Some(Value::Undefined).
    value: Option<Value>,
    target: Option<String>,
}

impl Completion {
    fn normal(value: Option<Value>) -> Self {
        Self {
            kind: CompletionKind::Normal,
            value,
            target: None,
        }
    }

    // ECMA-262 14.7.1.1 LoopContinues.
    fn loop_continues(&self, labels: &[&str]) -> bool {
        self.kind == CompletionKind::Normal
            || (self.kind == CompletionKind::Continue
                && self
                    .target
                    .as_deref()
                    .is_none_or(|target| labels.contains(&target)))
    }

    // ECMA-262 14.13.4: loops and switches consume only unlabelled breaks.
    fn consume_unlabelled_break(mut self) -> Self {
        if self.kind == CompletionKind::Break && self.target.is_none() {
            self.kind = CompletionKind::Normal;
        }
        self
    }

    // ECMA-262 6.2.4.4 UpdateEmpty preserves the completion's kind.
    fn update_empty(mut self, value: Option<Value>) -> Self {
        self.value = self.value.or(value);
        self
    }
}

/// An isolated execution realm with persistent global lexical bindings.
///
/// Only the documented subset is implemented. Standard object globals and host
/// extensions are not installed. Each instance owns all of its state.
#[derive(Debug)]
pub struct Realm {
    scopes: Vec<BTreeMap<String, BindingState>>,
    globals: BTreeMap<String, Value>,
    limits: Limits,
    remaining_steps: usize,
    strict: bool,
}

impl Default for Realm {
    fn default() -> Self {
        Self::new(Limits::default())
    }
}

impl Realm {
    /// Creates a realm with explicit host limits.
    pub fn new(limits: Limits) -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
            globals: BTreeMap::from([
                ("undefined".into(), Value::Undefined),
                ("NaN".into(), Value::Number(f64::NAN)),
                ("Infinity".into(), Value::Number(f64::INFINITY)),
            ]),
            limits,
            remaining_steps: 0,
            strict: false,
        }
    }

    /// Parses, validates, and evaluates a Script in this realm.
    ///
    /// Parsing and early errors occur before any execution. Runtime failures
    /// preserve effects that have already happened, as required by ECMAScript.
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let script = parse_script(source).map_err(Error::Parse)?;
        self.evaluate(&script)
    }

    /// Evaluates a Script already validated by the parser.
    pub fn evaluate(&mut self, script: &Script) -> Result<Value, Error> {
        self.remaining_steps = self.limits.max_steps;
        self.strict = script.is_strict();
        self.instantiate(script.statements().iter(), true)?;
        let completion = self.statements(script.statements())?;
        // Validated Scripts cannot leave an unhandled control transfer.
        debug_assert_eq!(completion.kind, CompletionKind::Normal);
        Ok(completion.value.unwrap_or(Value::Undefined))
    }

    fn exception(kind: ExceptionKind, span: Span, message: impl Into<String>) -> Error {
        Error::Exception {
            kind,
            span,
            message: message.into(),
        }
    }
    fn unsupported(span: Span, message: impl Into<String>) -> Error {
        Error::Unsupported {
            span,
            message: message.into(),
        }
    }
    fn tick(&mut self, span: Span) -> Result<(), Error> {
        self.remaining_steps = self
            .remaining_steps
            .checked_sub(1)
            .ok_or_else(|| Error::Limit {
                span,
                message: "evaluation step limit exceeded".into(),
            })?;
        Ok(())
    }
    fn check_string(&self, value: &Value, span: Span) -> Result<(), Error> {
        if let Value::String(s) = value {
            if s.len() > self.limits.max_string_units {
                return Err(Error::Limit {
                    span,
                    message: "string length limit exceeded".into(),
                });
            }
        }
        Ok(())
    }

    fn instantiate<'a>(
        &mut self,
        statements: impl Iterator<Item = &'a Statement> + Clone,
        global: bool,
    ) -> Result<(), Error> {
        let scope = self
            .scopes
            .last_mut()
            .expect("a realm always has a global scope");
        // Check all global conflicts before creating any bindings.
        for statement in statements.clone() {
            if let StatementKind::Lexical { bindings, .. } = &statement.kind {
                for binding in bindings {
                    if scope.contains_key(&binding.name)
                        || (global && restricted_global(&binding.name))
                    {
                        return Err(Self::exception(
                            ExceptionKind::SyntaxError,
                            binding.span,
                            "conflicting lexical declaration",
                        ));
                    }
                }
            }
        }
        for statement in statements {
            if let StatementKind::Lexical { mutable, bindings } = &statement.kind {
                for binding in bindings {
                    scope.insert(
                        binding.name.clone(),
                        BindingState {
                            value: None,
                            mutable: *mutable,
                        },
                    );
                }
            }
        }
        Ok(())
    }

    fn initialize_bindings(&mut self, bindings: &[Binding]) -> Result<(), Error> {
        for binding in bindings {
            self.tick(binding.span)?;
            let value = if let Some(expr) = &binding.initializer {
                self.expression(expr)?
            } else {
                Value::Undefined
            };
            let scope = self.scopes.last_mut().expect("a realm always has a scope");
            scope
                .get_mut(&binding.name)
                .expect("declaration was instantiated")
                .value = Some(value);
        }
        Ok(())
    }

    fn statements(&mut self, statements: &[Statement]) -> Result<Completion, Error> {
        let mut completion = Completion::normal(None);
        for statement in statements {
            // ECMA-262 14.2.2: even an abrupt statement inherits the previous
            // value when its own value is empty, then stops the statement list.
            completion = self.statement(statement)?.update_empty(completion.value);
            if completion.kind != CompletionKind::Normal {
                break;
            }
        }
        Ok(completion)
    }
    fn statement(&mut self, statement: &Statement) -> Result<Completion, Error> {
        self.labelled_statement(statement, &[])
    }

    fn labelled_statement(
        &mut self,
        statement: &Statement,
        labels: &[&str],
    ) -> Result<Completion, Error> {
        self.tick(statement.span)?;
        match &statement.kind {
            // ECMA-262 14.16.1: no debugging facility is active in this host.
            StatementKind::Empty | StatementKind::Debugger => Ok(Completion::normal(None)),
            StatementKind::Expression(expr) => Ok(Completion::normal(Some(self.expression(expr)?))),
            StatementKind::Break(target) | StatementKind::Continue(target) => Ok(Completion {
                kind: if matches!(statement.kind, StatementKind::Break(_)) {
                    CompletionKind::Break
                } else {
                    CompletionKind::Continue
                },
                value: None,
                target: target.as_ref().map(|label| label.name.clone()),
            }),
            StatementKind::Labelled { label, body } => {
                // Only a chain of labels passes its label set into a loop.
                // Blocks and conditional branches call statement with a fresh set.
                let mut nested = labels.to_vec();
                nested.push(&label.name);
                let mut result = self.labelled_statement(body, &nested)?;
                if result.kind == CompletionKind::Break
                    && result.target.as_deref() == Some(&label.name)
                {
                    result.kind = CompletionKind::Normal;
                    result.target = None;
                }
                Ok(result)
            }
            StatementKind::Throw(expr) => Err(Error::Thrown(self.expression(expr)?)),
            StatementKind::Lexical { bindings, .. } => {
                self.initialize_bindings(bindings)?;
                Ok(Completion::normal(None))
            }
            StatementKind::Block(body) => {
                self.scopes.push(BTreeMap::new());
                let result = self
                    .instantiate(body.iter(), false)
                    .and_then(|()| self.statements(body));
                self.scopes.pop();
                result
            }
            StatementKind::Try {
                body,
                handler,
                finalizer,
            } => {
                let mut result = self.statement(body);
                // ECMA-262 14.15.2/14.15.3: catch without a parameter evaluates
                // its block only on a throw, without an extra binding environment.
                if result.as_ref().is_err_and(|e| e.is_language_exception()) {
                    if let Some(handler) = handler {
                        result = self.statement(handler);
                    }
                }
                // Host failures cannot enter or be suppressed by language control.
                if result.as_ref().is_err_and(|e| !e.is_language_exception()) {
                    return result;
                }
                // ECMA-262 14.15.3: a normal finalizer preserves the protected
                // completion, ignoring its own value. An abrupt one replaces it.
                if let Some(finalizer) = finalizer {
                    let finalizer_result = self.statement(finalizer)?;
                    if finalizer_result.kind != CompletionKind::Normal {
                        result = Ok(finalizer_result);
                    }
                }
                result.map(|c| c.update_empty(Some(Value::Undefined)))
            }
            StatementKind::Switch {
                discriminant,
                clauses,
            } => {
                // ECMA-262 14.12.4 evaluates the discriminant before creating
                // the shared case-block environment. Selectors use that new scope.
                let input = self.expression(discriminant)?;
                self.scopes.push(BTreeMap::new());
                let result = self
                    .instantiate(
                        clauses.iter().flat_map(|clause| clause.statements.iter()),
                        false,
                    )
                    .and_then(|()| self.case_block(clauses, &input));
                self.scopes.pop();
                result.map(Completion::consume_unlabelled_break)
            }
            // ECMA-262 14.7.3.2: the result is the last non-empty body value,
            // initially undefined. Condition values never replace it.
            StatementKind::While { test, body } => {
                let mut value = Value::Undefined;
                while self.expression(test)?.to_boolean() {
                    let result = self.statement(body)?.update_empty(Some(value));
                    if !result.loop_continues(labels) {
                        return Ok(result.consume_unlabelled_break());
                    }
                    value = result.value.expect("loop UpdateEmpty supplies a value");
                }
                Ok(Completion::normal(Some(value)))
            }
            // ECMA-262 14.7.2.2: the first body precedes the first condition.
            StatementKind::DoWhile { body, test } => {
                let mut value = Value::Undefined;
                loop {
                    let result = self.statement(body)?.update_empty(Some(value));
                    if !result.loop_continues(labels) {
                        return Ok(result.consume_unlabelled_break());
                    }
                    value = result.value.expect("loop UpdateEmpty supplies a value");
                    if !self.expression(test)?.to_boolean() {
                        return Ok(Completion::normal(Some(value)));
                    }
                }
            }
            StatementKind::For {
                initializer,
                test,
                update,
                body,
            } => {
                // ECMA-262 14.7.4.2: all header bindings exist before any
                // initializer runs, and the outer scope is restored on every exit.
                match initializer {
                    Some(ForInitializer::Lexical { mutable, bindings }) => {
                        let scope = bindings
                            .iter()
                            .map(|binding| {
                                (
                                    binding.name.clone(),
                                    BindingState {
                                        value: None,
                                        mutable: *mutable,
                                    },
                                )
                            })
                            .collect();
                        self.scopes.push(scope);
                        let per_iteration = if *mutable { bindings.as_slice() } else { &[] };
                        let result = self.initialize_bindings(bindings).and_then(|()| {
                            self.for_body(
                                test.as_ref(),
                                update.as_ref(),
                                body,
                                per_iteration,
                                labels,
                            )
                        });
                        self.scopes.pop();
                        result
                    }
                    _ => {
                        if let Some(ForInitializer::Expression(expr)) = initializer {
                            self.expression(expr)?;
                        }
                        self.for_body(test.as_ref(), update.as_ref(), body, &[], labels)
                    }
                }
            }
            StatementKind::If {
                test,
                consequent,
                alternate,
            } => {
                let result = if self.expression(test)?.to_boolean() {
                    self.statement(consequent)?
                } else if let Some(alternate) = alternate {
                    self.statement(alternate)?
                } else {
                    Completion::normal(None)
                };
                // ECMA-262 14.6.2 applies UpdateEmpty even to break/continue.
                Ok(result.update_empty(Some(Value::Undefined)))
            }
        }
    }

    // ECMA-262 14.12.2/14.12.3: locate the first strictly equal selector,
    // skipping default during the search. After a match, fall through in source
    // order without evaluating any further selectors. Only when no selector
    // matches does execution begin at default (if present).
    fn case_block(&mut self, clauses: &[SwitchClause], input: &Value) -> Result<Completion, Error> {
        let mut selected = None;
        let mut default = None;
        for (index, clause) in clauses.iter().enumerate() {
            self.tick(clause.span)?;
            if let Some(test) = &clause.test {
                if input.strictly_equal(&self.expression(test)?) {
                    selected = Some(index);
                    break;
                }
            } else {
                default = Some(index);
            }
        }
        let mut value = Value::Undefined;
        if let Some(start) = selected.or(default) {
            for clause in &clauses[start..] {
                // Empty fall-through clauses still consume the host work budget.
                self.tick(clause.span)?;
                let result = self
                    .statements(&clause.statements)?
                    .update_empty(Some(value));
                if result.kind != CompletionKind::Normal {
                    return Ok(result);
                }
                value = result
                    .value
                    .expect("case-block UpdateEmpty supplies a value");
            }
        }
        Ok(Completion::normal(Some(value)))
    }

    // ECMA-262 14.7.4.3 ForBodyEvaluation. The body's completion determines
    // whether the update runs, and only body values contribute to the result.
    fn for_body(
        &mut self,
        test: Option<&Expr>,
        update: Option<&Expr>,
        body: &Statement,
        per_iteration: &[Binding],
        labels: &[&str],
    ) -> Result<Completion, Error> {
        let mut value = Value::Undefined;
        self.create_per_iteration_environment(per_iteration)?;
        loop {
            if let Some(test) = test {
                if !self.expression(test)?.to_boolean() {
                    return Ok(Completion::normal(Some(value)));
                }
            }
            // Statement evaluation consumes host budget even for `for (;;) ;`.
            let result = self.statement(body)?.update_empty(Some(value));
            if !result.loop_continues(labels) {
                return Ok(result.consume_unlabelled_break());
            }
            value = result.value.expect("loop UpdateEmpty supplies a value");
            self.create_per_iteration_environment(per_iteration)?;
            if let Some(update) = update {
                self.expression(update)?;
            }
        }
    }

    // ECMA-262 14.7.4.4: copy let values into a fresh environment with the
    // same outer environment. Const declarations do not request this operation.
    fn create_per_iteration_environment(&mut self, bindings: &[Binding]) -> Result<(), Error> {
        if bindings.is_empty() {
            return Ok(());
        }
        let mut next = BTreeMap::new();
        for binding in bindings {
            self.tick(binding.span)?;
            let scope = self.scopes.last().expect("loop environment exists");
            let value = scope[&binding.name]
                .value
                .clone()
                .expect("loop binding is initialized");
            next.insert(
                binding.name.clone(),
                BindingState {
                    value: Some(value),
                    mutable: true,
                },
            );
        }
        *self.scopes.last_mut().expect("loop environment exists") = next;
        Ok(())
    }

    fn resolve<'a>(&self, name: &'a str) -> Reference<'a> {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Reference::Lexical(index, name);
            }
        }
        if self.globals.contains_key(name) {
            Reference::Global(name)
        } else if standard_global(name) {
            Reference::UnsupportedGlobal(name)
        } else {
            Reference::Unresolvable(name)
        }
    }
    fn get(&self, reference: &Reference<'_>, span: Span) -> Result<Value, Error> {
        match reference {
            Reference::Lexical(index, name) => {
                self.scopes[*index][*name].value.clone().ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::ReferenceError,
                        span,
                        format!("{name} is uninitialized"),
                    )
                })
            }
            Reference::Global(name) => Ok(self.globals[*name].clone()),
            Reference::Unresolvable(name) => Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is not defined"),
            )),
            Reference::UnsupportedGlobal(name) => Err(Self::unsupported(
                span,
                format!("{name} is not implemented"),
            )),
        }
    }
    fn put(&mut self, reference: Reference<'_>, value: Value, span: Span) -> Result<(), Error> {
        match reference {
            Reference::Lexical(index, name) => {
                let binding = self.scopes[index]
                    .get_mut(name)
                    .expect("resolved binding exists");
                if binding.value.is_none() {
                    return Err(Self::exception(
                        ExceptionKind::ReferenceError,
                        span,
                        format!("{name} is uninitialized"),
                    ));
                }
                if !binding.mutable {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        format!("{name} is immutable"),
                    ));
                }
                binding.value = Some(value);
            }
            Reference::Global(name) if restricted_global(name) => {
                if self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        format!("{name} is not writable"),
                    ));
                }
            }
            Reference::Unresolvable(name) if self.strict => {
                return Err(Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    format!("{name} is not defined"),
                ));
            }
            Reference::Global(name) | Reference::Unresolvable(name) => {
                self.globals.insert(name.to_owned(), value);
            }
            Reference::UnsupportedGlobal(name) => {
                return Err(Self::unsupported(
                    span,
                    format!("{name} is not implemented"),
                ));
            }
        }
        Ok(())
    }

    fn expression(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.tick(expr.span)?;
        let result = match &expr.kind {
            ExprKind::Literal(literal) => match literal {
                Literal::Null => Value::Null,
                Literal::Boolean(v) => Value::Boolean(*v),
                Literal::Number(v) => Value::Number(*v),
                Literal::String(v) => Value::String(v.clone()),
            },
            ExprKind::Identifier(name) => self.get(&self.resolve(name), expr.span)?,
            ExprKind::Parenthesized(inner) => self.expression(inner)?,
            ExprKind::Assign(name, right) => {
                let reference = self.resolve(name);
                let value = self.expression(right)?;
                self.put(reference, value.clone(), expr.span)?;
                value
            }
            ExprKind::CompoundAssign(op, name, right) => {
                // ECMA-262 13.15.2: read the reference before the RHS. Logical
                // assignments that short-circuit perform neither RHS nor PutValue.
                let reference = self.resolve(name);
                let left = self.get(&reference, expr.span)?;
                match op {
                    BinaryOp::And if !left.to_boolean() => left,
                    BinaryOp::Or if left.to_boolean() => left,
                    BinaryOp::Nullish if !matches!(left, Value::Null | Value::Undefined) => left,
                    _ => {
                        let right = self.expression(right)?;
                        let value = self.binary(*op, left, right, expr.span)?;
                        self.put(reference, value.clone(), expr.span)?;
                        value
                    }
                }
            }
            ExprKind::Update {
                op,
                argument,
                prefix,
            } => {
                // ECMA-262 13.4.2–13.4.5: GetValue and ToNumeric precede
                // PutValue. Postfix returns the numeric old value, not its input.
                let name = identifier(argument).expect("parser checked update target");
                let reference = self.resolve(name);
                let old = self.get(&reference, argument.span)?.to_number();
                let new = match op {
                    UpdateOp::Increment => old + 1.0,
                    UpdateOp::Decrement => old - 1.0,
                };
                self.put(reference, Value::Number(new), expr.span)?;
                Value::Number(if *prefix { new } else { old })
            }
            ExprKind::Conditional(test, yes, no) => {
                if self.expression(test)?.to_boolean() {
                    self.expression(yes)?
                } else {
                    self.expression(no)?
                }
            }
            ExprKind::Unary(op, inner) => {
                if *op == UnaryOp::Typeof {
                    if let Some(name) = identifier(inner) {
                        if matches!(self.resolve(name), Reference::Unresolvable(_)) {
                            let value = Value::String(JsString::from("undefined"));
                            self.check_string(&value, expr.span)?;
                            return Ok(value);
                        }
                    }
                }
                let value = self.expression(inner)?;
                match op {
                    UnaryOp::Plus => Value::Number(value.to_number()),
                    UnaryOp::Minus => Value::Number(-value.to_number()),
                    UnaryOp::Not => Value::Boolean(!value.to_boolean()),
                    UnaryOp::BitNot => {
                        Value::Number((!(to_uint32(value.to_number()) as i32)) as f64)
                    }
                    UnaryOp::Void => Value::Undefined,
                    UnaryOp::Typeof => Value::String(JsString::from(value.type_name())),
                }
            }
            ExprKind::Binary(op, left, right) => {
                let left = self.expression(left)?;
                match op {
                    BinaryOp::And if !left.to_boolean() => left,
                    BinaryOp::Or if left.to_boolean() => left,
                    BinaryOp::Nullish if !matches!(left, Value::Null | Value::Undefined) => left,
                    _ => {
                        let right = self.expression(right)?;
                        self.binary(*op, left, right, expr.span)?
                    }
                }
            }
        };
        self.check_string(&result, expr.span)?;
        Ok(result)
    }

    fn binary(&self, op: BinaryOp, left: Value, right: Value, span: Span) -> Result<Value, Error> {
        use BinaryOp::*;
        match op {
            And | Or | Nullish | Comma => return Ok(right),
            StrictEqual => return Ok(Value::Boolean(left.strictly_equal(&right))),
            StrictNotEqual => return Ok(Value::Boolean(!left.strictly_equal(&right))),
            Equal => return Ok(Value::Boolean(left.loosely_equal(&right))),
            NotEqual => return Ok(Value::Boolean(!left.loosely_equal(&right))),
            Add if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) => {
                let a = left.to_js_string();
                let b = right.to_js_string();
                if a.len()
                    .checked_add(b.len())
                    .is_none_or(|length| length > self.limits.max_string_units)
                {
                    return Err(Error::Limit {
                        span,
                        message: "string length limit exceeded".into(),
                    });
                }
                return Ok(Value::String(a.concat(&b)));
            }
            Less | LessEqual | Greater | GreaterEqual => {
                let order = match (&left, &right) {
                    (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
                    _ => left.to_number().partial_cmp(&right.to_number()),
                };
                return Ok(Value::Boolean(order.is_some_and(|order| match op {
                    Less => order.is_lt(),
                    LessEqual => order.is_le(),
                    Greater => order.is_gt(),
                    GreaterEqual => order.is_ge(),
                    _ => unreachable!(),
                })));
            }
            _ => {}
        }
        let a = left.to_number();
        let b = right.to_number();
        Ok(Value::Number(match op {
            Add => a + b,
            Subtract => a - b,
            Multiply => a * b,
            Divide => a / b,
            Remainder => a % b,
            Exponentiate => exponentiate(a, b),
            BitAnd => ((to_uint32(a) & to_uint32(b)) as i32) as f64,
            BitOr => ((to_uint32(a) | to_uint32(b)) as i32) as f64,
            BitXor => ((to_uint32(a) ^ to_uint32(b)) as i32) as f64,
            LeftShift => ((to_uint32(a) << (to_uint32(b) & 31)) as i32) as f64,
            RightShift => ((to_uint32(a) as i32) >> (to_uint32(b) & 31)) as f64,
            UnsignedRightShift => (to_uint32(a) >> (to_uint32(b) & 31)) as f64,
            _ => unreachable!("non-numeric operators returned above"),
        }))
    }
}

fn identifier(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name),
        ExprKind::Parenthesized(e) => identifier(e),
        _ => None,
    }
}
fn restricted_global(name: &str) -> bool {
    matches!(name, "undefined" | "NaN" | "Infinity")
}
fn standard_global(name: &str) -> bool {
    matches!(
        name,
        "globalThis"
            | "eval"
            | "isFinite"
            | "isNaN"
            | "parseFloat"
            | "parseInt"
            | "decodeURI"
            | "decodeURIComponent"
            | "encodeURI"
            | "encodeURIComponent"
            | "Object"
            | "Function"
            | "Boolean"
            | "Symbol"
            | "Error"
            | "AggregateError"
            | "EvalError"
            | "RangeError"
            | "ReferenceError"
            | "SyntaxError"
            | "TypeError"
            | "URIError"
            | "Number"
            | "BigInt"
            | "Math"
            | "Date"
            | "String"
            | "RegExp"
            | "Array"
            | "Int8Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Int16Array"
            | "Uint16Array"
            | "Int32Array"
            | "Uint32Array"
            | "BigInt64Array"
            | "BigUint64Array"
            | "Float16Array"
            | "Float32Array"
            | "Float64Array"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "ArrayBuffer"
            | "SharedArrayBuffer"
            | "DataView"
            | "Atomics"
            | "JSON"
            | "WeakRef"
            | "FinalizationRegistry"
            | "Iterator"
            | "Promise"
            | "Reflect"
            | "Proxy"
            | "DisposableStack"
            | "AsyncDisposableStack"
            | "SuppressedError"
    )
}
