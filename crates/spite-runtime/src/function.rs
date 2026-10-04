//! Builtin function objects and the initial Object/Function prototype graph.

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span, WellKnownSymbol};

mod arguments;
mod array;
mod arrow;
mod boolean;
mod bound;
mod builtin;
mod construct;
mod error;
mod instance;
mod iterator;
mod method;
pub(crate) use method::MethodFunction;
mod number;
mod object;
mod ordinary;
mod string;
mod symbol;
mod wrapper;
pub(crate) use arrow::ScriptFunction;
pub(crate) use bound::BoundFunction;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Builtin {
    FunctionPrototype,
    FunctionCall,
    FunctionApply,
    FunctionBind,
    FunctionHasInstance,
    FunctionToString,
    ThrowTypeError,
    ObjectToString,
    ObjectValueOf,
    Object,
    ObjectHasOwnProperty,
    ObjectPropertyIsEnumerable,
    ObjectIsPrototypeOf,
    ObjectToLocaleString,
    ObjectDefineProperty,
    ObjectGetOwnPropertyDescriptor,
    ObjectHasOwn,
    ObjectIs,
    ObjectGetPrototypeOf,
    ObjectSetPrototypeOf,
    ObjectIsExtensible,
    ObjectPreventExtensions,
    ObjectCreate,
    ObjectDefineProperties,
    ObjectFreeze,
    ObjectSeal,
    ObjectIsFrozen,
    ObjectIsSealed,
    ObjectAssign,
    ObjectGetOwnPropertyDescriptors,
    ObjectGetOwnPropertyNames,
    ObjectGetOwnPropertySymbols,
    ObjectKeys,
    ObjectValues,
    ObjectEntries,
    Error(error::ErrorConstructor),
    ErrorToString,
    ErrorIsError,
    Boolean,
    BooleanToString,
    BooleanValueOf,
    Array,
    ArrayIsArray,
    ArrayOf,
    ArraySpecies,
    ArrayKeys,
    ArrayValues,
    ArrayEntries,
    ArrayIteratorNext,
    IteratorIdentity,
    IteratorTagGet,
    IteratorTagSet,
    StringIterator,
    StringIteratorNext,
    ArrayJoin,
    ArrayAt,
    ArrayPush,
    ArrayPop,
    ArrayShift,
    ArrayUnshift,
    ArrayReverse,
    ArraySort,
    ArrayToSorted,
    ArrayToReversed,
    ArrayToSpliced,
    ArrayWith,
    ArrayFill,
    ArrayCopyWithin,
    ArrayForEach,
    ArrayEvery,
    ArraySome,
    ArrayMap,
    ArrayFilter,
    ArraySlice,
    ArrayConcat,
    ArrayFlat,
    ArrayFlatMap,
    ArrayFind,
    ArrayFindIndex,
    ArrayFindLast,
    ArrayFindLastIndex,
    ArrayIncludes,
    ArrayIndexOf,
    ArrayLastIndexOf,
    ArrayReduce,
    ArrayReduceRight,
    ArrayToString,
    ArrayToLocaleString,
    Symbol,
    SymbolFor,
    SymbolKeyFor,
    SymbolToString,
    SymbolValueOf,
    SymbolDescription,
    SymbolToPrimitive,
    String,
    StringToString,
    StringValueOf,
    StringFromCharCode,
    StringFromCodePoint,
    StringRaw,
    StringAt,
    StringCharAt,
    StringCharCodeAt,
    StringCodePointAt,
    StringIsWellFormed,
    StringToWellFormed,
    StringConcat,
    StringSlice,
    StringSubstring,
    StringTrim,
    StringTrimStart,
    StringTrimEnd,
    StringRepeat,
    StringPadStart,
    StringPadEnd,
    StringIndexOf,
    StringLastIndexOf,
    StringIncludes,
    StringStartsWith,
    StringEndsWith,
    Number,
    NumberValueOf,
    NumberToString,
    NumberToFixed,
    NumberToPrecision,
    NumberToExponential,
    NumberToLocaleString,
    ParseFloat,
    ParseInt,
    IsFinite,
    IsNaN,
    NumberIsFinite,
    NumberIsNaN,
    NumberIsInteger,
    NumberIsSafeInteger,
}

