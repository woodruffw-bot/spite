//! Explicit Test262 host capabilities; ordinary Realms receive no host object.

use super::Builtin;
use crate::{
    Error, ExceptionKind, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span};

impl Realm {
    /// Installs a fresh partial Test262 `$262` host object in this realm.
    ///
    /// The object exposes `global` and native `detachArrayBuffer`. Accessing
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
        let detach = self.new_builtin(&function_prototype, Builtin::DetachArrayBuffer, span)?;
        self.define_builtin_property(
            &host,
            "detachArrayBuffer",
            Value::Object(detach),
            true,
            span,
        )?;
        self.define_builtin_property(
            &host,
            "global",
            Value::Object(self.global_object()),
            true,
            span,
        )?;
        for (name, getter_name) in [
            ("createRealm", "get createRealm"),
            ("evalScript", "get evalScript"),
            ("gc", "get gc"),
            ("agent", "get agent"),
            ("IsHTMLDDA", "get IsHTMLDDA"),
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
