//! Reflect object, apply, and construct (28.1.1–2, 28.1.14).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct ReflectIntrinsics {
    pub object: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl ReflectIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        std::iter::once(&self.object).chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn reflect_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ReflectIntrinsics, Error> {
        let object = self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Reflect"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        let mut methods = Vec::new();
        for builtin in [Builtin::ReflectApply, Builtin::ReflectConstruct] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &object,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        Ok(ReflectIntrinsics { object, methods })
    }

    pub(super) fn reflect_construct(
        &mut self,
        target: Value,
        list: Value,
        new_target: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // Validate constructors before reading the argument list. Explicit
        // undefined differs from an absent third argument (28.1.2).
        if !self.is_constructor(&target, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Reflect.construct requires a constructor target",
            ));
        }
        let new_target = if let Some(value) = new_target {
            if !self.is_constructor(&value, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Reflect.construct requires a constructor newTarget",
                ));
            }
            let Value::Object(object) = value else {
                unreachable!("constructor object")
            };
            Some(object)
        } else {
            None
        };
        let arguments = self.argument_list_from_array_like(list, span)?;
        self.construct_with_new_target(target, arguments, new_target, span)
    }
}
