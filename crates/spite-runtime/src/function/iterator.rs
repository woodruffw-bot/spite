//! Iterator constructor/prototype graph and IteratorResult creation (27.1, 7.4.16).

use super::Builtin;
use crate::{
    Error, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span, WellKnownSymbol};

mod callback;
mod concat;
mod consume;
mod from;
mod operations;
mod tag;

#[derive(Debug)]
pub(crate) struct IteratorIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    wrapper_prototype: ObjectHandle,
    from: ObjectHandle,
    wrapper_next: ObjectHandle,
    wrapper_return: ObjectHandle,
    helper_prototype: ObjectHandle,
    concat: ObjectHandle,
    helper_next: ObjectHandle,
    helper_return: ObjectHandle,
    to_array: ObjectHandle,
    for_each: ObjectHandle,
    every: ObjectHandle,
    some: ObjectHandle,
    find: ObjectHandle,
    reduce: ObjectHandle,
    map: ObjectHandle,
    filter: ObjectHandle,
    pub array_prototype: ObjectHandle,
    pub string_prototype: ObjectHandle,
    identity: ObjectHandle,
    constructor_get: ObjectHandle,
    constructor_set: ObjectHandle,
    array_next: ObjectHandle,
    string_next: ObjectHandle,
    tag_get: ObjectHandle,
    tag_set: ObjectHandle,
}

impl IteratorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.constructor,
            &self.prototype,
            &self.wrapper_prototype,
            &self.from,
            &self.wrapper_next,
            &self.wrapper_return,
            &self.helper_prototype,
            &self.concat,
            &self.helper_next,
            &self.helper_return,
            &self.to_array,
            &self.for_each,
            &self.every,
            &self.some,
            &self.find,
            &self.reduce,
            &self.map,
            &self.filter,
            &self.array_prototype,
            &self.string_prototype,
            &self.identity,
            &self.constructor_get,
            &self.constructor_set,
            &self.array_next,
            &self.string_next,
            &self.tag_get,
            &self.tag_set,
        ]
        .into_iter()
    }
}

impl Realm {
    pub(super) fn iterator_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<IteratorIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Iterator, span)?;
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let wrapper_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let from = self.new_builtin(function_prototype, Builtin::IteratorFrom, span)?;
        let concat = self.new_builtin(function_prototype, Builtin::IteratorConcat, span)?;
        let helper_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let helper_next =
            self.new_builtin(function_prototype, Builtin::IteratorHelperNext, span)?;
        let helper_return =
            self.new_builtin(function_prototype, Builtin::IteratorHelperReturn, span)?;
        let to_array = self.new_builtin(function_prototype, Builtin::IteratorToArray, span)?;
        let for_each = self.new_builtin(function_prototype, Builtin::IteratorForEach, span)?;
        let every = self.new_builtin(function_prototype, Builtin::IteratorEvery, span)?;
        let some = self.new_builtin(function_prototype, Builtin::IteratorSome, span)?;
        let find = self.new_builtin(function_prototype, Builtin::IteratorFind, span)?;
        let reduce = self.new_builtin(function_prototype, Builtin::IteratorReduce, span)?;
        let map = self.new_builtin(function_prototype, Builtin::IteratorMap, span)?;
        let filter = self.new_builtin(function_prototype, Builtin::IteratorFilter, span)?;
        let wrapper_next =
            self.new_builtin(function_prototype, Builtin::IteratorWrapperNext, span)?;
        let wrapper_return =
            self.new_builtin(function_prototype, Builtin::IteratorWrapperReturn, span)?;
        for (object, name, function) in [
            (&prototype, "toArray", &to_array),
            (&prototype, "forEach", &for_each),
            (&prototype, "every", &every),
            (&prototype, "some", &some),
            (&prototype, "find", &find),
            (&prototype, "reduce", &reduce),
            (&prototype, "map", &map),
            (&prototype, "filter", &filter),
            (&constructor, "concat", &concat),
            (&constructor, "from", &from),
            (&helper_prototype, "next", &helper_next),
            (&helper_prototype, "return", &helper_return),
            (&wrapper_prototype, "next", &wrapper_next),
            (&wrapper_prototype, "return", &wrapper_return),
        ] {
            self.define_builtin_property(
                object,
                name,
                Value::Object(function.clone()),
                true,
                span,
            )?;
        }
        let array_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let string_prototype =
            self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        let identity = self.new_builtin(function_prototype, Builtin::IteratorIdentity, span)?;
        let constructor_get =
            self.new_builtin(function_prototype, Builtin::IteratorConstructorGet, span)?;
        let constructor_set =
            self.new_builtin(function_prototype, Builtin::IteratorConstructorSet, span)?;
        let tag_get = self.new_builtin(function_prototype, Builtin::IteratorTagGet, span)?;
        let tag_set = self.new_builtin(function_prototype, Builtin::IteratorTagSet, span)?;
        let array_next = self.new_builtin(function_prototype, Builtin::ArrayIteratorNext, span)?;
        let string_next =
            self.new_builtin(function_prototype, Builtin::StringIteratorNext, span)?;
        self.define_builtin_property(
            &string_prototype,
            "next",
            Value::Object(string_next.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &array_prototype,
            "next",
            Value::Object(array_next.clone()),
            true,
            span,
        )?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &helper_prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Iterator Helper"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &constructor,
                "prototype",
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )?;
            objects.define(
                &prototype,
                "constructor",
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(constructor_get.clone())),
                        set: Some(Some(constructor_set.clone())),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(tag_get.clone())),
                        set: Some(Some(tag_set.clone())),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &prototype,
                WellKnownSymbol::Iterator.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(identity.clone())),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &string_prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("String Iterator"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            objects.define(
                &array_prototype,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("Array Iterator"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            Ok(())
        })?;
        Ok(IteratorIntrinsics {
            constructor,
            prototype,
            wrapper_prototype,
            from,
            wrapper_next,
            wrapper_return,
            helper_prototype,
            concat,
            helper_next,
            helper_return,
            to_array,
            for_each,
            every,
            some,
            find,
            reduce,
            map,
            filter,
            array_prototype,
            string_prototype,
            identity,
            constructor_get,
            constructor_set,
            array_next,
            string_next,
            tag_get,
            tag_set,
        })
    }

    pub(crate) fn iterator_result(
        &mut self,
        value: Value,
        done: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .object_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            let result = objects.create(Some(&prototype))?;
            for (key, value) in [("value", value), ("done", Value::Boolean(done))] {
                objects.define(
                    &result,
                    key,
                    DataDescriptor {
                        value: Some(value),
                        writable: Some(true),
                        enumerable: Some(true),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            Ok(Value::Object(result))
        })
    }
}
