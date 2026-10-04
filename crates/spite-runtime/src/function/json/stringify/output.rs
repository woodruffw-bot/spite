//! QuoteJSONString and fallible UTF-16 output growth.

use super::capacity_error;
use crate::{Error, Realm};
use spite_core::{JsString, Span};

impl Realm {
    pub(super) fn json_append(
        &mut self,
        output: &mut Vec<u16>,
        units: &[u16],
        span: Span,
    ) -> Result<(), Error> {
        self.object_work(span, |_, budget| budget.charge(units.len()))?;
        let length = output
            .len()
            .checked_add(units.len())
            .ok_or_else(|| capacity_error(span))?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(Error::Limit {
                span,
                message: "JSON output exceeds host string limit".into(),
            });
        }
        output
            .try_reserve(units.len())
            .map_err(|_| capacity_error(span))?;
        output.extend_from_slice(units);
        Ok(())
    }

    pub(super) fn json_quote(
        &mut self,
        output: &mut Vec<u16>,
        string: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        self.json_append(output, &[0x22], span)?;
        let units = string.code_units();
        let mut index = 0;
        while index < units.len() {
            let unit = units[index];
            index += 1;
            let escape = match unit {
                0x08 => Some(0x62),
                0x09 => Some(0x74),
                0x0a => Some(0x6e),
                0x0c => Some(0x66),
                0x0d => Some(0x72),
                0x22 | 0x5c => Some(unit),
                _ => None,
            };
            if let Some(escape) = escape {
                self.json_append(output, &[0x5c, escape], span)?;
            } else if (0xd800..=0xdbff).contains(&unit)
                && units
                    .get(index)
                    .is_some_and(|next| (0xdc00..=0xdfff).contains(next))
            {
                self.json_append(output, &[unit, units[index]], span)?;
                index += 1;
            } else if unit < 0x20 || (0xd800..=0xdfff).contains(&unit) {
                let mut escape = [0x5c, 0x75, 0, 0, 0, 0];
                for (digit, shift) in escape[2..].iter_mut().zip([12, 8, 4, 0]) {
                    let value = (unit >> shift) & 15;
                    *digit = if value < 10 {
                        0x30 + value
                    } else {
                        0x61 + value - 10
                    };
                }
                self.json_append(output, &escape, span)?;
            } else {
                self.json_append(output, &[unit], span)?;
            }
        }
        self.json_append(output, &[0x22], span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spite_parser::json::{JsonKind, parse_json};

    #[test]
    fn every_utf16_unit_roundtrips_and_quote_escapes_are_lowercase() {
        let original = JsString::from_code_units((0..=u16::MAX).collect());
        let mut output = Vec::new();
        Realm::default()
            .json_quote(&mut output, &original, Span::new(0, 0))
            .unwrap();
        let quoted = JsString::from_code_units(output);
        let parsed = parse_json(&quoted).unwrap();
        assert_eq!(parsed.nodes[parsed.root].kind, JsonKind::String(original));
        let mut output = Vec::new();
        Realm::default()
            .json_quote(
                &mut output,
                &JsString::from_code_units(vec![0x0b, 0xdabc, 0xdcab]),
                Span::new(0, 0),
            )
            .unwrap();
        // A paired surrogate is preserved; only unpaired units are escaped.
        assert_eq!(
            output,
            vec![
                0x22, 0x5c, 0x75, 0x30, 0x30, 0x30, 0x62, 0xdabc, 0xdcab, 0x22
            ]
        );
        let mut output = Vec::new();
        Realm::default()
            .json_quote(
                &mut output,
                &JsString::from_code_units(vec![0xdabc, 0x78, 0xdcab]),
                Span::new(0, 0),
            )
            .unwrap();
        assert_eq!(
            JsString::from_code_units(output).to_utf8().unwrap(),
            r#""\udabcx\udcab""#
        );
    }
}
