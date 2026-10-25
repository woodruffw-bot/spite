//! EnumerateObjectProperties and the internal for-in iterator (14.7.5.9–10).

use crate::{Completion, Error, Realm, Value};
use spite_core::{JsString, PropertyKey, Span};
use spite_heap::Handle;
use spite_parser::ast::{Expr, ForBinding, Statement};
use std::collections::BTreeSet;

#[derive(Debug)]
pub(crate) struct ForInIterator {
    object: Option<Handle>,
    keys: Option<std::vec::IntoIter<JsString>>,
    visited: BTreeSet<JsString>,
}

impl ForInIterator {
    pub(crate) fn object_handle(&self) -> Option<&Handle> {
        self.object.as_ref()
    }
}

impl Realm {
    pub(super) fn get_for_in_iterator(
        &mut self,
        source: Value,
        span: Span,
    ) -> Result<Option<ForInIterator>, Error> {
        // ForIn/OfHeadEvaluation returns a break completion for nullish input.
        if matches!(source, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        let Value::Object(object) = self.box_primitive(source, span)? else {
            unreachable!("ToObject")
        };
        Ok(Some(ForInIterator {
            object: Some(object),
            keys: None,
            visited: BTreeSet::new(),
        }))
    }

    pub(super) fn for_in_next(
        &mut self,
        cursor: &mut ForInIterator,
        span: Span,
    ) -> Result<Option<JsString>, Error> {
        while let Some(object) = &cursor.object {
            if cursor.keys.is_none() {
                self.tick(span)?;
                // Snapshot when each object is reached. Symbols never enter
                // the pending names or the visited set.
                let own_keys = self.own_property_keys(object, span)?;
                let mut keys = Vec::new();
                keys.try_reserve(own_keys.len()).map_err(|_| Error::Limit {
                    span,
                    message: "for-in key snapshot allocation failed".into(),
                })?;
                for key in own_keys {
                    if let PropertyKey::String(key) = key {
                        keys.push(key)
                    }
                }
                cursor.keys = Some(keys.into_iter());
            }
            for key in cursor.keys.as_mut().expect("for-in own keys") {
                self.tick(span)?;
                // Charge copied key units before comparisons and insertion.
                self.object_work(span, |_, budget| budget.charge(key.len() + 1))?;
                if cursor.visited.contains(&key) {
                    continue;
                }
                let Some(descriptor) = self.own_property_descriptor(object, &key, span)? else {
                    // A deleted key does not suppress a prototype key.
                    continue;
                };
                cursor.visited.insert(key.clone());
                // Read descriptors live without invoking accessors. A present
                // non-enumerable name still shadows its inherited counterpart.
                if descriptor.enumerable() {
                    return Ok(Some(key));
                }
            }
            // Body execution may have replaced this object's prototype.
            cursor.object = self.get_prototype_of(object, span)?;
            cursor.keys = None;
        }
        Ok(None)
    }

    pub(super) fn for_in(
        &mut self,
        binding: &ForBinding,
        expression: &Expr,
        body: &Statement,
        labels: &[&str],
        span: Span,
    ) -> Result<Completion, Error> {
        let source = self.iteration_source(binding, expression, span)?;
        let Some(mut iterator) = self.get_for_in_iterator(source, span)? else {
            return Ok(Completion::normal(Some(Value::Undefined)));
        };
        let mut value = Value::Undefined;
        while let Some(key) = self.for_in_next(&mut iterator, span)? {
            let result = self
                .iteration_body(binding, Value::String(key), body, span)?
                .update_empty(Some(value));
            if !result.loop_continues(labels) {
                // The internal for-in iterator has no observable return hook.
                return Ok(result.consume_unlabelled_break());
            }
            value = result.value.expect("loop UpdateEmpty supplies a value");
        }
        Ok(Completion::normal(Some(value)))
    }
}
