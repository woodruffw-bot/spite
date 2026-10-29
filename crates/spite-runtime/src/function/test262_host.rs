//! Explicit Test262 host capabilities; ordinary Realms receive no host object.

use super::Builtin;
use crate::{
    Error, ExceptionKind, Realm, Value,
    object::{
        DataDescriptor, DescriptorKind, PropertyDescriptor, source_context::SourceExecutionContext,
    },
};
use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::parse_script_utf16;

impl Realm {
    /// Installs a fresh partial Test262 `$262` host object in this realm.
    ///
    /// The object exposes `global`, `detachArrayBuffer`, global `evalScript` and
    /// requested collection through `gc`. Service collection requests with
    /// [`Self::run_requested_gc`] after JavaScript returns. Accessing
    /// unavailable host facilities reports Unsupported, including through typeof;
    /// they cannot masquerade as missing values or catchable JavaScript errors.
    /// Ordinary realms have no Test262 host unless this method is called.
    pub fn install_test262_host(&mut self) -> Result<(), Error> {
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let span = Span::new(0, 0);
        let intrinsics = self.intrinsics.as_ref().expect("initialized realm");
        let object_prototype = intrinsics.object_prototype.clone();
        let function_prototype = intrinsics.function_prototype.clone();
        let host = self.object_work(span, |objects, _| objects.create(Some(&object_prototype)))?;
        for (name, builtin) in [
            ("detachArrayBuffer", Builtin::DetachArrayBuffer),
            ("evalScript", Builtin::EvalScript),
            ("gc", Builtin::RequestGc),
        ] {
            let method = self.new_builtin(&function_prototype, builtin, span)?;
            self.define_builtin_property(&host, name, Value::Object(method), true, span)?;
        }
        self.define_builtin_property(
            &host,
            "global",
            Value::Object(self.global_object()),
            true,
            span,
        )?;
        for (name, getter_name) in [
            ("createRealm", "get createRealm"),
            ("agent", "get agent"),
            ("IsHTMLDDA", "get IsHTMLDDA"),
            ("AbstractModuleSource", "get AbstractModuleSource"),
        ] {
            let getter = self.new_builtin(
                &function_prototype,
                Builtin::UnavailableHost(getter_name),
                span,
            )?;
            self.define_property_or_throw(
                &host,
                JsString::from(name),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(getter)),
                        set: Some(None),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                span,
            )?;
        }
        self.define_property_or_throw(
            &self.global_object(),
            JsString::from("$262"),
            DataDescriptor {
                value: Some(Value::Object(host)),
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )
    }

    pub(super) fn host_eval_script(&mut self, source: Value, span: Span) -> Result<Value, Error> {
        // Test262 INTERPRETING.md: ParseScript followed by ScriptEvaluation.
        // This is a String-taking host API, independent of public eval hooks.
        let Value::String(source) = source else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "evalScript requires a String",
            ));
        };
        let bytes = char::decode_utf16(source.code_units().iter().copied())
            .try_fold(0usize, |length, point| {
                length.checked_add(point.map_or(3, char::len_utf8))
            })
            .ok_or_else(|| Error::Limit {
                span,
                message: "Script source capacity exceeded".into(),
            })?;
        if self
            .limits
            .max_source_bytes
            .is_some_and(|limit| bytes > limit)
        {
            return Err(Error::Limit {
                span,
                message: "Script source size limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(bytes))?;
        let script = parse_script_utf16(&source).map_err(|diagnostic| match diagnostic.kind {
            DiagnosticKind::Syntax => Self::exception(
                ExceptionKind::SyntaxError,
                diagnostic.span,
                diagnostic.message,
            ),
            DiagnosticKind::Unsupported => Self::unsupported(diagnostic.span, diagnostic.message),
            DiagnosticKind::Limit => Error::Limit {
                span: diagnostic.span,
                message: diagnostic.message,
            },
        })?;
        self.object_work(span, |_, budget| budget.charge(1))?;
        let global = self.scopes.first().expect("global environment").clone();
        let mut scopes = Vec::new();
        scopes.try_reserve_exact(1).map_err(|_| Error::Limit {
            span,
            message: "Script scope allocation failed".into(),
        })?;
        scopes.push(global.clone());
        let context = SourceExecutionContext {
            scopes,
            variable: Some(global),
            strict: false,
            module: None,
        };
        // Re-entry belongs to the current execution sequence. In particular,
        // evaluate() must not reset the work allowance for each nested Script.
        let caller_job = std::mem::replace(&mut self.job_active, true);
        let result = self.with_source_execution_context(context, |realm| realm.evaluate(&script));
        self.job_active = caller_job;
        result
    }

    pub(super) fn host_detach_array_buffer(
        &mut self,
        mut arguments: impl Iterator<Item = Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let value = arguments.next().unwrap_or(Value::Undefined);
        let Value::Object(buffer) = value else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "detachArrayBuffer requires an ArrayBuffer",
            ));
        };
        let ordinary = self.object_work(span, |objects, _| {
            Ok(objects.inspect(&buffer)?.array_buffer_data().is_some())
        })?;
        if !ordinary {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "detachArrayBuffer requires an ordinary ArrayBuffer",
            ));
        }
        // DetachArrayBuffer: all currently exposed ordinary buffers
        // have the undefined detach key; key comparison performs no coercion.
        if !matches!(
            arguments.next().unwrap_or(Value::Undefined),
            Value::Undefined
        ) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "ArrayBuffer detach key mismatch",
            ));
        }
        self.buffer_value_work(span, |objects, budget| {
            objects.detach_array_buffer(&buffer, budget)
        })?;
        Ok(Value::Undefined)
    }
}
