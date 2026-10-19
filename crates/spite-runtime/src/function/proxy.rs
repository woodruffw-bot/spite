//! Proxy creation, revocation and invocation (10.5.12–15, 28.2).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, ProxyState},
};
use spite_core::{JsString, Span};

pub(super) enum ProxyInvocation {
    Forward(ObjectHandle),
    Complete(Value),
}

impl Realm {
    pub(super) fn proxy_constructor(
        &mut self,
        target: Value,
        handler: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // ProxyCreate validates target before handler; revoked Proxy objects
        // remain objects and are valid creation inputs. No newTarget Get occurs.
        let Value::Object(target) = target else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Proxy target must be an object",
            ));
        };
        let Value::Object(handler) = handler else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Proxy handler must be an object",
            ));
        };
        self.object_work(span, |objects, budget| {
            objects.create_proxy(&target, &handler, budget)
        })
        .map(Value::Object)
    }

    pub(super) fn proxy_revocable(
        &mut self,
        target: Value,
        handler: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(proxy) = self.proxy_constructor(target, handler, span)? else {
            unreachable!("ProxyCreate returns an object")
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .function_prototype
            .clone();
        let revoker = self.object_work(span, |objects, budget| {
            objects.create_proxy_revoker(&prototype, &proxy, budget)
        })?;
        self.define_builtin_property(&revoker, "length", Value::Number(0.0), false, span)?;
        self.define_builtin_property(
            &revoker,
            "name",
            Value::String(JsString::from("")),
            false,
            span,
        )?;
        let object_prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .object_prototype
            .clone();
        let result =
            self.object_work(span, |objects, _| objects.create(Some(&object_prototype)))?;
        for (key, value) in [("proxy", proxy), ("revoke", revoker)] {
            self.define_property_or_throw(
                &result,
                JsString::from(key),
                DataDescriptor {
                    value: Some(Value::Object(value)),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(Value::Object(result))
    }

    pub(super) fn proxy_call(
        &mut self,
        state: ProxyState,
        this: &Value,
        arguments: &[Value],
        span: Span,
    ) -> Result<ProxyInvocation, Error> {
        let ProxyState::Live(snapshot) = state else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "revoked Proxy",
            ));
        };
        let trap =
            self.proxy_get_method(&snapshot.handler, JsString::from("apply").into(), span)?;
        let Some(trap) = trap else {
            return Ok(ProxyInvocation::Forward(snapshot.target));
        };
        // The real Array is created only after GetMethod succeeds. Snapshot
        // target/handler survive reentrant revocation during that lookup.
        let array = self.proxy_argument_array(arguments, span)?;
        let this = self.proxy_copy_value(this, span)?;
        let values =
            self.proxy_trap_arguments([Value::Object(snapshot.target), this, array], span)?;
        self.call(
            Value::Object(trap),
            Value::Object(snapshot.handler),
            values,
            span,
        )
        .map(ProxyInvocation::Complete)
    }

    pub(super) fn proxy_construct(
        &mut self,
        state: ProxyState,
        arguments: &[Value],
        new_target: &ObjectHandle,
        span: Span,
    ) -> Result<ProxyInvocation, Error> {
        let ProxyState::Live(snapshot) = state else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "revoked Proxy",
            ));
        };
        let trap =
            self.proxy_get_method(&snapshot.handler, JsString::from("construct").into(), span)?;
        let Some(trap) = trap else {
            return Ok(ProxyInvocation::Forward(snapshot.target));
        };
        let array = self.proxy_argument_array(arguments, span)?;
        let values = self.proxy_trap_arguments(
            [
                Value::Object(snapshot.target),
                array,
                Value::Object(new_target.clone()),
            ],
            span,
        )?;
        let result = self.call(
            Value::Object(trap),
            Value::Object(snapshot.handler),
            values,
            span,
        )?;
        if !matches!(result, Value::Object(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Proxy construct trap must return an object",
            ));
        }
        Ok(ProxyInvocation::Complete(result))
    }

    // CreateArrayFromList over borrowed arguments: charge an arbitrary Value
    // before cloning it, then let the ordinary own definition store that copy.
    fn proxy_argument_array(&mut self, arguments: &[Value], span: Span) -> Result<Value, Error> {
        let array = self.create_intrinsic_array(0, span)?;
        for (index, value) in arguments.iter().enumerate() {
            let value = self.proxy_copy_value(value, span)?;
            self.define_property_or_throw(
                &array,
                JsString::from(index.to_string().as_str()),
                DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(Value::Object(array))
    }

    fn proxy_copy_value(&mut self, value: &Value, span: Span) -> Result<Value, Error> {
        self.object_work(span, |_, budget| {
            budget.value(value)?;
            Ok(value.clone())
        })
    }

    fn proxy_trap_arguments(
        &mut self,
        values: [Value; 3],
        span: Span,
    ) -> Result<Vec<Value>, Error> {
        self.object_work(span, |_, budget| budget.charge(values.len()))?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(values.len())
            .map_err(|_| Error::Limit {
                span,
                message: "Proxy trap argument allocation failed".into(),
            })?;
        result.extend(values);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_bigint_receiver_copy_aborts_before_the_native_trap() {
        for large in [false, true] {
            let mut realm = Realm::default();
            let Value::Object(proxy) = realm
                .eval(
                    "let getterCalls=0; function target(){}; \
                     new Proxy(target,{get apply(){getterCalls+=1; \
                     return Object.preventExtensions;}})",
                )
                .unwrap()
            else {
                panic!("Proxy")
            };
            let Value::Object(target) = realm.eval("target").unwrap() else {
                panic!("ordinary callable target")
            };
            let this = if large {
                realm.eval("1n << 400000n").unwrap()
            } else {
                Value::Number(0.0)
            };
            if let Value::BigInt(value) = &this {
                assert_eq!(value.bit_length(), 400_001);
            }
            let span = Span::new(0, 0);
            let state = realm
                .object_work(span, |objects, budget| objects.proxy_state(&proxy, budget))
                .unwrap()
                .expect("Proxy state");
            // The native trap uses only argument0 (the target), avoiding any
            // second language-function argument copy of the BigInt receiver.
            realm.remaining_steps = Some(10_000);
            let result = realm.proxy_call(state, &this, &[], span);
            realm.remaining_steps = None;
            assert_eq!(realm.eval("getterCalls").unwrap(), Value::Number(1.0));
            let extensible = realm.objects.inspect(&target).unwrap().is_extensible();
            if large {
                assert!(matches!(result, Err(Error::Limit { .. })));
                assert!(extensible);
            } else {
                assert!(
                    matches!(result, Ok(ProxyInvocation::Complete(Value::Object(value))) if value == target)
                );
                assert!(!extensible);
            }
        }
    }
}
