//! Source Text Module records, linking and standalone evaluation (16.2.1).

mod async_evaluation;
mod binding;
mod evaluation;
mod graph;
mod link;
mod meta;
mod resolution;
pub(crate) use binding::ModuleBinding;
mod namespace;

use crate::{BindingState, Error, ObjectHandle, Realm, Value, environment::ThisBinding};
use spite_core::Span;
use spite_parser::{
    ast::{BindingPattern, BindingPatternKind, ExportKind, Module, StatementKind},
    parse_module, parse_module_with_source_limit,
};
use std::collections::BTreeMap;

/// A retained Source Text Module record owned by one Realm.
///
/// Clones refer to the same record. The record, bindings and evaluation Promise
/// remain reachable until the last handle is dropped. Loaded graphs support
/// linking and dependency evaluation, including top-level Await.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleHandle {
    root: crate::object::Root,
}

impl std::hash::Hash for ModuleHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.root.handle(), state);
    }
}

impl Realm {
    /// Parses and evaluates a fresh Module, returning its evaluation Promise.
    ///
    /// Drive pending awaits with [`Self::run_next_promise_job`]. Parsing/linking
    /// failures are returned directly; language evaluation failures reject the
    /// Promise. Returned values are unrooted; retain them with [`Self::root_value`]
    /// across collection. Evaluation never collects.
    ///
    /// Use retained records to load dependencies before evaluation.
    pub fn eval_module(&mut self, source: &str) -> Result<Value, Error> {
        let module = self.create_module(source)?;
        self.evaluate_module_record(&module)
    }

    /// Creates a retained Module record without executing its body.
    pub fn create_module(&mut self, source: &str) -> Result<ModuleHandle, Error> {
        let module = match self.limits.max_source_bytes {
            Some(limit) => parse_module_with_source_limit(source, limit),
            None => parse_module(source),
        }
        .map_err(Error::Parse)?;
        self.create_module_from_ast(&module)
    }

    /// Creates a retained record from a validated Module, including UTF-16 input.
    pub fn create_module_from_ast(&mut self, module: &Module) -> Result<ModuleHandle, Error> {
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let root = self.object_work(Span::new(0, 0), |objects, budget| {
            let record = objects.create_module_record(module.clone(), budget)?;
            objects.root(&record, budget)
        })?;
        Ok(ModuleHandle { root })
    }

    /// Evaluates a fresh instance of a parsed Module in its own strict environment.
    ///
    /// See [`Self::eval_module`] for Promise handling and remaining boundaries.
    pub fn evaluate_module(&mut self, module: &Module) -> Result<Value, Error> {
        let module = self.create_module_from_ast(module)?;
        self.evaluate_module_record(&module)
    }

    /// Evaluates a retained record, reusing its original evaluation Promise.
    ///
    /// Repeated calls never replay its body, including after rejection or a host
    /// abort. A host abort leaves the evaluation Promise unsettled. Object values
    /// returned from this method remain unrooted; the Module handle retains the
    /// Promise until its last clone is dropped.
    pub fn evaluate_module_record(&mut self, module: &ModuleHandle) -> Result<Value, Error> {
        let record = module.root.handle();
        let record = self
            .objects
            .module_execution(record)
            .map_err(Error::InvalidObject)?
            .and_then(|execution| execution.cycle_root.clone())
            .unwrap_or_else(|| record.clone());
        let record = &record;
        if let Some(promise) = self
            .objects
            .module_evaluation(record)
            .map_err(Error::InvalidObject)?
        {
            return Ok(promise);
        }
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        self.link_module_record(record, Span::new(0, 0))?;
        self.evaluate_module_graph(record, Span::new(0, 0))
    }

    fn initialize_module_record(
        &mut self,
        module: &Module,
        record: &ObjectHandle,
        imports: BTreeMap<String, ModuleBinding>,
    ) -> Result<(), Error> {
        let default_export = module_default_index(module);
        let body = module.body();
        let span = Span::new(
            0,
            body.statements()
                .last()
                .map_or(0, |statement| statement.span.end),
        );
        let global = self.scopes.first().expect("global environment").clone();
        let mut bindings = BTreeMap::new();
        for name in body.var_declarations() {
            self.tick(name.span)?;
            self.object_work(name.span, |_, budget| budget.charge(name.name.len() + 1))?;
            bindings
                .entry(name.name.to_owned())
                .or_insert(BindingState {
                    value: Some(Value::Undefined),
                    mutable: true,
                    deletable: false,
                    strict: true,
                });
        }
        if default_export.is_some() {
            self.object_work(span, |_, budget| budget.charge("*default*".len() + 1))?;
            // IsConstantDeclaration is false even for default expressions.
            bindings.insert(
                "*default*".into(),
                BindingState {
                    value: None,
                    mutable: true,
                    deletable: false,
                    strict: true,
                },
            );
        }
        let environment = self.object_work(span, |objects, budget| {
            let environment = objects.create_environment(Some(global.clone()), bindings, budget)?;
            objects.environment_mut(&environment)?.this = Some(ThisBinding::Module(record.clone()));
            Ok(environment)
        })?;
        self.object_work(span, |objects, budget| {
            objects.store_module_imports(&environment, imports, budget)
        })?;
        self.instantiate_module_body(&body, environment, default_export, record, span)
    }

