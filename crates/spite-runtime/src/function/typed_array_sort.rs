//! Stable numeric TypedArray sorting (23.2.3.29, 23.2.3.33).

use super::typed_array_methods::Method;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};
use std::cmp::Ordering;

impl Realm {
    pub(super) fn typed_array_sort_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let comparator = arguments.next().unwrap_or(Value::Undefined);
        // Both entries reject a non-callable comparator before validating even
        // an empty, detached, out-of-bounds, or incorrectly branded receiver.
        if !matches!(comparator, Value::Undefined) && !self.is_callable(&comparator, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray comparator must be callable or undefined",
            ));
        }
        let witness = self.typed_array_validate(receiver, span)?;
        let output = match method {
            // The intrinsic allocation precedes indexed reads, including any
            // host failure while decoding a BigInt element. No species hooks.
            Method::ToSorted => {
                self.typed_array_create_same_type(witness.element, witness.length, span)?
            }
            Method::Sort => witness.object.clone(),
            _ => unreachable!("sorting methods dispatched separately"),
        };

        // SortIndexedProperties uses read-through-holes (23.1.3.30.1). Every
        // numeric value is captured before any comparison or callback executes.
        // Grow from observed values instead of narrowing the logical u64 len.
        let mut items = Vec::new();
        for index in 0..witness.length {
            self.tick(span)?;
            let value = self.get_property(
                &witness.object,
                &JsString::from(index.to_string().as_str()),
                span,
            )?;
            self.object_work(span, |_, budget| {
                if items.len() == items.capacity() {
                    budget.charge(items.len())?;
                }
                budget.charge(1)
            })?;
            items.try_reserve(1).map_err(|_| allocation_limit(span))?;
            items.push(value);
        }
        let items = self.stable_typed_array_sort(items, &comparator, span)?;
        for (index, value) in items.into_iter().enumerate() {
            self.tick(span)?;
            // Comparator calls and result conversions may invalidate source.
            // Keep the captured range and the ordinary live setter's behavior;
            // no extra ValidateTypedArray or deletion phase is permitted here.
            self.set_property_or_throw(
                &output,
                JsString::from(index.to_string().as_str()),
                value,
                span,
            )?;
        }
        Ok(Value::Object(output))
    }

    // Stable bottom-up merging propagates abrupt completions immediately and
    // never recurses on the native stack. Payloads move between the two lists;
    // only a user comparator needs copies of its two argument values.
    fn stable_typed_array_sort(
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
                            && self.compare_typed_array_elements(
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
            self.object_work(span, |_, budget| budget.charge(length))?;
            buffer.clear();
            width = width.saturating_mul(2);
        }
        Ok(items)
    }

    /// CompareTypedArrayElements (23.2.4.7). A supplied comparator runs even
    /// for NaN, and its result undergoes ToNumber rather than ToNumeric.
    fn compare_typed_array_elements(
        &mut self,
        left: &Value,
        right: &Value,
        comparator: &Value,
        span: Span,
    ) -> Result<Ordering, Error> {
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
            // NaN and either zero are stable equality, while infinities retain
            // their signs. Result conversion is reentrant; do not revalidate.
            return Ok(if number < 0.0 {
                Ordering::Less
            } else if number > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            });
        }
        match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
                Ok(if left.is_nan() && right.is_nan() {
                    Ordering::Equal
                } else if left.is_nan() {
                    Ordering::Greater
                } else if right.is_nan() || left < right {
                    Ordering::Less
                } else if left > right {
                    Ordering::Greater
                } else {
                    // Equal nonzero numbers have equal signs. Among zeroes,
                    // -0 precedes +0 without subtracting or losing their signs.
                    right.is_sign_negative().cmp(&left.is_sign_negative())
                })
            }
            (Value::BigInt(left_integer), Value::BigInt(right_integer)) => {
                self.comparison_work(left, right, span)?;
                Ok(left_integer.cmp(right_integer))
            }
            _ => unreachable!("captured TypedArray elements have one numeric content type"),
        }
    }
}

