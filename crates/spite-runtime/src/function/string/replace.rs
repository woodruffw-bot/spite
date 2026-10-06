//! String replace/replaceAll (22.1.3.19–20) and uncaptured GetSubstitution.

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, ReplacementPart, Span, WellKnownSymbol, replacement_parts};

enum Replacement {
    Function(Value),
    Text(JsString),
}

pub(crate) struct Substitution<'a> {
    pub string: &'a JsString,
    pub matched: &'a JsString,
    pub position: usize,
    // Elements are already converted Strings or undefined.
    pub captures: &'a [Value],
    pub named: Option<&'a ObjectHandle>,
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
        self.append_substitution(
            result,
            Substitution {
                string,
                matched,
                position,
                captures: &[],
                named: None,
            },
            replacement,
            span,
        )
    }

    // GetSubstitution, 22.1.3.19.1. Named reads and String conversions happen
    // as each reference is consumed; expansion text is never scanned again.
    pub(crate) fn append_substitution(
        &mut self,
        result: &mut Vec<u16>,
        context: Substitution<'_>,
        replacement: &JsString,
        span: Span,
    ) -> Result<(), Error> {
        let units = replacement.code_units();
        self.object_work(span, |_, budget| {
            budget.charge(units.len())?;
            if context.named.is_some() {
                budget.charge(units.len())?;
            }
            Ok(())
        })?;
        for part in replacement_parts(replacement, context.captures.len(), context.named.is_some())
        {
            match part {
                ReplacementPart::Literal(units) => {
                    self.append_replacement_units(result, units, span)?
                }
                ReplacementPart::Matched => {
                    self.append_replacement_units(result, context.matched.code_units(), span)?
                }
                ReplacementPart::Prefix => self.append_replacement_units(
                    result,
                    &context.string.code_units()[..context.position],
                    span,
                )?,
                ReplacementPart::Suffix => {
                    // Custom exec can supply a match extending past the input.
                    let tail = context
                        .position
                        .saturating_add(context.matched.len())
                        .min(context.string.len());
                    self.append_replacement_units(
                        result,
                        &context.string.code_units()[tail..],
                        span,
                    )?;
                }
                ReplacementPart::Capture(index) => match &context.captures[index] {
                    Value::String(string) => {
                        self.append_replacement_units(result, string.code_units(), span)?
                    }
                    Value::Undefined => {}
                    _ => unreachable!("captures were converted before substitution"),
                },
                ReplacementPart::NamedCapture(units) => {
                    let Value::String(key) = self.copy_string_units(units, span)? else {
                        unreachable!("String copy");
                    };
                    let capture =
                        self.get_property(context.named.expect("named reference"), &key, span)?;
                    if !matches!(capture, Value::Undefined) {
                        let capture = self.string(capture, span)?;
                        self.append_replacement_units(result, capture.code_units(), span)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn append_replacement_units(
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