impl Builtin {
    // [[InitialName]] does not change when the public name property is altered.
    fn initial_name(self) -> &'static str {
        match self {
            Self::FunctionPrototype | Self::ThrowTypeError => "",
            Self::FunctionCall => "call",
            Self::FunctionApply => "apply",
            Self::FunctionBind => "bind",
            Self::FunctionHasInstance => "[Symbol.hasInstance]",
            Self::FunctionToString
            | Self::ArrayToString
            | Self::ObjectToString
            | Self::BooleanToString
            | Self::SymbolToString
            | Self::StringToString
            | Self::ErrorToString
            | Self::NumberToString => "toString",
            Self::ObjectValueOf
            | Self::BooleanValueOf
            | Self::NumberValueOf
            | Self::SymbolValueOf
            | Self::StringValueOf => "valueOf",
            Self::Boolean => "Boolean",
            Self::Array => "Array",
            Self::ArrayIsArray => "isArray",
            Self::ArrayOf => "of",
            Self::ArraySpecies => "get [Symbol.species]",
            Self::ArrayKeys => "keys",
            Self::ArrayValues => "values",
            Self::ArrayEntries => "entries",
            Self::ArrayIteratorNext | Self::StringIteratorNext => "next",
            Self::IteratorIdentity | Self::StringIterator => "[Symbol.iterator]",
            Self::IteratorTagGet => "get [Symbol.toStringTag]",
            Self::IteratorTagSet => "set [Symbol.toStringTag]",
            Self::ArrayJoin => "join",
            Self::ArrayPush => "push",
            Self::ArrayPop => "pop",
            Self::ArrayShift => "shift",
            Self::ArrayUnshift => "unshift",
            Self::ArrayReverse => "reverse",
            Self::ArraySort => "sort",
            Self::ArrayToSorted => "toSorted",
            Self::ArrayToReversed => "toReversed",
            Self::ArrayToSpliced => "toSpliced",
            Self::ArrayWith => "with",
            Self::ArrayFill => "fill",
            Self::ArrayCopyWithin => "copyWithin",
            Self::ArrayForEach => "forEach",
            Self::ArrayEvery => "every",
            Self::ArraySome => "some",
            Self::ArrayMap => "map",
            Self::ArrayFilter => "filter",
            Self::ArraySlice => "slice",
            Self::ArrayConcat => "concat",
            Self::ArrayFlat => "flat",
            Self::ArrayFlatMap => "flatMap",
            Self::ArrayReduce => "reduce",
            Self::ArrayReduceRight => "reduceRight",
            Self::ArrayFind => "find",
            Self::ArrayFindIndex => "findIndex",
            Self::ArrayFindLast => "findLast",
            Self::ArrayFindLastIndex => "findLastIndex",
            Self::Symbol => "Symbol",
            Self::SymbolFor => "for",
            Self::SymbolKeyFor => "keyFor",
            Self::SymbolDescription => "get description",
            Self::SymbolToPrimitive => "[Symbol.toPrimitive]",
            Self::String => "String",
            Self::StringFromCharCode => "fromCharCode",
            Self::StringFromCodePoint => "fromCodePoint",
            Self::StringRaw => "raw",
            Self::StringAt | Self::ArrayAt => "at",
            Self::StringCharAt => "charAt",
            Self::StringCharCodeAt => "charCodeAt",
            Self::StringCodePointAt => "codePointAt",
            Self::StringIsWellFormed => "isWellFormed",
            Self::StringToWellFormed => "toWellFormed",
            Self::StringConcat => "concat",
            Self::StringSlice => "slice",
            Self::StringSubstring => "substring",
            Self::StringTrim => "trim",
            Self::StringTrimStart => "trimStart",
            Self::StringTrimEnd => "trimEnd",
            Self::StringRepeat => "repeat",
            Self::StringPadStart => "padStart",
            Self::StringPadEnd => "padEnd",
            Self::StringIndexOf | Self::ArrayIndexOf => "indexOf",
            Self::StringLastIndexOf | Self::ArrayLastIndexOf => "lastIndexOf",
            Self::StringIncludes | Self::ArrayIncludes => "includes",
            Self::StringStartsWith => "startsWith",
            Self::StringEndsWith => "endsWith",
            Self::Object => "Object",
            Self::ObjectHasOwnProperty => "hasOwnProperty",
            Self::ObjectPropertyIsEnumerable => "propertyIsEnumerable",
            Self::ObjectIsPrototypeOf => "isPrototypeOf",
            Self::ObjectDefineProperty => "defineProperty",
            Self::ObjectGetOwnPropertyDescriptor => "getOwnPropertyDescriptor",
            Self::ObjectHasOwn => "hasOwn",
            Self::ObjectIs => "is",
            Self::ObjectGetPrototypeOf => "getPrototypeOf",
            Self::ObjectSetPrototypeOf => "setPrototypeOf",
            Self::ObjectIsExtensible => "isExtensible",
            Self::ObjectPreventExtensions => "preventExtensions",
            Self::ObjectCreate => "create",
            Self::ObjectDefineProperties => "defineProperties",
            Self::ObjectFreeze => "freeze",
            Self::ObjectSeal => "seal",
            Self::ObjectIsFrozen => "isFrozen",
            Self::ObjectIsSealed => "isSealed",
            Self::ObjectAssign => "assign",
            Self::ObjectGetOwnPropertyDescriptors => "getOwnPropertyDescriptors",
            Self::ObjectGetOwnPropertyNames => "getOwnPropertyNames",
            Self::ObjectGetOwnPropertySymbols => "getOwnPropertySymbols",
            Self::ObjectKeys => "keys",
            Self::ObjectValues => "values",
            Self::ObjectEntries => "entries",
            Self::Number => "Number",
            Self::Error(kind) => kind.name(),
            Self::ErrorIsError => "isError",
            Self::NumberIsFinite | Self::IsFinite => "isFinite",
            Self::NumberIsNaN | Self::IsNaN => "isNaN",
            Self::NumberIsInteger => "isInteger",
            Self::NumberIsSafeInteger => "isSafeInteger",
            Self::NumberToFixed => "toFixed",
            Self::NumberToPrecision => "toPrecision",
            Self::NumberToExponential => "toExponential",
            Self::NumberToLocaleString | Self::ObjectToLocaleString | Self::ArrayToLocaleString => {
                "toLocaleString"
            }
            Self::ParseFloat => "parseFloat",
            Self::ParseInt => "parseInt",
        }
    }

    fn length(self) -> f64 {
        match self {
            Self::FunctionCall
            | Self::IteratorTagSet
            | Self::FunctionHasInstance
            | Self::FunctionBind
            | Self::Boolean
            | Self::Array
            | Self::ArrayIsArray
            | Self::ArrayJoin
            | Self::ArrayAt
            | Self::ArrayPush
            | Self::ArrayUnshift
            | Self::ArrayFill
            | Self::ArrayForEach
            | Self::ArrayEvery
            | Self::ArraySome
            | Self::ArrayMap
            | Self::ArrayFilter
            | Self::ArrayConcat
            | Self::ArrayFlatMap
            | Self::ArrayFind
            | Self::ArrayFindIndex
            | Self::ArrayFindLast
            | Self::ArrayFindLastIndex
            | Self::ArrayIncludes
            | Self::ArrayIndexOf
            | Self::ArrayLastIndexOf
            | Self::ArrayReduce
            | Self::ArrayReduceRight
            | Self::ArraySort
            | Self::ArrayToSorted
            | Self::SymbolFor
            | Self::SymbolKeyFor
            | Self::SymbolToPrimitive
            | Self::String
            | Self::StringFromCharCode
            | Self::StringFromCodePoint
            | Self::StringRaw
            | Self::StringAt
            | Self::StringCharAt
            | Self::StringCharCodeAt
            | Self::StringCodePointAt
            | Self::StringConcat
            | Self::StringRepeat
            | Self::StringPadStart
            | Self::StringPadEnd
            | Self::StringIndexOf
            | Self::StringLastIndexOf
            | Self::StringIncludes
            | Self::StringStartsWith
            | Self::StringEndsWith
            | Self::Object
            | Self::ObjectHasOwnProperty
            | Self::ObjectPropertyIsEnumerable
            | Self::ObjectIsPrototypeOf
            | Self::ObjectGetPrototypeOf
            | Self::ObjectIsExtensible
            | Self::ObjectPreventExtensions
            | Self::ObjectFreeze
            | Self::ObjectSeal
            | Self::ObjectIsFrozen
            | Self::ObjectIsSealed
            | Self::ObjectGetOwnPropertyDescriptors
            | Self::ObjectGetOwnPropertyNames
            | Self::ObjectGetOwnPropertySymbols
            | Self::ObjectKeys
            | Self::ObjectValues
            | Self::ObjectEntries
            | Self::Number
            | Self::Error(_)
            | Self::ErrorIsError
            | Self::NumberToString
            | Self::NumberToFixed
            | Self::NumberToPrecision
            | Self::NumberToExponential
            | Self::NumberIsFinite
            | Self::NumberIsNaN
            | Self::NumberIsInteger
            | Self::NumberIsSafeInteger
            | Self::IsFinite
            | Self::IsNaN
            | Self::ParseFloat => 1.0,
            Self::FunctionApply
            | Self::ArraySlice
            | Self::ArrayCopyWithin
            | Self::ArrayWith
            | Self::ArrayToSpliced
            | Self::StringSlice
            | Self::StringSubstring
            | Self::ParseInt
            | Self::ObjectGetOwnPropertyDescriptor
            | Self::ObjectHasOwn
            | Self::ObjectSetPrototypeOf
            | Self::ObjectCreate
            | Self::ObjectDefineProperties
            | Self::ObjectAssign
            | Self::ObjectIs => 2.0,
            Self::ObjectDefineProperty => 3.0,
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
    Method(Box<MethodFunction>),
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
            Self::Method(method) => FunctionText::Script(method.code.source.clone()),
        }
    }
}

