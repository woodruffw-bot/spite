//! Symbol primitive conversion; global APIs and hooks arrive separately.

use crate::{Error, Realm};
use spite_core::{JsString, JsSymbol, Span};

impl Realm {
    /// SymbolDescriptiveString (20.4.3.3.1), preserving UTF-16 description units.
    pub(crate) fn symbol_descriptive_string(
        &mut self,
        symbol: &JsSymbol,
        span: Span,
    ) -> Result<JsString, Error> {
        let limit = || Error::Limit {
            span,
            message: "Symbol description output limit exceeded".into(),
        };
        let description = symbol.description();
        let length = description
            .map_or(0, JsString::len)
            .checked_add(8)
            .ok_or_else(limit)?;
        if length > self.limits.max_string_units {
            return Err(limit());
        }
        self.object_work(span, |_, budget| budget.charge(length))?;
        let mut units = Vec::new();
        units.try_reserve_exact(length).map_err(|_| limit())?;
        units.extend("Symbol(".encode_utf16());
        if let Some(description) = description {
            units.extend_from_slice(description.code_units());
        }
        units.push(u16::from(b')'));
        Ok(JsString::from_code_units(units))
    }
}

#[cfg(test)]
mod tests;
