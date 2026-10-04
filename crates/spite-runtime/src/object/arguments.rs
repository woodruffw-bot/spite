//! Mapped-argument aliases into checked environment cells (10.4.4).

use super::{Budget, Error, Objects, array_index};
use crate::{Value, environment::EnvironmentHandle};
use spite_core::JsString;
use spite_heap::Handle;
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct ParameterMap {
    pub environment: EnvironmentHandle,
    pub names: BTreeMap<u32, String>,
}

impl Objects {
    pub(crate) fn map_arguments(
        &mut self,
        object: &Handle,
        environment: EnvironmentHandle,
        names: BTreeMap<u32, String>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let record = self.inspect(object)?;
        if !record.arguments || record.parameter_map.is_some() {
            return Err(Error::WrongKind);
        }
        let bindings = &self.environment(&environment)?.bindings;
        for (index, name) in &names {
            budget.charge(name.len() + 1)?;
            let key = JsString::from(index.to_string().as_str());
            budget.lookup(record, &key)?;
            if !bindings.get(name).is_some_and(|binding| binding.mutable)
                || !record
                    .own_property(&key)
                    .is_some_and(|property| property.as_data().is_some_and(|data| data.writable))
            {
                return Err(Error::WrongKind);
            }
        }
        self.object_mut(object)?.parameter_map =
            (!names.is_empty()).then_some(ParameterMap { environment, names });
        Ok(())
    }

    pub(super) fn mapped_value(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Option<&Value>, Error> {
        let Some(map) = &self.inspect(object)?.parameter_map else {
            return Ok(None);
        };
        budget.charge(map.names.len() + 1)?;
        let Some(name) = array_index(key).and_then(|index| map.names.get(&index)) else {
            return Ok(None);
        };
        let binding = self
            .environment(&map.environment)?
            .bindings
            .get(name)
            .ok_or(Error::WrongKind)?;
        let value = binding.value.as_ref().ok_or(Error::WrongKind)?;
        Ok(Some(value))
    }

    pub(super) fn mapped_target(
        &self,
        object: &Handle,
        key: &JsString,
        budget: &mut Budget,
    ) -> Result<Option<(EnvironmentHandle, String, u32)>, Error> {
        let Some(map) = &self.inspect(object)?.parameter_map else {
            return Ok(None);
        };
        budget.charge(map.names.len() + 1)?;
        let Some(index) = array_index(key) else {
            return Ok(None);
        };
        let Some(name) = map.names.get(&index) else {
            return Ok(None);
        };
        let binding = self
            .environment(&map.environment)?
            .bindings
            .get(name)
            .ok_or(Error::WrongKind)?;
        if !binding.mutable || binding.value.is_none() {
            return Err(Error::WrongKind);
        }
        budget.charge(name.len() + 1)?;
        Ok(Some((map.environment.clone(), name.clone(), index)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Realm,
        object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
    };

    fn setup() -> (Realm, Handle) {
        let mut realm = Realm::default();
        let Value::Object(arguments)=realm.eval("let get,set,args;function f(a){get=()=>a;set=x=>a=x;args=arguments;}f.call({},1);args").unwrap() else {panic!()};
        (realm, arguments)
    }
    fn descriptor(realm: &Realm, arguments: &Handle) -> super::super::Property {
        realm
            .objects
            .get_own(arguments, &"0".into(), &mut Budget::new(1000))
            .unwrap()
            .unwrap()
    }

    #[test]
    fn non_writable_changes_snapshot_current_parameter_value_and_remove_alias() {
        let (mut realm, arguments) = setup();
        realm.eval("set(7)").unwrap();
        assert_eq!(
            descriptor(&realm, &arguments).as_data().unwrap().value,
            Value::Number(7.0)
        );
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        writable: Some(false),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        realm.eval("set(9)").unwrap();
        assert_eq!(realm.eval("args[0]"), Ok(Value::Number(7.0)));
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        writable: Some(true),
                        value: Some(Value::Number(3.0)),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert_eq!(realm.eval("get()"), Ok(Value::Number(9.0)));
        let (mut realm, arguments) = setup();
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        writable: Some(false),
                        value: Some(Value::Number(7.0)),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert_eq!(realm.eval("get()"), Ok(Value::Number(7.0)));
        realm.eval("set(9)").unwrap();
        assert_eq!(realm.eval("args[0]"), Ok(Value::Number(7.0)));
    }

    #[test]
    fn accessor_conversion_detaches_only_after_successful_definition() {
        let (mut realm, arguments) = setup();
        let Value::Object(getter) = realm.eval("()=>7").unwrap() else {
            panic!()
        };
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(getter)),
                            set: Some(None)
                        },
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert_eq!(realm.eval("set(9);args[0]"), Ok(Value::Number(7.0)));
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        value: Some(Value::Number(3.0)),
                        writable: Some(true),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert_eq!(realm.eval("args[0]=4;get()"), Ok(Value::Number(9.0)));
        let (mut realm, arguments) = setup();
        assert!(
            realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        configurable: Some(false),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert!(
            !realm
                .objects
                .delete(&arguments, &"0".into(), &mut Budget::new(1000))
                .unwrap()
        );
        assert!(
            !realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(None),
                            set: None
                        },
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert!(
            !realm
                .objects
                .define(
                    &arguments,
                    "0".into(),
                    DataDescriptor {
                        value: Some(Value::Number(3.0)),
                        configurable: Some(true),
                        ..Default::default()
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        assert_eq!(realm.eval("get()"), Ok(Value::Number(1.0)));
        assert_eq!(realm.eval("set(9);args[0]"), Ok(Value::Number(9.0)));
    }

    #[test]
    fn work_failure_leaves_both_the_descriptor_and_parameter_binding_unchanged() {
        let (mut realm, arguments) = setup();
        let before = descriptor(&realm, &arguments);
        assert_eq!(
            realm.objects.define(
                &arguments,
                "0".into(),
                DataDescriptor {
                    value: Some(Value::String("long replacement".into())),
                    writable: Some(false),
                    ..Default::default()
                },
                &mut Budget::new(15)
            ),
            Err(Error::WorkLimit)
        );
        assert_eq!(descriptor(&realm, &arguments), before);
        assert_eq!(realm.eval("get()"), Ok(Value::Number(1.0)));
        assert_eq!(realm.eval("set(7);args[0]"), Ok(Value::Number(7.0)));
    }
}
