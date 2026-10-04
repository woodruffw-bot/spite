//! Eager direct-iterator consumers (27.1.3.3).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn iterator_to_array(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 27.1.3.3.12: use GetIteratorDirect, without consulting @@iterator.
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.toArray requires an object receiver",
            ));
        };
        let mut record = self.get_iterator_direct(iterator, span)?;
        let mut items = Vec::new();
        while let Some(value) = self.iterator_step_value(&mut record, span)? {
            items.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "iterator result list exceeds platform capacity".into(),
            })?;
            items.push(value);
        }
        // Materialize only after exhaustion, using intrinsic own data elements.
        // Neither iterator-step errors nor host failures run IteratorClose.
        self.create_array_from_list(items, span)
    }
}
