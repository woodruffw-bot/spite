//! String replace/replaceAll (22.1.3.19–20) and uncaptured GetSubstitution.

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

enum Replacement {
    Function(Value),
    Text(JsString),
}

impl Realm {
    pub(crate) fn string_replace(
        &mut self,
        receiver: Value,
        search: Value,
        replace: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        // Edition 17 ignores primitive prototype Symbol.replace hooks.
        if matches!(search, Value::Object(_)) {
            if let Some(method) =
                self.get_method(&search, &WellKnownSymbol::Replace.symbol(), span)?
            {
                return self.call(method, search, vec![receiver, replace], span);
            }
        }
        let string = self.string(receiver, span)?;
        let search = self.string(search, span)?;
        let replacement = if self.is_callable(&replace, span)? {
            Replacement::Function(replace)
        } else {
            // Convert non-callable replacements even when no match exists.
            Replacement::Text(self.string(replace, span)?)
        };
        let Some(position) = self.find_string(&string, &search, 0, false, span)? else {
            return Ok(Value::String(string));
        };
        // Functional replacement runs before result construction. Its returned
        // String is literal text, without dollar-pattern substitution.
        let (replacement, substitute) = match replacement {
            Replacement::Function(function) => (
                self.functional_replacement(&function, &string, &search, position, span)?,
                false,
            ),
            Replacement::Text(text) => (text, true),
        };
        let mut result = Vec::new();
        self.append_replacement_units(&mut result, &string.code_units()[..position], span)?;
        if substitute {
            self.append_uncaptured_substitution(
                &mut result,
                &string,
                &search,
                position,
                &replacement,
                span,
            )?;
        } else {
            self.append_replacement_units(&mut result, replacement.code_units(), span)?;
        }
        self.append_replacement_units(
            &mut result,
            &string.code_units()[position + search.len()..],
            span,
        )?;
        Ok(Value::String(JsString::from_code_units(result)))
    }

    pub(crate) fn string_replace_all(
        &mut self,
        receiver: Value,
        search: Value,
        replace: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        if let Value::Object(object) = &search {
            // 22.1.3.20: IsRegExp and the global flag check precede GetMethod,
            // even when a custom Symbol.replace hook would return immediately.
            if self.is_regexp(&search, span)? {
                let flags = self.get_property(object, &JsString::from("flags"), span)?;
                Self::require_object_coercible(&flags, span)?;
                let flags = self.string(flags, span)?;
                self.object_work(span, |_, budget| budget.charge(flags.len()))?;
                if !flags.code_units().contains(&u16::from(b'g')) {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "String.replaceAll requires a global regular expression",
                    ));
                }
            }
            if let Some(method) =
                self.get_method(&search, &WellKnownSymbol::Replace.symbol(), span)?
            {
                return self.call(method, search, vec![receiver, replace], span);
            }
        }
        // Edition 17 skips primitive prototype hooks, as for replace.
        let string = self.string(receiver, span)?;
        let search = self.string(search, span)?;
        let replacement = if self.is_callable(&replace, span)? {
            Replacement::Function(replace)
        } else {
            Replacement::Text(self.string(replace, span)?)
        };
        // Collect non-overlapping positions before running any callbacks. An
        // empty search includes every UTF-16 boundary, including the last one.
        let advance = search.len().max(1);
        let mut positions = Vec::new();
        let mut start = 0;
        while start <= string.len() {
            let Some(position) = self.find_string(&string, &search, start, false, span)? else {
                break;
            };
            self.tick(span)?;
            positions
                .try_reserve(1)
                .map_err(|_| replacement_limit(span))?;
            positions.push(position);
            if position == string.len() {
                break;
            }
            start = position
                .checked_add(advance)
                .ok_or_else(|| replacement_limit(span))?;
        }
        if positions.is_empty() {
            return Ok(Value::String(string));
        }
        let mut result = Vec::new();
        let mut end = 0;
        for position in positions {
            let prefix = &string.code_units()[end..position];
            match &replacement {
                Replacement::Function(function) => {
                    let text =
                        self.functional_replacement(function, &string, &search, position, span)?;
                    self.append_replacement_units(&mut result, prefix, span)?;
                    self.append_replacement_units(&mut result, text.code_units(), span)?;
                }
                Replacement::Text(text) => {
                    self.append_replacement_units(&mut result, prefix, span)?;
                    self.append_uncaptured_substitution(
                        &mut result,
                        &string,
                        &search,
                        position,
                        text,
                        span,
                    )?;
                }
            }
            end = position + search.len();
        }
        self.append_replacement_units(&mut result, &string.code_units()[end..], span)?;
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn functional_replacement(
        &mut self,
        function: &Value,
        string: &JsString,
        search: &JsString,
        position: usize,
        span: Span,
    ) -> Result<JsString, Error> {
        self.object_work(span, |_, budget| {
            budget.value(function)?;
            budget.charge(search.len())?;
            budget.charge(string.len())
        })?;
        let value = self.call(
            function.clone(),
            Value::Undefined,
            vec![
                Value::String(search.clone()),
                Value::Number(position as f64),
                Value::String(string.clone()),
            ],
            span,
        )?;
        self.string(value, span)
    }

    fn append_uncaptured_substitution(
        &mut self,
        result: &mut Vec<u16>,
        string: &JsString,
        matched: &JsString,
        position: usize,
        replacement: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        let units = replacement.code_units();
        // Every code unit participates in at most one scan. Copy charges below
        // separately cover expanded prefixes/suffixes and literal output.
        self.object_work(span, |_, budget| budget.charge(units.len()))?;
        let mut cursor = 0;
        while cursor < units.len() {
            let Some(offset) = units[cursor..].iter().position(|&u| u == u16::from(b'$')) else {
                return self.append_replacement_units(result, &units[cursor..], span);
            };
            let dollar = cursor + offset;
            self.append_replacement_units(result, &units[cursor..dollar], span)?;
            let part = match units.get(dollar + 1).copied() {
                Some(u) if u == u16::from(b'$') => Some(&units[dollar..dollar + 1]),
                Some(u) if u == u16::from(b'&') => Some(matched.code_units()),
                Some(u) if u == u16::from(b'`') => Some(&string.code_units()[..position]),
                Some(u) if u == u16::from(b'\'') => {
                    Some(&string.code_units()[position + matched.len()..])
                }
                _ => None,
            };
            if let Some(part) = part {
                self.append_replacement_units(result, part, span)?;
                cursor = dollar + 2;
            } else {
                // With no captures or named captures, $n/$nn/$<name> and all
                // unrecognized dollar sequences remain literal text.
                self.append_replacement_units(result, &units[dollar..dollar + 1], span)?;
                cursor = dollar + 1;
            }
        }
        Ok(())
    }

    fn append_replacement_units(
        &mut self,
        result: &mut Vec<u16>,
        part: &[u16],
        span: Span,
    ) -> Result<(), Error> {
        let length = result
            .len()
            .checked_add(part.len())
            .ok_or_else(|| replacement_limit(span))?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(replacement_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(part.len()))?;
        result
            .try_reserve(part.len())
            .map_err(|_| replacement_limit(span))?;
        result.extend_from_slice(part);
        Ok(())
    }
}

fn replacement_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "replacement output exceeds host or platform capacity".into(),
    }
}
