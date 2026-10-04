//! Declarative environment identity and tracing, shared with the object heap.

use crate::Value;
use spite_heap::{Handle, Trace};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvironmentHandle(pub(crate) Handle);

#[derive(Debug)]
pub(crate) struct BindingState {
    pub value: Option<Value>,
    pub mutable: bool,
    // Immutable named-function bindings reject writes only from strict code.
    pub strict: bool,
}

#[derive(Debug)]
pub(crate) struct Environment {
    pub outer: Option<EnvironmentHandle>,
    pub bindings: BTreeMap<String, BindingState>,
    // None for declarative/arrow environments; Some(undefined) is a real binding.
    pub this: Option<Value>,
}

impl Trace for Environment {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(self.outer.as_ref().map(|outer| &outer.0))
            .chain(std::iter::once(self.this.as_ref().and_then(
                |value| match value {
                    Value::Object(handle) => Some(handle),
                    _ => None,
                },
            )))
            .chain(self.bindings.values().map(|binding| match &binding.value {
                Some(Value::Object(handle)) => Some(handle),
                _ => None,
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Error, Limits, Realm, Reference,
        object::{Budget, DataDescriptor, Error as ObjectError, Objects},
    };
    use spite_core::{JsString, Span};
    use spite_parser::{
        ast::{ForInitializer, StatementKind},
        parse_script,
    };

    fn binding(value: Value) -> BindingState {
        BindingState {
            value: Some(value),
            mutable: true,
            strict: true,
        }
    }

    #[test]
    fn references_retain_environment_identity_and_lookup_follows_outer_links() {
        let mut realm = Realm::default();
        realm.eval("let x = 0").unwrap();
        let span = Span::new(0, 0);
        realm
            .push_scope(
                BTreeMap::from([("x".into(), binding(Value::Number(1.0)))]),
                span,
            )
            .unwrap();
        let captured = realm.scopes.last().unwrap().clone();
        let reference = realm.resolve("x", span).unwrap();
        realm.scopes.pop();
        realm
            .push_scope(
                BTreeMap::from([("x".into(), binding(Value::Number(2.0)))]),
                span,
            )
            .unwrap();
        realm.put(reference, Value::Number(7.0), span).unwrap();
        assert_eq!(realm.eval("x"), Ok(Value::Number(2.0)));
        let nested = realm
            .objects
            .create_environment(Some(captured), BTreeMap::new(), &mut Budget::new(100))
            .unwrap();
        realm.scopes.push(nested);
        assert_eq!(realm.eval("x"), Ok(Value::Number(7.0)));
        assert_eq!(realm.collect(1000).unwrap().live, 14);
        realm.scopes.pop();
        realm.scopes.pop();
        assert_eq!(realm.eval("x"), Ok(Value::Number(0.0)));
        assert_eq!(realm.collect(1000).unwrap().reclaimed, 3);
    }

    #[test]
    fn per_iteration_copies_preserve_old_bindings_and_share_the_same_outer() {
        let mut realm = Realm::default();
        realm.eval("let outer = 9").unwrap();
        let script = parse_script("for (let i = 0; false;) {}").unwrap();
        let StatementKind::For {
            initializer: Some(ForInitializer::Lexical { bindings, .. }),
            ..
        } = &script.statements()[0].kind
        else {
            panic!("for")
        };
        let span = Span::new(0, 0);
        realm
            .push_scope(
                BTreeMap::from([("i".into(), binding(Value::Number(1.0)))]),
                span,
            )
            .unwrap();
        let old = realm.scopes.last().unwrap().clone();
        realm.create_per_iteration_environment(bindings).unwrap();
        let current = realm.scopes.last().unwrap().clone();
        assert_ne!(old, current);
        assert_eq!(
            realm.objects.environment(&old).unwrap().outer,
            realm.objects.environment(&current).unwrap().outer
        );
        realm
            .put(Reference::Lexical(current, "i"), Value::Number(2.0), span)
            .unwrap();
        assert_eq!(
            realm.get(&mut Reference::Lexical(old, "i"), span),
            Ok(Value::Number(1.0))
        );
        assert_eq!(realm.eval("i + outer"), Ok(Value::Number(11.0)));
        realm.scopes.pop();
        assert_eq!(realm.collect(1000).unwrap().reclaimed, 2);
    }

    #[test]
    fn environment_handles_cannot_be_used_as_objects_and_all_edges_are_checked() {
        let mut objects = Objects::new(5, 2);
        let object = objects.create(None).unwrap();
        let environment = objects
            .create_environment(None, BTreeMap::new(), &mut Budget::new(100))
            .unwrap();
        assert!(matches!(
            objects.inspect(&environment.0),
            Err(ObjectError::WrongKind)
        ));
        assert!(matches!(
            objects.root(&environment.0, &mut Budget::new(100)),
            Err(ObjectError::WrongKind)
        ));
        assert_eq!(
            objects.create(Some(&environment.0)),
            Err(ObjectError::WrongKind)
        );
        assert_eq!(
            objects.define(
                &object,
                JsString::from("bad"),
                DataDescriptor {
                    value: Some(Value::Object(environment.0.clone())),
                    ..Default::default()
                },
                &mut Budget::new(100)
            ),
            Err(ObjectError::WrongKind)
        );
        assert_eq!(
            objects.create_environment(
                Some(EnvironmentHandle(object.clone())),
                BTreeMap::new(),
                &mut Budget::new(100)
            ),
            Err(ObjectError::WrongKind)
        );
        let bad = BTreeMap::from([("bad".into(), binding(Value::Object(environment.0.clone())))]);
        assert_eq!(
            objects.create_environment(None, bad, &mut Budget::new(100)),
            Err(ObjectError::WrongKind)
        );
        objects.collect([&object], 100).unwrap();
        assert!(matches!(
            objects.environment(&environment),
            Err(ObjectError::Heap(spite_heap::Error::StaleHandle))
        ));
        assert_eq!(
            objects.create_environment(Some(environment), BTreeMap::new(), &mut Budget::new(100)),
            Err(ObjectError::Heap(spite_heap::Error::StaleHandle))
        );
    }

    #[test]
    fn environment_tracing_retains_outer_and_object_bindings_and_budgets_primitive_fields() {
        let mut objects = Objects::new(5, 0);
        let value = objects.create(None).unwrap();
        let garbage = objects.create(None).unwrap();
        let outer = objects
            .create_environment(
                None,
                BTreeMap::from([("object".into(), binding(Value::Object(value.clone())))]),
                &mut Budget::new(100),
            )
            .unwrap();
        let bindings = (0..100)
            .map(|index| {
                (
                    format!("b{index}"),
                    BindingState {
                        value: None,
                        mutable: true,
                        strict: true,
                    },
                )
            })
            .collect();
        let inner = objects
            .create_environment(Some(outer), bindings, &mut Budget::new(1000))
            .unwrap();
        assert_eq!(
            objects.collect([&inner.0], 50),
            Err(spite_heap::Error::Limit)
        );
        assert!(objects.inspect(&garbage).is_ok());
        let result = objects.collect([&inner.0], 1000).unwrap();
        assert_eq!(result.live, 3);
        assert_eq!(result.reclaimed, 1);
        assert!(objects.inspect(&value).is_ok());
        assert_eq!(objects.collect([], 1000).unwrap().reclaimed, 3);
    }

    #[test]
    fn environment_allocation_is_lazy_bounded_and_reusable_only_after_collection() {
        let mut empty = Realm::new(Limits {
            max_heap_entries: 0,
            ..Limits::default()
        });
        assert!(matches!(empty.eval("const x;"), Err(Error::Parse(_))));
        assert!(matches!(empty.eval("0"), Err(Error::Limit { .. })));
        assert_eq!(empty.collect(100).unwrap().live, 0);
        let mut realm = Realm::new(Limits {
            max_heap_entries: 13,
            ..Limits::default()
        });
        for _ in 0..2 {
            realm.eval("{ let transient = 1; }").unwrap();
        }
        assert!(matches!(realm.eval("{}"), Err(Error::Limit { .. })));
        assert_eq!(
            realm.eval("typeof transient"),
            Ok(Value::String(JsString::from("undefined")))
        );
        let result = realm.collect(1000).unwrap();
        assert_eq!(result.live, 11);
        assert_eq!(result.reclaimed, 2);
        realm.eval("{ let transient = 2; }").unwrap();
    }
}