    pub(crate) fn prepare_module_default(
        &mut self,
        body: &spite_parser::ast::FunctionBody,
        environment: &crate::environment::EnvironmentHandle,
        default_export: Option<usize>,
    ) -> Result<(), Error> {
        if let Some(index) = default_export
            && let StatementKind::Function(function) = &body.statements()[index].kind
        {
            let span = body.statements()[index].span;
            let value = self.ordinary_function(function, false, span)?;
            let Value::Object(handle) = &value else {
                unreachable!("default function")
            };
            self.set_function_name(
                handle,
                spite_core::JsString::from("default").into(),
                None,
                span,
            )?;
            self.initialize_module_default(environment, value, span)?;
        }
        Ok(())
    }

    pub(crate) fn initialize_module_default(
        &mut self,
        environment: &crate::environment::EnvironmentHandle,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.initialize_pattern(
            &BindingPattern {
                kind: BindingPatternKind::Identifier("*default*".into()),
                span,
            },
            value,
            environment,
        )
    }
}

fn module_default_index(module: &Module) -> Option<usize> {
    module.exports().iter().find_map(|export| {
        if matches!(export.kind, ExportKind::Default { local: None, .. }) {
            Some(
                module
                    .statements()
                    .iter()
                    .position(|statement| {
                        export.span.start <= statement.span.start
                            && statement.span.end <= export.span.end
                    })
                    .expect("retained default export statement"),
            )
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PromiseJobOutcome, PromiseState, function::Callable};
    use spite_core::JsString;

    fn default_value(realm: &mut Realm) -> Option<Value> {
        let Value::Object(capture) = realm.eval("capture").unwrap() else {
            panic!("capture")
        };
        let Callable::Arrow(function) =
            realm.objects.inspect(&capture).unwrap().callable().unwrap()
        else {
            panic!("arrow")
        };
        let environment = function.environment.clone();
        realm.objects.environment(&environment).unwrap().bindings["*default*"]
            .value
            .clone()
    }
    fn drain(realm: &mut Realm) {
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
            }
        }
    }

    #[test]
    fn anonymous_default_functions_initialize_before_awaited_body_and_keep_original_source() {
        for declaration in [
            "function(){}",
            "function*(){}",
            "async function(){}",
            "async function*(){}",
        ] {
            let mut realm = Realm::default();
            realm.eval("let gate=new Promise(()=>{})").unwrap();
            let promise = realm
                .eval_module(&format!(
                    "globalThis.capture=()=>0;await gate;export default {declaration}"
                ))
                .unwrap();
            let Value::Object(promise) = promise else {
                panic!("Promise")
            };
            assert!(matches!(
                realm.inspect_object(&promise).unwrap().promise_state(),
                Some(PromiseState::Pending)
            ));
            let value = default_value(&mut realm).expect("initialized function binding");
            assert_eq!(
                realm.read_property(&value, &JsString::from("name")),
                Ok(Value::String(JsString::from("default")))
            );
            let Value::Object(function) = value else {
                panic!("function")
            };
            let Callable::Ordinary(code) = realm
                .objects
                .inspect(&function)
                .unwrap()
                .callable()
                .unwrap()
            else {
                panic!("ordinary")
            };
            assert_eq!(code.source.to_js_string(), JsString::from(declaration));
        }
    }

    #[test]
    fn default_expression_naming_distinguishes_anonymous_definitions_from_values() {
        for (expression, name) in [
            ("()=>0", "default"),
            ("(function(){})", "default"),
            ("(function named(){})", "named"),
            ("(class{})", "default"),
            ("(class Named{})", "Named"),
            ("(0,()=>0)", ""),
            ("await (()=>0)", ""),
        ] {
            let mut realm = Realm::default();
            realm
                .eval_module(&format!(
                    "globalThis.capture=()=>0;export default {expression}"
                ))
                .unwrap();
            drain(&mut realm);
            let value = default_value(&mut realm).unwrap();
            assert_eq!(
                realm.read_property(&value, &JsString::from("name")),
                Ok(Value::String(JsString::from(name))),
                "{expression}"
            );
        }
    }

    #[test]
    fn awaited_default_expression_retains_tdz_until_settlement_and_initializes_once() {
        let mut realm = Realm::default();
        realm
            .eval("let release,events=[];let gate=new Promise(r=>release=r)")
            .unwrap();
        realm
            .eval_module("globalThis.capture=()=>0;export default await gate;events.push('after')")
            .unwrap();
        assert_eq!(default_value(&mut realm), None);
        realm.eval("release(7)").unwrap();
        drain(&mut realm);
        assert_eq!(default_value(&mut realm), Some(Value::Number(7.0)));
        assert_eq!(
            realm.eval("events.join(',')==='after'"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn anonymous_default_class_names_precede_static_initialization_and_preserve_name_methods() {
        let mut realm = Realm::default();
        realm.eval_module("globalThis.capture=()=>0;export default class{static{globalThis.observed=this.name}}").unwrap();
        assert_eq!(realm.eval("observed==='default'"), Ok(Value::Boolean(true)));
        let value = default_value(&mut realm).unwrap();
        assert_eq!(
            realm.read_property(&value, &JsString::from("name")),
            Ok(Value::String(JsString::from("default")))
        );
        realm
            .eval_module("globalThis.capture=()=>0;export default class{static name(){return 7}}")
            .unwrap();
        let value = default_value(&mut realm).unwrap();
        let method = realm
            .read_property(&value, &JsString::from("name"))
            .unwrap();
        assert_eq!(
            realm.read_property(&method, &JsString::from("name")),
            Ok(Value::String(JsString::from("name")))
        );
    }
}
