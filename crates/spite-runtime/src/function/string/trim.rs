//! TrimString and its three mandatory methods (22.1.3.32–34).

use super::Builtin;
use crate::{Error, Realm, Value};
use spite_core::{Span, is_line_terminator, is_whitespace};

impl Realm {
    pub(crate) fn string_trim(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        let mut units = string.code_units();
        if !matches!(builtin, Builtin::StringTrimEnd) {
            let start = units
                .iter()
                .position(|&unit| !is_space(unit))
                .unwrap_or(units.len());
            units = &units[start..];
        }
        if !matches!(builtin, Builtin::StringTrimStart) {
            let end = units
                .iter()
                .rposition(|&unit| !is_space(unit))
                .map_or(0, |last| last + 1);
            units = &units[..end];
        }
        self.copy_string_units(units, span)
    }
}

fn is_space(unit: u16) -> bool {
    // Every current WhiteSpace/LineTerminator code point is in the BMP.
    // Surrogates, whether paired or not, cannot form a trimmable code point.
    char::from_u32(u32::from(unit)).is_some_and(|ch| is_whitespace(ch) || is_line_terminator(ch))
}
