//! Native original-slot getters and the explicit matching boundary (22.2.6).

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
        self.regexp_require_data(receiver, span)?;
        self.string(argument, span)?;
        Err(Self::unsupported(
            span,
            "native regular expression matching",
        ))
    }

    pub(super) fn regexp_builtin_exec(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // RegExpExec already converted its String and checked the live exec
        // property. Its non-callable fallback still requires the native brand.
        self.regexp_require_data(receiver, span)?;
        Err(Self::unsupported(
            span,
            "native regular expression matching",
        ))
    }
}
