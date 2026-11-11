//! Resource stacks (sec-disposablestack-objects, sec-disposeresources).

use super::Builtin;
use crate::object::{
    DataDescriptor, DescriptorKind, DisposableResource, DisposableStackKind as StackKind,
    DisposalHint, PropertyDescriptor,
};
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Member {
    Adopt,
    Defer,
    Dispose,
    Disposed,
    Move,
    Use,
}

impl Member {
    pub(super) fn name(self, kind: StackKind) -> &'static str {
        match self {
            Self::Adopt => "adopt",
            Self::Defer => "defer",
            Self::Dispose => {
                if kind == StackKind::Sync {
                    "dispose"
                } else {
                    "disposeAsync"
                }
            }
            Self::Disposed => "get disposed",
            Self::Move => "move",
            Self::Use => "use",
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::Adopt => 2.0,
            Self::Defer | Self::Use => 1.0,
            Self::Dispose | Self::Disposed | Self::Move => 0.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DisposableStackIntrinsics {
    pub constructor: ObjectHandle,
    prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl DisposableStackIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn disposable_stack_intrinsics(
        &mut self,
        kind: StackKind,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<DisposableStackIntrinsics, Error> {
        let constructor =
            self.new_builtin(function_prototype, Builtin::DisposableStack(kind), span)?;
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.define_property_or_throw(
            &constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        let mut dispose = None;
        for member in [
            Member::Adopt,
            Member::Defer,
            Member::Dispose,
            Member::Disposed,
            Member::Move,
            Member::Use,
        ] {
            let method = self.new_builtin(
                function_prototype,
                Builtin::DisposableStackMember(kind, member),
                span,
            )?;
            if member == Member::Disposed {
                self.define_property_or_throw(
                    &prototype,
                    JsString::from("disposed"),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(method.clone())),
                            set: Some(None),
                        },
                        enumerable: Some(false),
                        configurable: Some(true),
                    },
                    span,
                )?;
            } else {
                self.define_builtin_property(
                    &prototype,
                    member.name(kind),
                    Value::Object(method.clone()),
                    true,
                    span,
                )?;
            }
            if member == Member::Dispose {
                dispose = Some(method.clone());
            }
            methods.push(method);
        }
        for (symbol, value, writable) in [
            (
                if kind == StackKind::Sync {
                    WellKnownSymbol::Dispose
                } else {
                    WellKnownSymbol::AsyncDispose
                },
                Value::Object(dispose.expect("dispose method")),
                true,
            ),
            (
                WellKnownSymbol::ToStringTag,
                Value::String(JsString::from(kind.name())),
                false,
            ),
        ] {
            self.define_property_or_throw(
                &prototype,
                symbol.symbol(),
                DataDescriptor {
                    value: Some(value),
                    writable: Some(writable),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(DisposableStackIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    pub(super) fn disposable_stack_constructor(
        &mut self,
        kind: StackKind,
        new_target: ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.function_realm_intrinsic(&new_target, span, |intrinsics| match kind {
                StackKind::Sync => intrinsics.disposable_stack.prototype.clone(),
                StackKind::Async => intrinsics.async_disposable_stack.prototype.clone(),
            })?
        };
        self.object_work(span, |objects, _| {
            objects.create_disposable_stack(&prototype, kind)
        })
        .map(Value::Object)
    }

    pub(super) fn disposable_stack_member(
        &mut self,
        kind: StackKind,
        member: Member,
        this: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if kind == StackKind::Async && member == Member::Dispose {
            return self.async_stack_dispose(this, span);
        }
        // RequireInternalSlot never unwraps proxies or observes receiver properties.
        let Value::Object(stack) = this else {
            return Err(Self::disposable_stack_brand_error(span));
        };
        let Some(disposed) = self.object_work(span, |objects, _| {
            objects.disposable_stack_state(&stack, kind)
        })?
        else {
            return Err(Self::disposable_stack_brand_error(span));
        };
        if member == Member::Disposed {
            return Ok(Value::Boolean(disposed));
        }
        if member == Member::Dispose {
            if disposed {
                return Ok(Value::Undefined);
            }
            return self.dispose_stack_resources(&stack, span);
        }
        if disposed {
            return Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                "DisposableStack has been disposed",
            ));
        }
        if member == Member::Move {
            let prototype = self.disposable_stack_intrinsic_prototype(kind);
            let target = self.object_work(span, |objects, _| {
                objects.create_disposable_stack(&prototype, kind)
            })?;
            self.object_work(span, |objects, budget| {
                objects.move_disposable_resources(&stack, &target, budget)
            })?;
            return Ok(Value::Object(target));
        }
        let resources =
            self.object_work(span, |objects, _| objects.disposable_resource_list(&stack))?;
        let value = arguments.next().unwrap_or(Value::Undefined);
        let (result, resource) = match member {
            Member::Use => {
                let Some(resource) = self.create_disposable_resource(&value, kind, span)? else {
                    return Ok(value);
                };
                (value, resource)
            }
            Member::Adopt | Member::Defer => {
                let method = if member == Member::Adopt {
                    arguments.next().unwrap_or(Value::Undefined)
                } else {
                    value.clone()
                };
                if !self.is_callable(&method, span)? {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "disposal callback is not callable",
                    ));
                }
                let (result, argument) = if member == Member::Adopt {
                    (value.clone(), Some(value))
                } else {
                    (Value::Undefined, None)
                };
                (
                    result,
                    DisposableResource {
                        hint: if kind == StackKind::Sync {
                            DisposalHint::Sync
                        } else {
                            DisposalHint::Async
                        },
                        value: Value::Undefined,
                        method,
                        argument,
                        wrapper_realm: (member == Member::Adopt)
                            .then(|| self.record.global_environment.clone().expect("initialized")),
                    },
                )
            }
            Member::Dispose | Member::Disposed | Member::Move => unreachable!("handled above"),
        };
        // Lookup may have reentered and disposed/moved the stack. The spec does
        // not repeat the state check after GetDisposeMethod returns.
        self.object_work(span, |objects, budget| {
            objects.push_disposable_resource(&resources, resource, budget)
        })?;
        Ok(result)
    }

    pub(crate) fn call_disposable_resource(
        &mut self,
        resource: DisposableResource,
        span: Span,
    ) -> Result<Value, Error> {
        let invoke = |realm: &mut Self| {
            let arguments = resource.argument.into_iter().collect();
            if resource.hint != DisposalHint::AsyncFromSync {
                return realm.call(resource.method, resource.value, arguments, span);
            }
            // GetDisposeMethod creates this builtin in the registration Realm.
            // Create its Promise there before calling the synchronous disposer;
            // ignore the return value and translate language throws to rejection.
            let capability = realm.intrinsic_promise_capability(span)?;
            match realm.call(resource.method, resource.value, arguments, span) {
                Ok(_) => {
                    realm.call(
                        capability.resolve,
                        Value::Undefined,
                        vec![Value::Undefined],
                        span,
                    )?;
                }
                Err(error) => {
                    let reason = realm.language_exception_value(error)?;
                    realm.call(capability.reject, Value::Undefined, vec![reason], span)?;
                }
            }
            Ok(capability.promise)
        };
        if let Some(target) = resource.wrapper_realm {
            self.with_function_realm(&target, span, invoke)
        } else {
            invoke(self)
        }
    }

    pub(crate) fn create_disposable_resource(
        &mut self,
        value: &Value,
        kind: StackKind,
        span: Span,
    ) -> Result<Option<DisposableResource>, Error> {
        if matches!(value, Value::Undefined | Value::Null) {
            return Ok((kind == StackKind::Async).then_some(DisposableResource {
                hint: DisposalHint::Async,
                value: Value::Undefined,
                method: Value::Undefined,
                argument: None,
                wrapper_realm: None,
            }));
        }
        if !matches!(value, Value::Object(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "disposable resource must be an object",
            ));
        }
        let (method, hint) = if kind == StackKind::Async {
            if let Some(method) =
                self.get_method(value, &WellKnownSymbol::AsyncDispose.symbol(), span)?
            {
                (Some(method), DisposalHint::Async)
            } else {
                (
                    self.get_method(value, &WellKnownSymbol::Dispose.symbol(), span)?,
                    DisposalHint::AsyncFromSync,
                )
            }
        } else {
            (
                self.get_method(value, &WellKnownSymbol::Dispose.symbol(), span)?,
                DisposalHint::Sync,
            )
        };
        let Some(method) = method else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "resource has no disposal method",
            ));
        };
        Ok(Some(DisposableResource {
            hint,
            value: value.clone(),
            method,
            argument: None,
            wrapper_realm: (hint == DisposalHint::AsyncFromSync)
                .then(|| self.record.global_environment.clone().expect("initialized")),
        }))
    }

    fn dispose_stack_resources(
        &mut self,
        stack: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        // Set disposed before invoking user code. Taking the list also releases
        // all captures on completion or host failure; no default quota applies.
        let resources = self.object_work(span, |objects, budget| {
            objects.take_disposable_resources(stack, budget)
        })?;
        self.dispose_resource_list(resources, Ok(crate::Completion::normal(None)), span)?;
        Ok(Value::Undefined)
    }

    fn disposable_stack_intrinsic_prototype(&self, kind: StackKind) -> ObjectHandle {
        let intrinsic = self.record.intrinsics.as_ref().expect("initialized");
        match kind {
            StackKind::Sync => intrinsic.disposable_stack.prototype.clone(),
            StackKind::Async => intrinsic.async_disposable_stack.prototype.clone(),
        }
    }

    pub(super) fn disposable_stack_brand_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no matching disposable state internal slot",
        )
    }
}
