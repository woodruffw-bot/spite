//! Date objects and UTC timestamp operations (21.4).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor, realm_object::Hint,
};
use spite_core::{
    JsString, Span, WellKnownSymbol,
    date::{UtcDateTime, format_iso_date_time, parse_date_time_string, time_clip},
};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Method {
    GetDate,
    GetDay,
    GetFullYear,
    GetHours,
    GetMilliseconds,
    GetMinutes,
    GetMonth,
    GetSeconds,
    GetTime,
    GetTimezoneOffset,
    GetUtcDate,
    GetUtcDay,
    GetUtcFullYear,
    GetUtcHours,
    GetUtcMilliseconds,
    GetUtcMinutes,
    GetUtcMonth,
    GetUtcSeconds,
    SetDate,
    SetFullYear,
    SetHours,
    SetMilliseconds,
    SetMinutes,
    SetMonth,
    SetSeconds,
    SetTime,
    SetUtcDate,
    SetUtcFullYear,
    SetUtcHours,
    SetUtcMilliseconds,
    SetUtcMinutes,
    SetUtcMonth,
    SetUtcSeconds,
    ToDateString,
    ToIsoString,
    ToJson,
    ToLocaleDateString,
    ToLocaleString,
    ToLocaleTimeString,
    ToString,
    ToTimeString,
    ToUtcString,
    ValueOf,
    ToPrimitive,
}

impl Method {
    const ALL: [Self; 44] = [
        Self::GetDate,
        Self::GetDay,
        Self::GetFullYear,
        Self::GetHours,
        Self::GetMilliseconds,
        Self::GetMinutes,
        Self::GetMonth,
        Self::GetSeconds,
        Self::GetTime,
        Self::GetTimezoneOffset,
        Self::GetUtcDate,
        Self::GetUtcDay,
        Self::GetUtcFullYear,
        Self::GetUtcHours,
        Self::GetUtcMilliseconds,
        Self::GetUtcMinutes,
        Self::GetUtcMonth,
        Self::GetUtcSeconds,
        Self::SetDate,
        Self::SetFullYear,
        Self::SetHours,
        Self::SetMilliseconds,
        Self::SetMinutes,
        Self::SetMonth,
        Self::SetSeconds,
        Self::SetTime,
        Self::SetUtcDate,
        Self::SetUtcFullYear,
        Self::SetUtcHours,
        Self::SetUtcMilliseconds,
        Self::SetUtcMinutes,
        Self::SetUtcMonth,
        Self::SetUtcSeconds,
        Self::ToDateString,
        Self::ToIsoString,
        Self::ToJson,
        Self::ToLocaleDateString,
        Self::ToLocaleString,
        Self::ToLocaleTimeString,
        Self::ToString,
        Self::ToTimeString,
        Self::ToUtcString,
        Self::ValueOf,
        Self::ToPrimitive,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::GetDate => "getDate",
            Self::GetDay => "getDay",
            Self::GetFullYear => "getFullYear",
            Self::GetHours => "getHours",
            Self::GetMilliseconds => "getMilliseconds",
            Self::GetMinutes => "getMinutes",
            Self::GetMonth => "getMonth",
            Self::GetSeconds => "getSeconds",
            Self::GetTime => "getTime",
            Self::GetTimezoneOffset => "getTimezoneOffset",
            Self::GetUtcDate => "getUTCDate",
            Self::GetUtcDay => "getUTCDay",
            Self::GetUtcFullYear => "getUTCFullYear",
            Self::GetUtcHours => "getUTCHours",
            Self::GetUtcMilliseconds => "getUTCMilliseconds",
            Self::GetUtcMinutes => "getUTCMinutes",
            Self::GetUtcMonth => "getUTCMonth",
            Self::GetUtcSeconds => "getUTCSeconds",
            Self::SetDate => "setDate",
            Self::SetFullYear => "setFullYear",
            Self::SetHours => "setHours",
            Self::SetMilliseconds => "setMilliseconds",
            Self::SetMinutes => "setMinutes",
            Self::SetMonth => "setMonth",
            Self::SetSeconds => "setSeconds",
            Self::SetTime => "setTime",
            Self::SetUtcDate => "setUTCDate",
            Self::SetUtcFullYear => "setUTCFullYear",
            Self::SetUtcHours => "setUTCHours",
            Self::SetUtcMilliseconds => "setUTCMilliseconds",
            Self::SetUtcMinutes => "setUTCMinutes",
            Self::SetUtcMonth => "setUTCMonth",
            Self::SetUtcSeconds => "setUTCSeconds",
            Self::ToDateString => "toDateString",
            Self::ToIsoString => "toISOString",
            Self::ToJson => "toJSON",
            Self::ToLocaleDateString => "toLocaleDateString",
            Self::ToLocaleString => "toLocaleString",
            Self::ToLocaleTimeString => "toLocaleTimeString",
            Self::ToString => "toString",
            Self::ToTimeString => "toTimeString",
            Self::ToUtcString => "toUTCString",
            Self::ValueOf => "valueOf",
            Self::ToPrimitive => "[Symbol.toPrimitive]",
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::SetFullYear | Self::SetMinutes | Self::SetUtcFullYear | Self::SetUtcMinutes => {
                3.0
            }
            Self::SetHours | Self::SetUtcHours => 4.0,
            Self::SetMonth | Self::SetSeconds | Self::SetUtcMonth | Self::SetUtcSeconds => 2.0,
            Self::SetDate
            | Self::SetMilliseconds
            | Self::SetTime
            | Self::SetUtcDate
            | Self::SetUtcMilliseconds
            | Self::ToJson
            | Self::ToPrimitive => 1.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DateIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl DateIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(self.methods.iter())
    }
}