#[derive(Debug)]
pub(super) struct Intrinsics {
    pub object: object::ObjectIntrinsics,
    pub errors: error::ErrorIntrinsics,
    pub is_finite: ObjectHandle,
    pub is_nan: ObjectHandle,
    pub object_prototype: ObjectHandle,
    pub function_prototype: ObjectHandle,
    pub object_to_string: ObjectHandle,
    pub object_value_of: ObjectHandle,
    pub throw_type_error: ObjectHandle,
    pub function_call: ObjectHandle,
    pub function_apply: ObjectHandle,
    pub function_bind: ObjectHandle,
    pub function_has_instance: ObjectHandle,
    pub function_to_string: ObjectHandle,
    pub boolean: boolean::BooleanIntrinsics,
    pub number: number::NumberIntrinsics,
    pub string: string::StringIntrinsics,
    pub symbol: symbol::SymbolIntrinsics,
    pub array: array::ArrayIntrinsics,
    pub iterator: iterator::IteratorIntrinsics,
}

impl Intrinsics {
    pub fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.is_finite,
            &self.is_nan,
            &self.object_prototype,
            &self.function_prototype,
            &self.object_to_string,
            &self.object_value_of,
            &self.throw_type_error,
            &self.function_call,
            &self.function_apply,
            &self.function_bind,
            &self.function_has_instance,
            &self.function_to_string,
        ]
        .into_iter()
        .chain(self.boolean.roots())
        .chain(self.number.roots())
        .chain(self.errors.roots())
        .chain(self.object.roots())
        .chain(self.string.roots())
        .chain(self.symbol.roots())
        .chain(self.array.roots())
        .chain(self.iterator.roots())
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
        // 20.2.3.6: fixed, non-enumerable and non-constructible default hook.
        let function_has_instance =
            self.new_builtin(&function_prototype, Builtin::FunctionHasInstance, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &function_prototype,
                WellKnownSymbol::HasInstance.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(function_has_instance.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        let boolean = self.boolean_intrinsics(&object_prototype, &function_prototype, span)?;
        let number = self.number_intrinsics(&object_prototype, &function_prototype, span)?;
        let is_finite = self.new_builtin(&function_prototype, Builtin::IsFinite, span)?;
        let is_nan = self.new_builtin(&function_prototype, Builtin::IsNaN, span)?;
        let errors = self.error_intrinsics(&object_prototype, &function_prototype, span)?;
        let object =
            self.object_constructor_intrinsics(&object_prototype, &function_prototype, span)?;
        let string = self.string_intrinsics(&object_prototype, &function_prototype, span)?;
        let symbol = self.symbol_intrinsics(&object_prototype, &function_prototype, span)?;
        let iterator = self.iterator_intrinsics(&object_prototype, &function_prototype, span)?;
        let array = self.array_intrinsics(&object_prototype, &function_prototype, span)?;
        // Publish only after the graph is fully initialized. A failed attempt
        // leaves unreachable allocations that explicit collection can reclaim.
        self.intrinsics = Some(Intrinsics {
            object,
            errors,
            is_finite,
            is_nan,
            object_prototype: object_prototype.clone(),
            function_prototype,
            object_to_string,
            object_value_of,
            throw_type_error,
            function_call,
            function_apply,
            function_bind,
            function_has_instance,
            function_to_string,
            boolean,
            number,
            string,
            symbol,
            array,
            iterator,
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

    pub(super) fn define_builtin_property(
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
        if self.limits.max_arguments.is_some_and(|limit| count > limit) {
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
        let length = self.length_of_array_like(&object, span)?;
        if self
            .limits
            .max_arguments
            .is_some_and(|limit| length > limit as u64)
            || self
                .remaining_steps
                .is_some_and(|work| length > work as u64)
        {
            return Err(Error::Limit {
                span,
                message: "array-like argument list exceeds host limits".into(),
            });
        }
        let mut values = Vec::new();
        // Retain ToLength's full width and grow storage as Get succeeds. A huge
        // array-like can throw at its first getter without allocating the list.
        for index in 0..length {
            values.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "call argument allocation failed".into(),
            })?;
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
        if self.call_depth >= 32 {
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
                Some(Callable::Method(method)) => {
                    let Value::Object(callee) = function else {
                        unreachable!("callable object")
                    };
                    return self.call_ordinary(method.code, callee, this, None, arguments, span);
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
                builtin => self.call_builtin(builtin, this, arguments, span),
            }?;
            self.check_string(&result, span)?;
            return Ok(result);
        }
    }
}
