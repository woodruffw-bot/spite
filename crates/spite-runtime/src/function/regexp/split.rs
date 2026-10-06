//! Generic RegExp splitting via species construction and custom exec (22.2.6.14).

use super::{advance_string_index, regexp_output_limit};
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, value::to_uint32};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    // SpeciesConstructor (7.3.23) rejects primitive constructors before reading
    // species. Its default is the intrinsic, regardless of the public binding.
    pub(super) fn regexp_species_constructor(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let constructor = self.get_property(object, &JsString::from("constructor"), span)?;
        let species = match constructor {
            Value::Undefined => Value::Undefined,
            Value::Object(constructor) => {
                self.get_property(&constructor, &WellKnownSymbol::Species.symbol(), span)?
            }
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "RegExp constructor property must be an Object or undefined",
                ));
            }
        };
        if matches!(species, Value::Undefined | Value::Null) {
            return Ok(Value::Object(
                self.intrinsics
                    .as_ref()
                    .expect("initialized")
                    .regexp
                    .constructor
                    .clone(),
            ));
        }
        if self.is_constructor(&species, span)? {
            Ok(species)
        } else {
            Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "RegExp species must be a constructor",
            ))
        }
    }

    #[inline(never)]
    pub(super) fn regexp_split(
        &mut self,
        receiver: Value,
        argument: Value,
        limit: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let string = self.string(argument, span)?;
        let constructor = self.regexp_species_constructor(&object, span)?;
        let flags = self.get_property(&object, &JsString::from("flags"), span)?;
        let flags = self.string(flags, span)?;
        self.object_work(span, |_, budget| budget.charge(flags.len()))?;
        let mut unicode = false;
        let mut sticky = false;
        for &unit in flags.code_units() {
            unicode |= unit == u16::from(b'u') || unit == u16::from(b'v');
            sticky |= unit == u16::from(b'y');
        }
        let flags = if sticky {
            flags
        } else {
            let length = flags
                .len()
                .checked_add(1)
                .ok_or_else(|| regexp_output_limit(span))?;
            let mut units = self.regexp_string_buffer(length, span)?;
            units.extend_from_slice(flags.code_units());
            units.push(u16::from(b'y'));
            JsString::from_code_units(units)
        };
        let Value::Object(splitter) = self.construct(
            constructor,
            vec![Value::Object(object), Value::String(flags)],
            span,
        )?
        else {
            unreachable!("Construct returns an Object");
        };
        let array = self.create_intrinsic_array(0, span)?;
        let limit = if matches!(limit, Value::Undefined) {
            u32::MAX
        } else {
            to_uint32(self.number(limit, span)?)
        };
        if limit == 0 {
            return Ok(Value::Object(array));
        }
        if string.is_empty() {
            if matches!(self.regexp_exec(&splitter, &string, span)?, Value::Null) {
                self.create_array_element(&array, 0, Value::String(string), span)?;
            }
            return Ok(Value::Object(array));
        }
        let last_index = JsString::from("lastIndex");
        let mut start = 0usize;
        let mut position = 0usize;
        let mut count = 0u32;
        while position < string.len() {
            self.tick(span)?;
            self.set_property_or_throw(
                &splitter,
                last_index.clone(),
                Value::Number(position as f64),
                span,
            )?;
            let result = self.regexp_exec(&splitter, &string, span)?;
            if let Value::Object(result) = result {
                let value = self.get_property(&splitter, &last_index, span)?;
                let end = self
                    .length_from_value(value, span)?
                    .min(string.len() as u64) as usize;
                if end != start {
                    let value =
                        self.regexp_substring(&string.code_units()[start..position], span)?;
                    self.create_array_element(&array, u64::from(count), value, span)?;
                    count += 1;
                    if count == limit {
                        return Ok(Value::Object(array));
                    }
                    start = end;
                    let length = self.length_of_array_like(&result, span)?;
                    for index in 1..length {
                        self.tick(span)?;
                        // Captures are copied as-is: no ToString, result[0],
                        // result.index or result.groups lookup is performed.
                        let value = self.get_property(
                            &result,
                            &JsString::from(index.to_string().as_str()),
                            span,
                        )?;
                        self.create_array_element(&array, u64::from(count), value, span)?;
                        count += 1;
                        if count == limit {
                            return Ok(Value::Object(array));
                        }
                    }
                    position = start;
                    continue;
                }
            }
            // position is in the String, so advancing one code point stays
            // within its length and conversion back to usize is exact.
            position = advance_string_index(&string, position as u64, unicode) as usize;
        }
        let value = self.regexp_substring(&string.code_units()[start..], span)?;
        self.create_array_element(&array, u64::from(count), value, span)?;
        Ok(Value::Object(array))
    }

    fn regexp_substring(&mut self, units: &[u16], span: Span) -> Result<Value, Error> {
        let mut output = self.regexp_string_buffer(units.len(), span)?;
        output.extend_from_slice(units);
        Ok(Value::String(JsString::from_code_units(output)))
    }
}
