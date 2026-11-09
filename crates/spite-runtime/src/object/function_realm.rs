//! Function creation Realms use traced, checked environment handles (9.3, 7.3.24).

use super::{Error, Objects};
use crate::{environment::EnvironmentHandle, function::Callable};
use spite_heap::Handle;

impl Objects {
    pub(crate) fn set_active_function_realm(
        &mut self,
        realm: &EnvironmentHandle,
    ) -> Result<(), Error> {
        self.environment(realm)?;
        self.active_realm = Some(realm.clone());
        Ok(())
    }

    pub(crate) fn clear_active_function_realm(&mut self) {
        self.active_realm = None;
    }

    pub(super) fn active_function_realm(&self) -> Result<Option<EnvironmentHandle>, Error> {
        if let Some(realm) = &self.active_realm {
            self.environment(realm)?;
        }
        Ok(self.active_realm.clone())
    }

    pub(super) fn initialize_callable(
        &mut self,
        function: &Handle,
        callable: Callable,
    ) -> Result<(), Error> {
        let realm = self.active_function_realm()?;
        let record = self.object_mut(function)?;
        if record.callable.is_some() || record.global {
            return Err(Error::WrongKind);
        }
        record.realm = realm;
        record.callable = Some(callable);
        Ok(())
    }

    pub(crate) fn initialize_global_realm(
        &mut self,
        global: &Handle,
        realm: &EnvironmentHandle,
    ) -> Result<(), Error> {
        self.environment(realm)?;
        let record = self.object_mut(global)?;
        if record.realm.is_some() || record.global || record.callable.is_some() {
            return Err(Error::WrongKind);
        }
        record.realm = Some(realm.clone());
        record.global = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BindingState, Value, object::Budget};
    use std::collections::BTreeMap;

    #[test]
    fn global_realm_is_immutable_traced_and_separate_from_function_realm() {
        let mut objects = Objects::with_limits(None, None);
        let mut budget = Budget::with_limits(None, None);
        let realm = objects
            .create_environment(None, BTreeMap::new(), &mut budget)
            .unwrap();
        let global = objects.create(None).unwrap();
        objects.initialize_global_realm(&global, &realm).unwrap();
        let object = objects.inspect(&global).unwrap();
        assert_eq!(object.global_realm(), Some(&realm));
        assert!(object.function_realm().is_none());
        assert!(!object.is_callable());
        assert_eq!(
            objects.initialize_global_realm(&global, &realm),
            Err(Error::WrongKind)
        );
        assert_eq!(
            objects
                .initialize_callable(&global, Callable::Builtin(crate::function::Builtin::Number)),
            Err(Error::WrongKind)
        );
        objects.collect([&global], usize::MAX).unwrap();
        assert!(objects.environment(&realm).is_ok());
        objects.collect([], usize::MAX).unwrap();
        assert!(objects.environment(&realm).is_err());
    }

    #[test]
    fn builtin_creation_realm_is_immutable_and_traces_its_global_bindings() {
        let mut objects = Objects::with_limits(None, None);
        let mut budget = Budget::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let held = objects.create(None).unwrap();
        let first = objects
            .create_environment(
                None,
                BTreeMap::from([(
                    "held".into(),
                    BindingState {
                        value: Some(Value::Object(held.clone())),
                        mutable: true,
                        deletable: false,
                        strict: false,
                    },
                )]),
                &mut budget,
            )
            .unwrap();
        let second = objects
            .create_environment(None, BTreeMap::new(), &mut budget)
            .unwrap();
        objects.set_active_function_realm(&first).unwrap();
        let one = objects
            .create_builtin(&prototype, crate::function::Builtin::Number)
            .unwrap();
        objects.set_active_function_realm(&second).unwrap();
        let two = objects
            .create_builtin(&prototype, crate::function::Builtin::Number)
            .unwrap();
        assert_eq!(
            objects.inspect(&one).unwrap().function_realm(),
            Some(&first)
        );
        assert_eq!(
            objects.inspect(&two).unwrap().function_realm(),
            Some(&second)
        );
        objects.set_active_function_realm(&first).unwrap();
        objects.collect([&one], usize::MAX).unwrap();
        assert!(objects.environment(&first).is_ok());
        assert!(objects.inspect(&held).is_ok());
        assert!(objects.environment(&second).is_err());
        assert!(objects.inspect(&two).is_err());
    }
}
