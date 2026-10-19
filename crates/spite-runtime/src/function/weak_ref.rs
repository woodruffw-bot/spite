//! WeakRef construction and dereferencing (26.1), with explicit host checkpoints.

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn weak_ref_constructor(
        &mut self,
        new_target: ObjectHandle,
        target: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 26.1.1.1: no-new is rejected by the ordinary builtin call dispatcher.
        // CanBeHeldWeakly precedes every observable newTarget.prototype access.
        if !self.can_be_held_weakly(&target, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "WeakRef target cannot be held weakly",
            ));
        }
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .global_metadata
                .weak_ref_prototype
                .clone()
        };
        let weak_ref = self.object_work(span, |objects, budget| {
            objects.create_weak_ref(&prototype, &target, budget)
        })?;
        // The weak slot is unpublished until return. No callback or collection
        // occurs between its allocation and this AddToKeptObjects operation.
        self.add_to_kept_objects(&target, span)?;
        Ok(Value::Object(weak_ref))
    }

    pub(super) fn weak_ref_deref(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let Value::Object(weak_ref) = receiver else {
            return Err(Self::weak_ref_brand_error(span));
        };
        // RequireInternalSlot does not unwrap Proxy receivers or consult their
        // properties/prototypes, including when the Proxy has been revoked.
        if !self.object_work(span, |objects, _| objects.weak_ref_has_slot(&weak_ref))? {
            return Err(Self::weak_ref_brand_error(span));
        }
        let Some(target) = self.object_work(span, |objects, budget| {
            objects.weak_ref_target(&weak_ref, budget)
        })?
        else {
            return Ok(Value::Undefined);
        };
        // An upgraded Symbol is already a strong owner across this transfer.
        // A live object cannot be collected during evaluation.
        self.add_to_kept_objects(&target, span)?;
        Ok(target)
    }

    fn weak_ref_brand_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no WeakRefTarget internal slot",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kept_root_work_failure_preserves_old_targets_without_keeping_the_new_one() {
        let mut realm = Realm::default();
        let old = realm
            .eval("let keptFailureRef=new WeakRef({});keptFailureRef.deref()")
            .unwrap();
        let new = realm.eval("({})").unwrap();
        let Value::Object(old_handle) = old else {
            panic!("Expected old target")
        };
        let Value::Object(new_handle) = &new else {
            panic!("Expected new target")
        };
        let kept_count = realm.kept_alive.len();
        // This allowance reaches the kept-list precharge but cannot register
        // the new object root. The fixture itself uses unlimited setup.
        realm.remaining_steps = Some(2);
        assert!(matches!(
            realm.add_to_kept_objects(&new, Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.kept_alive.len(), kept_count);
        realm.collect(usize::MAX).unwrap();
        assert!(realm.inspect_object(&old_handle).is_ok());
        assert!(realm.inspect_object(new_handle).is_err());
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(realm.inspect_object(&old_handle).is_err());
    }

    #[test]
    fn active_checkpoint_refusal_preserves_roots_and_inactive_clear_ignores_work_quota() {
        let mut realm = Realm::default();
        realm.clear_kept_objects().unwrap();
        assert!(realm.intrinsics.is_none());
        let Value::Object(target) = realm
            .eval("let checkpointRef=new WeakRef({});checkpointRef.deref()")
            .unwrap()
        else {
            panic!("Expected target")
        };
        let kept_count = realm.kept_alive.len();
        realm.remaining_steps = Some(0);
        for (call_depth, evaluation_depth) in [(1, 0), (0, 1)] {
            realm.call_depth = call_depth;
            realm.evaluation_depth = evaluation_depth;
            assert!(matches!(
                realm.clear_kept_objects(),
                Err(Error::Host { .. })
            ));
            assert_eq!(realm.kept_alive.len(), kept_count);
            assert_eq!(realm.remaining_steps, Some(0));
        }
        realm.call_depth = 0;
        realm.evaluation_depth = 0;
        realm.collect(usize::MAX).unwrap();
        assert!(realm.inspect_object(&target).is_ok());
        realm.clear_kept_objects().unwrap();
        assert_eq!(realm.remaining_steps, Some(0));
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(realm.inspect_object(&target).is_err());
    }
}
