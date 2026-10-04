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
            Builtin::ArraySpecies | Builtin::IteratorIdentity => Ok(this),
            Builtin::IteratorTagGet => Ok(Value::String(JsString::from("Iterator"))),
            Builtin::IteratorTagSet => {
                self.iterator_tag_setter(this, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::ArrayKeys | Builtin::ArrayValues | Builtin::ArrayEntries => {
                let kind = match builtin {
                    Builtin::ArrayKeys => crate::object::ArrayIterationKind::Key,
                    Builtin::ArrayValues => crate::object::ArrayIterationKind::Value,
                    _ => crate::object::ArrayIterationKind::KeyValue,
                };
                self.array_iterator(this, kind, span)
            }
            Builtin::ArrayIteratorNext => self.array_iterator_next(this, span),
            Builtin::StringIterator => self.string_iterator(this, span),
            Builtin::StringIteratorNext => self.string_iterator_next(this, span),
            Builtin::Object => {
                self.object_constructor(None, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::ObjectGetOwnPropertyNames | Builtin::ObjectGetOwnPropertySymbols => self
                .object_get_own_property_keys(
                    arguments.next().unwrap_or(Value::Undefined),
                    matches!(builtin, Builtin::ObjectGetOwnPropertySymbols),
                    span,
                ),
            Builtin::ObjectKeys | Builtin::ObjectValues | Builtin::ObjectEntries => self
                .object_enumerable_properties(
                    arguments.next().unwrap_or(Value::Undefined),
                    builtin,
                    span,
                ),
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
            Builtin::Array => self.array_constructor(None, arguments, span),
            Builtin::ArrayIsArray => self.array_is_array(arguments.next(), span),
            Builtin::ArrayOf => self.array_of(this, arguments, span),
            Builtin::ArrayJoin => {
                self.array_join(this, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::ArrayToString => self.array_to_string(this, span),
            Builtin::ArrayToLocaleString => self.array_to_locale_string(this, span),
            Builtin::ArrayPush => self.array_push(this, arguments, span),
            Builtin::ArrayPop => self.array_pop(this, span),
            Builtin::ArrayShift => self.array_shift(this, span),
            Builtin::ArrayUnshift => self.array_unshift(this, arguments, span),
            Builtin::ArrayReverse => self.array_reverse(this, span),
            Builtin::ArraySlice => self.array_slice(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArraySort | Builtin::ArrayToSorted => self.array_sort(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::ArrayToSorted),
                span,
            ),
            Builtin::ArrayToReversed => self.array_to_reversed(this, span),
            Builtin::ArrayToSpliced => self.array_to_spliced(this, arguments, span),
            Builtin::ArrayWith => self.array_with(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArrayFill => self.array_fill(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArrayCopyWithin => self.array_copy_within(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArrayIncludes | Builtin::ArrayIndexOf | Builtin::ArrayLastIndexOf => self
                .array_search(
                    builtin,
                    this,
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next(),
                    span,
                ),
            Builtin::ArrayReduce | Builtin::ArrayReduceRight => self.array_reduce(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next(),
                matches!(builtin, Builtin::ArrayReduceRight),
                span,
            ),
            Builtin::ArrayFind
            | Builtin::ArrayFindIndex
            | Builtin::ArrayFindLast
            | Builtin::ArrayFindLastIndex => self.array_find(
                builtin,
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArrayForEach
            | Builtin::ArrayEvery
            | Builtin::ArraySome
            | Builtin::ArrayMap
            | Builtin::ArrayFilter => self.array_callback(
                builtin,
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::ArrayAt => {
                self.array_at(this, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::String => self.string_constructor(None, arguments.next(), span),
            Builtin::StringRaw => {
                let template = arguments.next().unwrap_or(Value::Undefined);
                self.string_raw(template, arguments, span)
            }
            Builtin::StringConcat => self.string_concat(this, arguments, span),
            Builtin::StringIncludes | Builtin::StringStartsWith | Builtin::StringEndsWith => self
                .string_search_predicate(
                    builtin,
                    this,
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                    span,
                ),
            Builtin::StringIndexOf | Builtin::StringLastIndexOf => self.string_index_of(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::StringLastIndexOf),
                span,
            ),
            Builtin::StringRepeat => {
                self.string_repeat(this, arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::StringPadStart | Builtin::StringPadEnd => self.string_pad(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::StringPadStart),
                span,
            ),
            Builtin::StringTrim | Builtin::StringTrimStart | Builtin::StringTrimEnd => {
                self.string_trim(builtin, this, span)
            }
            Builtin::StringSlice | Builtin::StringSubstring => self.string_substring(
                this,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                matches!(builtin, Builtin::StringSlice),
                span,
            ),
            Builtin::StringFromCharCode | Builtin::StringFromCodePoint => self.string_from_codes(
                arguments,
                matches!(builtin, Builtin::StringFromCodePoint),
                span,
            ),
            Builtin::StringAt
            | Builtin::StringCharAt
            | Builtin::StringCharCodeAt
            | Builtin::StringCodePointAt => self.string_character(
                builtin,
                this,
                arguments.next().unwrap_or(Value::Undefined),
                span,
            ),
            Builtin::StringToString | Builtin::StringValueOf => {
                self.this_string_value(&this, span).map(Value::String)
            }
            Builtin::StringIsWellFormed | Builtin::StringToWellFormed => {
                self.string_well_formed(this, matches!(builtin, Builtin::StringToWellFormed), span)
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
            Builtin::FunctionHasInstance => self
                .ordinary_has_instance(this, arguments.next().unwrap_or(Value::Undefined), span)
                .map(Value::Boolean),
            Builtin::Symbol => {
                self.symbol_constructor(arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::SymbolFor => {
                self.symbol_for(arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::SymbolKeyFor => {
                self.symbol_key_for(arguments.next().unwrap_or(Value::Undefined), span)
            }
            Builtin::SymbolToString => {
                let symbol = self.this_symbol_value(&this, span)?;
                self.symbol_descriptive_string(&symbol, span)
                    .map(Value::String)
            }
            Builtin::SymbolValueOf | Builtin::SymbolToPrimitive => {
                self.this_symbol_value(&this, span).map(Value::Symbol)
            }
            Builtin::SymbolDescription => self.symbol_description(&this, span),
            Builtin::ObjectValueOf => self.box_primitive(this, span),
            Builtin::ObjectToString => self.object_to_string(this, span),
        }
    }
}
