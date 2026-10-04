//! Native algorithms are kept out of the recursive script dispatch frame.

use super::{Builtin, Callable, FunctionText, number};
use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    // Preserve a separate frame so adding native algorithms does not enlarge
    // every recursive script call, including calls from parameter defaults.
    #[inline(never)]
    pub(super) fn call_builtin(
        &mut self,
        builtin: Builtin,
        this: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        match builtin {
            Builtin::FunctionCall | Builtin::FunctionApply | Builtin::FunctionBind => {
                unreachable!("tail transfers are handled by callable dispatch")
            }
            Builtin::FunctionToString => {
                // 20.2.3.5: builtin source uses immutable [[InitialName]], never
                // an observable read of the mutable public name property.
                let callable = if let Value::Object(object) = &this {
                    self.object_work(span, |objects, _| {
                        Ok(objects
                            .inspect(object)?
                            .callable()
                            .map(Callable::source_text))
                    })?
                } else {
                    None
                };
                match callable {
                    Some(FunctionText::Script(source)) => {
                        self.object_work(span, |_, budget| budget.charge(source.as_str().len()))?;
                        Ok(Value::String(JsString::from(source.as_str())))
                    }
                    Some(FunctionText::Native(name)) => Ok(Value::String(JsString::from(
                        format!("function {name}() {{ [native code] }}").as_str(),
                    ))),
                    None => Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Function.prototype.toString requires a callable receiver",
                    )),
                }
            }
            Builtin::FunctionPrototype => Ok(Value::Undefined),
            Builtin::Object => {
                self.object_constructor(None, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::ObjectHasOwnProperty | Builtin::ObjectPropertyIsEnumerable => self
                .object_property_predicate(
                    this,
                    arguments.next().unwrap_or(Value::Undefined),
                    matches!(builtin, Builtin::ObjectPropertyIsEnumerable),
                    span,
                ),
            Builtin::ObjectIsPrototypeOf => self.object_is_prototype_of(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectToLocaleString => self.object_to_locale_string(this, span),
            Builtin::ObjectAssign => self.object_assign(
                arguments.next().unwrap_or(Value::Undefined),
                arguments,
                span,
            ),
            Builtin::ObjectGetOwnPropertyDescriptors => self.object_get_own_property_descriptors(
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectFreeze | Builtin::ObjectSeal => self.object_set_integrity(
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::ObjectFreeze),
                span,
            ),
            Builtin::ObjectIsFrozen | Builtin::ObjectIsSealed => self.object_test_integrity(
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::ObjectIsFrozen),
                span,
            ),
            Builtin::ObjectCreate => self.object_create(
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectDefineProperties => self.object_define_properties(
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectGetPrototypeOf => {
                self.object_get_prototype_of(arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::ObjectSetPrototypeOf => self.object_set_prototype_of(
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectIsExtensible | Builtin::ObjectPreventExtensions => self
                .object_extensibility(
                    arguments.next().unwrap_or(Value::Undefined),
                    matches!(builtin, Builtin::ObjectPreventExtensions),
                    span,
                ),
            Builtin::ObjectDefineProperty => self.object_define_property(
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ObjectGetOwnPropertyDescriptor | Builtin::ObjectHasOwn => self
                .object_own_property(
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                    matches!(builtin, Builtin::ObjectHasOwn),
                    span,
                ),
            Builtin::ObjectIs => {
                let first = arguments.next().unwrap_or(Value::Undefined);
                let second = arguments.next().unwrap_or(Value::Undefined);
                self.object_work(span, |_, budget| {
                    budget.value(&first)?;
                    budget.value(&second)?;
                    Ok(Value::Boolean(first.same_value(&second)))
                })
            }
            Builtin::Error(kind) => self.error_constructor(kind, None, arguments, span),
            Builtin::ErrorToString => self.error_to_string(this, span),
            Builtin::ErrorIsError => {
                self.error_is_error(arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::IsFinite | Builtin::IsNaN => {
                // 19.2.2–3 use ToNumber, unlike the non-coercing Number methods.
                let number = self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
                Ok(Value::Boolean(match builtin {
                    Builtin::IsFinite => number.is_finite(),
                    Builtin::IsNaN => number.is_nan(),
                    _ => unreachable!("global numeric predicate"),
                }))
            }
            Builtin::ParseFloat => self
                .parse_float(arguments.next().unwrap_or(Value::Undefined), span)
                .map(Value::Number),
            Builtin::ParseInt => self
                .parse_int(
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                    span,
                )
                .map(Value::Number),
            Builtin::Number => Ok(Value::Number(
                self.number_constructor_value(arguments.next(), span)?,
            )),
            Builtin::NumberValueOf => Ok(Value::Number(self.this_number_value(&this, span)?)),
            Builtin::NumberToString => {
                self.number_prototype_to_string(&this, arguments.next(), span)
            }
            Builtin::NumberToFixed => self.number_prototype_to_fixed(&this, arguments.next(), span),
            Builtin::NumberToPrecision => {
                self.number_prototype_to_precision(&this, arguments.next(), span)
            }
            Builtin::NumberToExponential => {
                self.number_prototype_to_exponential(&this, arguments.next(), span)
            }
            Builtin::NumberToLocaleString => {
                // 21.1.3.4 explicitly permits ordinary numeric formatting
                // without ECMA-402. Reserved arguments remain unused.
                let number = self.this_number_value(&this, span)?;
                Ok(Value::String(JsString::from(
                    crate::value::number_to_string(number).as_str(),
                )))
            }
            Builtin::NumberIsFinite
            | Builtin::NumberIsNaN
            | Builtin::NumberIsInteger
            | Builtin::NumberIsSafeInteger => {
                Ok(Value::Boolean(number::predicate(builtin, arguments.next())))
            }
            Builtin::Boolean => Ok(Value::Boolean(
                arguments.next().unwrap_or(Value::Undefined).to_boolean(),
            )),
            Builtin::String => self.string_constructor(None, arguments.next(), span),
            Builtin::StringToString | Builtin::StringValueOf => {
                self.this_string_value(&this, span).map(Value::String)
            }
            Builtin::BooleanValueOf => Ok(Value::Boolean(self.this_boolean_value(&this, span)?)),
            Builtin::BooleanToString => Ok(Value::String(JsString::from(
                if self.this_boolean_value(&this, span)? {
                    "true"
                } else {
                    "false"
                },
            ))),
            Builtin::ThrowTypeError => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "restricted function property",
            )),
            Builtin::ObjectValueOf => self.box_primitive(this, span),
            Builtin::ObjectToString => {
                // 20.1.3.6. No Symbol keys or additional exotic object kinds are
                // exposed yet; their tags and hooks must join this dispatch later.
                let tag = match &this {
                    Value::Undefined => "Undefined",
                    Value::Null => "Null",
                    Value::Boolean(_) => "Boolean",
                    Value::Number(_) => "Number",
                    Value::BigInt(_) => "BigInt",
                    Value::String(_) => "String",
                    Value::Object(handle) => self.object_work(span, |objects, _| {
                        let object = objects.inspect(handle)?;
                        Ok(if object.is_error() {
                            "Error"
                        } else if object.boolean_data().is_some() {
                            "Boolean"
                        } else if object.number_data().is_some() {
                            "Number"
                        } else if object.string_data().is_some() {
                            "String"
                        } else if object.is_arguments() {
                            "Arguments"
                        } else if object.is_callable() {
                            "Function"
                        } else {
                            "Object"
                        })
                    })?,
                };
                Ok(Value::String(JsString::from(
                    format!("[object {tag}]").as_str(),
                )))
            }
        }
    }
}
