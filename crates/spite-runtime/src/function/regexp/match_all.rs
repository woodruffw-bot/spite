//! Generic RegExp matchAll and the RegExp String Iterator (22.2.6.9, 22.2.9).

use super::advance_string_index;
use crate::{Error, ExceptionKind, Realm, Value, object::RegExpStringIterator};
use spite_core::{JsString, Span};

impl Realm {
    #[inline(never)]
    pub(super) fn regexp_match_all(
        &mut self,
        receiver: Value,
        argument: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let string = self.string(argument, span)?;
        let constructor = self.regexp_species_constructor(&object, span)?;
        let flags = self.get_property(&object, &JsString::from("flags"), span)?;
        let flags = self.string(flags, span)?;
        let Value::Object(matcher) = self.construct(
            constructor,
            vec![Value::Object(object.clone()), Value::String(flags.clone())],
            span,
        )?
        else {
            unreachable!("Construct returns an Object");
        };
        let last_index = JsString::from("lastIndex");
        let value = self.get_property(&object, &last_index, span)?;
        let index = self.length_from_value(value, span)?;
        self.set_property_or_throw(&matcher, last_index, Value::Number(index as f64), span)?;
        self.object_work(span, |_, budget| budget.charge(flags.len()))?;
        let mut global = false;
        let mut unicode = false;
        for &unit in flags.code_units() {
            global |= unit == u16::from(b'g');
            unicode |= unit == u16::from(b'u') || unit == u16::from(b'v');
        }
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .regexp_prototype
            .clone();
        self.object_work(span, |objects, _| {
            objects.create_regexp_string_iterator(
                &prototype,
                RegExpStringIterator {
                    matcher,
                    string,
                    global,
                    unicode,
                    done: false,
                },
            )
        })
        .map(Value::Object)
    }

    #[inline(never)]
    pub(crate) fn regexp_string_iterator_next(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a RegExp String Iterator",
            )
        };
        let Value::Object(iterator) = receiver else {
            return Err(brand_error());
        };
        let state = self
            .object_work(span, |objects, _| {
                Ok(objects
                    .inspect(&iterator)?
                    .regexp_string_iterator()
                    .cloned())
            })?
            .ok_or_else(brand_error)?;
        if state.done {
            return self.iterator_result(Value::Undefined, true, span);
        }
        let result = self.regexp_exec(&state.matcher, &state.string, span)?;
        if matches!(result, Value::Null) {
            self.object_work(span, |objects, _| {
                objects.finish_regexp_string_iterator(&iterator)
            })?;
            return self.iterator_result(Value::Undefined, true, span);
        }
        if !state.global {
            self.object_work(span, |objects, _| {
                objects.finish_regexp_string_iterator(&iterator)
            })?;
            return self.iterator_result(result, false, span);
        }
        let Value::Object(matched) = &result else {
            unreachable!("RegExpExec result");
        };
        let value = self.get_property(matched, &JsString::from("0"), span)?;
        let string = self.string(value, span)?;
        if string.is_empty() {
            let last_index = JsString::from("lastIndex");
            let value = self.get_property(&state.matcher, &last_index, span)?;
            let index = self.length_from_value(value, span)?;
            let next = advance_string_index(&state.string, index, state.unicode);
            self.set_property_or_throw(
                &state.matcher,
                last_index,
                Value::Number(next as f64),
                span,
            )?;
        }
        // Reentrant next calls may already have completed the iterator. Never
        // reset Done when the outer call yields its snapshotted exec result.
        self.iterator_result(result, false, span)
    }
}
