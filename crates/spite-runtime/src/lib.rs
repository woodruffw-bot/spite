//! A tree-walking interpreter for the implemented ECMAScript subset.

#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod test_support;

mod assignment_pattern;
mod binding;
mod environment;
mod eval;
mod for_in;
mod for_of;
mod function;
mod global;
mod iterator_count;
mod optional_chain;
mod super_property;
mod with;
use environment::{BindingState, EnvironmentHandle};
pub mod object;
mod realm_object;
mod value;
pub use realm_object::RootedValue;
pub use spite_heap::{Collection, Handle as ObjectHandle};
pub use value::{ConversionError, Value};

use realm_object::Hint;
use spite_bigint::{BigInt, BitwiseOp, Budget, Error as IntegerError};
use spite_core::{Diagnostic, JsString, Span};
use spite_parser::{ast::*, parse_script, parse_script_with_source_limit};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};
use value::{exponentiate, to_uint32};

/// Built-in error categories produced by implemented runtime operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExceptionKind {
    /// A runtime syntax failure, such as an invalid declaration or integer string.
    SyntaxError,
    /// An unresolvable or uninitialized binding.
    ReferenceError,
    /// An operation forbidden by a value or binding's type.
    TypeError,
    /// A numeric argument is outside an operation's domain.
    RangeError,
    /// URI encoding or decoding encountered malformed Unicode or escape syntax.
    URIError,
}

/// An evaluation failure. Host failures are distinct from JavaScript exceptions.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An embedding supplied a foreign, stale, or non-object handle.
    InvalidObject(object::Error),
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
            Self::InvalidObject(_)
            | Self::Parse(_)
            | Self::Unsupported { .. }
            | Self::Limit { .. } => false,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject(error) => write!(f, "invalid embedding object: {error}"),
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

/// Optional host resource quotas. All default to `None` (no quota).
/// They do not alter ECMAScript exceptions or disable platform safety checks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Limits {
    /// Optional maximum encoded bytes in Script, eval, or dynamic Function source.
    /// Scalars use UTF-8 lengths; each lone surrogate occupies three bytes.
    pub max_source_bytes: Option<usize>,
    /// Optional maximum work per Script, including bindings, clauses, and arithmetic.
    /// Defaults to `None`, which disables the execution work limit.
    /// Realm initialization is outside this per-Script allowance.
    pub max_steps: Option<usize>,
    /// Optional maximum code units in any produced string.
    pub max_string_units: Option<usize>,
    /// Optional maximum magnitude bits in a produced BigInt.
    pub max_bigint_bits: Option<usize>,
    /// Optional maximum values in a call argument list, including apply and bound arguments.
    pub max_arguments: Option<usize>,
    /// Optional maximum heap slots shared by objects and lexical environments.
    pub max_heap_entries: Option<usize>,
    /// Optional maximum own properties in each ordinary object.
    pub max_properties: Option<usize>,
}

