//! Exact UnicodePropertyValueExpression grammar and property-name early errors.

use super::{Failure, Pattern, property_data, syntax};

impl Pattern {
    pub(super) fn property_escape(&mut self, negated: bool) -> Result<bool, Failure> {
        // The enclosing p/P is consumed. No loose matching, escapes, whitespace
        // or Unicode identifiers are permitted in the property expression.
        // https://262.ecma-international.org/17.0/#sec-patterns
        // https://262.ecma-international.org/17.0/#sec-patterns-static-semantics-early-errors
        if !self.eat(b'{') {
            return Err(syntax(
                "regular expression Unicode property requires braces",
            ));
        }
        let name = self.property_word()?;
        if self.eat(b'=') {
            if name.bytes().any(|byte| byte.is_ascii_digit()) {
                return Err(syntax("invalid regular expression Unicode property name"));
            }
            let value = self.property_word()?;
            self.property_close()?;
            let values = match name.as_str() {
                "General_Category" | "gc" => property_data::GENERAL_CATEGORY,
                "Script" | "sc" | "Script_Extensions" | "scx" => property_data::SCRIPT,
                _ => return Err(syntax("unknown regular expression Unicode property name")),
            };
            if values.binary_search(&value.as_str()).is_err() {
                return Err(syntax("unknown regular expression Unicode property value"));
            }
            return Ok(false);
        }
        self.property_close()?;
        if property_data::GENERAL_CATEGORY
            .binary_search(&name.as_str())
            .is_ok()
            || property_data::BINARY.binary_search(&name.as_str()).is_ok()
        {
            return Ok(false);
        }
        if self.mode.sets && property_data::STRINGS.binary_search(&name.as_str()).is_ok() {
            if negated {
                return Err(syntax(
                    "regular expression string property cannot be negated",
                ));
            }
            // All specified string properties satisfy MayContainStrings, even
            // when some of their members happen to be individual characters.
            return Ok(true);
        }
        Err(syntax("unknown regular expression Unicode property"))
    }

    fn property_word(&mut self) -> Result<String, Failure> {
        let mut word = String::new();
        while let Some(point) = self.peek() {
            if !matches!(point, 0x41..=0x5a | 0x61..=0x7a | 0x30..=0x39 | 0x5f) {
                break;
            }
            // The ranges above contain only ASCII and therefore fit in u8.
            word.push(char::from(point as u8));
            self.pos += 1;
        }
        if word.is_empty() {
            return Err(syntax(
                "expected regular expression Unicode property identifier",
            ));
        }
        Ok(word)
    }

    fn property_close(&mut self) -> Result<(), Failure> {
        if !self.eat(b'}') {
            return Err(syntax(
                "invalid regular expression Unicode property expression",
            ));
        }
        Ok(())
    }
}
