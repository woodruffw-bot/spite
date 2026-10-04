//! Unmapped arguments objects (10.4.4.6), before Symbol.iterator is exposed.

use crate::{
    Error, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn unmapped_arguments(
        &mut self,
        arguments: &[Value],
        span: Span,
    ) -> Result<Value, Error> {
        let intrinsics = self
            .intrinsics
            .as_ref()
            .expect("function intrinsics initialized");
        let prototype = intrinsics.object_prototype.clone();
        let thrower = intrinsics.throw_type_error.clone();
        let object = self.object_work(span, |objects, _| objects.create_arguments(&prototype))?;
        self.define_builtin_property(
            &object,
            "length",
            Value::Number(arguments.len() as f64),
            true,
            span,
        )?;
        for (index, value) in arguments.iter().enumerate() {
            self.object_work(span, |objects, budget| {
                budget.value(value)?;
                objects.define(
                    &object,
                    JsString::from(index.to_string().as_str()),
                    DataDescriptor {
                        value: Some(value.clone()),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )
            })?;
        }
        // The required @@iterator hook will be installed with Symbol/Array iteration;
        // no Symbol property keys or reflection operations are exposed yet.
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                JsString::from("callee"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(thrower.clone())),
                        set: Some(Some(thrower)),
                    },
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        Ok(Value::Object(object))
    }
}
