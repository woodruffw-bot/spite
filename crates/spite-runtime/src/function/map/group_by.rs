//! GroupBy with collection keys and intrinsic result creation (24.1.2.1).

use super::*;
use crate::object::CollectionKey;
use std::collections::HashMap;

type Groups = Vec<(Value, Vec<Value>)>;

impl Realm {
    pub(crate) fn map_group_by(
        &mut self,
        items: Value,
        callback: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let groups = self.collection_groups(items, callback, 0, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .map
            .prototype
            .clone();
        // Materialization follows exhaustion and bypasses public constructors and
        // Map.prototype.set. Allocation failures cannot close a finished iterator.
        let map = self.object_work(span, |objects, _| objects.create_map(&prototype))?;
        for (key, values) in groups {
            self.tick(span)?;
            let value = self.create_array_from_list(values, span)?;
            self.object_work(span, |objects, budget| {
                objects.map_set(&map, key, value, budget)
            })?;
        }
        Ok(Value::Object(map))
    }

    fn collection_groups(
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
        let mut groups: Groups = Vec::new();
        let mut lookup: HashMap<CollectionKey, usize> = HashMap::new();
        loop {
            self.tick(span)?;
            // GroupBy's specification bound precedes the next step, even if the
            // next step would finish. This bound is not a default host quota.
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
                let key = self.call(
                    callback.clone(),
                    Value::Undefined,
                    vec![value.clone(), Value::Number(index as f64)],
                    span,
                )?;
                self.object_work(span, |_, budget| {
                    budget.value(&key)?;
                    budget.value(&value)
                })?;
                let key = if matches!(key, Value::Number(number) if number == 0.0) {
                    Value::Number(0.0)
                } else {
                    key
                };
                let index_key = CollectionKey::new(&key);
                let group = if let Some(group) = lookup.get(&index_key) {
                    *group
                } else {
                    groups
                        .try_reserve(1)
                        .map_err(|_| Self::map_group_capacity_error(span))?;
                    lookup
                        .try_reserve(1)
                        .map_err(|_| Self::map_group_capacity_error(span))?;
                    let group = groups.len();
                    groups.push((key, Vec::new()));
                    lookup.insert(index_key, group);
                    group
                };
                groups[group]
                    .1
                    .try_reserve(1)
                    .map_err(|_| Self::map_group_capacity_error(span))?;
                groups[group].1.push(value);
                Ok(())
            })();
            if let Err(error) = add {
                return Err(self.iterator_close_error(&iterator, error, span));
            }
            index += 1;
        }
    }

    fn map_group_capacity_error(span: Span) -> Error {
        Error::Limit {
            span,
            message: "Map group storage capacity exceeded".into(),
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
        let callback = realm.eval("(v,k)=>{seen=k;return v;}").unwrap();
        assert!(matches!(
            realm.collection_groups(items, callback, 9_007_199_254_740_990, Span::new(0, 0)),
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
