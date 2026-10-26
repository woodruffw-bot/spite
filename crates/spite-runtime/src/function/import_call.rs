//! EvaluateImportCall operand validation and intrinsic rejection (13.3.10.2).

use super::Builtin;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn import_call_values(
        &mut self,
        specifier: Value,
        options: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Both operand evaluations precede capability creation. Conversion and
        // option failures reject this intrinsic Promise; host failures escape.
        let capability = self.intrinsic_promise_capability(span)?;
        let validation = (|| {
            self.string(specifier, span)?;
            if matches!(options, Value::Undefined) {
                return Ok(());
            }
            let Value::Object(options) = options else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "import options must be an object",
                ));
            };
            let attributes = self.get_property(&options, &JsString::from("with"), span)?;
            if matches!(attributes, Value::Undefined) {
                return Ok(());
            }
            if !matches!(attributes, Value::Object(_)) {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "import attributes must be an object",
                ));
            }
            // EnumerableOwnProperties collects every value before the algorithm
            // checks any attribute type. Keys are already filtered to strings;
            // temporary entry pairs are unnecessary for this validation prefix.
            let values =
                self.enumerable_own_property_list(attributes, Builtin::ObjectValues, span)?;
            if values
                .iter()
                .any(|value| !matches!(value, Value::String(_)))
            {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "import attribute values must be strings",
                ));
            }
            Ok(())
        })();
        if let Err(error) = validation {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            return Ok(capability.promise);
        }
        Err(Self::unsupported(
            span,
            "dynamic module loading is not implemented",
        ))
    }
}
