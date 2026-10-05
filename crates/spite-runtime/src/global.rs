//! Ordinary global object and the object half of the global environment (9.1.1.4).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, Property},
};
use spite_core::{JsString, Span};
use std::collections::BTreeMap;

impl Realm {
    pub(super) fn initialize_realm(&mut self) -> Result<(), Error> {
        if self.global_object.is_some() {
            return Ok(());
        }
        // Realm initialization (9.3.1) precedes the per-Script work allowance.
        // Its fixed intrinsic graph runs no user code. Opted-in heap/property
        // quotas and checked allocation still apply.
        self.remaining_steps = None;
        let span = Span::new(0, 0);
        if self.scopes.is_empty() {
            self.push_scope(BTreeMap::new(), span)?;
        }
        let prototype = self.ensure_object_intrinsics(span)?;
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        for (name, value) in [
            ("undefined", Value::Undefined),
            ("NaN", Value::Number(f64::NAN)),
            ("Infinity", Value::Number(f64::INFINITY)),
            ("globalThis", Value::Object(object.clone())),
        ] {
            let mutable = name == "globalThis";
            self.object_work(span, |objects, budget| {
                objects.define(
                    &object,
                    JsString::from(name),
                    DataDescriptor {
                        value: Some(value),
                        writable: Some(mutable),
                        enumerable: Some(false),
                        configurable: Some(mutable),
                    },
                    budget,
                )
            })?;
        }
        let boolean = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .boolean
            .constructor
            .clone();
        self.define_builtin_property(&object, "Boolean", Value::Object(boolean), true, span)?;
        let bigint = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .bigint
            .constructor
            .clone();
        self.define_builtin_property(&object, "BigInt", Value::Object(bigint), true, span)?;
        let number = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .number
            .constructor
            .clone();
        self.define_builtin_property(&object, "Number", Value::Object(number), true, span)?;
        let intrinsics = self.intrinsics.as_ref().expect("initialized");
        let global_functions = [
            ("eval", intrinsics.eval.clone()),
            ("parseFloat", intrinsics.number.parse_float.clone()),
            ("parseInt", intrinsics.number.parse_int.clone()),
            ("isFinite", intrinsics.is_finite.clone()),
            ("isNaN", intrinsics.is_nan.clone()),
            ("decodeURI", intrinsics.decode_uri.clone()),
            (
                "decodeURIComponent",
                intrinsics.decode_uri_component.clone(),
            ),
            ("encodeURI", intrinsics.encode_uri.clone()),
            (
                "encodeURIComponent",
                intrinsics.encode_uri_component.clone(),
            ),
        ];
        for (name, function) in global_functions {
            self.define_builtin_property(&object, name, Value::Object(function), true, span)?;
        }
        let errors: Vec<_> = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .errors
            .entries
            .iter()
            .map(|entry| (entry.kind.name(), entry.constructor.clone()))
            .collect();
        for (name, constructor) in errors {
            self.define_builtin_property(&object, name, Value::Object(constructor), true, span)?;
        }
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object
            .constructor
            .clone();
        self.define_builtin_property(&object, "Object", Value::Object(constructor), true, span)?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_constructor
            .clone();
        self.define_builtin_property(&object, "Function", Value::Object(constructor), true, span)?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .string
            .constructor
            .clone();
        self.define_builtin_property(&object, "String", Value::Object(constructor), true, span)?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .symbol
            .constructor
            .clone();
        self.define_builtin_property(&object, "Symbol", Value::Object(constructor), true, span)?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .array
            .constructor
            .clone();
        self.define_builtin_property(&object, "Array", Value::Object(constructor), true, span)?;
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .constructor
            .clone();
        self.define_builtin_property(&object, "Iterator", Value::Object(constructor), true, span)?;
        let reflect = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .reflect
            .object
            .clone();
        self.define_builtin_property(&object, "Reflect", Value::Object(reflect), true, span)?;
        let math = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .math
            .object
            .clone();
        self.define_builtin_property(&object, "Math", Value::Object(math), true, span)?;
        let json = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .json
            .object
            .clone();
        self.define_builtin_property(&object, "JSON", Value::Object(json), true, span)?;
        let map = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .map
            .constructor
            .clone();
        self.define_builtin_property(&object, "Map", Value::Object(map), true, span)?;
        let set = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .set
            .constructor
            .clone();
        self.define_builtin_property(&object, "Set", Value::Object(set), true, span)?;
        self.global_object = Some(object);
        Ok(())
    }