/// Milliseconds since the epoch, including clocks that precede it. Unlike an
/// input Date timestamp, the current instant is floored to its containing ms.
pub(super) fn current_time_value() -> f64 {
    let milliseconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as f64,
        Err(error) => {
            let duration = error.duration();
            -(duration.as_millis() as f64 + f64::from(duration.subsec_nanos() % 1_000_000 != 0))
        }
    };
    time_clip(milliseconds)
}

impl Realm {
    pub(super) fn date_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<DateIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Date, span)?;
        // 21.4.4: Date.prototype is ordinary, with no [[DateValue]].
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                "prototype",
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        methods.try_reserve_exact(47).map_err(|_| Error::Limit {
            span,
            message: "Date intrinsic allocation failed".into(),
        })?;
        for builtin in [Builtin::DateNow, Builtin::DateParse, Builtin::DateUtc] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &constructor,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        // Materialize the complete edition-17 property graph. Pending bodies
        // report Unsupported, so inherited methods never silently disappear.
        for method in Method::ALL {
            let function =
                self.new_builtin(function_prototype, Builtin::DateMethod(method), span)?;
            if matches!(method, Method::ToPrimitive) {
                self.object_work(span, |objects, budget| {
                    objects.define(
                        &prototype,
                        WellKnownSymbol::ToPrimitive.symbol(),
                        DataDescriptor {
                            value: Some(Value::Object(function.clone())),
                            writable: Some(false),
                            enumerable: Some(false),
                            configurable: Some(true),
                        },
                        budget,
                    )
                })?;
            } else {
                self.define_builtin_property(
                    &prototype,
                    method.name(),
                    Value::Object(function.clone()),
                    true,
                    span,
                )?;
            }
            methods.push(function);
        }
        Ok(DateIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    pub(super) fn parse_date_value(&mut self, text: &JsString, span: Span) -> Result<f64, Error> {
        self.object_work(span, |_, budget| budget.charge(text.len()))?;
        let Some(parsed) = parse_date_time_string(text) else {
            return Ok(f64::NAN);
        };
        parsed
            .utc_time_value()
            .ok_or_else(|| Self::unsupported(span, "local Date time zone resolution"))
    }

    #[inline(never)]
    pub(super) fn date_constructor(
        &mut self,
        new_target: ObjectHandle,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let value = match arguments.len() {
            0 => current_time_value(),
            1 => {
                let value = arguments.into_iter().next().expect("one argument");
                let copied = if let Value::Object(object) = &value {
                    self.object_work(span, |objects, _| Ok(objects.inspect(object)?.date_value()))?
                } else {
                    None
                };
                if let Some(time) = copied {
                    time
                } else {
                    match self.primitive(value, Hint::Default, span)? {
                        Value::String(text) => self.parse_date_value(&text, span)?,
                        other => self.number(other, span)?,
                    }
                }
            }
            _ => {
                // All seven conversions precede calendar arithmetic, including
                // later abrupt completions when an earlier result is NaN.
                for argument in arguments.into_iter().take(7) {
                    self.number(argument, span)?;
                }
                return Err(Self::unsupported(
                    span,
                    "numeric Date calendar construction and local time zones",
                ));
            }
        };
        // 21.4.2.1: input conversion precedes GetPrototypeFromConstructor.
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .date
                .prototype
                .clone()
        };
        self.object_work(span, |objects, _| objects.create_date(&prototype, value))
            .map(Value::Object)
    }

    pub(super) fn date_utc(
        &mut self,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
        for argument in arguments.take(6) {
            self.number(argument, span)?;
        }
        Err(Self::unsupported(span, "numeric Date calendar arithmetic"))
    }

    fn this_date_value(&mut self, value: &Value, span: Span) -> Result<f64, Error> {
        let time = if let Value::Object(object) = value {
            self.object_work(span, |objects, _| Ok(objects.inspect(object)?.date_value()))?
        } else {
            None
        };
        time.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a Date value",
            )
        })
    }

    pub(super) fn date_method(
        &mut self,
        method: Method,
        this: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if matches!(method, Method::ToPrimitive) {
            let Value::Object(object) = this else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Date primitive conversion requires an object",
                ));
            };
            let hint = match arguments.next().unwrap_or(Value::Undefined) {
                Value::String(hint)
                    if hint == JsString::from("string") || hint == JsString::from("default") =>
                {
                    Hint::String
                }
                Value::String(hint) if hint == JsString::from("number") => Hint::Number,
                _ => {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "invalid Date primitive conversion hint",
                    ));
                }
            };
            return self.ordinary_to_primitive(object, hint, span);
        }
        if matches!(method, Method::ToJson) {
            let object = self.box_primitive(this, span)?;
            let primitive = self.primitive(object.clone(), Hint::Number, span)?;
            if matches!(primitive, Value::Number(value) if !value.is_finite()) {
                return Ok(Value::Null);
            }
            let Value::Object(handle) = &object else {
                unreachable!("ToObject");
            };
            let method = self.get_property(handle, &JsString::from("toISOString"), span)?;
            return self.call(method, object, Vec::new(), span);
        }
        let time = self.this_date_value(&this, span)?;
        match method {
            Method::GetTime | Method::ValueOf => Ok(Value::Number(time)),
            Method::SetTime => {
                let value =
                    time_clip(self.number(arguments.next().unwrap_or(Value::Undefined), span)?);
                let Value::Object(object) = this else {
                    unreachable!("Date brand");
                };
                self.object_work(span, |objects, _| objects.set_date_value(&object, value))?;
                Ok(Value::Number(value))
            }
            Method::ToIsoString => {
                if time.is_nan() {
                    return Err(Self::exception(
                        ExceptionKind::RangeError,
                        span,
                        "invalid Date time value",
                    ));
                }
                let fields = UtcDateTime::from_time_value(time as i64).expect("clipped Date value");
                self.date_string_work(
                    if (0..=9999).contains(&fields.year) {
                        24
                    } else {
                        27
                    },
                    span,
                )?;
                Ok(Value::String(
                    format_iso_date_time(time as i64).expect("clipped Date value"),
                ))
            }
            Method::ToString
            | Method::ToDateString
            | Method::ToTimeString
            | Method::ToUtcString
                if time.is_nan() =>
            {
                self.date_string_work(12, span)?;
                Ok(Value::String(JsString::from("Invalid Date")))
            }
            Method::GetUtcDate
            | Method::GetUtcDay
            | Method::GetUtcFullYear
            | Method::GetUtcHours
            | Method::GetUtcMilliseconds
            | Method::GetUtcMinutes
            | Method::GetUtcMonth
            | Method::GetUtcSeconds => {
                if time.is_nan() {
                    return Ok(Value::Number(f64::NAN));
                }
                let fields = UtcDateTime::from_time_value(time as i64).expect("clipped Date value");
                let value = match method {
                    Method::GetUtcDate => f64::from(fields.day),
                    Method::GetUtcDay => f64::from(fields.weekday),
                    Method::GetUtcFullYear => f64::from(fields.year),
                    Method::GetUtcHours => f64::from(fields.hour),
                    Method::GetUtcMilliseconds => f64::from(fields.millisecond),
                    Method::GetUtcMinutes => f64::from(fields.minute),
                    Method::GetUtcMonth => f64::from(fields.month),
                    Method::GetUtcSeconds => f64::from(fields.second),
                    _ => unreachable!("UTC getter"),
                };
                Ok(Value::Number(value))
            }
            _ => Err(Self::unsupported(
                span,
                "Date calendar mutation and local/legacy string operations",
            )),
        }
    }

    fn date_string_work(&mut self, length: usize, span: Span) -> Result<(), Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(Error::Limit {
                span,
                message: "Date string output limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(length))
    }
}
