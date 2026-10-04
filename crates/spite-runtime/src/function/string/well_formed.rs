//! Unicode well-formedness and replacement (22.1.3.10/31, 7.2.7).

use crate::{Error, Realm, Value};
use spite_core::{JsString, Span};

impl Realm {
    pub(crate) fn string_well_formed(
        &mut self,
        receiver: Value,
        replace: bool,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        let mut decoded = char::decode_utf16(string.code_units().iter().copied());
        if !replace {
            return Ok(Value::Boolean(decoded.all(|point| point.is_ok())));
        }
        // Each replacement occupies exactly one UTF-16 unit. Valid pairs retain
        // both units, so output length equals input length even for ill-formed S.
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| string.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        let mut result = Vec::with_capacity(string.len());
        for point in decoded {
            let point = point.unwrap_or(char::REPLACEMENT_CHARACTER);
            result.extend_from_slice(point.encode_utf16(&mut [0; 2]));
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }
}
