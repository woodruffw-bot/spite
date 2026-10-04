//! Array.from: iterable and array-like sources with ordered construction (23.1.2.1).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn array_from(
        &mut self,
        receiver: Value,
        items: Value,
        mapper: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let mapping = !matches!(mapper, Value::Undefined);
        if mapping && !self.is_callable(&mapper, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Array mapper is not callable",
            ));
        }
        let using_iterator = self.get_method(&items, &WellKnownSymbol::Iterator.symbol(), span)?;
        if let Some(method) = using_iterator {
            // Iterable construction receives no arguments and precedes calling
            // the iterator method. Species is never consulted.
            let result = self.array_from_result(receiver, None, span)?;
            let mut iterator = self.get_iterator_from_method(items, method, span)?;
            let mut index = 0;
            loop {
                self.tick(span)?;
                // Check before advancing even when the next result would be done.
                if index >= 9_007_199_254_740_991 {
                    let error = Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Array.from exceeds the maximum safe integer length",
                    );
                    return Err(self.iterator_close_error(&iterator, error, span));
                }
                // Step failures do not close the iterator (IteratorStepValue).
                let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                    return self.finish_array_from(result, index, span);
                };
                let define = self
                    .array_from_map(value, index, mapping.then_some(&mapper), &this_arg, span)
                    .and_then(|value| self.create_array_element(&result, index, value, span));
                if let Err(error) = define {
                    return Err(self.iterator_close_error(&iterator, error, span));
                }
                index += 1;
            }
        }
        let Value::Object(source) = self.box_primitive(items, span)? else {
            unreachable!("ToObject");
        };
        let length = self.length_of_array_like(&source, span)?;
        let result = self.array_from_result(receiver, Some(length), span)?;
        for index in 0..length {
            self.tick(span)?;
            // Get every index, including holes; only the source length is fixed.
            let value =
                self.get_property(&source, &JsString::from(index.to_string().as_str()), span)?;
            let value =
                self.array_from_map(value, index, mapping.then_some(&mapper), &this_arg, span)?;
            self.create_array_element(&result, index, value, span)?;
        }
        self.finish_array_from(result, length, span)
    }

    fn array_from_result(
        &mut self,
        receiver: Value,
        length: Option<u64>,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let constructor = if let Value::Object(object) = &receiver {
            self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.is_constructor())
            })?
        } else {
            false
        };
        if constructor {
            let arguments = length
                .map(|n| Value::Number(n as f64))
                .into_iter()
                .collect();
            let Value::Object(result) = self.construct(receiver, arguments, span)? else {
                unreachable!("Construct returns an object");
            };
            Ok(result)
        } else {
            self.create_intrinsic_array(length.unwrap_or(0), span)
        }
    }

    fn array_from_map(
        &mut self,
        value: Value,
        index: u64,
        mapper: Option<&Value>,
        this_arg: &Value,
        span: Span,
    ) -> Result<Value, Error> {
        if let Some(mapper) = mapper {
            self.object_work(span, |_, budget| {
                budget.value(mapper)?;
                budget.value(this_arg)
            })?;
            self.call(
                mapper.clone(),
                this_arg.clone(),
                vec![value, Value::Number(index as f64)],
                span,
            )
        } else {
            Ok(value)
        }
    }

    fn finish_array_from(
        &mut self,
        result: ObjectHandle,
        length: u64,
        span: Span,
    ) -> Result<Value, Error> {
        self.set_property_or_throw(
            &result,
            JsString::from("length"),
            Value::Number(length as f64),
            span,
        )?;
        Ok(Value::Object(result))
    }
}
