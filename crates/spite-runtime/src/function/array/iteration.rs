//! Array keys/values/entries and ArrayIterator.next (23.1.3.5/19/38, 23.1.5.2.1).

use super::*;
use crate::object::ArrayIterationKind;

impl Realm {
    pub(crate) fn array_iterator(
        &mut self,
        receiver: Value,
        kind: ArrayIterationKind,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(array) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .array_prototype
            .clone();
        self.object_work(span, |objects, _| {
            objects.create_array_iterator(&prototype, &array, kind)
        })
        .map(Value::Object)
    }

    pub(crate) fn array_iterator_next(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not an Array Iterator",
            )
        };
        let Value::Object(iterator) = receiver else {
            return Err(brand_error());
        };
        let state = self
            .object_work(span, |objects, _| {
                Ok(objects.inspect(&iterator)?.array_iterator().cloned())
            })?
            .ok_or_else(brand_error)?;
        let Some(array) = state.array else {
            return self.iterator_result(Value::Undefined, true, span);
        };
        // Index is snapshotted before live length lookup. The TypedArray-specific
        // length/detachment branch joins here when that object kind exists.
        let index = state.next_index;
        let length = self.length_of_array_like(&array, span)?;
        if index >= length {
            self.object_work(span, |objects, _| objects.finish_array_iterator(&iterator))?;
            return self.iterator_result(Value::Undefined, true, span);
        }
        // LengthOfArrayLike is at most MAX_SAFE_INTEGER, so this cannot overflow.
        // Update only the index: a reentrant length getter may have cleared array.
        self.object_work(span, |objects, _| {
            objects.set_array_iterator_index(&iterator, index + 1)
        })?;
        let result = if state.kind == ArrayIterationKind::Key {
            Value::Number(index as f64)
        } else {
            let value =
                self.get_property(&array, &JsString::from(index.to_string().as_str()), span)?;
            if state.kind == ArrayIterationKind::Value {
                value
            } else {
                let entry = self.create_intrinsic_array(2, span)?;
                self.create_array_element(&entry, 0, Value::Number(index as f64), span)?;
                self.create_array_element(&entry, 1, value, span)?;
                Value::Object(entry)
            }
        };
        self.iterator_result(result, false, span)
    }
}
