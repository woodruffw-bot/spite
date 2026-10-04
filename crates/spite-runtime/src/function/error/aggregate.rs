//! AggregateError's synchronous IteratorToList and intrinsic Array copy (20.5.7.1.1).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{Span, WellKnownSymbol};

impl Realm {
    pub(super) fn aggregate_errors(&mut self, errors: Value, span: Span) -> Result<Value, Error> {
        // GetIterator(sync) requires an iterable; a direct iterator without the
        // hook is not sufficient. Primitive Strings iterate their code points.
        let method = self
            .get_method(&errors, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(ExceptionKind::TypeError, span, "errors is not iterable")
            })?;
        let mut record = self.get_iterator_from_method(errors, method, span)?;
        let mut values = Vec::new();
        loop {
            self.tick(span)?;
            // IteratorToList does not close on step/done/value failures. Host
            // aborts also propagate directly, without running JavaScript cleanup.
            let Some(value) = self.iterator_step_value(&mut record, span)? else {
                break;
            };
            self.object_work(span, |_, budget| budget.value(&value))?;
            values.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "aggregate error list exceeds platform capacity".into(),
            })?;
            values.push(value);
        }
        // Allocate the Array only after complete iteration, using intrinsics
        // rather than public constructors, species hooks, or indexed setters.
        self.create_array_from_list(values, span)
    }
}
