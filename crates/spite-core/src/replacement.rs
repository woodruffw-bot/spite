//! Allocation-free GetSubstitution template scanning (22.1.3.19.1).

use crate::JsString;

/// One literal or reference in an ECMAScript replacement template.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplacementPart<'a> {
    /// Literal UTF-16 units, including unrecognized reference text.
    Literal(&'a [u16]),
    /// The complete matched String, referenced by `$&`.
    Matched,
    /// The input preceding the match, referenced by the dollar/backtick sequence.
    Prefix,
    /// The input following the match, referenced by `$'`.
    Suffix,
    /// A numbered capture, with a zero-based index into the captures List.
    Capture(usize),
    /// A named capture key, to be read and converted when this part is consumed.
    NamedCapture(&'a [u16]),
}

/// Scan a replacement template without allocating or rewriting UTF-16 units.
///
/// Capture references depend on the number of captures and whether a named
/// captures Object is present. This scanner performs no property reads or
/// conversions. Consumers resolve references in order and never rescan their
/// replacement text. Scanning takes linear work, including unterminated named
/// references; consumers can charge at most two scans of the template's units.
pub fn replacement_parts(
    template: &JsString,
    capture_count: usize,
    named_captures: bool,
) -> impl Iterator<Item = ReplacementPart<'_>> + Clone + '_ {
    Parts {
        units: template.code_units(),
        cursor: 0,
        capture_count,
        named_captures,
        next_gt: 0,
    }
}

#[derive(Clone)]
struct Parts<'a> {
    units: &'a [u16],
    cursor: usize,
    capture_count: usize,
    named_captures: bool,
    // The next greater-than position, or usize::MAX when none remains.
    next_gt: usize,
}

impl<'a> Iterator for Parts<'a> {
    type Item = ReplacementPart<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.cursor;
        if start == self.units.len() {
            return None;
        }
        let dollar = self.units[start..]
            .iter()
            .position(|&unit| unit == u16::from(b'$'))
            .map_or(self.units.len(), |offset| start + offset);
        if dollar != start {
            self.cursor = dollar;
            return Some(ReplacementPart::Literal(&self.units[start..dollar]));
        }
        let next = self.units.get(start + 1).copied();
        if let Some(unit) = next {
            match unit {
                0x24 => {
                    self.cursor = start + 2;
                    return Some(ReplacementPart::Literal(&self.units[start..start + 1]));
                }
                0x26 => {
                    self.cursor = start + 2;
                    return Some(ReplacementPart::Matched);
                }
                0x60 => {
                    self.cursor = start + 2;
                    return Some(ReplacementPart::Prefix);
                }
                0x27 => {
                    self.cursor = start + 2;
                    return Some(ReplacementPart::Suffix);
                }
                0x30..=0x39 => {
                    let first = usize::from(unit - 0x30);
                    let mut index = first;
                    self.cursor = start + 2;
                    if let Some(&second @ 0x30..=0x39) = self.units.get(self.cursor) {
                        index = first * 10 + usize::from(second - 0x30);
                        self.cursor += 1;
                        if index > self.capture_count {
                            index = first;
                            self.cursor -= 1;
                        }
                    }
                    return Some(if index != 0 && index <= self.capture_count {
                        ReplacementPart::Capture(index - 1)
                    } else {
                        ReplacementPart::Literal(&self.units[start..self.cursor])
                    });
                }
                0x3c => {
                    self.cursor = start + 2;
                    if self.named_captures {
                        if self.next_gt < self.cursor {
                            self.next_gt = self.units[self.cursor..]
                                .iter()
                                .position(|&unit| unit == u16::from(b'>'))
                                .map_or(usize::MAX, |offset| self.cursor + offset);
                        }
                        if self.next_gt < self.units.len() {
                            let key = &self.units[self.cursor..self.next_gt];
                            self.cursor = self.next_gt + 1;
                            return Some(ReplacementPart::NamedCapture(key));
                        }
                    }
                    return Some(ReplacementPart::Literal(&self.units[start..self.cursor]));
                }
                _ => {}
            }
        }
        self.cursor = start + 1;
        Some(ReplacementPart::Literal(&self.units[start..self.cursor]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn describe(template: &JsString, captures: usize, named: bool) -> String {
        let mut text = String::new();
        for part in replacement_parts(template, captures, named) {
            if !text.is_empty() {
                text.push_str(" | ");
            }
            match part {
                ReplacementPart::Literal(units) => {
                    write!(
                        text,
                        "text({:?})",
                        JsString::from_code_units(units.to_vec())
                    )
                    .unwrap();
                }
                ReplacementPart::Matched => text.push_str("match"),
                ReplacementPart::Prefix => text.push_str("prefix"),
                ReplacementPart::Suffix => text.push_str("suffix"),
                ReplacementPart::Capture(index) => write!(text, "capture[{}]", index + 1).unwrap(),
                ReplacementPart::NamedCapture(units) => {
                    write!(
                        text,
                        "named({:?})",
                        JsString::from_code_units(units.to_vec())
                    )
                    .unwrap();
                }
            }
        }
        if text.is_empty() {
            "<empty>".into()
        } else {
            text
        }
    }

    #[test]
    fn numbered_references_follow_decimal_fallback_without_rescanning_literals() {
        let mut snapshot = String::new();
        for captures in [0, 1, 3, 9, 10, 99] {
            for template in [
                "",
                "plain",
                "$$$&$`$'",
                "$0|$00|$01|$1|$03|$3|$04|$4|$09|$9|$10|$11|$99|$100",
                "$001$011$999",
                "$1x$10",
                "$z$",
                "$$1$$01",
            ] {
                let template = JsString::from(template);
                writeln!(
                    snapshot,
                    "captures={captures} template={template:?} => {}",
                    describe(&template, captures, false)
                )
                .unwrap();
            }
        }
        insta::assert_snapshot!(snapshot);
    }

    #[test]
    fn named_references_preserve_utf16_keys_and_stop_at_the_first_closing_delimiter() {
        let mut templates: Vec<JsString> = [
            "$<foo>",
            "$<>",
            "$<foo",
            "$<a$&>",
            "$<$<x>>",
            "$$<foo>$&",
            "$<foo>$<foo>",
            "$<bad$<bad",
            "$<$1>",
            "$<𐐀>",
        ]
        .into_iter()
        .map(JsString::from)
        .collect();
        templates.push(JsString::from_code_units(vec![
            0xd800, 0x24, 0x3c, 0xdc00, 0x3e,
        ]));
        let mut snapshot = String::new();
        for named in [false, true] {
            for template in &templates {
                writeln!(
                    snapshot,
                    "named={named} template={template:?} => {}",
                    describe(template, 1, named)
                )
                .unwrap();
            }
        }
        insta::assert_snapshot!(snapshot);
    }

    #[test]
    fn large_unterminated_templates_and_cloned_iterators_have_no_default_size_cap() {
        let template = JsString::from(("$<".repeat(120_000) + "$1").as_str());
        let mut parts = replacement_parts(&template, 1, true);
        assert_eq!(parts.next(), Some(ReplacementPart::Literal(&[0x24, 0x3c])));
        let mut clone = parts.clone();
        assert_eq!(clone.next(), parts.next());
        assert_eq!(parts.count(), 119_999);
        assert_eq!(
            replacement_parts(&template, 1, true).last(),
            Some(ReplacementPart::Capture(0))
        );
    }
}
