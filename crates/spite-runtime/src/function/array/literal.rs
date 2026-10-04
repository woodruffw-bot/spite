//! ArrayAccumulation without spread (13.2.4.1–2).

use crate::{Error, ExceptionKind, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};
use spite_parser::ast::Expr;

impl Realm {
    pub(crate) fn array_literal(
        &mut self,
        elements: &[Option<Expr>],
        span: Span,
    ) -> Result<Value, Error> {
        let array = self.create_intrinsic_array(0, span)?;
        // Source and parser limits normally bound this well below u32::MAX.
        u32::try_from(elements.len()).map_err(|_| {
            Self::exception(ExceptionKind::RangeError, span, "invalid Array length")
        })?;
        for (index, element) in elements.iter().enumerate() {
            let index = index as u32;
            if let Some(expression) = element {
                // Element evaluation does not infer a name for anonymous functions.
                let value = self.expression(expression)?;
                self.create_array_element(&array, u64::from(index), value, expression.span)?;
            } else {
                // Elisions grow length without defining an indexed property.
                // The fresh Array owns a writable length, so this Set cannot
                // invoke inherited setters or any other JavaScript code.
                self.object_work(span, |objects, budget| {
                    objects.define(
                        &array,
                        JsString::from("length"),
                        DataDescriptor {
                            value: Some(Value::Number(f64::from(index + 1))),
                            ..Default::default()
                        },
                        budget,
                    )
                })?;
            }
        }
        Ok(Value::Object(array))
    }
}
