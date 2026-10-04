//! EnumerateObjectProperties and the internal for-in iterator (14.7.5.9–10).

use crate::{Completion, Error, Realm, Value};
use spite_core::{PropertyKey, Span};
use spite_parser::ast::{Expr, ForBinding, Statement};
use std::collections::BTreeSet;

impl Realm {
    pub(super) fn for_in(
        &mut self,
        binding: &ForBinding,
        expression: &Expr,
        body: &Statement,
        labels: &[&str],
        span: Span,
    ) -> Result<Completion, Error> {
        let source = self.iteration_source(binding, expression, span)?;
        // ForIn/OfHeadEvaluation returns a break completion for nullish input.
        if matches!(source, Value::Undefined | Value::Null) {
            return Ok(Completion::normal(Some(Value::Undefined)));
        }
        let Value::Object(source) = self.box_primitive(source, span)? else {
            unreachable!("ToObject");
        };
        let mut current = Some(source);
        let mut visited = BTreeSet::new();
        let mut value = Value::Undefined;
        while let Some(object) = current {
            self.tick(span)?;
            // Snapshot each object's own keys when it is reached. Descriptors
            // remain live between bodies, and getters are never evaluated.
            for key in self.own_property_keys(&object, span)? {
                self.tick(span)?;
                let PropertyKey::String(key) = key else {
                    continue;
                };
                // Charge copied key units before set comparisons and insertion.
                self.object_work(span, |_, budget| budget.charge(key.len() + 1))?;
                if visited.contains(&key) {
                    continue;
                }
                let Some(descriptor) = self.own_property_descriptor(&object, &key, span)? else {
                    // A deleted key does not suppress a later prototype key.
                    continue;
                };
                visited.insert(key.clone());
                // Non-enumerable own keys still suppress inherited names.
                if !descriptor.enumerable() {
                    continue;
                }
                let result = self
                    .iteration_body(binding, Value::String(key), body, span)?
                    .update_empty(Some(value));
                if !result.loop_continues(labels) {
                    // The internal for-in iterator has no observable return hook.
                    return Ok(result.consume_unlabelled_break());
                }
                value = result.value.expect("loop UpdateEmpty supplies a value");
            }
            current = self.object_work(span, |objects, _| {
                Ok(objects.inspect(&object)?.prototype().cloned())
            })?;
        }
        Ok(Completion::normal(Some(value)))
    }
}
