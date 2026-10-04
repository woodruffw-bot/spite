//! Unicode default conversion and the locale-neutral host fallback (22.1.3.29–31, 35).

use crate::{Error, Realm, Value};
use spite_core::{
    JsString, Span, is_unicode_case_ignorable, is_unicode_cased, unicode_case_mapping,
};

impl Realm {
    pub(crate) fn string_case(
        &mut self,
        receiver: Value,
        uppercase: bool,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        let mut decoded = char::decode_utf16(string.code_units().iter().copied());
        let mut result = Vec::new();
        let mut preceding_cased = false;
        while let Some(point) = decoded.next() {
            self.tick(span)?;
            let point = match point {
                Ok(point) => point,
                Err(error) => {
                    self.append_case_units(&mut result, &[error.unpaired_surrogate()], span)?;
                    preceding_cased = false;
                    continue;
                }
            };
            // Unicode 3.13 Final_Sigma: scan the original text, skipping all
            // Case_Ignorable points even when they also have the Cased property.
            let final_sigma = !uppercase
                && point == '\u{03a3}'
                && preceding_cased
                && !self.following_cased(decoded.clone(), span)?;
            if final_sigma {
                self.append_case_units(&mut result, &[0x03c2], span)?;
            } else if let Some(mapping) = unicode_case_mapping(point, uppercase) {
                self.append_case_units(&mut result, mapping, span)?;
            } else {
                self.append_case_units(&mut result, point.encode_utf16(&mut [0; 2]), span)?;
            }
            if !uppercase && !is_unicode_case_ignorable(point) {
                preceding_cased = is_unicode_cased(point);
            }
        }
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn following_cased(
        &mut self,
        decoded: impl Iterator<Item = Result<char, std::char::DecodeUtf16Error>>,
        span: Span,
    ) -> Result<bool, Error> {
        for point in decoded {
            self.tick(span)?;
            let Ok(point) = point else { return Ok(false) };
            if !is_unicode_case_ignorable(point) {
                return Ok(is_unicode_cased(point));
            }
        }
        Ok(false)
    }

    fn append_case_units(
        &mut self,
        result: &mut Vec<u16>,
        units: &[u16],
        span: Span,
    ) -> Result<(), Error> {
        let limit = || Error::Limit {
            span,
            message: "case conversion output exceeds string limit or platform capacity".into(),
        };
        let length = result.len().checked_add(units.len()).ok_or_else(limit)?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(limit());
        }
        self.object_work(span, |_, budget| budget.charge(units.len()))?;
        result.try_reserve(units.len()).map_err(|_| limit())?;
        result.extend_from_slice(units);
        Ok(())
    }
}
