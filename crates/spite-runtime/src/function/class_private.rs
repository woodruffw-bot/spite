//! Private method/accessor definition and brand installation (15.7.14, 7.3.28/33).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    private::{PrivateMethod, PrivateMethodKind, PrivateName},
};
use spite_core::{PropertyKey, Span};
use spite_parser::ast::{ExprKind, ObjectProperty, PropertyKind};

impl Realm {
    pub(super) fn private_method_definition(
        &mut self,
        home: &ObjectHandle,
        name: PrivateName,
        property: &ObjectProperty,
        container: &mut Vec<PrivateMethod>,
    ) -> Result<(), Error> {
        let ExprKind::Function(syntax) = &property.value.kind else {
            unreachable!("method syntax")
        };
        let function = self.method_function(
            syntax,
            home,
            PropertyKey::String(name.description().clone()),
            property.kind,
            property.span,
        )?;
        let kind = match property.kind {
            PropertyKind::Method => PrivateMethodKind::Method(function),
            PropertyKind::Getter => PrivateMethodKind::Accessor {
                get: Some(function),
                set: None,
            },
            PropertyKind::Setter => PrivateMethodKind::Accessor {
                get: None,
                set: Some(function),
            },
            _ => unreachable!("private method kind"),
        };
        self.object_work(property.span, |_, budget| {
            budget.charge(container.len() + 1)
        })?;
        if let Some(previous) = container.iter_mut().find(|method| method.name == name) {
            let (
                PrivateMethodKind::Accessor { get, set },
                PrivateMethodKind::Accessor {
                    get: previous_get,
                    set: previous_set,
                },
            ) = (kind, &mut previous.kind)
            else {
                unreachable!("parser permits only paired private accessors")
            };
            debug_assert!(get.is_none() || previous_get.is_none());
            debug_assert!(set.is_none() || previous_set.is_none());
            if get.is_some() {
                *previous_get = get;
            }
            if set.is_some() {
                *previous_set = set;
            }
        } else {
            container.try_reserve(1).map_err(|_| Error::Limit {
                span: property.span,
                message: "private method allocation capacity exceeded".into(),
            })?;
            container.push(PrivateMethod { name, kind });
        }
        Ok(())
    }

    pub(super) fn initialize_private_methods(
        &mut self,
        receiver: &Value,
        methods: &[PrivateMethod],
        span: Span,
    ) -> Result<(), Error> {
        let Value::Object(receiver) = receiver else {
            unreachable!("class initialization receiver")
        };
        // InitializeInstanceElements installs every method/accessor before the
        // first field. ClassDefinitionEvaluation also does this before its
        // ordered static fields and blocks, regardless of declaration position.
        for method in methods {
            let added = self.object_work(span, |objects, budget| {
                objects.private_method_add(receiver, method, budget)
            })?;
            if !added {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "object already has private method or accessor",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        function::Callable,
        object::{Budget, Error as ObjectError},
    };
    use spite_core::JsString;
    use std::rc::Rc;

    fn object(value: Value) -> ObjectHandle {
        let Value::Object(handle) = value else {
            panic!("object")
        };
        handle
    }

    #[test]
    fn accessor_metadata_and_checked_class_method_edges_preserve_original_records() {
        let mut realm = Realm::default();
        let constructor = object(
            realm
                .eval("class C{get #x(){return 7;}set #x(v){}#m(a,b){}read(){return this.#x;}}C")
                .unwrap(),
        );
        let methods = realm
            .instance_elements(&constructor, Span::new(0, 0))
            .unwrap()
            .private_methods;
        assert_eq!(methods.len(), 2);
        let PrivateMethodKind::Accessor {
            get: Some(get),
            set: Some(set),
        } = &methods[0].kind
        else {
            panic!("paired accessors")
        };
        for (handle, name, length) in [(get, "get #x", 0.0), (set, "set #x", 1.0)] {
            assert_eq!(
                realm.get_property(handle, &JsString::from("name"), Span::new(0, 0)),
                Ok(Value::String(name.into()))
            );
            assert_eq!(
                realm.get_property(handle, &JsString::from("length"), Span::new(0, 0)),
                Ok(Value::Number(length))
            );
            assert!(!realm.objects.inspect(handle).unwrap().is_constructor());
        }
        let stale = object(realm.eval("(()=>{})").unwrap());
        realm.collect(usize::MAX).unwrap();
        let mut other = Realm::default();
        let foreign = object(other.eval("(()=>{})").unwrap());
        let plain = object(realm.eval("({})").unwrap());
        for (bad, expected) in [
            (foreign, ObjectError::Heap(spite_heap::Error::ForeignHandle)),
            (stale, ObjectError::Heap(spite_heap::Error::StaleHandle)),
            (plain.clone(), ObjectError::NotCallable),
            (realm.scopes[0].0.clone(), ObjectError::WrongKind),
        ] {
            let replacement: Rc<[PrivateMethod]> = vec![PrivateMethod {
                name: methods[0].name.clone(),
                kind: PrivateMethodKind::Accessor {
                    get: Some(get.clone()),
                    set: Some(bad.clone()),
                },
            }]
            .into();
            assert_eq!(
                realm.objects.set_class_private_methods(
                    &constructor,
                    replacement.clone(),
                    &mut Budget::with_work_limit(None)
                ),
                Err(expected)
            );
            let target = object(realm.eval("({})").unwrap());
            assert_eq!(
                realm.objects.private_method_add(
                    &target,
                    &replacement[0],
                    &mut Budget::with_work_limit(None)
                ),
                Err(expected)
            );
            assert!(
                !realm
                    .objects
                    .private_has(
                        &target,
                        &replacement[0].name,
                        &mut Budget::with_work_limit(None)
                    )
                    .unwrap()
            );
            assert_eq!(realm.eval("new C().read()"), Ok(Value::Number(7.0)));
        }
        assert_eq!(
            realm.objects.set_class_private_methods(
                &constructor,
                methods.clone(),
                &mut Budget::new(0)
            ),
            Err(ObjectError::WorkLimit)
        );
        assert_eq!(
            realm
                .objects
                .private_method_add(&plain, &methods[0], &mut Budget::new(1)),
            Err(ObjectError::WorkLimit)
        );
        assert!(
            !realm
                .objects
                .private_has(&plain, &methods[0].name, &mut Budget::with_work_limit(None))
                .unwrap()
        );
        assert_eq!(
            realm.objects.set_class_private_methods(
                &plain,
                methods,
                &mut Budget::with_work_limit(None)
            ),
            Err(ObjectError::WrongKind)
        );
        let Some(Callable::ClassConstructor(class)) =
            realm.objects.inspect(&constructor).unwrap().callable()
        else {
            panic!("class")
        };
        assert_eq!(class.elements.private_methods.len(), 2);
    }
}
