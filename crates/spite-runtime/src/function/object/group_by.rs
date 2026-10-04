//! Object.groupBy (20.1.2.13), GroupBy, and AddValueToKeyedGroup.

use crate::{Error, ExceptionKind, Realm, Value, object::DataDescriptor};
use spite_core::{PropertyKey, Span, WellKnownSymbol};

type Groups = Vec<(PropertyKey, Vec<Value>)>;

impl Realm {
    pub(crate) fn object_group_by(
        &mut self,
        items: Value,
        callback: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let groups = self.property_groups(items, callback, 0, span)?;
        // Materialize only after GroupBy finishes. Array or result allocation
        // failures at this point cannot execute iterator cleanup.
        let result = self.object_work(span, |objects, _| objects.create(None))?;
        for (key, values) in groups {
            self.tick(span)?;
            let value = self.create_array_from_list(values, span)?;
            self.define_property_or_throw(
                &result,
                key,
                DataDescriptor {
                    value: Some(value),
                    writable: Some(true),
                    enumerable: Some(true),
                    configurable: Some(true),
                }
                .into(),
                span,
            )?;
        }
        Ok(Value::Object(result))
    }

    fn property_groups(
        &mut self,
        items: Value,
        callback: Value,
        mut index: u64,
        span: Span,
    ) -> Result<Groups, Error> {
        Self::require_object_coercible(&items, span)?;
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "groupBy callback is not callable",
            ));
        }
        let method = self
            .get_method(&items, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "groupBy input is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(items, method, span)?;
        let mut groups = Vec::new();
        loop {
            self.tick(span)?;
            // This specification bound precedes the next step, even when that
            // step would exhaust the iterator. It is not a host work quota.
            if index >= 9_007_199_254_740_991 {
                let error = Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "groupBy exceeds the maximum safe integer index",
                );
                return Err(self.iterator_close_error(&iterator, error, span));
            }
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(groups);
            };
            let add = (|| {
                self.object_work(span, |_, budget| {
                    budget.value(&callback)?;
                    budget.value(&value)
                })?;
                let key = self.call(
                    callback.clone(),
                    Value::Undefined,
                    vec![value.clone(), Value::Number(index as f64)],
                    span,
                )?;
                let key = self.property_key(key, span)?;
                self.add_property_group(&mut groups, key, value, span)
            })();
            if let Err(error) = add {
                return Err(self.iterator_close_error(&iterator, error, span));
            }
            index += 1;
        }
    }

    fn add_property_group(
        &mut self,
        groups: &mut Groups,
        key: PropertyKey,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        // AddValueToKeyedGroup compares converted strings by code units and
        // Symbols by identity. Preserve first-key and within-group value order.
        for (existing, values) in groups.iter_mut() {
            self.object_work(span, |_, budget| {
                budget.charge(key.as_string().map_or(1, |s| s.len() + 1))
            })?;
            if existing == &key {
                values
                    .try_reserve(1)
                    .map_err(|_| Self::group_capacity_error(span))?;
                values.push(value);
                return Ok(());
            }
        }
        groups
            .try_reserve(1)
            .map_err(|_| Self::group_capacity_error(span))?;
        let mut values = Vec::new();
        values
            .try_reserve(1)
            .map_err(|_| Self::group_capacity_error(span))?;
        values.push(value);
        groups.push((key, values));
        Ok(())
    }

    fn group_capacity_error(span: Span) -> Error {
        Error::Limit {
            span,
            message: "groupBy storage exceeds platform capacity".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_integer_guard_closes_before_the_next_step() {
        let mut realm = Realm::default();
        let items = realm.eval("let steps=0,closed=0,seen=0;let i={next(){steps++;return {value:7};},return(){closed++;return {};}};({[Symbol.iterator](){return i;}})").unwrap();
        let callback = realm.eval("(v,k)=>{seen=k;return 'x';}").unwrap();
        assert!(matches!(
            realm.property_groups(items, callback, 9_007_199_254_740_990, Span::new(0, 0)),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("steps===1 && closed===1 && seen===9007199254740990"),
            Ok(Value::Boolean(true))
        );
    }
}
