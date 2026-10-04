//! Builtin function objects and the initial Object/Function prototype graph.

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span};

mod arguments;
mod arrow;
mod bound;
mod construct;
mod instance;
mod ordinary;
pub(crate) use arrow::ScriptFunction;
pub(crate) use bound::BoundFunction;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Builtin {
    FunctionPrototype,
    FunctionCall,
    FunctionApply,
    FunctionBind,
    FunctionToString,
    ThrowTypeError,
    ObjectToString,
    ObjectValueOf,
}

impl Builtin {
    // [[InitialName]] does not change when the public name property is altered.
    fn initial_name(self) -> &'static str {
        match self {
            Self::FunctionPrototype | Self::ThrowTypeError => "",
            Self::FunctionCall => "call",
            Self::FunctionApply => "apply",
            Self::FunctionBind => "bind",
            Self::FunctionToString | Self::ObjectToString => "toString",
            Self::ObjectValueOf => "valueOf",
        }
    }

    fn length(self) -> f64 {
        match self {
            Self::FunctionCall | Self::FunctionBind => 1.0,
            Self::FunctionApply => 2.0,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub(crate) enum Callable {
    Builtin(Builtin),
    Bound(BoundFunction),
    Arrow(ScriptFunction),
    Ordinary(ScriptFunction),
}

pub(super) enum FunctionText {
    Native(&'static str),
    Script(spite_parser::ast::FunctionSource),
}

impl Callable {
    fn source_text(&self) -> FunctionText {
        match self {
            Self::Builtin(builtin) => FunctionText::Native(builtin.initial_name()),
            Self::Bound(_) => FunctionText::Native(""),
            Self::Arrow(function) | Self::Ordinary(function) => {
                FunctionText::Script(function.source.clone())
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Intrinsics {
    pub object_prototype: ObjectHandle,
    pub function_prototype: ObjectHandle,
    pub object_to_string: ObjectHandle,
    pub object_value_of: ObjectHandle,
    pub throw_type_error: ObjectHandle,
    pub function_call: ObjectHandle,
    pub function_apply: ObjectHandle,
    pub function_bind: ObjectHandle,
    pub function_to_string: ObjectHandle,
}

impl Intrinsics {
    pub fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.object_prototype,
            &self.function_prototype,
            &self.object_to_string,
            &self.object_value_of,
            &self.throw_type_error,
            &self.function_call,
            &self.function_apply,
            &self.function_bind,
            &self.function_to_string,
        ]
        .into_iter()
    }
}

impl Realm {
    pub(super) fn ensure_object_intrinsics(&mut self, span: Span) -> Result<ObjectHandle, Error> {
        if let Some(intrinsics) = &self.intrinsics {
            return Ok(intrinsics.object_prototype.clone());
        }
        let object_prototype =
            self.object_work(span, |objects, _| objects.create_object_prototype())?;
        let function_prototype =
            self.new_builtin(&object_prototype, Builtin::FunctionPrototype, span)?;
        let object_to_string =
            self.new_builtin(&function_prototype, Builtin::ObjectToString, span)?;
        let object_value_of =
            self.new_builtin(&function_prototype, Builtin::ObjectValueOf, span)?;
        for (name, handle) in [
            ("toString", &object_to_string),
            ("valueOf", &object_value_of),
        ] {
            self.define_builtin_property(
                &object_prototype,
                name,
                Value::Object(handle.clone()),
                true,
                span,
            )?;
        }
        // 9.3.2 / 10.2.4: Function.prototype owns the shared restricted accessors.
        let throw_type_error =
            self.new_builtin(&function_prototype, Builtin::ThrowTypeError, span)?;
        for name in ["name", "length"] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &throw_type_error,
                    JsString::from(name),
                    DataDescriptor {
                        configurable: Some(false),
                        ..Default::default()
                    },
                    budget,
                )
            })?;
        }
        self.object_work(span, |objects, _| {
            objects.prevent_extensions(&throw_type_error)
        })?;
        for name in ["caller", "arguments"] {
            self.object_work(span, |objects, budget| {
                objects.define(
                    &function_prototype,
                    JsString::from(name),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(throw_type_error.clone())),
                            set: Some(Some(throw_type_error.clone())),
                        },
                        enumerable: Some(false),
                        configurable: Some(true),
                    },
                    budget,
                )
            })?;
        }
        let function_call = self.new_builtin(&function_prototype, Builtin::FunctionCall, span)?;
        let function_apply = self.new_builtin(&function_prototype, Builtin::FunctionApply, span)?;
        let function_bind = self.new_builtin(&function_prototype, Builtin::FunctionBind, span)?;
        let function_to_string =
            self.new_builtin(&function_prototype, Builtin::FunctionToString, span)?;
        for (name, handle) in [
            ("call", &function_call),
            ("apply", &function_apply),
            ("bind", &function_bind),
            ("toString", &function_to_string),
        ] {
            self.define_builtin_property(
                &function_prototype,
                name,
                Value::Object(handle.clone()),
                true,
                span,
            )?;
        }
        // Publish only after the graph is fully initialized. A failed attempt
        // leaves unreachable allocations that explicit collection can reclaim.
        self.intrinsics = Some(Intrinsics {
            object_prototype: object_prototype.clone(),
            function_prototype,
            object_to_string,
            object_value_of,
            throw_type_error,
            function_call,
            function_apply,
            function_bind,
            function_to_string,
        });
        Ok(object_prototype)
    }

    fn new_builtin(
        &mut self,
        prototype: &ObjectHandle,
        builtin: Builtin,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let object = self.object_work(span, |objects, _| {
            objects.create_builtin(prototype, builtin)
        })?;
        self.define_builtin_property(
            &object,
            "length",
            Value::Number(builtin.length()),
            false,
            span,
        )?;
        self.define_builtin_property(
            &object,
            "name",
            Value::String(JsString::from(builtin.initial_name())),
            false,
            span,
        )?;
        Ok(object)
    }

    fn define_builtin_property(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        value: Value,
        writable: bool,
        span: Span,
    ) -> Result<(), Error> {
        let defined = self.object_work(span, |objects, budget| {
            objects.define(
                object,
                JsString::from(name),
                DataDescriptor {
                    value: Some(value),
                    writable: Some(writable),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        debug_assert!(defined, "fresh intrinsic property");
        Ok(())
    }

    pub(super) fn is_callable(&mut self, value: &Value, span: Span) -> Result<bool, Error> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        self.object_work(span, |objects, _| {
            Ok(objects.inspect(object)?.is_callable())
        })
    }

    pub(super) fn check_argument_count(&self, count: usize, span: Span) -> Result<(), Error> {
        if count > self.limits.max_arguments {
            return Err(Error::Limit {
                span,
                message: "call argument limit exceeded".into(),
            });
        }
        Ok(())
    }

    fn argument_list_from_array_like(
        &mut self,
        value: Value,
        span: Span,
    ) -> Result<Vec<Value>, Error> {
        // CreateListFromArrayLike / LengthOfArrayLike, 7.3.19 / 7.3.18.
        let Value::Object(object) = value else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "apply argument list must be an object",
            ));
        };
        let length = self.get_property(&object, &JsString::from("length"), span)?;
        let number = self.number(length, span)?;
        // ToIntegerOrInfinity then ToLength, 7.1.5 / 7.1.20. NaN maps to +0.
        let length = if number.is_nan() || number <= 0.0 {
            0.0
        } else {
            number.trunc().min(9_007_199_254_740_991.0)
        };
        if length > self.limits.max_arguments as f64 || length > self.remaining_steps as f64 {
            return Err(Error::Limit {
                span,
                message: "array-like argument list exceeds host limits".into(),
            });
        }
        let mut values = Vec::new();
        for index in 0..length as usize {
            values.push(self.get_property(
                &object,
                &JsString::from(index.to_string().as_str()),
                span,
            )?);
        }
        Ok(values)
    }

    pub(super) fn call(
        &mut self,
        function: Value,
        this: Value,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        self.enter_call(span)?;
        let result = self.call_inner(function, this, arguments, span);
        self.call_depth -= 1;
        result
    }

    fn enter_call(&mut self, span: Span) -> Result<(), Error> {
        // Calls and construction share a bound. Bound/call/apply tail transfers
        // stay iterative; user code and getter/coercion re-entry grow the stack.
        if self.call_depth >= 64 {
            return Err(Error::Limit {
                span,
                message: "call nesting limit exceeded".into(),
            });
        }
        self.call_depth += 1;
        Ok(())
    }

    fn call_inner(
        &mut self,
        mut function: Value,
        mut this: Value,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        self.check_argument_count(arguments.len(), span)?;
        let mut arguments = arguments.into_iter();
        loop {
            // 13.3.6.2: callers evaluate arguments before entering this callable check.
            let callable = if let Value::Object(object) = &function {
                self.object_work(span, |objects, budget| {
                    objects
                        .inspect(object)?
                        .callable()
                        .map(|callable| callable.copy_with_budget(budget))
                        .transpose()
                })?
            } else {
                None
            };
            let builtin = match callable {
                Some(Callable::Builtin(builtin)) => builtin,
                Some(Callable::Arrow(arrow)) => return self.call_arrow(arrow, arguments, span),
                Some(Callable::Ordinary(code)) => {
                    let Value::Object(callee) = function else {
                        unreachable!("callable object")
                    };
                    return self.call_ordinary(code, callee, this, None, arguments, span);
                }
                Some(Callable::Bound(bound)) => {
                    let count = bound
                        .arguments
                        .len()
                        .checked_add(arguments.len())
                        .ok_or_else(|| Error::Limit {
                            span,
                            message: "call argument limit exceeded".into(),
                        })?;
                    self.check_argument_count(count, span)?;
                    let mut values = bound.arguments;
                    values.extend(arguments);
                    function = Value::Object(bound.target);
                    this = bound.this;
                    arguments = values.into_iter();
                    continue;
                }
                None => {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "value is not callable",
                    ));
                }
            };
            let result = match builtin {
                Builtin::FunctionCall => {
                    // 20.2.3.3 is a tail call. Transfer ownership of the receiver and
                    // advance the argument iterator without Rust stack recursion.
                    function = this;
                    this = arguments.next().unwrap_or(Value::Undefined);
                    continue;
                }
                Builtin::FunctionBind => return self.bind_function(this, arguments, span),
                Builtin::FunctionApply => {
                    // 20.2.3.1 checks the target before inspecting the argument list.
                    if !self.is_callable(&this, span)? {
                        return Err(Self::exception(
                            ExceptionKind::TypeError,
                            span,
                            "apply requires a callable receiver",
                        ));
                    }
                    let this_argument = arguments.next().unwrap_or(Value::Undefined);
                    let array_like = arguments.next().unwrap_or(Value::Undefined);
                    let values = if matches!(array_like, Value::Undefined | Value::Null) {
                        Vec::new()
                    } else {
                        self.argument_list_from_array_like(array_like, span)?
                    };
                    function = this;
                    this = this_argument;
                    arguments = values.into_iter();
                    continue;
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
                            self.object_work(span, |_, budget| {
                                budget.charge(source.as_str().len())
                            })?;
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
                Builtin::ThrowTypeError => Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "restricted function property",
                )),
                Builtin::ObjectValueOf => {
                    Self::require_object_coercible(&this, span)?;
                    if matches!(this, Value::Object(_)) {
                        Ok(this)
                    } else {
                        Err(Self::unsupported(
                            span,
                            "returning primitive wrapper objects is not implemented",
                        ))
                    }
                }
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
                            Ok(if object.is_arguments() {
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
            }?;
            self.check_string(&result, span)?;
            return Ok(result);
        }
    }
}