    pub(super) fn global_object(&self) -> ObjectHandle {
        self.global_object
            .as_ref()
            .expect("initialized realm")
            .clone()
    }

    pub(super) fn global_own(&mut self, name: &str, span: Span) -> Result<Option<Property>, Error> {
        let object = self.global_object();
        self.object_work(span, |objects, budget| {
            objects.get_own(&object, &JsString::from(name), budget)
        })
    }

    pub(super) fn restricted_global_property(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<bool, Error> {
        Ok(self
            .global_own(name, span)?
            .is_some_and(|property| !property.configurable()))
    }

    fn global_extensible(&mut self, span: Span) -> Result<bool, Error> {
        let object = self.global_object();
        self.object_work(span, |objects, _| {
            Ok(objects.inspect(&object)?.is_extensible())
        })
    }

    pub(super) fn can_declare_global_var(&mut self, name: &str, span: Span) -> Result<bool, Error> {
        Ok(self.global_own(name, span)?.is_some() || self.global_extensible(span)?)
    }

    pub(super) fn can_declare_global_function(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<bool, Error> {
        // 9.1.1.4.15: a non-configurable function binding must be writable data
        // and enumerable. Inherited properties do not constrain declarations.
        match self.global_own(name, span)? {
            None => self.global_extensible(span),
            Some(property) => Ok(property.configurable()
                || property
                    .as_data()
                    .is_some_and(|data| data.writable && data.enumerable)),
        }
    }

    pub(super) fn create_global_var(&mut self, name: &str, span: Span) -> Result<(), Error> {
        self.create_global_var_binding(name, false, span)
    }

    pub(super) fn create_global_var_binding(
        &mut self,
        name: &str,
        deletable: bool,
        span: Span,
    ) -> Result<(), Error> {
        // 9.1.1.4.16: preserve existing own properties, including accessors and
        // configurable properties. Only newly created Script vars are fixed.
        if self.global_own(name, span)?.is_none() && self.global_extensible(span)? {
            let object = self.global_object();
            let created = self.object_work(span, |objects, budget| {
                objects.define(
                    &object,
                    JsString::from(name),
                    DataDescriptor {
                        value: Some(Value::Undefined),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(deletable),
                    },
                    budget,
                )
            })?;
            debug_assert!(created, "global declaration was checked");
        }
        Ok(())
    }

    pub(super) fn create_global_function(
        &mut self,
        name: &str,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.create_global_function_binding(name, value, false, span)
    }

    pub(super) fn create_global_function_binding(
        &mut self,
        name: &str,
        value: Value,
        deletable: bool,
        span: Span,
    ) -> Result<(), Error> {
        let descriptor = if self
            .global_own(name, span)?
            .is_none_or(|property| property.configurable())
        {
            DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(deletable),
            }
        } else {
            DataDescriptor {
                value: Some(value),
                ..Default::default()
            }
        };
        let object = self.global_object();
        let created = self.object_work(span, |objects, budget| {
            objects.define(&object, JsString::from(name), descriptor, budget)
        })?;
        // For this ordinary global object the following Set in 9.1.1.4.17
        // writes the same value to a writable data property, with no callbacks.
        debug_assert!(created, "global function declaration was checked");
        Ok(())
    }

    pub(super) fn get_global(&mut self, name: &str, span: Span) -> Result<Value, Error> {
        let object = self.global_object();
        let key = JsString::from(name);
        if !self.has_property(&object, &key, span)? && self.strict {
            return Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is not defined"),
            ));
        }
        self.get_property(&object, &key, span)
    }

    pub(super) fn put_global(
        &mut self,
        name: &str,
        value: Value,
        resolved: bool,
        span: Span,
    ) -> Result<(), Error> {
        let object = self.global_object();
        let key = JsString::from(name);
        // An RHS can delete an already-resolved property (9.1.1.2.5).
        if resolved && self.strict && !self.has_property(&object, &key, span)? {
            return Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is not defined"),
            ));
        }
        let written = self.set_property_value(&Value::Object(object), key, value, span)?;
        if !written && self.strict {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                format!("{name} is not writable"),
            ));
        }
        Ok(())
    }

    pub(super) fn missing_global_property(&self, key: &JsString) -> bool {
        // Identifier names are UTF-8 scalar strings, while arbitrary property
        // names may contain lone surrogates; those never name these placeholders.
        String::from_utf16(key.code_units()).is_ok_and(|name| {
            crate::standard_global(&name) || self.unsupported_host_globals.contains(&name)
        })
    }

    pub(super) fn check_global_property_operation<'key>(
        &mut self,
        object: &ObjectHandle,
        key: impl Into<spite_core::PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<(), Error> {
        let Some(key) = key.into().as_string() else {
            return Ok(());
        };
        if self.global_object.as_ref() == Some(object)
            && self.missing_global_property(key)
            && !self.object_work(span, |objects, budget| objects.has_own(object, key, budget))?
        {
            return Err(Self::unsupported(
                span,
                "global property is not implemented",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Reference,
        object::{Budget, DescriptorKind, PropertyDescriptor},
    };

    fn define(realm: &mut Realm, name: &str, descriptor: impl Into<PropertyDescriptor>) {
        let global = realm.global_object();
        assert!(
            realm
                .objects
                .define(&global, name, descriptor, &mut Budget::new(1000))
                .unwrap()
        );
    }

    #[test]
    fn global_accessors_receive_the_global_object_and_var_preserves_descriptors() {
        let mut realm = Realm::default();
        let Value::Object(getter) = realm
            .eval("(function(){'use strict';return this;})")
            .unwrap()
        else {
            panic!()
        };
        let Value::Object(setter) = realm
            .eval("let received; (function(v){'use strict';received=this;this.last=v;})")
            .unwrap()
        else {
            panic!()
        };
        define(
            &mut realm,
            "access",
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(getter)),
                    set: Some(Some(setter)),
                },
                enumerable: Some(false),
                configurable: Some(true),
            },
        );
        assert_eq!(
            realm.eval("var access;access===this"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("access=7;received===this && last===7"),
            Ok(Value::Boolean(true))
        );
        let property = realm
            .global_own("access", Span::new(0, 0))
            .unwrap()
            .unwrap();
        assert!(matches!(property, Property::Accessor(_)));
        assert_eq!(
            realm.eval("function access(){}typeof access==='function' && !delete access"),
            Ok(Value::Boolean(true))
        );
        let property = realm
            .global_own("access", Span::new(0, 0))
            .unwrap()
            .unwrap();
        let data = property.as_data().unwrap();
        assert!(data.writable && data.enumerable && !data.configurable);
        assert_eq!(realm.eval("last"), Ok(Value::Number(7.0)));
    }

    #[test]
    fn declarations_check_real_descriptors_before_creating_any_bindings() {
        for (writable, enumerable, allowed) in [
            (true, true, true),
            (true, false, false),
            (false, true, false),
        ] {
            let mut realm = Realm::default();
            realm.eval("").unwrap();
            define(
                &mut realm,
                "f",
                DataDescriptor {
                    value: Some(Value::Number(7.0)),
                    writable: Some(writable),
                    enumerable: Some(enumerable),
                    configurable: Some(false),
                },
            );
            let result = realm.eval("let fresh=1;function f(){}var later=2;");
            if allowed {
                assert!(result.is_ok());
                assert_eq!(realm.eval("typeof f"), Ok(Value::String("function".into())));
            } else {
                assert!(matches!(
                    result,
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ));
                assert_eq!(
                    realm.eval("typeof fresh==='undefined' && typeof later==='undefined' && f===7"),
                    Ok(Value::Boolean(true))
                );
            }
        }
        let mut realm = Realm::default();
        realm.eval("").unwrap();
        define(
            &mut realm,
            "fixed",
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(None),
                    set: Some(None),
                },
                enumerable: Some(true),
                configurable: Some(false),
            },
        );
        assert!(matches!(
            realm.eval("function fixed(){}"),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert!(matches!(
            realm.eval("let fresh;let fixed;"),
            Err(Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("var fixed;typeof fresh==='undefined'"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn non_extensible_globals_distinguish_existing_properties_from_new_declarations() {
        let mut realm = Realm::default();
        realm.eval("this.present=7").unwrap();
        realm
            .objects
            .prevent_extensions(&realm.global_object())
            .unwrap();
        for source in ["let fresh;var absent;", "let fresh;function absent(){}"] {
            assert!(matches!(
                realm.eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
            assert_eq!(
                realm.eval("typeof fresh"),
                Ok(Value::String("undefined".into()))
            );
        }
        assert_eq!(realm.eval("var present;present"), Ok(Value::Number(7.0)));
        assert_eq!(
            realm.eval("function present(){}typeof present"),
            Ok(Value::String("function".into()))
        );
        assert_eq!(
            realm.eval("absent=7;typeof absent"),
            Ok(Value::String("undefined".into()))
        );
        assert!(matches!(
            realm.eval("'use strict';this.absent=7"),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("let independent=7;independent"),
            Ok(Value::Number(7.0))
        );
    }

    #[test]
    fn inherited_bindings_use_global_receivers_and_declarations_create_own_properties() {
        let mut realm = Realm::default();
        let Value::Object(prototype) = realm.eval("({inherited:7})").unwrap() else {
            panic!()
        };
        let Value::Object(getter) = realm
            .eval("(function(){'use strict';return this;})")
            .unwrap()
        else {
            panic!()
        };
        realm
            .objects
            .define(
                &prototype,
                "access",
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(getter)),
                        set: Some(None),
                    },
                    enumerable: Some(true),
                    configurable: Some(true),
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        realm
            .objects
            .set_prototype(
                &realm.global_object(),
                Some(&prototype),
                &mut Budget::new(1000),
            )
            .unwrap();
        assert_eq!(
            realm.eval("inherited===7 && access===this && delete inherited && inherited===7"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.eval("var inherited,access;inherited===undefined && access===undefined && !delete inherited"),Ok(Value::Boolean(true)));
    }

    #[test]
    fn saved_global_references_observe_deletion_in_strict_and_non_strict_code() {
        let mut realm = Realm::default();
        realm.eval("this.x=1;delete this.x").unwrap();
        let span = Span::new(0, 0);
        assert_eq!(
            realm.get(&mut Reference::Global("x"), span),
            Ok(Value::Undefined)
        );
        realm.strict = true;
        assert!(matches!(
            realm.get(&mut Reference::Global("x"), span),
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ));
    }

    #[test]
    fn intrinsic_work_failure_does_not_publish_partial_graphs() {
        for work in 0..1000 {
            let mut realm = Realm {
                remaining_steps: Some(work),
                ..Realm::default()
            };
            let result = realm.ensure_object_intrinsics(Span::new(0, 0));
            assert!(result.is_ok() || matches!(result, Err(Error::Limit { .. })));
            assert_eq!(realm.intrinsics.is_some(), result.is_ok());
            assert_eq!(
                realm.collect(usize::MAX).unwrap().live,
                if result.is_ok() {
                    crate::test_support::REALM_ENTRIES - 2
                } else {
                    0
                }
            );
        }
    }
}
