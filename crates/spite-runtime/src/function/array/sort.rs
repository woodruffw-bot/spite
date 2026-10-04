//! Stable Array sorting and comparison (23.1.3.30, 23.1.3.34).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};
use std::cmp::Ordering;

impl Realm {
    pub(crate) fn array_sort(
        &mut self,
        receiver: Value,
        comparator: Value,
        copy: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // Both entry points validate the comparator before ToObject/length.
        if !matches!(comparator, Value::Undefined) && !self.is_callable(&comparator, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array comparator must be callable or undefined",
            ));
        }
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let output = if copy {
            // ArrayCreate and its length bound precede all indexed reads.
            self.create_intrinsic_array(length, span)?
        } else {
            object.clone()
        };
        let mut items = Vec::new();
        for index in 0..length {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            if copy || self.has_property(&object, &key, span)? {
                let value = self.get_property(&object, &key, span)?;
                // Grow only as values are observed, never from an untrusted len.
                items.try_reserve(1).map_err(|_| allocation_limit(span))?;
                items.push(value);
            }
        }
        let items = self.stable_array_sort(items, &comparator, span)?;
        let count = items.len() as u64;
        for (index, value) in items.into_iter().enumerate() {
            self.tick(span)?;
            if copy {
                // ArrayCreate already established that every index fits u32.
                self.create_array_element(&output, index as u64, value, span)?;
            } else {
                self.set_property_or_throw(
                    &output,
                    JsString::from(index.to_string().as_str()),
                    value,
                    span,
                )?;
            }
        }
        if !copy {
            for index in count..length {
                self.tick(span)?;
                self.delete_property_or_throw(
                    &output,
                    &JsString::from(index.to_string().as_str()),
                    span,
                )?;
            }
        }
        Ok(Value::Object(output))
    }

    // Bottom-up merge sort is stable, fallible, and uses no native recursion.
    // Values move between buffers; only observable comparisons copy payloads.
    fn stable_array_sort(
        &mut self,
        mut items: Vec<Value>,
        comparator: &Value,
        span: Span,
    ) -> Result<Vec<Value>, Error> {
        let length = items.len();
        if length < 2 {
            return Ok(items);
        }
        self.object_work(span, |_, budget| budget.charge(length))?;
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(length)
            .map_err(|_| allocation_limit(span))?;
        let mut width = 1usize;
        while width < length {
            let mut start = 0usize;
            while start < length {
                let middle = start.saturating_add(width).min(length);
                let end = middle.saturating_add(width).min(length);
                let (mut left, mut right) = (start, middle);
                while left < middle || right < end {
                    self.tick(span)?;
                    let take_left = right == end
                        || left < middle
                            && self.compare_array_elements(
                                &items[left],
                                &items[right],
                                comparator,
                                span,
                            )? != Ordering::Greater;
                    let index = if take_left {
                        let index = left;
                        left += 1;
                        index
                    } else {
                        let index = right;
                        right += 1;
                        index
                    };
                    buffer.push(std::mem::replace(&mut items[index], Value::Undefined));
                }
                start = end;
            }
            std::mem::swap(&mut items, &mut buffer);
            buffer.clear();
            width = width.saturating_mul(2);
        }
        Ok(items)
    }

    fn compare_array_elements(
        &mut self,
        left: &Value,
        right: &Value,
        comparator: &Value,
        span: Span,
    ) -> Result<Ordering, Error> {
        match (left, right) {
            (Value::Undefined, Value::Undefined) => return Ok(Ordering::Equal),
            (Value::Undefined, _) => return Ok(Ordering::Greater),
            (_, Value::Undefined) => return Ok(Ordering::Less),
            _ => {}
        }
        if !matches!(comparator, Value::Undefined) {
            self.object_work(span, |_, budget| {
                budget.value(left)?;
                budget.value(right)
            })?;
            let result = self.call(
                comparator.clone(),
                Value::Undefined,
                vec![left.clone(), right.clone()],
                span,
            )?;
            let number = self.number(result, span)?;
            // NaN, +0, and -0 all preserve the original relative order.
            return Ok(if number < 0.0 {
                Ordering::Less
            } else if number > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            });
        }
        self.object_work(span, |_, budget| budget.value(left))?;
        let left = self.string(left.clone(), span)?;
        self.object_work(span, |_, budget| budget.value(right))?;
        let right = self.string(right.clone(), span)?;
        self.object_work(span, |_, budget| budget.charge(left.len().min(right.len())))?;
        Ok(left.code_units().cmp(right.code_units()))
    }
}

fn allocation_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "Array sort allocation limit exceeded".into(),
    }
}
