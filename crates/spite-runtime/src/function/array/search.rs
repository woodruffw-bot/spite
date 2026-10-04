//! Generic element searches with distinct equality/presence rules (23.1.3.16–17/20).

use super::Builtin;
use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn array_search(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        search: Value,
        from: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = self.box_primitive(receiver, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&object, span)?;
        let includes = matches!(builtin, Builtin::ArrayIncludes);
        let backwards = matches!(builtin, Builtin::ArrayLastIndexOf);
        let missing = if includes {
            Value::Boolean(false)
        } else {
            Value::Number(-1.0)
        };
        // Empty receivers do not coerce fromIndex. For lastIndexOf only an
        // absent argument defaults to length - 1; explicit undefined means 0.
        if length == 0 {
            return Ok(missing);
        }
        let number = match from {
            None if backwards => (length - 1) as f64,
            value => self.number(value.unwrap_or(Value::Undefined), span)?,
        };
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        let start = if integer >= 0.0 {
            if backwards {
                integer.min((length - 1) as f64)
            } else {
                integer
            }
        } else if backwards {
            length as f64 + integer
        } else {
            (length as f64 + integer).max(0.0)
        };
        if start < 0.0 || start >= length as f64 {
            return Ok(missing);
        }
        // LengthOfArrayLike bounds every visited integral index below 2^53 - 1.
        let start = start as u64;
        let count = if backwards { start + 1 } else { length - start };
        for offset in 0..count {
            self.tick(span)?;
            let index = if backwards {
                start - offset
            } else {
                start + offset
            };
            let key = JsString::from(index.to_string().as_str());
            // includes uses Get even for holes. The index searches skip absent
            // properties and compare only present own or inherited elements.
            if !includes && !self.has_property(&object, &key, span)? {
                continue;
            }
            let element = self.get_property(&object, &key, span)?;
            let equal = self.object_work(span, |_, budget| {
                budget.value(&search)?;
                budget.value(&element)?;
                Ok(match (&search, &element) {
                    (Value::Number(a), Value::Number(b)) if includes => {
                        a == b || (a.is_nan() && b.is_nan())
                    }
                    _ => search.strictly_equal(&element),
                })
            })?;
            if equal {
                return Ok(if includes {
                    Value::Boolean(true)
                } else {
                    Value::Number(index as f64)
                });
            }
        }
        Ok(missing)
    }
}
