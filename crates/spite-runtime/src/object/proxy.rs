//! Proxy state and revocation (10.5, 28.2). JavaScript hooks belong to the Realm.

use super::{Budget, Error, Objects};
use crate::function::Callable;
use spite_heap::Handle;

#[derive(Clone, Debug)]
pub(crate) struct ProxySnapshot {
    pub(crate) target: Handle,
    pub(crate) handler: Handle,
}

#[derive(Clone, Debug)]
pub(crate) enum ProxyState {
    Live(ProxySnapshot),
    Revoked,
}

#[derive(Debug)]
pub(super) struct ProxyData {
    // One optional pair prevents partially cleared target/handler state.
    pub(super) snapshot: Option<ProxySnapshot>,
    pub(super) callable: bool,
}

impl Objects {
    pub(crate) fn proxy_state(
        &self,
        object: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<ProxyState>, Error> {
        let Some(proxy) = &self.inspect(object)?.proxy else {
            // Ordinary operations retain their exact existing work accounting.
            return Ok(None);
        };
        let Some(snapshot) = &proxy.snapshot else {
            return Ok(Some(ProxyState::Revoked));
        };
        budget.charge(2)?;
        Ok(Some(ProxyState::Live(snapshot.clone())))
    }

    pub(crate) fn revoke_proxy_from(
        &mut self,
        revoker: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let Some(Callable::ProxyRevoker(capture)) = self.inspect(revoker)?.callable() else {
            return Err(Error::WrongKind);
        };
        let Some(proxy) = capture else {
            return Ok(());
        };
        if !self.inspect(proxy)?.is_proxy() {
            return Err(Error::WrongKind);
        }
        // All checks and charges precede either mutation. Exclusive Objects
        // access and the absence of callbacks/allocation preserve these handles.
        budget.charge(3)?;
        let proxy = proxy.clone();
        let Some(Callable::ProxyRevoker(capture)) = &mut self
            .object_mut(revoker)
            .expect("validated revoker remains live")
            .callable
        else {
            unreachable!("validated revoker retains its callable kind")
        };
        *capture = None;
        self.object_mut(&proxy)
            .expect("validated Proxy remains live")
            .proxy
            .as_mut()
            .expect("validated Proxy retains its slot")
            .snapshot = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Value,
        function::{BoundFunction, Builtin},
        object::{
            DataDescriptor, DescriptorKind, GetAction, Property, PropertyDescriptor, SetAction,
            complete_property_descriptor, is_compatible_property_descriptor,
        },
    };
    use spite_core::{JsString, JsSymbol};

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    #[test]
    fn snapshots_preserve_checked_identity_and_do_not_charge_ordinary_records() {
        let mut objects = Objects::with_limits(None, None);
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let mut none = Budget::new(0);
        assert!(objects.proxy_state(&target, &mut none).unwrap().is_none());
        assert_eq!(none.remaining_work(), Some(0));
        assert!(matches!(
            objects.proxy_state(&proxy, &mut Budget::new(1)),
            Err(Error::WorkLimit)
        ));
        let mut exact = Budget::new(2);
        let Some(ProxyState::Live(snapshot)) = objects.proxy_state(&proxy, &mut exact).unwrap()
        else {
            panic!("attached Proxy")
        };
        assert_eq!((&snapshot.target, &snapshot.handler), (&target, &handler));
        assert_eq!(exact.remaining_work(), Some(0));
        let mut other = Objects::with_limits(None, None);
        let foreign = other.create(None).unwrap();
        assert!(matches!(
            objects.proxy_state(&foreign, &mut unlimited()),
            Err(Error::Heap(_))
        ));
    }

    #[test]
    fn callable_and_constructor_capabilities_survive_revocation_without_copying_brands() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let constructor = objects.create_builtin(&prototype, Builtin::Object).unwrap();
        let callable = objects
            .create_builtin(&prototype, Builtin::ReflectApply)
            .unwrap();
        let array = objects.create_array(None, 0, &mut unlimited()).unwrap();
        for (target, call, construct) in [
            (&constructor, true, true),
            (&callable, true, false),
            (&array, false, false),
        ] {
            let proxy = objects
                .create_proxy(target, &handler, &mut unlimited())
                .unwrap();
            let revoker = objects
                .create_proxy_revoker(&prototype, &proxy, &mut unlimited())
                .unwrap();
            assert_eq!(objects.inspect(&proxy).unwrap().is_callable(), call);
            assert_eq!(objects.inspect(&proxy).unwrap().is_constructor(), construct);
            assert!(!objects.inspect(&proxy).unwrap().is_array());
            objects
                .revoke_proxy_from(&revoker, &mut unlimited())
                .unwrap();
            assert_eq!(objects.inspect(&proxy).unwrap().is_callable(), call);
            assert_eq!(objects.inspect(&proxy).unwrap().is_constructor(), construct);
            // ProxyCreate accepts revoked object inputs without reading their slots.
            objects
                .create_proxy(&proxy, &proxy, &mut unlimited())
                .unwrap();
        }
    }

