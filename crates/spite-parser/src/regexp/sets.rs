//! Flat ClassUnion and ClassStringDisjunction grammar in UnicodeSetsMode.

use super::{Failure, Pattern, syntax, unsupported};

enum Operand {
    Character(u32),
    Set { may_contain_strings: bool },
}

impl Pattern {
    pub(super) fn unicode_sets_class(&mut self) -> Result<(), Failure> {
        // https://262.ecma-international.org/17.0/#sec-patterns
        // The enclosing '[' is consumed. This step supports flat ClassUnion;
        // nested operands and ClassIntersection/Subtraction remain explicit gaps.
        let invert = self.eat(b'^');
        let mut may_contain_strings = false;
        while !self.eat(b']') {
            self.set_operator_gap()?;
            let left = self.set_operand()?;
            self.set_operator_gap()?;
            if self.eat(b'-') {
                let right = self.set_operand()?;
                let (Operand::Character(left), Operand::Character(right)) = (left, right) else {
                    return Err(syntax(
                        "regular expression Unicode class range requires character endpoints",
                    ));
                };
                if left > right {
                    return Err(syntax("regular expression Unicode class range is reversed"));
                }
            } else if let Operand::Set {
                may_contain_strings: strings,
            } = left
            {
                may_contain_strings |= strings;
            }
        }
        // MayContainStrings for a union is true if any operand may contain
        // strings. Empty ClassContents and character ranges contribute false.
        // https://262.ecma-international.org/17.0/#sec-static-semantics-maycontainstrings
        // https://262.ecma-international.org/17.0/#sec-patterns-static-semantics-early-errors
        if invert && may_contain_strings {
            return Err(syntax(
                "negated regular expression class may contain strings",
            ));
        }
        Ok(())
    }

    fn set_operator_gap(&self) -> Result<(), Failure> {
        if self.points[self.pos..].starts_with(&[0x26, 0x26])
            || self.points[self.pos..].starts_with(&[0x2d, 0x2d])
        {
            return Err(unsupported(
                "regular expression Unicode class set operators are not implemented",
            ));
        }
        Ok(())
    }

    fn set_operand(&mut self) -> Result<Operand, Failure> {
        if self.peek() == Some(0x5b) {
            return Err(unsupported(
                "regular expression nested Unicode class validation is not implemented",
            ));
        }
        if self.peek() == Some(0x5c) {
            match self.points.get(self.pos + 1) {
                Some(0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57) => {
                    self.pos += 2;
                    return Ok(Operand::Set {
                        may_contain_strings: false,
                    });
                }
                Some(0x70 | 0x50) => {
                    return Err(unsupported(
                        "regular expression Unicode property validation is not implemented",
                    ));
                }
                Some(0x71) => {
                    self.pos += 2;
                    return self.class_string_disjunction().map(|may_contain_strings| {
                        Operand::Set {
                            may_contain_strings,
                        }
                    });
                }
                _ => {}
            }
        }
        self.class_set_character().map(Operand::Character)
    }

    fn class_string_disjunction(&mut self) -> Result<bool, Failure> {
        if !self.eat(b'{') {
            return Err(syntax(
                "regular expression class string disjunction requires braces",
            ));
        }
        let mut may_contain_strings = false;
        // MayContainStrings needs only the distinction between zero, one and
        // multiple characters; long strings retain no machine-integer bound.
        let mut length = 0u8;
        loop {
            match self.peek() {
                None => {
                    return Err(syntax(
                        "unterminated regular expression class string disjunction",
                    ));
                }
                Some(0x7d) => {
                    // }
                    self.pos += 1;
                    return Ok(may_contain_strings || length != 1);
                }
                Some(0x7c) => {
                    // |
                    self.pos += 1;
                    may_contain_strings |= length != 1;
                    length = 0;
                }
                _ => {
                    self.class_set_character()?;
                    length = (length + 1).min(2);
                }
            }
        }
    }

    fn class_set_character(&mut self) -> Result<u32, Failure> {
        let Some(point) = self.peek() else {
            return Err(syntax(
                "expected regular expression Unicode class character",
            ));
        };
        self.pos += 1;
        if point == 0x5c {
            // \
            let Some(escape) = self.peek() else {
                return Err(syntax(
                    "unterminated regular expression Unicode class escape",
                ));
            };
            self.pos += 1;
            if escape == 0x62 {
                // b is backspace, not an assertion.
                return Ok(8);
            }
            // ClassSetReservedPunctuator permits these escapes in v classes,
            // including inside strings, beyond CharacterEscape[+UnicodeMode].
            if b"&-!#%,:;<=>@`~"
                .iter()
                .any(|byte| u32::from(*byte) == escape)
            {
                return Ok(escape);
            }
            return self.character_escape(escape);
        }
        if b"()[]{}/-|".iter().any(|byte| u32::from(*byte) == point) {
            return Err(syntax(
                "unescaped regular expression Unicode class syntax character",
            ));
        }
        if self.peek() == Some(point)
            && b"&!#$%*+,.:;<=>?@^`~"
                .iter()
                .any(|byte| u32::from(*byte) == point)
        {
            return Err(syntax(
                "reserved double punctuation in regular expression Unicode class",
            ));
        }
        Ok(point)
    }
}
