//! Realm storage, host entry points and Script evaluation.

use crate::environment::EnvironmentHandle;
use crate::{CompletionKind, Error, Limits, ModuleImportRequest, ObjectHandle, RootedValue, Value};
use crate::{function, object, realm_record};
use spite_parser::ast::Script;
use spite_parser::{parse_script, parse_script_with_source_limit};
use std::collections::VecDeque;

/// An isolated execution realm with persistent global lexical bindings.
///
/// The global object exposes the standard bindings supported by this host.
/// Host extensions are not installed by default.
/// By default this agent cannot suspend: `Atomics.wait` validates its arguments
/// and throws TypeError. [`Self::new_suspending`] permits synchronous waiting.
/// `Atomics.waitAsync` supports immediate results and
/// notifications from agents sharing native backing. Foreign notifications and
/// finite timeouts progress through [`Self::run_next_atomics_wait_job`] at idle
/// host boundaries.
/// Execution uses bounded Rust recursion and requires a native thread stack of
/// at least 2 MiB. Call re-entry is capped at 32 native calls.
#[derive(Debug)]
pub struct Realm {
    pub(crate) record: realm_record::RealmRecord,
    pub(crate) test262_agents: Option<function::AgentContext>,
    pub(crate) scopes: Vec<EnvironmentHandle>,
    // Lexical scopes may nest without changing the current VariableEnvironment.
    pub(crate) variable_environment: Option<EnvironmentHandle>,
    pub(crate) limits: Limits,
    pub(crate) remaining_steps: Option<usize>,
    pub(crate) call_depth: usize,
    pub(crate) tail_scope: Option<usize>,
    pub(crate) evaluation_depth: usize,
    pub(crate) strict: bool,
    pub(crate) objects: object::Objects,
    // 9.11: retained until the embedding explicitly ends its synchronous sequence.
    pub(crate) kept_alive: Vec<RootedValue>,
    // One host job is one execution sequence and one work allowance.
    pub(crate) job_active: bool,
    pub(crate) promise_jobs: VecDeque<function::PromiseJob>,
    pub(crate) active_module: Option<ObjectHandle>,
    pub(crate) module_async_order: usize,
    pub(crate) module_import_requests: Option<VecDeque<ModuleImportRequest>>,
    pub(crate) gc_requests: VecDeque<function::GcRequest>,
    pub(crate) atomic_waits: Option<function::WaitRegistry>,
    pub(crate) can_suspend: bool,
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
            record: realm_record::RealmRecord::default(),
            test262_agents: None,
            scopes: Vec::new(),
            variable_environment: None,
            limits,
            remaining_steps: limits.max_steps,
            call_depth: 0,
            tail_scope: None,
            evaluation_depth: 0,
            strict: false,
            objects: object::Objects::with_limits(limits.max_heap_entries, limits.max_properties),
            kept_alive: Vec::new(),
            job_active: false,
            promise_jobs: VecDeque::new(),
            active_module: None,
            module_async_order: 0,
            module_import_requests: None,
            gc_requests: VecDeque::new(),
            atomic_waits: None,
            can_suspend: false,
        }
    }

    /// Creates an agent allowed to suspend its calling thread for `Atomics.wait`.
    ///
    /// The policy is fixed for this Realm. Notifications can arrive from other
    /// agents sharing its native backing. This constructor starts no thread;
    /// [`Self::new`] and [`Self::default`] create nonsuspending agents.
    pub fn new_suspending(limits: Limits) -> Self {
        let mut realm = Self::new(limits);
        realm.can_suspend = true;
        realm
    }

    /// Marks an unavailable host global so access reports Unsupported, not a
    /// misleading ReferenceError or an undefined result from typeof.
    ///
    /// This does not install a JavaScript value. Lexical bindings can shadow the
    /// name, and existing implemented globals retain their normal behavior.
    /// Ordinary realms reserve no extra host names.
    pub fn reserve_unsupported_global(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.record.inspectable_host_absences.remove(&name);
        self.record.unsupported_host_globals.insert(name);
    }

    /// Reserves an absent host global while permitting own-property inspection.
    ///
    /// An absent property's descriptor is undefined and own-property predicates
    /// return false. Reading or using the missing global still reports Unsupported.
    /// This supports optional host protocols such as Test262's `$DONE`, which is
    /// intentionally absent outside asynchronous tests. Existing properties and
    /// lexical shadows retain their ordinary behavior.
    pub fn reserve_absent_host_global(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.record.inspectable_host_absences.insert(name.clone());
        self.record.unsupported_host_globals.insert(name);
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
        let caller_variable =
            std::mem::replace(&mut self.variable_environment, self.scopes.first().cloned());
        let caller_module = self.active_module.take();
        let caller_strict = self.strict;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        self.strict = script.is_strict();
        let result = (|| {
            self.instantiate_global(script)?;
            let completion = self.statements(script.statements())?;
            // Validated Scripts cannot leave an unhandled control transfer.
            debug_assert_eq!(completion.kind, CompletionKind::Normal);
            Ok(completion.value.unwrap_or(Value::Undefined))
        })();
        if self.job_active {
            self.variable_environment = caller_variable;
            self.strict = caller_strict;
        }
        self.active_module = caller_module;
        result
    }
}