    #[test]
    fn failed_creation_preserves_inputs_and_revoker_kind_checks() {
        let mut objects = Objects::new(2, 4);
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        assert!(matches!(
            objects.create_proxy(&target, &handler, &mut Budget::new(2)),
            Err(Error::WorkLimit)
        ));
        assert!(matches!(
            objects.create_proxy(&target, &handler, &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.create_proxy_revoker(&handler, &target, &mut unlimited()),
            Err(Error::WrongKind)
        ));
        assert!(matches!(
            objects.revoke_proxy_from(&target, &mut unlimited()),
            Err(Error::WrongKind)
        ));
        assert_eq!(
            objects
                .collect([&target, &handler], usize::MAX)
                .unwrap()
                .live,
            2
        );
        assert!(!objects.inspect(&target).unwrap().is_proxy());
    }

    #[test]
    fn revocation_is_failure_atomic_at_full_heap_and_idempotent() {
        let mut objects = Objects::new(5, 4);
        let prototype = objects.create(None).unwrap();
        let target = objects.create_builtin(&prototype, Builtin::Object).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let revoker = objects
            .create_proxy_revoker(&prototype, &proxy, &mut unlimited())
            .unwrap();
        assert!(objects.create(None).is_err());
        assert!(matches!(
            objects.revoke_proxy_from(&revoker, &mut Budget::new(2)),
            Err(Error::WorkLimit)
        ));
        assert!(matches!(
            objects.proxy_state(&proxy, &mut unlimited()).unwrap(),
            Some(ProxyState::Live(_))
        ));
        assert!(
            matches!(objects.inspect(&revoker).unwrap().callable(), Some(Callable::ProxyRevoker(Some(handle))) if handle == &proxy)
        );
        let Some(ProxyState::Live(snapshot)) =
            objects.proxy_state(&proxy, &mut unlimited()).unwrap()
        else {
            panic!("attached Proxy")
        };
        objects
            .revoke_proxy_from(&revoker, &mut Budget::new(3))
            .unwrap();
        assert!(matches!(
            objects.proxy_state(&proxy, &mut Budget::new(0)).unwrap(),
            Some(ProxyState::Revoked)
        ));
        assert!(matches!(
            objects.inspect(&revoker).unwrap().callable(),
            Some(Callable::ProxyRevoker(None))
        ));
        objects
            .revoke_proxy_from(&revoker, &mut Budget::new(0))
            .unwrap();
        assert!(objects.inspect(&proxy).unwrap().is_callable());
        assert!(objects.inspect(&proxy).unwrap().is_constructor());
        assert_eq!((&snapshot.target, &snapshot.handler), (&target, &handler));
    }

    #[test]
    fn raw_internal_methods_fail_closed_on_direct_and_inherited_proxies() {
        let mut objects = Objects::with_limits(None, None);
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let child = objects.create(Some(&proxy)).unwrap();
        let key = JsString::from("x");
        for object in [&proxy, &child] {
            assert!(matches!(
                objects.get(object, &key, &mut unlimited()),
                Err(Error::RequiresRuntime)
            ));
            assert!(matches!(
                objects.has(object, &key, &mut unlimited()),
                Err(Error::RequiresRuntime)
            ));
            assert!(matches!(
                objects.set(
                    object,
                    key.clone(),
                    Value::Undefined,
                    Some(object),
                    &mut unlimited()
                ),
                Err(Error::RequiresRuntime)
            ));
        }
        assert!(matches!(
            objects.get_own(&proxy, &key, &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.has_own(&proxy, &key, &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.own_keys(&proxy, &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.define(
                &proxy,
                key.clone(),
                DataDescriptor::default(),
                &mut unlimited()
            ),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.delete(&proxy, &key, &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.prevent_extensions(&proxy),
            Err(Error::RequiresRuntime)
        ));
        assert!(matches!(
            objects.set_prototype(&proxy, None, &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        assert_eq!(
            objects.get_own(&child, &key, &mut unlimited()).unwrap(),
            None
        );
        assert!(objects.inspect(&proxy).unwrap().is_proxy());
    }

    #[test]
    fn raw_set_preserves_rejection_and_accessor_priority_before_proxy_receiver() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let key = JsString::from("x");
        objects
            .define(
                &target,
                key.clone(),
                DataDescriptor {
                    value: Some(Value::Number(1.0)),
                    writable: Some(false),
                    configurable: Some(true),
                    ..Default::default()
                },
                &mut unlimited(),
            )
            .unwrap();
        assert_eq!(
            objects
                .set(
                    &target,
                    key.clone(),
                    Value::Undefined,
                    Some(&proxy),
                    &mut unlimited()
                )
                .unwrap(),
            SetAction::Done(false)
        );
        objects
            .define(
                &target,
                key.clone(),
                DataDescriptor {
                    writable: Some(true),
                    ..Default::default()
                },
                &mut unlimited(),
            )
            .unwrap();
        assert!(matches!(
            objects.set(
                &target,
                key.clone(),
                Value::Undefined,
                Some(&proxy),
                &mut unlimited()
            ),
            Err(Error::RequiresRuntime)
        ));
        let setter = objects
            .create_builtin(&prototype, Builtin::ReflectApply)
            .unwrap();
        objects
            .define(
                &target,
                key.clone(),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: None,
                        set: Some(Some(setter.clone())),
                    },
                    ..Default::default()
                },
                &mut unlimited(),
            )
            .unwrap();
        assert_eq!(
            objects
                .set(
                    &target,
                    key,
                    Value::Undefined,
                    Some(&proxy),
                    &mut unlimited()
                )
                .unwrap(),
            SetAction::Call(setter)
        );
    }

    #[test]
    fn ordinary_prototype_cycle_walk_stops_at_live_and_revoked_proxy_candidates() {
        let mut objects = Objects::with_limits(None, None);
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        assert!(
            objects
                .set_prototype(&target, Some(&proxy), &mut unlimited())
                .unwrap()
        );
        let revoker = objects
            .create_proxy_revoker(&handler, &proxy, &mut unlimited())
            .unwrap();
        objects
            .revoke_proxy_from(&revoker, &mut unlimited())
            .unwrap();
        let next = objects.create(None).unwrap();
        assert!(
            objects
                .set_prototype(&next, Some(&proxy), &mut unlimited())
                .unwrap()
        );
        let a = objects.create(None).unwrap();
        let b = objects.create(Some(&a)).unwrap();
        assert!(
            !objects
                .set_prototype(&a, Some(&b), &mut unlimited())
                .unwrap()
        );
    }

    #[test]
    fn bound_allocation_uses_the_already_observed_prototype_and_cached_capability() {
        let mut objects = Objects::with_limits(None, None);
        let initial = objects.create(None).unwrap();
        let observed = objects.create(None).unwrap();
        let target = objects.create_builtin(&initial, Builtin::Object).unwrap();
        let proxy = objects
            .create_proxy(&target, &initial, &mut unlimited())
            .unwrap();
        let bound = || BoundFunction {
            target: proxy.clone(),
            this: Value::Undefined,
            arguments: Vec::new(),
        };
        assert!(matches!(
            objects.create_bound(bound(), &mut unlimited()),
            Err(Error::RequiresRuntime)
        ));
        let revoker = objects
            .create_proxy_revoker(&initial, &proxy, &mut unlimited())
            .unwrap();
        objects
            .revoke_proxy_from(&revoker, &mut unlimited())
            .unwrap();
        let function = objects
            .create_bound_with_prototype(bound(), Some(&observed), &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.inspect(&function).unwrap().prototype(),
            Some(&observed)
        );
        assert!(objects.inspect(&function).unwrap().is_constructor());
        assert_eq!(
            objects
                .get(&function, &JsString::from("missing"), &mut unlimited())
                .unwrap(),
            GetAction::Value(Value::Undefined)
        );
    }

    #[test]
    fn revoker_traces_proxy_target_handler_and_releases_all_three_on_revoke() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let revoker = objects
            .create_proxy_revoker(&prototype, &proxy, &mut unlimited())
            .unwrap();
        assert_eq!(objects.collect([&revoker], usize::MAX).unwrap().live, 5);
        objects
            .revoke_proxy_from(&revoker, &mut unlimited())
            .unwrap();
        assert_eq!(objects.collect([&revoker], usize::MAX).unwrap().live, 2);
        for handle in [&proxy, &target, &handler] {
            assert!(objects.inspect(handle).is_err());
        }
        assert!(objects.inspect(&revoker).is_ok());
    }

    #[test]
    fn symbol_ephemerons_follow_handle_edges_without_new_physical_symbol_cells() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        let handler = objects.create(None).unwrap();
        let proxy = objects
            .create_proxy(&target, &handler, &mut unlimited())
            .unwrap();
        let revoker = objects
            .create_proxy_revoker(&prototype, &proxy, &mut unlimited())
            .unwrap();
        let map = objects.create_weak_map(&prototype).unwrap();
        let value = objects.create(None).unwrap();
        let key = JsSymbol::new(None);
        objects
            .define(
                &handler,
                JsString::from("key"),
                DataDescriptor {
                    value: Some(Value::Symbol(key.clone())),
                    ..Default::default()
                },
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
            .unwrap();
        drop(key);
        assert_eq!(
            objects.collect([&map, &revoker], usize::MAX).unwrap().live,
            7
        );
        objects
            .revoke_proxy_from(&revoker, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.collect([&map, &revoker], usize::MAX).unwrap().live,
            3
        );
        for handle in [&value, &target, &handler, &proxy] {
            assert!(objects.inspect(handle).is_err());
        }
    }

    #[test]
    fn borrowed_compatibility_preserves_frozen_same_value_and_omitted_fields() {
        let current = complete_property_descriptor(
            DataDescriptor {
                value: Some(Value::Number(-0.0)),
                ..Default::default()
            }
            .into(),
        );
        let same = PropertyDescriptor::from(DataDescriptor {
            value: Some(Value::Number(-0.0)),
            ..Default::default()
        });
        let other = PropertyDescriptor::from(DataDescriptor {
            value: Some(Value::Number(0.0)),
            ..Default::default()
        });
        assert!(is_compatible_property_descriptor(
            false,
            &same,
            Some(&current)
        ));
        assert!(!is_compatible_property_descriptor(
            false,
            &other,
            Some(&current)
        ));
        assert!(!is_compatible_property_descriptor(false, &same, None));
        let generic = PropertyDescriptor {
            kind: DescriptorKind::Accessor {
                get: None,
                set: None,
            },
            ..Default::default()
        };
        assert!(is_compatible_property_descriptor(
            false,
            &generic,
            Some(&current)
        ));
        let accessor = PropertyDescriptor {
            kind: DescriptorKind::Accessor {
                get: Some(None),
                set: None,
            },
            ..Default::default()
        };
        assert!(!is_compatible_property_descriptor(
            false,
            &accessor,
            Some(&current)
        ));
        assert!(
            matches!(current, Property::Data(data) if matches!(data.value, Value::Number(value) if value.is_sign_negative()))
        );
    }
}
