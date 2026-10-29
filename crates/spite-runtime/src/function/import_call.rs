//! EvaluateImportCall operand validation and intrinsic rejection (13.3.10.2).

use super::Builtin;
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};
use spite_parser::ast::{ImportAttribute, ModuleRequest};

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
            let specifier = self.string(specifier, span)?;
            if matches!(options, Value::Undefined) {
                return Ok(ModuleRequest {
                    specifier,
                    attributes: Vec::new(),
                });
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
                return Ok(ModuleRequest {
                    specifier,
                    attributes: Vec::new(),
                });
            }
            if !matches!(attributes, Value::Object(_)) {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "import attributes must be an object",
                ));
            }
            // Snapshot all enumerable values before validating their types.
            // Entry arrays are private ordinary arrays, so reading their own
            // key/value cells cannot invoke attribute getters a second time.
            let entries =
                self.enumerable_own_property_list(attributes, Builtin::ObjectEntries, span)?;
            let mut attributes = Vec::new();
            attributes
                .try_reserve_exact(entries.len())
                .map_err(|_| Error::Limit {
                    span,
                    message: "import attribute capacity exceeded".into(),
                })?;
            for entry in entries {
                let Value::Object(entry) = entry else {
                    unreachable!("Object.entries pair")
                };
                let Value::String(key) = self.get_property(&entry, &JsString::from("0"), span)?
                else {
                    unreachable!("string property key")
                };
                let Value::String(value) = self.get_property(&entry, &JsString::from("1"), span)?
                else {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "import attribute values must be strings",
                    ));
                };
                attributes.push(ImportAttribute { key, value });
            }
            attributes.sort_unstable_by(|left, right| left.key.cmp(&right.key));
            Ok(ModuleRequest {
                specifier,
                attributes,
            })
        })();
        match validation {
            Ok(request) => self.queue_module_import(request, capability.clone(), span)?,
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.call(capability.reject, Value::Undefined, vec![reason], span)?;
            }
        }
        Ok(capability.promise)
    }
}