fn allocation_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "TypedArray sort allocation limit exceeded".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, object};

    #[test]
    fn comparator_rejection_precedes_receiver_validation() {
        for method in [Method::Sort, Method::ToSorted] {
            let result = Realm::default().typed_array_sort_method(
                method,
                Value::Null,
                vec![Value::Number(7.0)].into_iter(),
                Span::new(0, 0),
            );
            // Both failures are TypeError in JavaScript. Inspect the private
            // diagnostic here to distinguish which ordered check produced it.
            assert!(matches!(result, Err(Error::Exception {
                kind: ExceptionKind::TypeError, message, ..
            }) if message == "TypedArray comparator must be callable or undefined"));
        }
    }

    #[test]
    fn intrinsic_copy_allocation_precedes_element_decoding_with_a_full_heap() {
        const SETUP: &str =
            "let numbers=new Uint8Array([2,1]);let integers=BigUint64Array.of(256n);";

        // Derive the explicit fixture cap from the identical unlimited setup,
        // including its unreachable temporaries before the first collection.
        // No intrinsic-count constant or extra per-test allowance is needed.
        let mut probe = Realm::default();
        probe.eval(SETUP).unwrap();
        probe.eval("numbers").unwrap();
        probe.eval("integers").unwrap();
        let collection = probe.collect(usize::MAX).unwrap();
        let mut realm = Realm::new(Limits {
            max_heap_entries: Some(collection.live.checked_add(collection.reclaimed).unwrap()),
            ..Limits::default()
        });
        realm.eval(SETUP).unwrap();
        let numbers = realm.eval("numbers").unwrap();
        let integers = realm.eval("integers").unwrap();
        loop {
            match realm.objects.create(None) {
                Ok(_) => {}
                Err(object::Error::Heap(spite_heap::Error::Capacity)) => break,
                other => panic!("unexpected heap-fill result: {other:?}"),
            }
        }
        realm.limits.max_bigint_bits = Some(8);
        let span = Span::new(0, 0);
        let arguments = || Vec::new().into_iter();
        // Collection is explicit only. No eval or collect occurs after filling
        // the heap, so unreachable filler objects cannot be reclaimed here.
        assert_eq!(
            realm.typed_array_sort_method(Method::ToSorted, integers.clone(), arguments(), span),
            Err(Error::Limit {
                span,
                message: "heap storage capacity exceeded".into(),
            })
        );
        assert_eq!(
            realm.typed_array_sort_method(Method::Sort, integers, arguments(), span),
            Err(Error::Limit {
                span,
                message: "integer resource limit exceeded".into(),
            })
        );
        assert_eq!(
            realm.typed_array_sort_method(Method::Sort, numbers.clone(), arguments(), span),
            Ok(numbers.clone())
        );
        let Value::Object(numbers) = numbers else {
            unreachable!("fixture TypedArray");
        };
        assert_eq!(
            realm.get_property(&numbers, &JsString::from("0"), span),
            Ok(Value::Number(1.0))
        );
        assert_eq!(
            realm.get_property(&numbers, &JsString::from("1"), span),
            Ok(Value::Number(2.0))
        );
    }

    #[test]
    fn explicit_work_abort_in_a_comparator_preserves_prior_effects_and_skips_language_handlers() {
        for method in ["sort", "toSorted"] {
            let mut realm = Realm::default();
            realm
                .eval("let a=new Uint8Array([3,1,2]),calls=0,flag=0")
                .unwrap();
            // Enable the comparator-focused quota after constructing the fixture.
            realm.limits.max_steps = Some(5_000);
            assert!(matches!(realm.eval(&format!("try{{a.{method}(()=>{{calls++;a[0]=99;while(true){{}}}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit { .. })));
            assert_eq!(
                realm.eval("calls===1 && flag===0 && a.join()==='99,1,2'"),
                Ok(Value::Boolean(true))
            );
        }
    }
}