// Resolve references before evaluating assignment RHS expressions. GetValue and
// PutValue are separate operations, including the unresolvable typeof case.
enum Reference<'a> {
    Value(Value),
    Lexical(EnvironmentHandle, &'a str),
    ObjectBinding {
        object: ObjectHandle,
        name: &'a str,
    },
    Global(&'a str),
    Unresolvable(&'a str),
    UnsupportedGlobal(&'a str),
    Property {
        base: Value,
        // Edition 17 converts a computed name at GetValue/PutValue, not when
        // creating the reference. GetValue caches the converted property key.
        key: Value,
    },
    SuperProperty {
        base: Value,
        key: Value,
        this_value: Value,
    },
}

impl Reference<'_> {
    fn is_eval_environment_reference(&self) -> bool {
        matches!(
            self,
            Self::Lexical(_, "eval")
                | Self::ObjectBinding { name: "eval", .. }
                | Self::Global("eval")
        )
    }
    // ECMA-262 13.3.6.2 / 9.1.1.2.10: environment calls use WithBaseObject.
    fn call_receiver(self) -> Value {
        match self {
            Self::Property { base, .. } => base,
            Self::SuperProperty { this_value, .. } => this_value,
            Self::ObjectBinding { object, .. } => Value::Object(object),
            _ => Value::Undefined,
        }
    }
}

// Implemented statement completions. Throws already carry a non-empty value in
// Error::Thrown; host failures remain separate from these control transfers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompletionKind {
    Normal,
    Break,
    Continue,
    Return,
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
/// Only the documented subset is implemented. The ordinary global object exposes
/// implemented standard bindings; host extensions are not installed.
/// Execution uses bounded Rust recursion and requires a native thread stack of
/// at least 2 MiB. Call re-entry is capped at 32 until explicit execution frames
/// replace native recursion.
#[derive(Debug)]
pub struct Realm {
    scopes: Vec<EnvironmentHandle>,
    // Lexical scopes may nest without changing the current VariableEnvironment.
    variable_environment: Option<EnvironmentHandle>,
    global_object: Option<ObjectHandle>,
    unsupported_host_globals: BTreeSet<String>,
    limits: Limits,
    remaining_steps: Option<usize>,
    call_depth: usize,
    evaluation_depth: usize,
    strict: bool,
    objects: object::Objects,
    intrinsics: Option<function::Intrinsics>,
    template_map: Vec<(Rc<[TemplateElement]>, ObjectHandle)>,
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
            scopes: Vec::new(),
            variable_environment: None,
            global_object: None,
            unsupported_host_globals: BTreeSet::new(),
            limits,
            remaining_steps: limits.max_steps,
            call_depth: 0,
            evaluation_depth: 0,
            strict: false,
            objects: object::Objects::with_limits(limits.max_heap_entries, limits.max_properties),
            intrinsics: None,
            template_map: Vec::new(),
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
    /// Returned and thrown object values are unrooted. Use [`Self::root_value`]
    /// to retain them across explicit collection. Evaluation never collects.
    pub fn eval(&mut self, source: &str) -> Result<Value, Error> {
        let script = match self.limits.max_source_bytes {
            Some(limit) => parse_script_with_source_limit(source, limit),
            None => parse_script(source),
        }
        .map_err(Error::Parse)?;
        self.evaluate(&script)
    }

    /// Evaluates a Script already validated by the parser.
    pub fn evaluate(&mut self, script: &Script) -> Result<Value, Error> {
        self.initialize_realm()?;
        self.variable_environment = self.scopes.first().cloned();
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
        if let Some(remaining) = &mut self.remaining_steps {
            *remaining = remaining.checked_sub(1).ok_or_else(|| Error::Limit {
                span,
                message: "evaluation step limit exceeded".into(),
            })?;
        }
        Ok(())
    }

    fn integer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, IntegerError>,
    ) -> Result<T, Error> {
        let mut budget = Budget::with_limits(self.limits.max_bigint_bits, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::integer_error(error, span))
    }

    fn integer_error(error: IntegerError, span: Span) -> Error {
        match error {
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
        }
    }

    fn conversion_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, ConversionError>,
    ) -> Result<T, Error> {
        let mut budget = Budget::with_limits(self.limits.max_bigint_bits, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::conversion_error(error, span))
    }

    fn conversion_error(error: ConversionError, span: Span) -> Error {
        match error {
            ConversionError::BigIntToNumber => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert BigInt to Number",
            ),
            ConversionError::SymbolToNumber => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert Symbol to Number",
            ),
            ConversionError::SymbolToString => Self::exception(
                ExceptionKind::TypeError,
                span,
                "cannot convert Symbol to String",
            ),
            ConversionError::ObjectNeedsContext => Self::unsupported(
                span,
                "object conversion requires ToPrimitive and callable hooks",
            ),
            ConversionError::Integer(error) => Self::integer_error(error, span),
        }
    }

    fn number(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        self.primitive(value, Hint::Number, span)?
            .to_number()
            .map_err(|error| Self::conversion_error(error, span))
    }

    fn numeric(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        match self.primitive(value, Hint::Number, span)? {
            value @ (Value::BigInt(_) | Value::Number(_)) => Ok(value),
            other => Ok(Value::Number(self.number(other, span)?)),
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
            if self
                .limits
                .max_string_units
                .is_some_and(|limit| s.len() > limit)
            {
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
        if units.len().checked_add(part.len()).is_none_or(|len| {
            self.limits
                .max_string_units
                .is_some_and(|limit| len > limit)
        }) {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        units.extend_from_slice(part.code_units());
        Ok(())
    }

    fn push_scope(
        &mut self,
        bindings: BTreeMap<String, BindingState>,
        span: Span,
    ) -> Result<(), Error> {
        let outer = self.scopes.last().cloned();
        let scope = self.object_work(span, |objects, budget| {
            objects.create_environment(outer, bindings, budget)
        })?;
        self.scopes.push(scope);
        Ok(())
    }

    fn check_lexical_conflicts<'a>(
        &mut self,
        statements: impl Iterator<Item = &'a Statement>,
        global: bool,
        functions_lexical: bool,
    ) -> Result<(), Error> {
        let handle = self.scopes.last().expect("active environment").clone();
        for statement in statements {
            for (name, span, _) in lexical_declarations(statement, functions_lexical) {
                if self
                    .objects
                    .environment(&handle)
                    .expect("active environment")
                    .bindings
                    .contains_key(name)
                    || (global && self.restricted_global_property(name, span)?)
                {
                    return Err(Self::exception(
                        ExceptionKind::SyntaxError,
                        span,
                        "conflicting lexical declaration",
                    ));
                }
            }
        }
        Ok(())
    }

    fn instantiate<'a>(
        &mut self,
        statements: impl DoubleEndedIterator<Item = &'a Statement> + Clone,
        global: bool,
        functions_lexical: bool,
    ) -> Result<(), Error> {
        if !global {
            self.check_lexical_conflicts(statements.clone(), false, functions_lexical)?;
        }
        let handle = self.scopes.last().expect("active environment").clone();
        let scope = &mut self
            .objects
            .environment_mut(&handle)
            .expect("active environment")
            .bindings;
        for statement in statements.clone() {
            for (name, _, mutable) in lexical_declarations(statement, functions_lexical) {
                scope.insert(
                    name.to_owned(),
                    BindingState {
                        value: None,
                        mutable,
                        deletable: false,
                        strict: true,
                    },
                );
            }
        }
        if functions_lexical {
            self.initialize_functions(
                statements.filter_map(|statement| match &statement.kind {
                    StatementKind::Function(function) => Some(function.as_ref()),
                    _ => None,
                }),
                Some(&handle),
            )?;
        }
        Ok(())
    }

    fn initialize_bindings(&mut self, bindings: &[BindingElement]) -> Result<(), Error> {
        let environment = self.scopes.last().expect("active environment").clone();
        for binding in bindings {
            self.tick(binding.pattern.span)?;
            self.initialize_binding_element(binding, Value::Undefined, &environment)?;
        }
        Ok(())
    }

    fn instantiate_global(&mut self, script: &Script) -> Result<(), Error> {
        // ECMA-262 16.1.7: check conflicts before creating any new bindings.
        // All Script vars are instantiated, including vars in unreachable code.
        self.check_lexical_conflicts(script.statements().iter(), true, false)?;
        let functions = script.function_declarations();
        let function_names: BTreeSet<_> = functions
            .iter()
            .map(|f| f.name.as_ref().expect("named declaration").name.as_str())
            .collect();
        let declarations = script.var_declarations();
        for binding in &declarations {
            self.tick(binding.span)?;
            if self
                .objects
                .environment(&self.scopes[0])
                .expect("global environment")
                .bindings
                .contains_key(binding.name)
            {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    binding.span,
                    "var declaration conflicts with global lexical binding",
                ));
            }
            if (standard_global(binding.name)
                || self.unsupported_host_globals.contains(binding.name))
                && self.global_own(binding.name, binding.span)?.is_none()
                && !function_names.contains(binding.name)
            {
                return Err(Self::unsupported(
                    binding.span,
                    format!("{} is not implemented", binding.name),
                ));
            }
        }
        for function in &functions {
            let name = function.name.as_ref().expect("named declaration");
            self.tick(name.span)?;
            if self
                .objects
                .environment(&self.scopes[0])
                .expect("global environment")
                .bindings
                .contains_key(&name.name)
            {
                return Err(Self::exception(
                    ExceptionKind::SyntaxError,
                    name.span,
                    "function declaration conflicts with global lexical binding",
                ));
            }
        }
        for function in &functions {
            let name = function.name.as_ref().expect("named declaration");
            if !self.can_declare_global_function(&name.name, name.span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    name.span,
                    "cannot declare function over restricted global",
                ));
            }
        }
        for binding in &declarations {
            if !function_names.contains(binding.name)
                && !self.can_declare_global_var(binding.name, binding.span)?
            {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    binding.span,
                    "cannot declare var on non-extensible global object",
                ));
            }
        }
        self.instantiate(script.statements().iter(), true, false)?;
        self.initialize_functions(functions.iter().copied(), None)?;
        for binding in declarations {
            if !function_names.contains(binding.name) {
                self.create_global_var(binding.name, binding.span)?;
            }
        }
        Ok(())
    }

    fn evaluate_var_bindings(&mut self, bindings: &[BindingElement]) -> Result<(), Error> {
        for binding in bindings {
            self.tick(binding.pattern.span)?;
            // ECMA-262 14.3.2.1: a declaration without an initializer does not
            // assign. Identifier initializers resolve before the RHS; patterns
            // perform BindingInitialization only after evaluating their RHS.
            if let Some(expr) = &binding.initializer {
                if let BindingPatternKind::Identifier(name) = &binding.pattern.kind {
                    let reference = self.resolve(name, binding.pattern.span)?;
                    let value = self.named_expression(expr, JsString::from(name.as_str()))?;
                    self.put(reference, value, binding.pattern.span)?;
                } else {
                    let value = self.expression(expr)?;
                    self.assign_pattern(&binding.pattern, value)?;
                }
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
        self.enter_evaluation(statement.span)?;
        // Keep recursive expression statements out of the large general frame.
        let result = if let StatementKind::Expression(expression) = &statement.kind {
            self.tick(statement.span)
                .and_then(|()| self.expression(expression))
                .map(|value| Completion::normal(Some(value)))
        } else {
            self.statement_inner(statement, labels)
        };
        self.evaluation_depth -= 1;
        result
    }

    fn statement_inner(
        &mut self,
        statement: &Statement,
        labels: &[&str],
    ) -> Result<Completion, Error> {
        self.tick(statement.span)?;
        match &statement.kind {
            // ECMA-262 14.16.1: no debugging facility is active in this host.
            StatementKind::Empty | StatementKind::Debugger => Ok(Completion::normal(None)),
            // 15.2.6: declaration evaluation is empty; instantiation made the value.
            StatementKind::Function(_) => Ok(Completion::normal(None)),
            StatementKind::Class(class) => {
                let value = self.class_definition(class, None)?;
                let name = class.name.as_ref().expect("class declaration name");
                let environment = self.scopes.last().expect("declaration environment");
                self.objects
                    .environment_mut(environment)
                    .expect("declaration environment")
                    .bindings
                    .get_mut(&name.name)
                    .expect("instantiated class binding")
                    .value = Some(value);
                Ok(Completion::normal(None))
            }
            StatementKind::Expression(_) => {
                unreachable!("expression statement dispatched separately")
            }
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
            StatementKind::Return(expression) => Ok(Completion {
                kind: CompletionKind::Return,
                value: Some(match expression {
                    Some(expression) => self.expression(expression)?,
                    None => Value::Undefined,
                }),
                target: None,
            }),
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
                self.push_scope(BTreeMap::new(), statement.span)?;
                let result = self
                    .instantiate(body.iter(), false, true)
                    .and_then(|()| self.statements(body));
                self.scopes.pop();
                result
            }
            StatementKind::With { object, body } => {
                self.with_statement(object, body, statement.span)
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
                self.push_scope(BTreeMap::new(), statement.span)?;
                let result = self
                    .instantiate(
                        clauses.iter().flat_map(|clause| clause.statements.iter()),
                        false,
                        true,
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
                        let scope = self.pattern_bindings(
                            bindings.iter().map(|binding| &binding.pattern),
                            *mutable,
                        )?;
                        self.push_scope(scope, statement.span)?;
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
            StatementKind::ForOf {
                binding,
                iterable,
                body,
            } => self.for_of(binding, iterable, body, labels, statement.span),
            StatementKind::ForIn {
                binding,
                object,
                body,
            } => self.for_in(binding, object, body, labels, statement.span),
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
        per_iteration: &[BindingElement],
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
    fn create_per_iteration_environment(
        &mut self,
        bindings: &[BindingElement],
    ) -> Result<(), Error> {
        if bindings.is_empty() {
            return Ok(());
        }
        let mut next = BTreeMap::new();
        for (name, span) in bindings
            .iter()
            .flat_map(|binding| binding.pattern.bound_names())
        {
            self.tick(span)?;
            let handle = self.scopes.last().expect("loop environment exists");
            let scope = &self
                .objects
                .environment(handle)
                .expect("active environment")
                .bindings;
            let value = scope[name]
                .value
                .clone()
                .expect("loop binding is initialized");
            next.insert(
                name.to_owned(),
                BindingState {
                    value: Some(value),
                    mutable: true,
                    deletable: false,
                    strict: true,
                },
            );
        }
        if next.is_empty() {
            return Ok(());
        }
        let current = self.scopes.last().expect("loop environment exists");
        let outer = self
            .objects
            .environment(current)
            .expect("active environment")
            .outer
            .clone();
        let next = self.object_work(bindings[0].pattern.span, |objects, budget| {
            objects.create_environment(outer, next, budget)
        })?;
        *self.scopes.last_mut().expect("loop environment exists") = next;
        Ok(())
    }

    fn resolve<'a>(&mut self, name: &'a str, span: Span) -> Result<Reference<'a>, Error> {
        let mut next = self.scopes.last().cloned();
        while let Some(handle) = next {
            self.tick(span)?;
            let scope = self
                .objects
                .environment(&handle)
                .expect("active environment");
            if scope.bindings.contains_key(name) {
                return Ok(Reference::Lexical(handle, name));
            }
            // Release the heap borrow before observable unscopables getters.
            let object = scope.binding_object.clone();
            next = scope.outer.clone();
            if let Some(object) = object {
                if self.with_has_binding(&object, name, span)? {
                    return Ok(Reference::ObjectBinding { object, name });
                }
            }
        }
        if (standard_global(name) || self.unsupported_host_globals.contains(name))
            && self.global_own(name, span)?.is_none()
        {
            return Ok(Reference::UnsupportedGlobal(name));
        }
        let object = self.global_object();
        Ok(
            if self.has_property(&object, &JsString::from(name), span)? {
                Reference::Global(name)
            } else {
                Reference::Unresolvable(name)
            },
        )
    }

    fn reference<'a>(&mut self, target: &'a Expr) -> Result<Reference<'a>, Error> {
        match &target.kind {
            ExprKind::Identifier(name) => self.resolve(name, target.span),
            ExprKind::OptionalChain { base, steps } => self.optional_chain_reference(base, steps),
            ExprKind::Parenthesized(inner) => self.reference(inner),
            ExprKind::SuperProperty(name) => self.super_property_reference(name, target.span),
            ExprKind::Member(base, name) => {
                let base = self.expression(base)?;
                let key = match name {
                    PropertyName::Private(_) => {
                        return Err(Self::unsupported(
                            target.span,
                            "private elements are not implemented",
                        ));
                    }
                    PropertyName::Literal(literal) => self.literal_value(literal, target.span)?,
                    PropertyName::Computed(expression) => self.expression(expression)?,
                };
                Ok(Reference::Property { base, key })
            }
            _ => unreachable!("parser validated reference target"),
        }
    }
    fn get(&mut self, reference: &mut Reference<'_>, span: Span) -> Result<Value, Error> {
        match reference {
            Reference::Value(value) => {
                self.object_work(span, |_, budget| budget.value(value))?;
                Ok(value.clone())
            }
            Reference::Property { base, key } => {
                Self::require_object_coercible(base, span)?;
                let key = self.reference_key(key, span)?;
                self.get_property_value(base, &key, span)
            }
            Reference::SuperProperty {
                base,
                key,
                this_value,
            } => {
                Self::require_object_coercible(base, span)?;
                let key = self.reference_key(key, span)?;
                let Value::Object(base) = base else {
                    unreachable!("a non-null super base is an object")
                };
                self.get_property_with_receiver(base, &key, this_value.clone(), span)
            }
            Reference::ObjectBinding { object, name } => self.with_get_binding(object, name, span),
            Reference::Lexical(handle, name) => self
                .objects
                .environment(handle)
                .expect("resolved environment")
                .bindings[*name]
                .value
                .clone()
                .ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::ReferenceError,
                        span,
                        format!("{name} is uninitialized"),
                    )
                }),
            Reference::Global(name) => self.get_global(name, span),
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
            Reference::Value(_) => unreachable!("parser rejects optional-chain assignment targets"),
            Reference::Property { base, mut key } => {
                Self::require_object_coercible(&base, span)?;
                let key = self.reference_key(&mut key, span)?;
                let written = self.set_property_value(&base, key, value, span)?;
                if !written && self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "property is not writable",
                    ));
                }
            }
            Reference::ObjectBinding { object, name } => {
                self.with_set_binding(&object, name, value, span)?
            }
            Reference::SuperProperty {
                base,
                mut key,
                this_value,
            } => {
                Self::require_object_coercible(&base, span)?;
                let key = self.reference_key(&mut key, span)?;
                let Value::Object(base) = base else {
                    unreachable!("a non-null super base is an object")
                };
                let written =
                    self.set_property_with_receiver(&base, key, value, this_value, span)?;
                if !written && self.strict {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "super property is not writable",
                    ));
                }
            }
            Reference::Lexical(handle, name) => {
                let binding = self
                    .objects
                    .environment_mut(&handle)
                    .expect("resolved environment")
                    .bindings
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
                    if !binding.strict && !self.strict {
                        return Ok(());
                    }
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        format!("{name} is immutable"),
                    ));
                }
                binding.value = Some(value);
            }
            Reference::Unresolvable(name) if self.strict => {
                return Err(Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    format!("{name} is not defined"),
                ));
            }
            Reference::Global(name) => self.put_global(name, value, true, span)?,
            Reference::Unresolvable(name) => self.put_global(name, value, false, span)?,
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
            Error::Exception {
                kind,
                span,
                message,
            } => self.materialize_exception(kind, message, span)?,
            _ => unreachable!("only language throws enter a catch clause"),
        };
        let mut bindings = BTreeMap::new();
        for (name, span) in parameter.bound_names() {
            self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
            bindings.insert(
                name.to_owned(),
                BindingState {
                    value: None,
                    mutable: true,
                    deletable: false,
                    strict: true,
                },
            );
        }
        self.push_scope(bindings, handler.span)?;
        let environment = self.scopes.last().expect("catch environment").clone();
        let result = self
            .initialize_pattern(parameter, value, &environment)
            .and_then(|()| self.statement(&handler.body));
        self.scopes.pop();
        result
    }

    fn enter_evaluation(&mut self, span: Span) -> Result<(), Error> {
        // Bound combined statement/expression recursion across function calls.
        if self.evaluation_depth >= 64 {
            return Err(Error::Limit {
                span,
                message: "evaluation nesting limit exceeded".into(),
            });
        }
        self.evaluation_depth += 1;
        Ok(())
    }

    fn expression(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.enter_evaluation(expr.span)?;
        // Debug builds reserve space for every general expression branch. Do
        // not retain that frame through calls and recursive eval compilation.
        let result = match &expr.kind {
            ExprKind::Call { callee, arguments } => {
                self.call_expression(callee, arguments, expr.span)
            }
            ExprKind::SuperCall(arguments) => self.super_call(arguments, expr.span),
            _ => self.expression_inner(expr),
        };
        self.evaluation_depth -= 1;
        result
    }

    fn call_expression(
        &mut self,
        callee: &Expr,
        arguments: &[spite_parser::ast::Argument],
        span: Span,
    ) -> Result<Value, Error> {
        self.tick(span)?;
        let (function, this, direct_eval) = if reference_expression(callee) {
            let mut reference = self.reference(callee)?;
            let function = self.get(&mut reference, callee.span)?;
            // 13.3.6.1: with/local eval references are direct for %eval%.
            let direct_eval = reference.is_eval_environment_reference()
                && matches!(&function, Value::Object(handle) if handle == &self.intrinsics.as_ref().expect("initialized").eval);
            let this = reference.call_receiver();
            (function, this, direct_eval)
        } else {
            (self.expression(callee)?, Value::Undefined, false)
        };
        let values = self.argument_list(arguments)?;
        let result = if direct_eval {
            self.perform_eval(
                values.into_iter().next().unwrap_or(Value::Undefined),
                true,
                span,
            )?
        } else {
            self.call(function, this, values, span)?
        };
        self.check_string(&result, span)?;
        Ok(result)
    }

    fn expression_inner(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.tick(expr.span)?;
        let result = match &expr.kind {
            ExprKind::This => self.this_value(expr.span)?,
            ExprKind::NewTarget => self.new_target_value(expr.span)?,
            ExprKind::Function(function) => self.ordinary_function(function, true, expr.span)?,
            ExprKind::Class(class) => self.class_definition(class, None)?,
            ExprKind::Arrow {
                parameters,
                body,
                source,
            } => self.arrow_function(parameters, body, source, expr.span)?,
            ExprKind::Object(properties) => self.object_literal(properties, expr.span)?,
            ExprKind::Array(elements) => self.array_literal(elements, expr.span)?,
            ExprKind::New { callee, arguments } => {
                let constructor = self.expression(callee)?;
                let values = self.argument_list(arguments.as_deref().unwrap_or(&[]))?;
                self.construct(constructor, values, expr.span)?
            }
            ExprKind::Call { .. } | ExprKind::SuperCall(_) => {
                unreachable!("call dispatched separately")
            }
            ExprKind::TaggedTemplate {
                tag,
                elements,
                substitutions,
            } => self.tagged_template(tag, elements, substitutions, expr.span)?,
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
                        let value = self.string(value, substitution.span)?;
                        self.append_string(&mut units, &value, substitution.span)?;
                    }
                }
                Value::String(JsString::from_code_units(units))
            }
            ExprKind::Literal(literal) => self.literal_value(literal, expr.span)?,
            ExprKind::PrivateIn { .. } => {
                return Err(Self::unsupported(
                    expr.span,
                    "private brand checks are not implemented",
                ));
            }
            ExprKind::Identifier(name) => {
                let mut reference = self.resolve(name, expr.span)?;
                self.get(&mut reference, expr.span)?
            }
            ExprKind::Parenthesized(inner) => self.expression(inner)?,
            ExprKind::Member(..) | ExprKind::SuperProperty(_) | ExprKind::OptionalChain { .. } => {
                let mut reference = self.reference(expr)?;
                self.get(&mut reference, expr.span)?
            }
            ExprKind::Assign(target, right) => {
                let reference = self.reference(target)?;
                let value = self.assignment_expression(target, right)?;
                self.put(reference, value.clone(), expr.span)?;
                value
            }
            ExprKind::DestructuringAssign { pattern, value } => {
                let value = self.expression(value)?;
                self.destructuring_assignment(pattern, value.clone())?;
                value
            }
            ExprKind::CompoundAssign(op, target, right) => {
                // ECMA-262 13.15.2: read the reference before the RHS. Logical
                // assignments that short-circuit perform neither RHS nor PutValue.
                let mut reference = self.reference(target)?;
                let left = self.get(&mut reference, expr.span)?;
                match op {
                    BinaryOp::And if !left.to_boolean() => left,
                    BinaryOp::Or if left.to_boolean() => left,
                    BinaryOp::Nullish if !matches!(left, Value::Null | Value::Undefined) => left,
                    _ => {
                        let right =
                            if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Nullish) {
                                self.assignment_expression(target, right)?
                            } else {
                                self.expression(right)?
                            };
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
                let mut reference = self.reference(argument)?;
                let old = self.get(&mut reference, argument.span)?;
                let old = self.numeric(old, argument.span)?;
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
                if *op == UnaryOp::Delete && reference_expression(inner) {
                    // ECMA-262 13.5.1.2: deleting an environment reference
                    // does not GetValue, even for an uninitialized binding.
                    let reference = self.reference(inner)?;
                    let deleted = match reference {
                        Reference::Value(_) => true,
                        Reference::SuperProperty { .. } => {
                            // 13.5.1.2: fail before ToObject or ToPropertyKey.
                            return Err(Self::exception(
                                ExceptionKind::ReferenceError,
                                inner.span,
                                "cannot delete a super property",
                            ));
                        }
                        Reference::Property { base, mut key } => {
                            Self::require_object_coercible(&base, inner.span)?;
                            let key = self.reference_key(&mut key, inner.span)?;
                            let deleted = self.delete_property_value(&base, &key, inner.span)?;
                            if !deleted && self.strict {
                                return Err(Self::exception(
                                    ExceptionKind::TypeError,
                                    inner.span,
                                    "property is not configurable",
                                ));
                            }
                            deleted
                        }
                        Reference::Lexical(environment, name) => {
                            let bindings = &mut self
                                .objects
                                .environment_mut(&environment)
                                .expect("resolved environment")
                                .bindings;
                            if bindings.get(name).is_none_or(|binding| binding.deletable) {
                                bindings.remove(name);
                                true
                            } else {
                                false
                            }
                        }
                        Reference::ObjectBinding { object, name } => self.delete_property_value(
                            &Value::Object(object),
                            &JsString::from(name),
                            inner.span,
                        )?,
                        Reference::Global(name) => {
                            let object = Value::Object(self.global_object());
                            self.delete_property_value(&object, &JsString::from(name), inner.span)?
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
                let value = if *op == UnaryOp::Typeof && identifier(inner).is_some() {
                    // Resolve once: object-environment lookup can execute getters.
                    let mut reference = self.reference(inner)?;
                    if matches!(reference, Reference::Unresolvable(_)) {
                        Value::Undefined
                    } else {
                        self.get(&mut reference, inner.span)?
                    }
                } else {
                    self.expression(inner)?
                };
                match op {
                    UnaryOp::Plus => Value::Number(self.number(value, expr.span)?),
                    UnaryOp::Minus => match self.numeric(value, expr.span)? {
                        Value::BigInt(value) => {
                            Value::BigInt(self.integer_work(expr.span, |budget| value.neg(budget))?)
                        }
                        Value::Number(value) => Value::Number(-value),
                        _ => unreachable!("ToNumeric returns a numeric value"),
                    },
                    UnaryOp::Not => Value::Boolean(!value.to_boolean()),
                    UnaryOp::BitNot => match self.numeric(value, expr.span)? {
                        Value::BigInt(value) => Value::BigInt(
                            self.integer_work(expr.span, |budget| value.bitnot(budget))?,
                        ),
                        Value::Number(value) => Value::Number((!(to_uint32(value) as i32)) as f64),
                        _ => unreachable!("ToNumeric returns a numeric value"),
                    },
                    UnaryOp::Void => Value::Undefined,
                    UnaryOp::Typeof => {
                        Value::String(JsString::from(if self.is_callable(&value, expr.span)? {
                            "function"
                        } else {
                            value.type_name()
                        }))
                    }
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
        mut left: Value,
        mut right: Value,
        span: Span,
    ) -> Result<Value, Error> {
        use BinaryOp::*;
        match op {
            And | Or | Nullish | Comma => return Ok(right),
            Instanceof => return Ok(Value::Boolean(self.instance_of(left, right, span)?)),
            In => {
                let Value::Object(object) = right else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "right operand of in must be an object",
                    ));
                };
                let key = self.property_key(left, span)?;
                return Ok(Value::Boolean(self.has_property(&object, &key, span)?));
            }
            StrictEqual | StrictNotEqual => {
                let equal = self.strictly_equal(&left, &right, span)?;
                return Ok(Value::Boolean(if op == StrictEqual {
                    equal
                } else {
                    !equal
                }));
            }
            Equal | NotEqual => {
                if matches!(
                    (&left, &right),
                    (
                        Value::Object(_),
                        Value::String(_)
                            | Value::Number(_)
                            | Value::BigInt(_)
                            | Value::Boolean(_)
                            | Value::Symbol(_)
                    )
                ) {
                    left = self.primitive(left, Hint::Default, span)?;
                } else if matches!(
                    (&left, &right),
                    (
                        Value::String(_)
                            | Value::Number(_)
                            | Value::BigInt(_)
                            | Value::Boolean(_)
                            | Value::Symbol(_),
                        Value::Object(_)
                    )
                ) {
                    right = self.primitive(right, Hint::Default, span)?;
                }
                self.comparison_work(&left, &right, span)?;
                let equal =
                    self.conversion_work(span, |budget| left.loosely_equal(&right, budget))?;
                return Ok(Value::Boolean(if op == Equal { equal } else { !equal }));
            }
            Add => {
                left = self.primitive(left, Hint::Default, span)?;
                right = self.primitive(right, Hint::Default, span)?;
                if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
                    let a = self.string(left, span)?;
                    let b = self.string(right, span)?;
                    if a.len().checked_add(b.len()).is_none_or(|length| {
                        self.limits
                            .max_string_units
                            .is_some_and(|limit| length > limit)
                    }) {
                        return Err(Error::Limit {
                            span,
                            message: "string length limit exceeded".into(),
                        });
                    }
                    return Ok(Value::String(a.concat(&b)));
                }
            }
            Less | LessEqual | Greater | GreaterEqual => {
                left = self.primitive(left, Hint::Number, span)?;
                right = self.primitive(right, Hint::Number, span)?;
                self.comparison_work(&left, &right, span)?;
                let order = self.conversion_work(span, |budget| left.compare(&right, budget))?;
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
        let left = self.numeric(left, span)?;
        let right = self.numeric(right, span)?;
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
        let Value::Number(a) = left else {
            unreachable!("numeric types checked above")
        };
        let Value::Number(b) = right else {
            unreachable!("numeric types checked above")
        };
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

fn lexical_declarations(
    statement: &Statement,
    functions: bool,
) -> impl Iterator<Item = (&str, Span, bool)> {
    let (bindings, mutable) = match &statement.kind {
        StatementKind::Lexical { bindings, mutable } => (bindings.as_slice(), *mutable),
        _ => (&[][..], true),
    };
    let function = match &statement.kind {
        StatementKind::Function(function) if functions => function.name.as_ref(),
        StatementKind::Class(class) => class.name.as_ref(),
        _ => None,
    };
    bindings
        .iter()
        .flat_map(|binding| binding.pattern.bound_names())
        .map(move |(name, span)| (name, span, mutable))
        .chain(function.map(|name| (name.name.as_str(), name.span, true)))
}

fn identifier(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name),
        ExprKind::Parenthesized(e) => identifier(e),
        _ => None,
    }
}

fn reference_expression(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(_)
        | ExprKind::Member(..)
        | ExprKind::SuperProperty(_)
        | ExprKind::OptionalChain { .. } => true,
        ExprKind::Parenthesized(inner) => reference_expression(inner),
        _ => false,
    }
}
fn standard_global(name: &str) -> bool {
    matches!(
        name,
        "Date"
            | "RegExp"
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
            | "WeakMap"
            | "WeakSet"
            | "ArrayBuffer"
            | "SharedArrayBuffer"
            | "DataView"
            | "Atomics"
            | "WeakRef"
            | "FinalizationRegistry"
            | "Promise"
            | "Proxy"
            | "DisposableStack"
            | "AsyncDisposableStack"
            | "SuppressedError"
    )
}
