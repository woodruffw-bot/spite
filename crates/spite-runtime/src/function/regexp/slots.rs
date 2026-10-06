//! Native original-slot getters and RegExpBuiltinExec (22.2.6–7).

use super::Member;
use crate::{Error, ExceptionKind, Realm, Value, object::RegExpData};
use spite_core::{JsString, Span, regexp_pattern_source_units};

impl Realm {
    #[inline(never)]
    pub(super) fn regexp_slot_getter(
        &mut self,
        member: Member,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let data = self.object_work(span, |objects, _| {
            Ok(objects.inspect(&object)?.regexp_data().cloned())
        })?;
        let Some(data) = data else {
            if object
                == self
                    .intrinsics
                    .as_ref()
                    .expect("initialized")
                    .regexp
                    .prototype
            {
                if matches!(member, Member::Source) {
                    let mut units = self.regexp_string_buffer(4, span)?;
                    units.extend("(?:)".encode_utf16());
                    return Ok(Value::String(JsString::from_code_units(units)));
                }
                return Ok(Value::Undefined);
            }
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver has no RegExp internal slots",
            ));
        };
        if matches!(member, Member::Source) {
            self.object_work(span, |_, budget| budget.charge(data.source.len()))?;
            let stream = regexp_pattern_source_units(&data.source);
            let length = stream
                .clone()
                .try_fold(0usize, |n, _| n.checked_add(1))
                .ok_or_else(|| super::regexp_output_limit(span))?;
            let mut units = self.regexp_string_buffer(length, span)?;
            units.extend(stream);
            return Ok(Value::String(JsString::from_code_units(units)));
        }
        let flag = match member {
            Member::DotAll => b's',
            Member::Global => b'g',
            Member::HasIndices => b'd',
            Member::IgnoreCase => b'i',
            Member::Multiline => b'm',
            Member::Sticky => b'y',
            Member::Unicode => b'u',
            Member::UnicodeSets => b'v',
            _ => unreachable!("native RegExp flag or source getter"),
        };
        Ok(Value::Boolean(
            data.flags.code_units().contains(&u16::from(flag)),
        ))
    }

    fn regexp_require_data(&mut self, receiver: Value, span: Span) -> Result<RegExpData, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        self.object_work(span, |objects, _| {
            Ok(objects.inspect(&object)?.regexp_data().cloned())
        })?
        .ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver has no RegExpMatcher internal slot",
            )
        })
    }

    #[inline(never)]
    pub(super) fn regexp_native_exec(
        &mut self,
        receiver: Value,
        argument: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.regexp_require_data(receiver.clone(), span)?;
        let string = self.string(argument, span)?;
        self.regexp_builtin_exec(receiver, &string, span)
    }

    // RegExpBuiltinExec, 22.2.7.2. This step executes the supported ordinary
    // matcher subset; other valid Patterns retain the explicit Unsupported boundary.
    #[inline(never)]
    pub(super) fn regexp_builtin_exec(
        &mut self,
        receiver: Value,
        string: &JsString,
        span: Span,
    ) -> Result<Value, Error> {
        let data = self.regexp_require_data(receiver.clone(), span)?;
        let Value::Object(object) = receiver else {
            unreachable!("RegExp brand requires an Object");
        };
        let last_index = JsString::from("lastIndex");
        let index = self.get_property(&object, &last_index, span)?;
        let mut index = self.length_from_value(index, span)?;
        let flags = data.flags.code_units();
        let global = flags.contains(&u16::from(b'g'));
        let sticky = flags.contains(&u16::from(b'y'));
        let has_indices = flags.contains(&u16::from(b'd'));
        let update_index = global || sticky;
        if !update_index {
            index = 0;
        }
        let matcher = data.matcher.as_ref();
        let found = if index > string.len() as u64 {
            None
        } else {
            let matcher = matcher.ok_or_else(|| {
                Self::unsupported(span, "native regular expression matching for this Pattern")
            })?;
            let index = index as usize;
            let work = if sticky {
                data.source.len().min(string.len() - index)
            } else {
                string.len() - index
            };
            self.object_work(span, |_, budget| {
                if budget.remaining_work().is_some() {
                    // Matcher dispatch still consumes work for an empty suffix.
                    let passes = matcher.search_passes(sticky);
                    budget.charge(passes)?;
                    for _ in 0..passes {
                        budget.charge(work)?;
                        budget.charge(work)?;
                    }
                }
                Ok(())
            })?;
            matcher.find(string, index, sticky)
        };
        let Some(found) = found else {
            if update_index {
                self.set_property_or_throw(&object, last_index, Value::Number(0.0), span)?;
            }
            return Ok(Value::Null);
        };
        if update_index {
            self.set_property_or_throw(
                &object,
                last_index,
                Value::Number(found.range.end as f64),
                span,
            )?;
        }
        self.object_work(span, |_, budget| budget.charge(found.capture_count))?;
        let length = found.capture_count as u64 + 1;
        let array = self.create_intrinsic_array(length, span)?;
        self.regexp_match_property(
            &array,
            "index",
            Value::Number(found.range.start as f64),
            span,
        )?;
        self.regexp_match_property(&array, "input", Value::String(string.clone()), span)?;
        let matched = self.regexp_substring(&string.code_units()[found.range.clone()], span)?;
        self.create_array_element(&array, 0, matched, span)?;
        self.regexp_match_property(&array, "groups", Value::Undefined, span)?;
        // Every source-order group has an own element. A group in an unselected
        // alternative is undefined, distinct from a participating empty capture.
        for index in 0..found.capture_count {
            let value = if let Some(capture) = found.capture(index) {
                self.regexp_substring(&string.code_units()[capture], span)?
            } else {
                Value::Undefined
            };
            self.create_array_element(&array, index as u64 + 1, value, span)?;
        }
        if has_indices {
            let indices = self.create_intrinsic_array(length, span)?;
            self.regexp_match_property(&indices, "groups", Value::Undefined, span)?;
            let ranges = std::iter::once(Some(found.range.clone()))
                .chain((0..found.capture_count).map(|index| found.capture(index)));
            for (index, range) in ranges.enumerate() {
                let value = if let Some(range) = range {
                    let pair = self.create_intrinsic_array(2, span)?;
                    self.create_array_element(&pair, 0, Value::Number(range.start as f64), span)?;
                    self.create_array_element(&pair, 1, Value::Number(range.end as f64), span)?;
                    Value::Object(pair)
                } else {
                    Value::Undefined
                };
                self.create_array_element(&indices, index as u64, value, span)?;
            }
            self.regexp_match_property(&array, "indices", Value::Object(indices), span)?;
        }
        Ok(Value::Object(array))
    }

    fn regexp_match_property(
        &mut self,
        object: &crate::ObjectHandle,
        key: &str,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.define_property_or_throw(
            object,
            JsString::from(key),
            crate::object::DataDescriptor {
                value: Some(value),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )
    }
}
