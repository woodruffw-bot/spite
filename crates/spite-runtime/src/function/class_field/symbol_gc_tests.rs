//! Shared class-field Symbol ownership and weak-Rc recovery regressions.

use super::{ClassField, ClassFieldName};
use crate::object::{Budget, DataDescriptor, Error as ObjectError, Objects};
use crate::{ObjectHandle, Realm, RootedValue, Value};
use spite_core::{JsSymbol, PropertyKey, Span, WeakJsSymbol};
use std::{cell::RefCell, rc::Rc};

type SharedFields = Rc<[ClassField]>;
type RecoveredFields = (SharedFields, JsSymbol);

fn object(value: Value) -> ObjectHandle {
    let Value::Object(value) = value else {
        panic!("expected object")
    };
    value
}

fn unlimited() -> Budget {
    Budget::with_work_limit(None)
}

struct FieldsFixture {
    realm: Realm,
    constructor: ObjectHandle,
    map: ObjectHandle,
    value: ObjectHandle,
    fields: Option<SharedFields>,
    symbol: WeakJsSymbol,
    _map_root: RootedValue,
}

impl FieldsFixture {
    fn new() -> Self {
        let mut realm = Realm::default();
        // No global binding, instance, initializer, or host Symbol clone keeps
        // this anonymous constructor/field name live independently.
        let constructor = object(realm.eval("(class {[Symbol('field')];})").unwrap());
        let fields = realm
            .instance_fields(&constructor, Span::new(0, 0))
            .unwrap();
        assert_eq!(fields.len(), 1);
        assert!(fields[0].initializer.is_none());
        assert!(matches!(
            &fields[0].name,
            ClassFieldName::Public(PropertyKey::Symbol(_))
        ));
        let key = fields[0].symbol_name().unwrap();
        let symbol = key.downgrade();
        let map = object(realm.eval("new WeakMap()").unwrap());
        let value = object(realm.eval("({})").unwrap());
        assert!(
            realm
                .objects
                .define(
                    &value,
                    "back",
                    DataDescriptor {
                        value: Some(Value::Symbol(key.clone())),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    &mut unlimited()
                )
                .unwrap()
        );
        realm
            .objects
            .weak_map_set_symbol(&map, key, Value::Object(value.clone()), &mut unlimited())
            .unwrap();
        let map_root = realm
            .root_value(Value::Object(map.clone()), usize::MAX)
            .unwrap();
        Self {
            realm,
            constructor,
            map,
            value,
            fields: Some(fields),
            symbol,
            _map_root: map_root,
        }
    }

    fn assert_constructor_collected(&self) {
        assert!(matches!(
            self.realm.objects.inspect(&self.constructor),
            Err(ObjectError::Heap(spite_heap::Error::StaleHandle))
        ));
    }

    fn assert_value_collected(&self) {
        assert!(matches!(
            self.realm.objects.inspect(&self.value),
            Err(ObjectError::Heap(spite_heap::Error::StaleHandle))
        ));
        assert_eq!(self.symbol.upgrade(), None);
    }
}

#[test]
fn external_class_field_rc_owns_the_symbol_even_without_an_initializer_or_constructor_root() {
    let mut fixture = FieldsFixture::new();
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_constructor_collected();
    assert!(fixture.realm.objects.inspect(&fixture.value).is_ok());
    assert!(fixture.symbol.upgrade().is_some());
    // Shared metadata contains one physical JsSymbol wrapper. The extra host
    // Rc owner does not increment that Symbol's Arc count, but can recover it.
    let fields = fixture.fields.as_ref().unwrap();
    let key = fields[0].symbol_name().unwrap();
    assert!(
        fixture
            .realm
            .objects
            .weak_map_has_symbol(&fixture.map, key, &mut unlimited())
            .unwrap()
    );
    assert_eq!(
        fixture
            .realm
            .objects
            .weak_map_get_symbol(&fixture.map, key, &mut unlimited())
            .unwrap(),
        Some(Value::Object(fixture.value.clone()))
    );
    drop(fixture.fields.take());
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_value_collected();
}

#[test]
fn external_class_field_rc_weak_roots_the_conditional_cycle_for_a_safe_commit() {
    let mut fixture = FieldsFixture::new();
    let fields = fixture.fields.take().unwrap();
    let recoverable = Rc::downgrade(&fields);
    drop(fields);
    // There is no external strong Rc or direct strong JsSymbol at census.
    // A saved weak Rc is nevertheless a recovery capability until heap cleanup.
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_constructor_collected();
    assert!(fixture.realm.objects.inspect(&fixture.value).is_ok());
    assert!(fixture.symbol.upgrade().is_some());
    // After this collection the old allocation has actually been destroyed.
    // Its conditional Symbol/value cycle was conservatively kept for the commit.
    assert!(recoverable.upgrade().is_none());
    drop(recoverable);
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_value_collected();
}

#[test]
fn field_rc_promotion_after_preflight_is_rechecked_without_a_symbol_arc_clone() {
    let mut fixture = FieldsFixture::new();
    let fields = fixture.fields.take().unwrap();
    let recoverable = Rc::downgrade(&fields);
    drop(fields);
    let promoted: Rc<RefCell<Option<SharedFields>>> = Rc::new(RefCell::new(None));
    let sink = promoted.clone();
    let result = Objects::with_symbol_gc_test_hooks(
        Some(Box::new(move || {
            *sink.borrow_mut() = recoverable.upgrade();
        })),
        None,
        || fixture.realm.collect(usize::MAX),
    );
    result.unwrap();
    // The hook's weak Rc was dropped before the final recheck. Its newly
    // promoted strong Rc did not create another physical JsSymbol wrapper.
    assert!(promoted.borrow().is_some());
    fixture.assert_constructor_collected();
    assert!(fixture.symbol.upgrade().is_some());
    assert!(fixture.realm.objects.inspect(&fixture.value).is_ok());
    drop(promoted.borrow_mut().take());
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_value_collected();
}

#[test]
fn postcommit_weak_rc_recovery_cannot_expose_a_retired_strong_field_symbol() {
    let mut fixture = FieldsFixture::new();
    let fields = fixture.fields.take().unwrap();
    let recoverable = Rc::downgrade(&fields);
    drop(fields);
    let recovered: Rc<RefCell<Option<RecoveredFields>>> = Rc::new(RefCell::new(None));
    let sink = recovered.clone();
    let result = Objects::with_symbol_gc_test_hooks(
        None,
        Some(Box::new(move || {
            *sink.borrow_mut() = recoverable.upgrade().and_then(|fields| {
                let symbol = fields.first()?.symbol_name()?.clone();
                Some((fields, symbol))
            });
        })),
        || fixture.realm.collect(usize::MAX),
    );
    result.unwrap();
    fixture.assert_constructor_collected();
    assert!(fixture.realm.objects.inspect(&fixture.value).is_ok());
    {
        let saved = recovered.borrow();
        let (_, symbol) = saved
            .as_ref()
            .expect("metadata is recoverable before physical cleanup");
        assert_eq!(symbol.downgrade().upgrade(), Some(symbol.clone()));
        assert!(
            fixture
                .realm
                .objects
                .weak_map_has_symbol(&fixture.map, symbol, &mut unlimited())
                .unwrap()
        );
    }
    // Recovery cloned a strong Symbol directly out of Rc metadata, bypassing
    // WeakJsSymbol::upgrade. Only conservative weak-Rc rooting makes it safe.
    drop(recovered.borrow_mut().take());
    fixture.realm.collect(usize::MAX).unwrap();
    fixture.assert_value_collected();
}
