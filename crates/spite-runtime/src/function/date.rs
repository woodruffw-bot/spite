//! Date objects and UTC timestamp operations (21.4).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, TimeZoneError, Value, object::DataDescriptor,
    realm_object::Hint,
};
use spite_core::{
    JsString, Span, WellKnownSymbol,
    date::{
        MAX_TIME_VALUE, MS_PER_DAY, RecurringTimeZoneError, TzifTimeZoneError, UtcDateTime,
        format_iso_date_time, format_utc_date_string, make_date, make_day, make_full_year,
        make_time, parse_date_time_string, parse_utc_date_string, time_clip,
    },
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
        if let Some(parsed) = parse_date_time_string(text) {
            return match parsed.utc_time_value() {
                Some(value) => Ok(value),
                None => self.date_clip_local_value(
                    parsed
                        .nominal_epoch_milliseconds()
                        .expect("valid interchange fields") as f64,
                    span,
                ),
            };
        }
        // 21.4.3.2 also requires parsing our own toUTCString output for Dates
        // with zero milliseconds. No implementation-specific fallback is used.
        Ok(parse_utc_date_string(text).map_or(f64::NAN, |time| time as f64))
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
                let local = self.date_numeric_calendar(arguments.into_iter(), span)?;
                self.date_clip_local_value(local, span)?
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

    #[inline(never)]
    pub(super) fn date_utc(
        &mut self,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        Ok(Value::Number(time_clip(
            self.date_numeric_calendar(arguments, span)?,
        )))
    }

    #[inline(never)]
    fn date_numeric_calendar(
        &mut self,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<f64, Error> {
        // 21.4.2.1 and 21.4.3.4: every present component converts in order,
        // even after an earlier NaN. Absent fields retain numeric defaults.
        let mut fields = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        fields[0] = self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
        for (field, argument) in fields[1..].iter_mut().zip(arguments) {
            *field = self.number(argument, span)?;
        }
        let day =
            self.date_make_day_value(make_full_year(fields[0]), fields[1], fields[2], span)?;
        Ok(make_date(
            day,
            make_time(fields[3], fields[4], fields[5], fields[6]),
        ))
    }

    #[inline(never)]
    fn date_make_day_value(
        &mut self,
        year: f64,
        month: f64,
        date: f64,
        span: Span,
    ) -> Result<f64, Error> {
        // Date Number arithmetic may use wide native integers internally;
        // a JavaScript BigInt value quota does not apply to those intermediates.
        let mut budget = spite_bigint::Budget::with_limits(None, self.remaining_steps);
        let day = make_day(year, month, date, &mut budget);
        self.remaining_steps = budget.remaining_work();
        day.map_err(|error| Self::integer_error(error, span))
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
            Method::GetDate
            | Method::GetDay
            | Method::GetFullYear
            | Method::GetHours
            | Method::GetMilliseconds
            | Method::GetMinutes
            | Method::GetMonth
            | Method::GetSeconds
            | Method::GetTimezoneOffset => self.date_get_local(method, time, span),
            Method::SetDate
            | Method::SetMonth
            | Method::SetHours
            | Method::SetMinutes
            | Method::SetSeconds
            | Method::SetMilliseconds
                if time.is_nan() =>
            {
                self.date_set_invalid_local(method, arguments, span)
            }
            Method::SetTime => {
                let value =
                    time_clip(self.number(arguments.next().unwrap_or(Value::Undefined), span)?);
                let Value::Object(object) = this else {
                    unreachable!("Date brand");
                };
                self.object_work(span, |objects, _| objects.set_date_value(&object, value))?;
                Ok(Value::Number(value))
            }
            Method::SetUtcDate => {
                let Value::Object(object) = this else {
                    unreachable!("Date brand");
                };
                self.date_set_utc_date(
                    object,
                    time,
                    arguments.next().unwrap_or(Value::Undefined),
                    span,
                )
            }
            Method::SetUtcMonth | Method::SetUtcFullYear => {
                let Value::Object(object) = this else {
                    unreachable!("Date brand");
                };
                self.date_set_utc_calendar(method, object, time, arguments, span)
            }
            Method::SetUtcHours
            | Method::SetUtcMinutes
            | Method::SetUtcSeconds
            | Method::SetUtcMilliseconds => {
                let Value::Object(object) = this else {
                    unreachable!("Date brand");
                };
                self.date_set_utc_time(method, object, time, arguments, span)
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
            Method::ToUtcString => {
                let fields = UtcDateTime::from_time_value(time as i64).expect("clipped Date value");
                let magnitude = fields.year.unsigned_abs();
                let digits = if magnitude < 10_000 {
                    4
                } else if magnitude < 100_000 {
                    5
                } else {
                    6
                };
                self.date_string_work(25 + digits + usize::from(fields.year < 0), span)?;
                Ok(Value::String(
                    format_utc_date_string(time as i64).expect("clipped Date value"),
                ))
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

    // 21.4.1.25 and 21.4.4.2–11: all stored time values are clipped integers.
    // Adding even a full native i32 offset remains within i64 and below 2^53;
    // the local calendar intermediate must not itself be TimeClipped.
    #[inline(never)]
    fn date_get_local(&mut self, method: Method, time: f64, span: Span) -> Result<Value, Error> {
        if time.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        let offset = self
            .time_zone()
            .map_err(|error| Self::date_time_zone_error(error, span))?
            .offset_at(i128::from(time as i64));
        let local = time as i64 + i64::from(offset) * 1000;
        if matches!(method, Method::GetTimezoneOffset) {
            // Preserve the specified subtraction/division, including +0 in UTC.
            return Ok(Value::Number((time - local as f64) / 60_000.0));
        }
        let fields = UtcDateTime::from_epoch_milliseconds(local);
        let value = match method {
            Method::GetDate => f64::from(fields.day),
            Method::GetDay => f64::from(fields.weekday),
            Method::GetFullYear => f64::from(fields.year),
            Method::GetHours => f64::from(fields.hour),
            Method::GetMilliseconds => f64::from(fields.millisecond),
            Method::GetMinutes => f64::from(fields.minute),
            Method::GetMonth => f64::from(fields.month),
            Method::GetSeconds => f64::from(fields.second),
            _ => unreachable!("local getter"),
        };
        Ok(Value::Number(value))
    }

    fn date_time_zone_error(error: TimeZoneError, span: Span) -> Error {
        let message = error.to_string();
        match error {
            TimeZoneError::Tzif(TzifTimeZoneError::Allocation)
            | TimeZoneError::Recurring(RecurringTimeZoneError::Allocation) => {
                Error::Limit { span, message }
            }
            TimeZoneError::Tzif(
                TzifTimeZoneError::UnsupportedVersion | TzifTimeZoneError::UnsupportedLeapSeconds,
            ) => Error::Unsupported { span, message },
            _ => Error::Host { span, message },
        }
    }

    // 21.4.1.26/30: this helper implements the composition TimeClip(UTC(t)).
    // Non-finite inputs return before system-zone loading. A finite input is
    // never clipped before offset resolution; the bound below is a proof that
    // no native i32 offset could bring the final instant into Date's domain.
    #[inline(never)]
    fn date_clip_local_value(&mut self, local: f64, span: Span) -> Result<f64, Error> {
        if !local.is_finite() {
            return Ok(f64::NAN);
        }
        let zone = self
            .time_zone()
            .map_err(|error| Self::date_time_zone_error(error, span))?;
        let maximum_offset = i128::from(i32::MIN).abs() * 1000;
        if local.abs() > (i128::from(MAX_TIME_VALUE) + maximum_offset) as f64 {
            return Ok(f64::NAN);
        }
        // Calendar inputs within the proof's bound are exact integral Numbers
        // below 2^53. The resolved epoch also stays below 2^53 before TimeClip.
        debug_assert_eq!(local.fract(), 0.0);
        let utc = zone
            .resolve_local(local as i128)
            .map_err(|error| Error::Host {
                span,
                message: error.to_string(),
            })?;
        Ok(time_clip(utc as f64))
    }

    #[inline(never)]
    fn date_set_invalid_local(
        &mut self,
        method: Method,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 21.4.4.20, 22–26: all present arguments convert before testing the
        // captured NaN, and returning NaN does not write the slot. A conversion
        // hook may have revived the object. setFullYear has different rules.
        self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
        for argument in arguments.take(method.length() as usize - 1) {
            self.number(argument, span)?;
        }
        Ok(Value::Number(f64::NAN))
    }

    // 21.4.4.28 and 32: retain the captured calendar and time of day through
    // coercion. Only setUTCFullYear replaces a captured NaN with the epoch.
    #[inline(never)]
    fn date_set_utc_calendar(
        &mut self,
        method: Method,
        object: ObjectHandle,
        time: f64,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let (start, time) = match method {
            Method::SetUtcFullYear => (0, if time.is_nan() { 0.0 } else { time }),
            Method::SetUtcMonth => (1, time),
            _ => unreachable!("UTC calendar setter"),
        };
        let mut converted = [None; 3];
        converted[start] = Some(self.number(arguments.next().unwrap_or(Value::Undefined), span)?);
        for (slot, argument) in converted[start + 1..].iter_mut().zip(arguments) {
            *slot = Some(self.number(argument, span)?);
        }
        if time.is_nan() {
            // Conversion may have revived the object. setUTCMonth must return
            // NaN without overwriting the time installed by that hook.
            return Ok(Value::Number(f64::NAN));
        }
        let time = time as i64;
        let previous = UtcDateTime::from_time_value(time).expect("clipped Date value");
        let mut fields = [
            f64::from(previous.year),
            f64::from(previous.month),
            f64::from(previous.day),
        ];
        for (field, converted) in fields.iter_mut().zip(converted) {
            if let Some(value) = converted {
                *field = value;
            }
        }
        // Unlike Date.UTC, year setters preserve literal years 0 through 99.
        let day = self.date_make_day_value(fields[0], fields[1], fields[2], span)?;
        let value = time_clip(make_date(day, time.rem_euclid(MS_PER_DAY) as f64));
        self.object_work(span, |objects, _| objects.set_date_value(&object, value))?;
        Ok(Value::Number(value))
    }

    // 21.4.4.27: the captured year/month are already normalized, so their
    // month's first day follows exactly from Day(t) and DateFromTime(t).
    // Retain MakeDay's ordered Number additions and the captured time of day.
    #[inline(never)]
    fn date_set_utc_date(
        &mut self,
        object: ObjectHandle,
        time: f64,
        date: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let date = self.number(date, span)?;
        if time.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        let time = time as i64;
        let fields = UtcDateTime::from_time_value(time).expect("clipped Date value");
        // At the lower TimeClip boundary the month's first day is outside
        // the clipped domain, but is still a valid finite intermediate time.
        let first_day = time.div_euclid(MS_PER_DAY) - i64::from(fields.day) + 1;
        let day = (first_day as f64 + date.trunc()) - 1.0;
        let value = time_clip(make_date(day, time.rem_euclid(MS_PER_DAY) as f64));
        self.object_work(span, |objects, _| objects.set_date_value(&object, value))?;
        Ok(Value::Number(value))
    }

    // 21.4.4.29–30, 31 and 33: capture [[DateValue]] before any conversion.
    // A previously invalid Date returns NaN after converting all present
    // arguments and does not overwrite a value installed by a conversion hook.
    #[inline(never)]
    fn date_set_utc_time(
        &mut self,
        method: Method,
        object: ObjectHandle,
        time: f64,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let start = match method {
            Method::SetUtcHours => 0,
            Method::SetUtcMinutes => 1,
            Method::SetUtcSeconds => 2,
            Method::SetUtcMilliseconds => 3,
            _ => unreachable!("UTC time setter"),
        };
        let mut converted = [None; 4];
        converted[start] = Some(self.number(arguments.next().unwrap_or(Value::Undefined), span)?);
        for (slot, argument) in converted[start + 1..].iter_mut().zip(arguments) {
            *slot = Some(self.number(argument, span)?);
        }
        if time.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        let previous = UtcDateTime::from_time_value(time as i64).expect("clipped Date value");
        let mut fields = [
            f64::from(previous.hour),
            f64::from(previous.minute),
            f64::from(previous.second),
            f64::from(previous.millisecond),
        ];
        for (field, converted) in fields.iter_mut().zip(converted) {
            if let Some(value) = converted {
                *field = value;
            }
        }
        let day = (time as i64).div_euclid(MS_PER_DAY) as f64;
        let value = time_clip(make_date(
            day,
            make_time(fields[0], fields[1], fields[2], fields[3]),
        ));
        self.object_work(span, |objects, _| objects.set_date_value(&object, value))?;
        Ok(Value::Number(value))
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
