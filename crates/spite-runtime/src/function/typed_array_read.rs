//! TypedArray indexed access, searches, reductions, and strings (23.2.3).

use super::typed_array_methods::Method;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn typed_array_read_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // ValidateTypedArray precedes every argument conversion and callback
        // check. Later user code may invalidate indices in the captured range.
        let witness = self.typed_array_validate(receiver, span)?;
        match method {
            Method::At => self.typed_array_read_at(
                &witness.object,
                witness.length,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Method::Includes | Method::IndexOf | Method::LastIndexOf => self
                .typed_array_read_search(
                    method,
                    &witness.object,
                    witness.length,
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next(),
                    span,
                ),
            Method::Reduce | Method::ReduceRight => self.typed_array_read_reduce(
                &witness.object,
                witness.length,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next(),
                matches!(method, Method::ReduceRight),
                span,
            ),
            Method::Join | Method::ToLocaleString => {
                let locale = matches!(method, Method::ToLocaleString);
                let separator = if locale {
                    // The existing non-402 host uses comma. Reserved locale
                    // arguments are neither converted nor forwarded.
                    JsString::from(",")
                } else {
                    let separator = arguments.next().unwrap_or(Value::Undefined);
                    if matches!(separator, Value::Undefined) {
                        JsString::from(",")
                    } else {
                        // Even an empty TypedArray converts its separator.
                        self.string(separator, span)?
                    }
                };
                self.typed_array_read_string(
                    &witness.object,
                    witness.length,
                    &separator,
                    locale,
                    span,
                )
            }
            _ => unreachable!("TypedArray callback methods have their own dispatch"),
        }
    }

    fn typed_array_read_at(
        &mut self,
        object: &ObjectHandle,
        length: u64,
        index: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.2.3.1: use the captured length, then Get the live indexed value.
        let number = self.number(index, span)?;
        let relative = if number.is_nan() { 0.0 } else { number.trunc() };
        let index = if relative >= 0.0 {
            relative
        } else {
            length as f64 + relative
        };
        if index < 0.0 || index >= length as f64 {
            return Ok(Value::Undefined);
        }
        // The witness bounds finite integral indices below 2^53 - 1.
        self.get_property(
            object,
            &JsString::from((index as u64).to_string().as_str()),
            span,
        )
    }

    fn typed_array_read_search(
        &mut self,
        method: Method,
        object: &ObjectHandle,
        length: u64,
        search: Value,
        from: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.2.3.16/.17/.20 differ in equality, presence, and direction.
        let includes = matches!(method, Method::Includes);
        let backwards = matches!(method, Method::LastIndexOf);
        let missing = if includes {
            Value::Boolean(false)
        } else {
            Value::Number(-1.0)
        };
        if length == 0 {
            // All three skip fromIndex conversion for an initially empty view.
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
        // This handles both infinities before narrowing to an unsigned index.
        if start < 0.0 || start >= length as f64 {
            return Ok(missing);
        }
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
            // Invalidated indices remain terminal at the exotic object.
            // includes reads undefined; strict searches skip the absent index.
            if !includes && !self.has_property(object, &key, span)? {
                continue;
            }
            let element = self.get_property(object, &key, span)?;
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

    fn typed_array_read_reduce(
        &mut self,
        object: &ObjectHandle,
        length: u64,
        callback: Value,
        mut accumulator: Option<Value>,
        backwards: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.2.3.23/.24 validate callability before the empty/no-initial case.
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray reducer is not callable",
            ));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable object");
        };
        for offset in 0..length {
            self.tick(span)?;
            let index = if backwards {
                length - 1 - offset
            } else {
                offset
            };
            // TypedArray reduction never performs HasProperty, including after
            // a callback has detached the buffer or shrunk the view out of bounds.
            let value =
                self.get_property(object, &JsString::from(index.to_string().as_str()), span)?;
            accumulator = Some(match accumulator {
                None => value,
                Some(previous) => self.call(
                    Value::Object(callback.clone()),
                    Value::Undefined,
                    vec![
                        previous,
                        value,
                        Value::Number(index as f64),
                        Value::Object(object.clone()),
                    ],
                    span,
                )?,
            });
        }
        accumulator.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray reduction has no initial value or element",
            )
        })
    }

    fn typed_array_read_string(
        &mut self,
        object: &ObjectHandle,
        length: u64,
        separator: &JsString,
        locale: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.2.3.18/.31: keep the entry range and observe each value anew.
        let mut result = Vec::new();
        for index in 0..length {
            self.tick(span)?;
            if index > 0 {
                self.typed_array_read_append(&mut result, separator, span)?;
            }
            let element =
                self.get_property(object, &JsString::from(index.to_string().as_str()), span)?;
            if matches!(element, Value::Undefined) || (locale && matches!(element, Value::Null)) {
                continue;
            }
            let string = if locale {
                self.object_work(span, |_, budget| budget.value(&element))?;
                let method =
                    self.get_property_value(&element, &JsString::from("toLocaleString"), span)?;
                // Invoke uses the primitive Number/BigInt receiver and no
                // reserved locale/options arguments in this non-402 host.
                let value = self.call(method, element, Vec::new(), span)?;
                self.string(value, span)?
            } else {
                self.string(element, span)?
            };
            self.typed_array_read_append(&mut result, &string, span)?;
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn typed_array_read_append(
        &mut self,
        units: &mut Vec<u16>,
        part: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        // Preflight the same quota as append_string before fallible bulk
        // reservation. No receiver hook runs between this check and the append.
        let length = units
            .len()
            .checked_add(part.len())
            .ok_or_else(|| Error::Limit {
                span,
                message: "string length exceeds platform capacity".into(),
            })?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        let movement = if length > units.capacity() {
            units.len()
        } else {
            0
        };
        self.object_work(span, |_, budget| {
            budget.charge(movement)?;
            budget.charge(part.len())
        })?;
        units.try_reserve(part.len()).map_err(|_| Error::Limit {
            span,
            message: "string storage exceeds platform capacity".into(),
        })?;
        self.append_string(units, part, span)
    }
}
