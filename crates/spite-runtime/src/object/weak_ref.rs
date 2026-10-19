//! WeakRef targets do not contribute strong object or Symbol edges (26.1).

use super::{Budget, Error, Objects};
use crate::Value;
use spite_core::WeakJsSymbol;
use spite_heap::Handle;

#[derive(Debug)]
enum Target {
    Object(Handle),
    Symbol(WeakJsSymbol),
}

#[derive(Debug)]
pub(super) struct WeakRefData {
    target: Target,
}

impl Objects {
    // The Realm checks CanBeHeldWeakly, including its Symbol registry, first.
    // Storage checks heap identities and stores no owning target edge.
    pub(crate) fn create_weak_ref(
        &mut self,
        prototype: &Handle,
        target: &Value,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let target = match target {
            Value::Object(object) => {
                self.inspect(object)?;
                budget.charge(1)?;
                Target::Object(object.clone())
            }
            Value::Symbol(symbol) => {
                budget.charge(1)?;
                Target::Symbol(symbol.downgrade())
            }
            _ => return Err(Error::WrongKind),
        };
        let weak_ref = self.create(Some(prototype))?;
        self.object_mut(&weak_ref)?.weak_ref = Some(Box::new(WeakRefData { target }));
        Ok(weak_ref)
    }

    pub(crate) fn weak_ref_has_slot(&self, weak_ref: &Handle) -> Result<bool, Error> {
        Ok(self.inspect(weak_ref)?.weak_ref.is_some())
    }

    pub(crate) fn weak_ref_target(
        &self,
        weak_ref: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<Value>, Error> {
        let data = self
            .inspect(weak_ref)?
            .weak_ref
            .as_deref()
            .ok_or(Error::WrongKind)?;
        budget.charge(1)?;
        match &data.target {
            Target::Object(target) => match self.inspect(target) {
                Ok(_) => Ok(Some(Value::Object(target.clone()))),
                Err(Error::Heap(spite_heap::Error::StaleHandle)) => Ok(None),
                Err(error) => Err(error),
            },
            Target::Symbol(target) => Ok(target.upgrade().map(Value::Symbol)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spite_core::JsSymbol;

    #[test]
    fn weak_object_targets_do_not_trace_and_expired_generations_cannot_revive() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        let mut budget = Budget::with_work_limit(None);
        let weak_ref = objects
            .create_weak_ref(&prototype, &Value::Object(target.clone()), &mut budget)
            .unwrap();
        assert!(!objects.weak_ref_has_slot(&prototype).unwrap());
        assert!(objects.weak_ref_has_slot(&weak_ref).unwrap());
        assert_eq!(
            objects.weak_ref_target(&weak_ref, &mut budget).unwrap(),
            Some(Value::Object(target.clone()))
        );
        let collection = objects.collect([&weak_ref], usize::MAX).unwrap();
        assert_eq!((collection.live, collection.reclaimed), (2, 1));
        assert_eq!(
            objects.weak_ref_target(&weak_ref, &mut budget).unwrap(),
            None
        );
        let replacement = objects.create(None).unwrap();
        assert_ne!(target, replacement);
        objects
            .collect([&weak_ref, &replacement], usize::MAX)
            .unwrap();
        assert_eq!(
            objects.weak_ref_target(&weak_ref, &mut budget).unwrap(),
            None
        );
        assert!(objects.inspect(&replacement).is_ok());
    }

    #[test]
    fn weak_symbol_targets_expire_without_strong_owners() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let symbol = JsSymbol::new(None);
        let weak = symbol.downgrade();
        let mut budget = Budget::with_work_limit(None);
        let weak_ref = objects
            .create_weak_ref(&prototype, &Value::Symbol(symbol.clone()), &mut budget)
            .unwrap();
        assert_eq!(
            objects.weak_ref_target(&weak_ref, &mut budget).unwrap(),
            Some(Value::Symbol(symbol.clone()))
        );
        drop(symbol);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            objects.weak_ref_target(&weak_ref, &mut budget).unwrap(),
            None
        );
        assert_eq!(objects.collect([&weak_ref], usize::MAX).unwrap().live, 2);
    }

    #[test]
    fn invalid_handles_brands_and_work_fail_before_weak_slot_creation() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        let stale = objects.create(None).unwrap();
        objects.collect([&prototype, &target], usize::MAX).unwrap();
        let foreign = Objects::with_limits(None, None).create(None).unwrap();
        let mut budget = Budget::with_work_limit(None);
        for bad in [stale, foreign] {
            assert!(matches!(
                objects.create_weak_ref(&prototype, &Value::Object(bad), &mut budget),
                Err(Error::Heap(_))
            ));
        }
        assert!(matches!(
            objects.create_weak_ref(&prototype, &Value::Null, &mut budget),
            Err(Error::WrongKind)
        ));
        assert!(matches!(
            objects.weak_ref_target(&prototype, &mut budget),
            Err(Error::WrongKind)
        ));
        assert!(matches!(
            objects.create_weak_ref(
                &prototype,
                &Value::Object(target.clone()),
                &mut Budget::new(0),
            ),
            Err(Error::WorkLimit)
        ));
        let collection = objects.collect([&prototype, &target], usize::MAX).unwrap();
        assert_eq!((collection.live, collection.reclaimed), (2, 0));
    }

    #[test]
    fn a_full_heap_refuses_creation_without_keeping_the_target() {
        let mut objects = Objects::with_limits(Some(2), None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(None).unwrap();
        assert!(matches!(
            objects.create_weak_ref(
                &prototype,
                &Value::Object(target.clone()),
                &mut Budget::with_work_limit(None),
            ),
            Err(Error::Heap(spite_heap::Error::Capacity))
        ));
        assert!(objects.inspect(&target).is_ok());
        let collection = objects.collect([&prototype], usize::MAX).unwrap();
        assert_eq!((collection.live, collection.reclaimed), (1, 1));
    }
}
