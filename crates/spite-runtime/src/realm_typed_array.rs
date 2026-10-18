//! TypedArray element coercion outside object-store borrows (10.4.5.18).

use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn set_typed_array_element(
        &mut self,
        target: &ObjectHandle,
        index: f64,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        let element = self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(target)?
                .typed_array_data()
                .expect("typed element continuation has a TypedArray target")
                .element_type())
        })?;
        // TypedArraySetElement converts even when the index is invalid or the
        // buffer is detached. Conversion can resize, detach, or restore bounds;
        // storage performs IsValidIntegerIndex only after it completes.
        let value = if element.is_bigint() {
            Value::BigInt(self.bigint(value, span)?)
        } else {
            Value::Number(self.number(value, span)?)
        };
        self.object_work(span, |objects, budget| {
            objects.typed_array_set_element(target, index, value, budget)
        })
    }
}
