//! UnicodeSetsMode class grammar with iterative nesting and string containment.

use super::{Failure, Pattern, syntax};

enum Operand {
    Character(u32),
    Set { may_contain_strings: bool },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetMode {
    Empty,
    Single,
    Union,
    Intersection,
    Subtraction,
}

struct Class {
    invert: bool,
    mode: SetMode,
    need_operand: bool,
    may_contain_strings: bool,
    last_character: Option<u32>,
    last_range: bool,
}

impl Class {
    fn new(invert: bool) -> Self {
        Self {
            invert,
            mode: SetMode::Empty,
            need_operand: true,
            may_contain_strings: false,
            last_character: None,
            last_range: false,
        }
    }

    fn operand(&mut self, operand: Operand) {
        let (character, strings) = match operand {
            Operand::Character(character) => (Some(character), false),
            Operand::Set {
                may_contain_strings,
            } => (None, may_contain_strings),
        };
        // MayContainStrings is union OR, intersection AND, subtraction's left
        // value. It is conservative syntax analysis, independent of matching.
        // https://262.ecma-international.org/17.0/#sec-static-semantics-maycontainstrings
        match self.mode {
            SetMode::Empty => {
                self.mode = SetMode::Single;
                self.may_contain_strings = strings;
            }
            SetMode::Single | SetMode::Union => self.may_contain_strings |= strings,
            SetMode::Intersection => self.may_contain_strings &= strings,
            SetMode::Subtraction => {}
        }
        self.last_character = character;
        self.last_range = false;
        self.need_operand = false;
    }
}

impl Pattern {
    pub(super) fn unicode_sets_class(&mut self) -> Result<(), Failure> {
        // https://262.ecma-international.org/17.0/#sec-patterns
        // The enclosing '[' is consumed. Each frame retains the enclosing
        // expression while a NestedClass operand is scanned, without recursion.
        let mut current = Class::new(self.eat(b'^'));
        let mut parents = Vec::new();
        loop {
            if self.peek().is_none() {
                return Err(syntax("unterminated regular expression Unicode class"));
            }
            if self.eat(b']') {
                if current.need_operand && current.mode != SetMode::Empty {
                    return Err(syntax(
                        "regular expression Unicode class operator requires an operand",
                    ));
                }
                if current.invert && current.may_contain_strings {
                    return Err(syntax(
                        "negated regular expression class may contain strings",
                    ));
                }
                let strings = !current.invert && current.may_contain_strings;
                let Some(parent) = parents.pop() else {
                    return Ok(());
                };
                current = parent;
                current.operand(Operand::Set {
                    may_contain_strings: strings,
                });
                continue;
            }
            if current.need_operand {
                if self.eat(b'[') {
                    parents.push(current);
                    current = Class::new(self.eat(b'^'));
                } else {
                    current.operand(self.set_operand()?);
                }
                continue;
            }
            let operator = if self.points[self.pos..].starts_with(&[0x26, 0x26]) {
                // ClassIntersection explicitly excludes a third unescaped '&'.
                if self.points.get(self.pos + 2) == Some(&0x26) {
                    return Err(syntax(
                        "invalid regular expression Unicode class intersection",
                    ));
                }
                Some(SetMode::Intersection)
            } else if self.points[self.pos..].starts_with(&[0x2d, 0x2d]) {
                Some(SetMode::Subtraction)
            } else {
                None
            };
            if let Some(operator) = operator {
                if current.last_range
                    || (current.mode != SetMode::Single && current.mode != operator)
                {
                    return Err(syntax(
                        "regular expression Unicode class cannot mix unions, ranges and set operators",
                    ));
                }
                self.pos += 2;
                current.mode = operator;
                current.need_operand = true;
                continue;
            }
            if self.eat(b'-') {
                let Some(left) = current.last_character else {
                    return Err(syntax(
                        "regular expression Unicode class range requires character endpoints",
                    ));
                };
                if matches!(current.mode, SetMode::Intersection | SetMode::Subtraction) {
                    return Err(syntax(
                        "regular expression Unicode class cannot mix unions, ranges and set operators",
                    ));
                }
                // A NestedClass is an operand, never a ClassSetCharacter.
                if self.peek() == Some(0x5b) {
                    return Err(syntax(
                        "regular expression Unicode class range requires character endpoints",
                    ));
                }
                let Operand::Character(right) = self.set_operand()? else {
                    return Err(syntax(
                        "regular expression Unicode class range requires character endpoints",
                    ));
                };
                if left > right {
                    return Err(syntax("regular expression Unicode class range is reversed"));
                }
                current.last_character = None;
                current.last_range = true;
                continue;
            }
            match current.mode {
                SetMode::Single => current.mode = SetMode::Union,
                SetMode::Union => {}
                _ => {
                    return Err(syntax(
                        "regular expression Unicode class cannot mix unions, ranges and set operators",
                    ));
                }
            }
            current.need_operand = true;
        }
    }

    fn set_operand(&mut self) -> Result<Operand, Failure> {
        if self.peek() == Some(0x5c) {
            match self.points.get(self.pos + 1) {
                Some(0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57) => {
                    self.pos += 2;
                    return Ok(Operand::Set {
                        may_contain_strings: false,
                    });
                }
                Some(0x70 | 0x50) => {
                    let negated = self.points[self.pos + 1] == 0x50;
                    self.pos += 2;
                    return self
                        .property_escape(negated)
                        .map(|may_contain_strings| Operand::Set {
                            may_contain_strings,
                        });
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
