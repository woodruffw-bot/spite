//! A tree-walking interpreter for the implemented ECMAScript subset.

pub mod object;
mod value;
pub use value::Value;

use spite_bigint::{BigInt, BitwiseOp, Budget, Error as IntegerError};
use spite_core::{Diagnostic, JsString, Span};
use spite_parser::{ast::*, parse_script};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};
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
    /// A numeric argument is outside an operation's domain.
    RangeError,
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
    /// Maximum work per Script, including bindings, clauses, and integer arithmetic.
    pub max_steps: usize,
    /// Maximum code units in any produced string.
    pub max_string_units: usize,
    /// Maximum magnitude bits in a produced BigInt.
    pub max_bigint_bits: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: 100_000,
            max_string_units: 1024 * 1024,
            max_bigint_bits: 65_536,
        }
    }
}

#[derive(Debug)]
struct BindingState {
    value: Option<Value>,
    mutable: bool,
}

#[derive(Debug)]
struct GlobalBinding {
    value: Value,
    deletable: bool,
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
    globals: BTreeMap<String, GlobalBinding>,
    unsupported_host_globals: BTreeSet<String>,
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
            globals: [
                ("undefined", Value::Undefined),
                ("NaN", Value::Number(f64::NAN)),
                ("Infinity", Value::Number(f64::INFINITY)),
            ]
            .into_iter()
            .map(|(name, value)| {
                (
                    name.into(),
                    GlobalBinding {
                        value,
                        deletable: false,
                    },
                )
            })
            .collect(),
            unsupported_host_globals: BTreeSet::new(),
            limits,
            remaining_steps: 0,
            strict: false,
        }
    }

    /// Marks an unavailable host global so access reports Unsupported, not a
    /// misleading ReferenceError or an undefined result from typeof.
    ///
    /// This does not install a JavaScript value. Lexical bindings can shadow the
    /// name, and existing implemented globals retain their normal behavior.
    /// Ordinary realms reserve no extra host names.
    pub fn reserve_unsupported_global(&mut self, name: impl Into<String>) {
        self.unsupported_host_globals.insert(name.into());
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
        self.instantiate_global(script)?;
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

    fn integer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, IntegerError>,
    ) -> Result<T, Error> {
        let mut budget = Budget::new(self.limits.max_bigint_bits, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| match error {
            IntegerError::Limit => Error::Limit {
                span,
                message: "integer resource limit exceeded".into(),
            },
            IntegerError::DivisionByZero | IntegerError::NegativeExponent => {
                Self::exception(ExceptionKind::RangeError, span, error.to_string())
            }
            IntegerError::InvalidDigit | IntegerError::InvalidRadix => {
                unreachable!("integer input was validated before conversion")
            }
        })
    }

    fn number(value: &Value, span: Span) -> Result<f64, Error> {
        value
            .to_number()
            .map_err(|kind| Self::exception(kind, span, "cannot convert BigInt to Number"))
    }

    fn numeric(value: Value, span: Span) -> Result<Value, Error> {
        match value {
            Value::BigInt(_) | Value::Number(_) => Ok(value),
            other => Ok(Value::Number(Self::number(&other, span)?)),
        }
    }

    fn comparison_work(&mut self, left: &Value, right: &Value, span: Span) -> Result<(), Error> {
        let words = [left, right]
            .into_iter()
            .map(|value| match value {
                Value::BigInt(value) => value.bit_length().div_ceil(32),
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        self.integer_work(span, |budget| budget.charge(words))
    }

    fn strictly_equal(&mut self, left: &Value, right: &Value, span: Span) -> Result<bool, Error> {
        self.comparison_work(left, right, span)?;
        Ok(left.strictly_equal(right))
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

    fn append_string(
        &self,
        units: &mut Vec<u16>,
        part: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        if units
            .len()
            .checked_add(part.len())
            .is_none_or(|len| len > self.limits.max_string_units)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        units.extend_from_slice(part.code_units());
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
                        || (global
                            && self
                                .globals
                                .get(&binding.name)
                                .is_some_and(|b| !b.deletable))
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

    fn instantiate_global(&mut self, script: &Script) -> Result<(), Error> {
        // ECMA-262 16.1.7: check conflicts before creating any new bindings.
        // All Script vars are instantiated, including vars in unreachable code.
        let declarations = script.var_declarations();
        for binding in &declarations {
            self.tick(binding.span)?;
            if self.scopes[0].contains_key(&binding.name) {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    binding.span,
                    "var declaration conflicts with global lexical binding",
                ));
            }
            if (standard_global(&binding.name)
                || self.unsupported_host_globals.contains(&binding.name))
                && !self.globals.contains_key(&binding.name)
            {
                return Err(Self::unsupported(
                    binding.span,
                    format!("{} is not implemented", binding.name),
                ));
            }
        }
        self.instantiate(script.statements().iter(), true)?;
        for binding in declarations {
            // CreateGlobalVarBinding preserves existing properties. Only new
            // properties become non-deletable (ECMA-262 9.1.1.4.16).
            self.globals
                .entry(binding.name.clone())
                .or_insert(GlobalBinding {
                    value: Value::Undefined,
                    deletable: false,
                });
        }
        Ok(())
    }

    fn evaluate_var_bindings(&mut self, bindings: &[Binding]) -> Result<(), Error> {
        for binding in bindings {
            self.tick(binding.span)?;
            // ECMA-262 14.3.2.1: a declaration without an initializer does not
            // assign, and an initializer resolves its reference before the RHS.
            if let Some(expr) = &binding.initializer {
                let reference = self.resolve(&binding.name);
                let value = self.expression(expr)?;
                self.put(reference, value, binding.span)?;
            }
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
            StatementKind::Var(bindings) => {
                self.evaluate_var_bindings(bindings)?;
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
                let mut result = match self.statement(body) {
                    Err(error) if error.is_language_exception() => match handler {
                        Some(handler) => self.catch_clause(handler, error),
                        None => Err(error),
                    },
                    other => other,
                };
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
                    Some(ForInitializer::Var(bindings)) => {
                        self.evaluate_var_bindings(bindings)?;
                        self.for_body(test.as_ref(), update.as_ref(), body, &[], labels)
                    }
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
                let candidate = self.expression(test)?;
                if self.strictly_equal(input, &candidate, test.span)? {
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
        } else if standard_global(name) || self.unsupported_host_globals.contains(name) {
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
            Reference::Global(name) => Ok(self.globals[*name].value.clone()),
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
                self.globals
                    .entry(name.to_owned())
                    .or_insert(GlobalBinding {
                        value: Value::Undefined,
                        deletable: true,
                    })
                    .value = value;
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

    // ECMA-262 14.15.2: the parameter environment encloses the block environment
    // and is restored before any pending finally or outer handler executes.
    fn catch_clause(&mut self, handler: &CatchClause, error: Error) -> Result<Completion, Error> {
        let Some(parameter) = &handler.parameter else {
            return self.statement(&handler.body);
        };
        let value = match error {
            Error::Thrown(value) => value,
            Error::Exception { .. } => {
                return Err(Self::unsupported(
                    handler.span,
                    "binding built-in exceptions requires JavaScript Error objects",
                ));
            }
            _ => unreachable!("only language throws enter a catch clause"),
        };
        self.tick(parameter.span)?;
        self.scopes.push(BTreeMap::from([(
            parameter.name.clone(),
            BindingState {
                value: Some(value),
                mutable: true,
            },
        )]));
        let result = self.statement(&handler.body);
        self.scopes.pop();
        result
    }

    fn expression(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.tick(expr.span)?;
        let result = match &expr.kind {
            ExprKind::Template {
                elements,
                substitutions,
            } => {
                // ECMA-262 13.2.8.6: each substitution is evaluated and converted
                // with ToString before the next one. Raw text is not evaluated.
                let mut units = Vec::new();
                for (index, element) in elements.iter().enumerate() {
                    self.tick(element.span)?;
                    self.append_string(
                        &mut units,
                        element
                            .cooked
                            .as_ref()
                            .expect("validated untagged template"),
                        element.span,
                    )?;
                    if let Some(substitution) = substitutions.get(index) {
                        let value = self.expression(substitution)?;
                        let value = self
                            .integer_work(substitution.span, |budget| value.to_js_string(budget))?;
                        self.append_string(&mut units, &value, substitution.span)?;
                    }
                }
                Value::String(JsString::from_code_units(units))
            }
            ExprKind::Literal(literal) => match literal {
                Literal::Null => Value::Null,
                Literal::Boolean(v) => Value::Boolean(*v),
                Literal::Number(v) => Value::Number(*v),
                Literal::BigInt { digits, radix } => {
                    Value::BigInt(self.integer_work(expr.span, |budget| {
                        BigInt::parse_digits(digits, *radix, budget)
                    })?)
                }
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
                let old = Self::numeric(self.get(&reference, argument.span)?, argument.span)?;
                let one = if matches!(old, Value::BigInt(_)) {
                    Value::BigInt(BigInt::from(1))
                } else {
                    Value::Number(1.0)
                };
                let operation = match op {
                    UpdateOp::Increment => BinaryOp::Add,
                    UpdateOp::Decrement => BinaryOp::Subtract,
                };
                let new = self.binary(operation, old.clone(), one, expr.span)?;
                self.put(reference, new.clone(), expr.span)?;
                if *prefix { new } else { old }
            }
            ExprKind::Conditional(test, yes, no) => {
                if self.expression(test)?.to_boolean() {
                    self.expression(yes)?
                } else {
                    self.expression(no)?
                }
            }
            ExprKind::Unary(op, inner) => {
                if *op == UnaryOp::Delete {
                    if let Some(name) = identifier(inner) {
                        // ECMA-262 13.5.1.2: deleting an environment reference
                        // does not GetValue, even for an uninitialized binding.
                        let deleted = match self.resolve(name) {
                            Reference::Lexical(..) => false,
                            Reference::Global(name) if !self.globals[name].deletable => false,
                            Reference::Global(name) => {
                                self.globals.remove(name);
                                true
                            }
                            Reference::Unresolvable(_) => true,
                            Reference::UnsupportedGlobal(name) => {
                                return Err(Self::unsupported(
                                    inner.span,
                                    format!("{name} is not implemented"),
                                ));
                            }
                        };
                        return Ok(Value::Boolean(deleted));
                    }
                }
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
                    UnaryOp::Plus => Value::Number(Self::number(&value, expr.span)?),
                    UnaryOp::Minus => match value {
                        Value::BigInt(value) => {
                            Value::BigInt(self.integer_work(expr.span, |budget| value.neg(budget))?)
                        }
                        other => Value::Number(-Self::number(&other, expr.span)?),
                    },
                    UnaryOp::Not => Value::Boolean(!value.to_boolean()),
                    UnaryOp::BitNot => match value {
                        Value::BigInt(value) => Value::BigInt(
                            self.integer_work(expr.span, |budget| value.bitnot(budget))?,
                        ),
                        other => Value::Number(
                            (!(to_uint32(Self::number(&other, expr.span)?) as i32)) as f64,
                        ),
                    },
                    UnaryOp::Void => Value::Undefined,
                    UnaryOp::Typeof => Value::String(JsString::from(value.type_name())),
                    UnaryOp::Delete => Value::Boolean(true),
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

    fn binary(
        &mut self,
        op: BinaryOp,
        left: Value,
        right: Value,
        span: Span,
    ) -> Result<Value, Error> {
        use BinaryOp::*;
        match op {
            And | Or | Nullish | Comma => return Ok(right),
            StrictEqual | StrictNotEqual => {
                let equal = self.strictly_equal(&left, &right, span)?;
                return Ok(Value::Boolean(if op == StrictEqual {
                    equal
                } else {
                    !equal
                }));
            }
            Equal | NotEqual => {
                self.comparison_work(&left, &right, span)?;
                let equal = self.integer_work(span, |budget| left.loosely_equal(&right, budget))?;
                return Ok(Value::Boolean(if op == Equal { equal } else { !equal }));
            }
            Add if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) => {
                let a = self.integer_work(span, |budget| left.to_js_string(budget))?;
                let b = self.integer_work(span, |budget| right.to_js_string(budget))?;
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
                self.comparison_work(&left, &right, span)?;
                let order = self.integer_work(span, |budget| left.compare(&right, budget))?;
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
        if let (Value::BigInt(a), Value::BigInt(b)) = (&left, &right) {
            if op == UnsignedRightShift {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "BigInt has no unsigned right shift",
                ));
            }
            return self
                .integer_work(span, |budget| match op {
                    Add => a.add(b, budget),
                    Subtract => a.sub(b, budget),
                    Multiply => a.mul(b, budget),
                    Divide => a.div_rem(b, budget).map(|(quotient, _)| quotient),
                    Remainder => a.div_rem(b, budget).map(|(_, remainder)| remainder),
                    Exponentiate => a.pow(b, budget),
                    BitAnd => a.bitwise(b, BitwiseOp::And, budget),
                    BitOr => a.bitwise(b, BitwiseOp::Or, budget),
                    BitXor => a.bitwise(b, BitwiseOp::Xor, budget),
                    LeftShift => a.shl(b, budget),
                    RightShift => a.shr(b, budget),
                    _ => unreachable!("non-numeric operators returned above"),
                })
                .map(Value::BigInt);
        }
        // ToNumeric preserves BigInt. Arithmetic requires equal numeric types.
        if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot mix BigInt and Number operands",
            ));
        }
        let a = Self::number(&left, span)?;
        let b = Self::number(&right, span)?;
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
