//! RegExp allocation and initialization (22.2.3.1–3, 22.2.4.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, RegExpData, RegExpMatcher, RegExpMatcherBody, RegExpNamedGroup},
};
use spite_core::{
    DiagnosticKind, JsString, RegExpAnchoredMatcher, RegExpBackreferenceMatcher,
    RegExpCharacterMatcher, RegExpDisjunctionMatcher, RegExpImpossibleMatcher,
    RegExpLiteralMatcher, RegExpPrefixedMatcher, RegExpQuantifiedContinuationMatcher,
    RegExpQuantifiedMatcher, RegExpRepeatedCaptureMatcher, RegExpRepeatedContinuationMatcher,
    RegExpRepeatedLiteralMatcher, RegExpRepeatedPrefixedMatcher, RegExpRepeatedSequenceMatcher,
    RegExpSequenceMatcher, RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher,
    RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedNullableStringClassCaptureMatcher,
    RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher,
    RegExpUnicodeAssertedStringClassAlternativeMatcher,
    RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher,
    RegExpUnicodeBoundedStringClassCaptureMatcher, RegExpUnicodeCharacterMatcher,
    RegExpUnicodeClassLiteralClassMatcher, RegExpUnicodeClassLiteralClassWordMatcher,
    RegExpUnicodeClassLiteralMatcher, RegExpUnicodeClassMatcher,
    RegExpUnicodeClassPairLiteralMatcher, RegExpUnicodeClassPairLiteralWordMatcher,
    RegExpUnicodeClassPairMatcher, RegExpUnicodeClassSequenceMatcher,
    RegExpUnicodeClassTripleMatcher, RegExpUnicodeClassTripleWordMatcher,
    RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher,
    RegExpUnicodeCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeEmptyStringClassCaptureMatcher, RegExpUnicodeFoldedLiteralMatcher,
    RegExpUnicodeLiteralClassPairMatcher, RegExpUnicodeLiteralClassPairWordMatcher,
    RegExpUnicodeNullableClassCaptureMatcher, RegExpUnicodeNullableStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher,
    RegExpUnicodePredicateChoiceStringClassAlternativeMatcher,
    RegExpUnicodePredicateChoiceStringClassCaptureMatcher,
    RegExpUnicodePrefixFreeStringClassCaptureMatcher, RegExpUnicodePropertyDotMatcher,
    RegExpUnicodeRepeatedAtomMatcher, RegExpUnicodeRepeatedClassCaptureMatcher,
    RegExpUnicodeRepeatedStringClassCaptureMatcher, RegExpUnicodeRequiredStringClassCaptureMatcher,
    RegExpUnicodeSetCompileError, RegExpUnicodeSingleStringClassCaptureMatcher,
    RegExpUnicodeStringSetMatcher, RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableCountedNullableStringClassCaptureMatcher,
    RegExpUnicodeTruthTableExactOneGroupedOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableExactOneGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableGroupedOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableGroupedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableNestedOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableNestedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableOptionalSpineStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableQuantifiedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableRootStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedGroupedOptionalSpineStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedGroupedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedNestedOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedNestedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedOptionalSpineStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedQuantifiedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableScopedZeroCountSpineStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableSingleStringClassCaptureMatcher,
    RegExpUnicodeTruthTableStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableStringClassCaptureMatcher,
    RegExpUnicodeTruthTableSurroundingOptionalStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableSurroundingStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableZeroCountRootStringClassAlternativeMatcher,
    RegExpUnicodeTruthTableZeroCountSpineStringClassAlternativeMatcher,
    RegExpUnicodeTwoRequiredStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher,
    RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroMinimumSingleStringClassCaptureMatcher,
    RegExpUnicodeUnassertedZeroMinimumStringClassAlternativeMatcher,
    RegExpUnicodeUnassertedZeroMinimumStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher,
    RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher,
    RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher,
    RegExpUnicodeZeroMinimumAssertedSingleStringClassCaptureMatcher,
    RegExpUnicodeZeroMinimumAssertedStringClassAlternativeMatcher,
    RegExpUnicodeZeroMinimumAssertedStringClassCaptureMatcher, RegExpZeroCountClassMatcher,
    RegExpZeroCountGroupMatcher, RegExpZeroCountSequenceMatcher, RegExpZeroCountSpineMatcher, Span,
    regexp_outer_group_body,
};
use spite_parser::parse_regexp_pattern;
use std::{collections::HashMap, sync::Arc};

impl Realm {
    #[inline(never)]
    pub(in crate::function) fn regexp_constructor(
        &mut self,
        new_target: Option<ObjectHandle>,
        pattern: Value,
        flags: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // IsRegExp's observable Symbol.match lookup precedes every other Get.
        let pattern_is_regexp = self.is_regexp(&pattern, span)?;
        let intrinsic = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .regexp
            .constructor
            .clone();
        if new_target.is_none() && pattern_is_regexp && matches!(flags, Value::Undefined) {
            let Value::Object(object) = &pattern else {
                unreachable!("IsRegExp requires Object")
            };
            let constructor = self.get_property(object, &JsString::from("constructor"), span)?;
            if constructor.same_value(&Value::Object(intrinsic.clone())) {
                return Ok(pattern);
            }
        }
        let original = if let Value::Object(object) = &pattern {
            self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.regexp_data().cloned())
            })?
        } else {
            None
        };
        let (pattern, flags) = if let Some(original) = original {
            // Native slots take precedence even if Symbol.match is false.
            let flags = if matches!(flags, Value::Undefined) {
                Value::String(original.flags)
            } else {
                flags
            };
            (Value::String(original.source), flags)
        } else if pattern_is_regexp {
            let Value::Object(object) = pattern else {
                unreachable!("IsRegExp requires Object")
            };
            let pattern = self.get_property(&object, &JsString::from("source"), span)?;
            let flags = if matches!(flags, Value::Undefined) {
                self.get_property(&object, &JsString::from("flags"), span)?
            } else {
                flags
            };
            (pattern, flags)
        } else {
            (pattern, flags)
        };
        let new_target = new_target.unwrap_or(intrinsic);
        self.regexp_allocate_initialize(new_target, pattern, flags, span)
    }

    // RegExpCreate (22.2.3.1) bypasses IsRegExp, call identity and original-slot
    // copying. Its Pattern argument is always passed directly to ToString.
    pub(crate) fn regexp_create(
        &mut self,
        pattern: Value,
        flags: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .regexp
            .constructor
            .clone();
        self.regexp_allocate_initialize(constructor, pattern, flags, span)
    }

    #[inline(never)]
    fn regexp_allocate_initialize(
        &mut self,
        new_target: ObjectHandle,
        pattern: Value,
        flags: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // RegExpAlloc precedes both ToString conversions. Constructor input's
        // public source/flags Gets complete before GetPrototypeFromConstructor.
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .regexp
                .prototype
                .clone()
        };
        let object = self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
        self.define_property_or_throw(
            &object,
            JsString::from("lastIndex"),
            DataDescriptor {
                value: None,
                writable: Some(true),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        let source = if matches!(pattern, Value::Undefined) {
            JsString::default()
        } else {
            self.string(pattern, span)?
        };
        let flags = if matches!(flags, Value::Undefined) {
            JsString::default()
        } else {
            self.string(flags, span)?
        };
        self.object_work(span, |_, budget| {
            budget.charge(source.len())?;
            budget.charge(flags.len())
        })?;
        let metadata = parse_regexp_pattern(&source, &flags, span).map_err(|diagnostic| {
            match diagnostic.kind {
                DiagnosticKind::Syntax => {
                    Self::exception(ExceptionKind::SyntaxError, span, diagnostic.message)
                }
                DiagnosticKind::Unsupported => Self::unsupported(span, diagnostic.message),
                DiagnosticKind::Limit => Error::Limit {
                    span,
                    message: diagnostic.message,
                },
            }
        })?;
        let captures = metadata.capture_count;
        let (capture_source, named_groups, named_references) =
            self.object_work(span, |_, budget| {
                if metadata.named_captures.is_empty() {
                    return Ok((
                        source.clone(),
                        Arc::<[RegExpNamedGroup]>::from([]),
                        Vec::new(),
                    ));
                }
                budget.charge(source.len())?;
                budget.charge(metadata.named_captures.len())?;
                budget.charge(metadata.named_captures.len())?;
                let mut groups = Vec::<RegExpNamedGroup>::new();
                let mut names = HashMap::new();
                for capture in &metadata.named_captures {
                    let slot = (capture.index - 1) as usize;
                    let group = *names.entry(capture.name.clone()).or_insert_with(|| {
                        let index = groups.len();
                        groups.push(RegExpNamedGroup {
                            name: capture.name.clone(),
                            slots: Vec::new(),
                        });
                        index
                    });
                    groups[group].slots.push(slot);
                }
                budget.charge(metadata.named_references.len())?;
                let mut references = metadata.named_references.into_iter().peekable();
                let mut named_references = Vec::new();
                let mut captures = metadata.named_captures.into_iter().peekable();
                let mut normalized = Vec::new();
                let mut start = 0;
                while captures.peek().is_some() || references.peek().is_some() {
                    let remove_capture = captures.peek().is_some_and(|capture| {
                        references.peek().is_none_or(|reference| {
                            capture.specifier.start < reference.escape.start
                        })
                    });
                    if remove_capture {
                        let capture = captures.next().expect("capture selected");
                        normalized.extend_from_slice(
                            &source.code_units()[start..capture.specifier.start],
                        );
                        start = capture.specifier.end;
                    } else {
                        let reference = references.next().expect("reference selected");
                        normalized
                            .extend_from_slice(&source.code_units()[start..reference.escape.start]);
                        start = reference.escape.end;
                        budget.charge(reference.name.len())?;
                        let group = names[&reference.name];
                        let begin = normalized.len();
                        normalized.extend_from_slice(&source.code_units()[reference.escape]);
                        named_references.push(spite_core::RegExpBackreferenceNamedReference {
                            escape: begin..normalized.len(),
                            group,
                        });
                    }
                }
                normalized.extend_from_slice(&source.code_units()[start..]);
                Ok((
                    JsString::from_code_units(normalized),
                    Arc::<[RegExpNamedGroup]>::from(groups),
                    named_references,
                ))
            })?;
        let mut enclosing_captures = 0;
        let unicode = flags
            .code_units()
            .iter()
            .any(|&unit| matches!(unit, 0x75 | 0x76));
        // References use the complete capture layout: removing an enclosing
        // group would change DecimalEscape numbering and self-reference state.
        let references = if unicode {
            None
        } else {
            self.object_work(span, |_, budget| {
                budget.charge(named_groups.len())?;
                let groups: Vec<&[usize]> = named_groups
                    .iter()
                    .map(|group| group.slots.as_slice())
                    .collect();
                RegExpBackreferenceMatcher::compile_with_named_bindings_and_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    flags.code_units().contains(&u16::from(b's')),
                    spite_core::RegExpBackreferenceNamedBindings {
                        groups: &groups,
                        references: &named_references,
                    },
                    |work| budget.charge(work),
                )
            })?
        };
        let matcher = if let Some(matcher) = references {
            Some(RegExpMatcherBody::Backreferences(matcher))
        } else if unicode {
            // Unicode literal plans prove atom and capture boundaries before
            // flattening. Empty and scalar plans use the approved leading
            // boundary for an initial offset inside a surrogate pair (22.2.7.2).
            self.object_work(span, |_, budget| {
                budget.charge(capture_source.len())?;
                budget.charge(capture_source.len())?;
                budget.charge(capture_source.len())?;
                let literal = if !flags.code_units().contains(&u16::from(b'i')) {
                    RegExpLiteralMatcher::compile_unicode_code_points(&capture_source)
                        .map(RegExpMatcherBody::UnicodeCodePoints)
                } else {
                    None
                };
                if literal.is_some() {
                    Ok(literal)
                } else {
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    if let Some(body) = RegExpLiteralMatcher::compile_unicode_empty(&capture_source)
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeEmpty(body)));
                    }
                    if let Some(body) = RegExpLiteralMatcher::compile_unicode_pure_empty_with_work(
                        &capture_source,
                        |work| budget.charge(work),
                    )? {
                        return Ok(Some(RegExpMatcherBody::UnicodeEmpty(body)));
                    }
                    if let Some(body) =
                        RegExpLiteralMatcher::compile_unicode_enclosing_pure_empty_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeEmpty(body)));
                    }
                    if let Some(body) =
                        RegExpLiteralMatcher::compile_unicode_mandatory_pure_empty_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeEmpty(body)));
                    }
                    if let Some(body) =
                        RegExpLiteralMatcher::compile_unicode_positive_pure_empty_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeEmpty(body)));
                    }
                    Ok(
                        RegExpLiteralMatcher::compile_unicode_pure_empty_disjunction_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )?
                        .map(RegExpMatcherBody::UnicodeEmpty),
                    )
                }
            })?
        } else {
            let matching_source = if capture_source.code_units().first() == Some(&0x28) {
                let (body, captures) = self.object_work(span, |_, budget| {
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    if let Some(group) = regexp_outer_group_body(&capture_source) {
                        budget.charge(group.body.len())?;
                        Ok((
                            JsString::from_code_units(
                                capture_source.code_units()[group.body].to_vec(),
                            ),
                            group.captures,
                        ))
                    } else {
                        Ok((capture_source.clone(), 0))
                    }
                })?;
                enclosing_captures = captures;
                body
            } else {
                capture_source.clone()
            };
            self.object_work(span, |_, budget| {
                budget.charge(source.len())?;
                budget.charge(source.len())
            })?;
            let ignore_case = flags.code_units().contains(&u16::from(b'i'));
            if let Some(matcher) = RegExpLiteralMatcher::compile(&matching_source, ignore_case) {
                Some(RegExpMatcherBody::Literal(matcher))
            } else {
                // Cover the top-level scan and remaining per-branch compilation
                // passes. Accounting remains optional, as for literal plans.
                self.object_work(span, |_, budget| {
                    budget.charge(source.len())?;
                    budget.charge(source.len())
                })?;
                let disjunction = self.object_work(span, |_, budget| {
                    RegExpDisjunctionMatcher::compile_with_work(
                        &matching_source,
                        ignore_case,
                        flags.code_units().contains(&u16::from(b'm')),
                        flags.code_units().contains(&u16::from(b's')),
                        |work| budget.charge(work),
                    )
                })?;
                if let Some(matcher) = disjunction {
                    Some(RegExpMatcherBody::Disjunction(matcher))
                } else {
                    self.object_work(span, |_, budget| {
                        budget.charge(source.len())?;
                        budget.charge(source.len())
                    })?;
                    let anchored = self.object_work(span, |_, budget| {
                        RegExpAnchoredMatcher::compile_with_work(
                            &matching_source,
                            ignore_case,
                            flags.code_units().contains(&u16::from(b'm')),
                            flags.code_units().contains(&u16::from(b's')),
                            |work| budget.charge(work),
                        )
                    })?;
                    if let Some(matcher) = anchored {
                        Some(RegExpMatcherBody::Anchored(matcher))
                    } else {
                        self.object_work(span, |_, budget| {
                            let dot_all = flags.code_units().contains(&u16::from(b's'));
                            if let Some(matcher) = RegExpCharacterMatcher::compile_with_work(
                                &matching_source,
                                ignore_case,
                                dot_all,
                                |work| budget.charge(work),
                            )? {
                                Ok(Some(RegExpMatcherBody::Character(matcher)))
                            } else {
                                let sequence =
                                    RegExpSequenceMatcher::compile_with_assertions_and_work(
                                        &matching_source,
                                        ignore_case,
                                        flags.code_units().contains(&u16::from(b'm')),
                                        dot_all,
                                        |work| budget.charge(work),
                                    )?;
                                if let Some(matcher) = sequence {
                                    Ok(Some(RegExpMatcherBody::Sequence(matcher)))
                                } else {
                                    let quantified = RegExpQuantifiedMatcher::compile_with_work(
                                        &matching_source,
                                        ignore_case,
                                        dot_all,
                                        |work| budget.charge(work),
                                    )?;
                                    if let Some(matcher) = quantified {
                                        Ok(Some(RegExpMatcherBody::Quantified(matcher)))
                                    } else {
                                        if let Some(matcher) =
                                            RegExpQuantifiedContinuationMatcher::compile_with_assertions_and_work(
                                                &matching_source,
                                                ignore_case,
                                                flags.code_units().contains(&u16::from(b'm')),
                                                dot_all,
                                                |work| budget.charge(work),
                                            )?
                                        {
                                            Ok(Some(RegExpMatcherBody::QuantifiedContinuation(
                                                matcher,
                                            )))
                                        } else {
                                            if let Some(matcher) = RegExpPrefixedMatcher::compile_with_assertions_and_work(
                                                &matching_source,
                                                ignore_case,
                                                flags.code_units().contains(&u16::from(b'm')),
                                                dot_all,
                                                |work| budget.charge(work),
                                            )? {
                                                Ok(Some(RegExpMatcherBody::Prefixed(matcher)))
                                            } else {
                                                if let Some(matcher) = RegExpRepeatedLiteralMatcher::compile_with_work(
                                                    &matching_source,
                                                    ignore_case,
                                                    |work| budget.charge(work),
                                                )? {
                                                    Ok(Some(RegExpMatcherBody::RepeatedLiteral(matcher)))
                                                } else {
                                                    if let Some(matcher)=RegExpRepeatedSequenceMatcher::compile_with_assertions_and_work(
                                                        &matching_source,
                                                        ignore_case,
                                                        flags.code_units().contains(&u16::from(b'm')),
                                                        dot_all,
                                                        |work| budget.charge(work),
                                                    )? {Ok(Some(RegExpMatcherBody::RepeatedSequence(matcher)))}else{
                                                        if let Some(matcher)=RegExpRepeatedContinuationMatcher::compile_with_work(&matching_source,ignore_case,flags.code_units().contains(&u16::from(b'm')),dot_all,|work|budget.charge(work))? {Ok(Some(RegExpMatcherBody::RepeatedContinuation(matcher)))}else{
                                                            if let Some(matcher)=RegExpRepeatedPrefixedMatcher::compile_with_work(&matching_source,ignore_case,flags.code_units().contains(&u16::from(b'm')),dot_all,|work|budget.charge(work))? {Ok(Some(RegExpMatcherBody::RepeatedPrefixed(matcher)))}else{RegExpRepeatedCaptureMatcher::compile_with_work(&matching_source,ignore_case,flags.code_units().contains(&u16::from(b'm')),dot_all,|work|budget.charge(work)).map(|m|m.map(RegExpMatcherBody::RepeatedCaptures))}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        })?
                    }
                }
            }
        };
        // Unicode u/v atoms require complete-character matching.
        // Unicode dot/escape and flat-class plans precede the checked v-set and
        // legacy BMP proofs (22.2.2.7.1, 22.2.2.9).
        let matcher = if matcher.is_none() && unicode {
            // Complete wrappers around a proved consuming body
            // capture its whole original range (22.2.2.7). Compile the retained
            // body completely before admitting wrappers.
            let (matching_source, wrappers) = self.object_work(span, |_, budget| {
                // Assertions surrounding a complete ordinary atom wrapper are
                // zero-width, so moving them inside that wrapper preserves its
                // capture range (22.2.2.4). Word and line gates keep their order
                // and only complete transparent wrappers are removed. The whole
                // retained body must still pass a supported Unicode proof below.
                let units = capture_source.code_units();
                budget.charge(units.len().saturating_mul(4))?;
                let start_anchor = units.first() == Some(&0x5e);
                let end_anchor = units.last() == Some(&0x24)
                    && units[..units.len() - 1]
                        .iter()
                        .rev()
                        .take_while(|&&unit| unit == 0x5c)
                        .count()
                        % 2
                        == 0;
                let mut start = usize::from(start_anchor);
                let mut end = units.len() - usize::from(end_anchor);
                // A single word gate at either boundary is zero-width too.
                // Keep its original order relative to any outside line gate.
                if units.get(start) == Some(&0x5c)
                    && matches!(units.get(start + 1), Some(0x62 | 0x42))
                {
                    start += 2;
                }
                if end >= start + 2
                    && units[end - 2] == 0x5c
                    && matches!(units[end - 1], 0x62 | 0x42)
                    && units[..end - 2]
                        .iter()
                        .rev()
                        .take_while(|&&c| c == 0x5c)
                        .count()
                        % 2
                        == 0
                {
                    end -= 2;
                }
                let body_source = if start != 0 || end != units.len() {
                    let body = &units[start..end];
                    budget.charge(body.len())?;
                    JsString::from_code_units(body.to_vec())
                } else {
                    capture_source.clone()
                };
                if let Some(group) = regexp_outer_group_body(&body_source) {
                    let body = &body_source.code_units()[group.body];
                    let length = body.len() + start + (units.len() - end);
                    budget.charge(length)?;
                    let mut source = Vec::with_capacity(length);
                    source.extend_from_slice(&units[..start]);
                    source.extend_from_slice(body);
                    source.extend_from_slice(&units[end..]);
                    Ok((JsString::from_code_units(source), group.captures))
                } else {
                    Ok((capture_source.clone(), 0))
                }
            })?;
            let body = self.object_work(span, |_, budget| {
                budget.charge(matching_source.len())?;
                if let Some(body) = RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
                    &matching_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    flags.code_units().contains(&u16::from(b's')),
                    |work| budget.charge(work),
                )? {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) =
                    RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &matching_source,
                    flags.code_units().contains(&u16::from(b'v')),
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                )? {
                    return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                }
                if let Some(body) =
                    RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'v')),
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                }
                if let Some(body) =
                    RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_v_set_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                }
                if let Some(body) = RegExpUnicodePropertyDotMatcher::compile_with_work(
                    &matching_source,
                    flags.code_units().contains(&0x76),
                    flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x73),
                    |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodePropertyDot(body))); }
                if let Some(body) =
                    RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) =
                    RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) =
                    RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x6d),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) =
                    RegExpUnicodeCharacterMatcher::compile_assertion_sequence_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeCharacter(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_with_line_anchors_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) =
                        RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_line_anchors_and_work(
                            &matching_source,
                            flags.code_units().contains(&u16::from(b'i')),
                            flags.code_units().contains(&u16::from(b'm')),
                            |work| budget.charge(work),
                        )
                        .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_line_assertions_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_with_scoped_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_capturing_atom_with_assertions_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_with_composed_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_nested_assertion_groups_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_with_nested_assertion_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_capturing_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76),
                        flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d),
                        flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_alternatives_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_nested_alternative_assertions_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_scoped_alternative_words_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branches_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_words_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_lines_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_line_words_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertions_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_scoped_assertions_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_assertion_sequences_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_sequence_words_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertions_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_branch_nested_assertion_words_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_nullable_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) =
                    RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_alternatives_with_work(
                        &matching_source,
                        flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                        flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73),
                        |work| budget.charge(work),
                    )?
                { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_alternatives_with_work(
                    &matching_source, flags.code_units().contains(&0x76), flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d), flags.code_units().contains(&0x73), |work| budget.charge(work),
                )? { return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body))); }
                // Legacy BMP APIs retain their case-sensitive Unicode contract.
                if !flags.code_units().contains(&u16::from(b'i')) {
                    let body = if flags.code_units().contains(&u16::from(b'v')) {
                        RegExpCharacterMatcher::compile_bmp_unicode_sets_with_work(
                            &matching_source,
                            |work| budget.charge(work),
                        )
                    } else {
                        RegExpCharacterMatcher::compile_bmp_unicode_with_work(
                            &matching_source,
                            |work| budget.charge(work),
                        )
                    }?;
                    if let Some(body) = body {
                        return Ok(Some(RegExpMatcherBody::Character(body)));
                    }
                }
                if flags.code_units().contains(&u16::from(b'v')) {
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) =
                        RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                            &matching_source,
                            flags.code_units().contains(&u16::from(b'i')),
                            flags.code_units().contains(&u16::from(b'm')),
                            |work| budget.charge(work),
                        )
                        .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) =
                        RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                            &matching_source,
                            flags.code_units().contains(&u16::from(b'i')),
                            flags.code_units().contains(&u16::from(b'm')),
                            |work| budget.charge(work),
                        )
                        .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeLiteralClassPairMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeLiteralClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairLiteralMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPairLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteralClass(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassTripleMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassTriple(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassTripleWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeLiteralClassPairWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairLiteralWordMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPairLiteralWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralClassWordMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteralClassWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassSequenceMatcher::compile_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassSequence(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassSequenceMatcher::compile_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassSequence(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassSequenceMatcher::compile_mixed_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassSequence(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_word_boundaries_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteralClassWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeLiteralClassPairWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPairLiteralWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeLiteralClassPairWordMatcher::compile_checked_then_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeLiteralClassPairWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_then_checked_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeLiteralClassPairWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairLiteralWordMatcher::compile_checked_then_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPairLiteralWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairLiteralWordMatcher::compile_flat_then_checked_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPairLiteralWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralClassWordMatcher::compile_checked_then_flat_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteralClassWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_then_checked_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteralClassWord(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassLiteralMatcher::compile_v_set_then_literal_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassLiteral(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_pair_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_checked_then_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassPairMatcher::compile_v_set_then_checked_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClassPair(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeStringSet(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeClass(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_noncapturing_empty_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_finite_noncapturing_empty_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_noncapturing_empty_sequence_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_noncapturing_empty_sequence_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_noncapturing_empty_sequence_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_quantified_empty_sequence_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_quantified_empty_sequence_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_quantified_empty_sequence_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_empty_choice_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_empty_choice_nullable_v_set_with_single_character_strings_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedAtomMatcher::compile_counted_empty_choice_nullable_character_result_v_set_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    )
                    .map_err(unicode_set_compile_error)?
                    {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedAtom(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_empty_branch_captures_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_noncapturing_empty_branch_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_quantified_noncapturing_empty_branch_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_noncapturing_empty_choices_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_empty_capture_effects_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_with_boundary_empty_captures_and_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                    }
                    if let Some(body) = RegExpUnicodeRepeatedClassCaptureMatcher::compile_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedClassCaptures(body)));
                    }

                    if let Some(body) = RegExpUnicodeRepeatedClassCaptureMatcher::compile_single_with_work(
                        &matching_source,
                        flags.code_units().contains(&u16::from(b'i')),
                        flags.code_units().contains(&u16::from(b'm')),
                        |work| budget.charge(work),
                    ).map_err(unicode_set_compile_error)? {
                        return Ok(Some(RegExpMatcherBody::UnicodeRepeatedClassCaptures(body)));
                    }

                }
                Ok(None)
            })?;
            if body.is_some() {
                enclosing_captures = wrappers;
            }
            body
        } else {
            matcher
        };
        // Retain the original alternative owner. Wrapper removal can expose a
        // root alternative whose assertions belong to only one branch (22.2.2).
        let matcher = if matcher.is_none() && flags.code_units().contains(&u16::from(b'v')) {
            let body = self.object_work(span, |_, budget| {
                if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_single_with_boundary_empty_captures_and_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_single_branch_captures_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_single_complete_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeNullableClassCaptureMatcher::compile_single_root_alternative_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeEmptyStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeEmptyStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeNullableStringClassCaptureMatcher::compile_grouped_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeEmptyStringClassCaptureMatcher::compile_grouped_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeEmptyStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_outer_assertions_and_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_enclosed_outer_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_surrounding_line_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }

                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_surrounding_word_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_nested_assertion_wrappers_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_unconditional_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_reversed_line_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_grouped_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_surrounding_line_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_surrounding_word_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_nested_assertion_wrappers_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_reversed_surrounding_line_assertions_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_nested_endpoint_assertion_wrappers_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSurroundingAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeSurroundingAssertedStringClassAlternativeMatcher::compile_exact_single_counts_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSurroundingAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_exact_single_counts_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedStringClassAlternatives(body)));
                }
                if let Some(body) = RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeNullableStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeEmptyStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeEmptyStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeRepeatedStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeRepeatedStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeBoundedStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeBoundedStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeCountedNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeCountedNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodePrefixFreeStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodePrefixFreeStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTwoRequiredStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTwoRequiredStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeRequiredStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeRequiredStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedRepeatedStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedTwoRequiredNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedCountedNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedCountedNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeAssertedCountedNullableScalarStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeAssertedCountedNullableScalarStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroCountAssertedStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroCountAssertedStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroCountStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroCountStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroCountSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroCountAssertedSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroCountAssertedSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroCountStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroCountAssertedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroCountAssertedStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroMinimumAssertedStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroMinimumAssertedStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroMinimumStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroMinimumStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroMinimumSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroMinimumSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroMinimumAssertedSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroMinimumAssertedSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeUnassertedZeroMinimumStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeUnassertedZeroMinimumStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeZeroMinimumAssertedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeZeroMinimumAssertedStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodePredicateChoiceStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodePredicateChoiceStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodePredicateChoiceSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodePredicateChoiceSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodePredicateChoiceStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodePredicateChoiceStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeConjunctivePredicateChoiceStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeConjunctivePredicateChoiceStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeConjunctivePredicateChoiceSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeConjunctivePredicateChoiceSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeConjunctivePredicateChoiceStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeConjunctivePredicateChoiceStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableCountedNullableStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableCountedNullableStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableSingleStringClassCaptureMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableSingleStringClassCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableRootStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableRootStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableGroupedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableGroupedStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableExactOneGroupedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableExactOneGroupedStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableSurroundingStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableSurroundingStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableGroupedOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableGroupedOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableExactOneGroupedOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableExactOneGroupedOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableSurroundingOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableSurroundingOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableZeroCountRootStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableZeroCountRootStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableNestedOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableNestedOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableNestedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableNestedZeroCountStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedNestedOptionalStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedNestedOptionalStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedNestedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedNestedZeroCountStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableOptionalSpineStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableOptionalSpineStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableZeroCountSpineStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableZeroCountSpineStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableQuantifiedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableQuantifiedZeroCountStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedQuantifiedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedQuantifiedZeroCountStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedZeroCountSpineStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedZeroCountSpineStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedOptionalSpineStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedOptionalSpineStringClassAlternativeCaptures(body)));
                }
                if let Some(body) = RegExpUnicodeTruthTableScopedGroupedOptionalSpineStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)? {
                    return Ok(Some(RegExpMatcherBody::UnicodeTruthTableScopedGroupedOptionalSpineStringClassAlternativeCaptures(body)));
                }
                RegExpUnicodeTruthTableScopedGroupedZeroCountStringClassAlternativeMatcher::compile_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    |work| budget.charge(work),
                ).map_err(unicode_set_compile_error)
                 .map(|body| body.map(RegExpMatcherBody::UnicodeTruthTableScopedGroupedZeroCountStringClassAlternativeCaptures))

            })?;
            if body.is_some() {
                enclosing_captures = 0;
            }
            body
        } else {
            matcher
        };
        // Unicode literal concatenations resolve capture boundaries in original
        // input characters, rather than folded/source UTF-16 widths (22.2.2.7.3).
        let matcher = if matcher.is_none() && unicode {
            let body = self.object_work(span, |_, budget| {
                let body = RegExpUnicodeFoldedLiteralMatcher::compile_with_flags_and_work(
                    &capture_source,
                    flags.code_units().contains(&0x69),
                    flags.code_units().contains(&0x6d),
                    |work| budget.charge(work),
                )?;
                match body {
                    Some(body) => Ok(Some(body)),
                    None => {
                        RegExpUnicodeFoldedLiteralMatcher::compile_with_word_boundaries_and_work(
                            &capture_source,
                            flags.code_units().contains(&0x69),
                            flags.code_units().contains(&0x6d),
                            |work| budget.charge(work),
                        )
                    }
                }
            })?;
            if body.is_some() {
                // This plan compiles the complete original capture layout.
                enclosing_captures = 0;
            }
            body.map(RegExpMatcherBody::UnicodeFoldedLiteral)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() && !unicode {
            let ordinary = self.object_work(span, |_, budget| {
                budget.charge(named_groups.len())?;
                let groups: Vec<&[usize]> = named_groups
                    .iter()
                    .map(|group| group.slots.as_slice())
                    .collect();
                RegExpBackreferenceMatcher::compile_ordinary_with_named_bindings_and_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'i')),
                    flags.code_units().contains(&u16::from(b'm')),
                    flags.code_units().contains(&u16::from(b's')),
                    spite_core::RegExpBackreferenceNamedBindings {
                        groups: &groups,
                        references: &named_references,
                    },
                    |work| budget.charge(work),
                )
            })?;
            // The fallback compiles the complete original layout, including
            // captures peeled only for the specialized matchers above.
            enclosing_captures = 0;
            ordinary.map(RegExpMatcherBody::Backreferences)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            // Complete grammar and early-error validation above is required:
            // RepeatMatcher with max zero never invokes this group's body
            // (22.2.2.3.1). Keep all older successful plans and their work intact.
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountGroupMatcher::compile_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountGroup)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountGroupMatcher::compile_enclosing_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            zero.map(|(body, captures)| {
                // Mandatory ordinary wrappers capture the empty whole match;
                // the retained zero-count body never defines a slot (22.2.2.7).
                enclosing_captures = captures;
                RegExpMatcherBody::ZeroCountGroup(body)
            })
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountClassMatcher::compile_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountClass)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountClassMatcher::compile_enclosing_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            zero.map(|(body, captures)| {
                enclosing_captures = captures;
                RegExpMatcherBody::ZeroCountClass(body)
            })
        } else {
            matcher
        };
        let matcher = if matcher.is_none() && unicode {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountClassMatcher::compile_unicode_property_validated_with_work(
                    &capture_source,
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountClass)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() && unicode {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountClassMatcher::compile_enclosing_unicode_property_validated_with_work(
                    &capture_source,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            zero.map(|(body, captures)| {
                enclosing_captures = captures;
                RegExpMatcherBody::ZeroCountClass(body)
            })
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSpineMatcher::compile_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                // This plan carries the complete original capture layout,
                // including the undefined suffix beneath optional groups.
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSpine)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSpine)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_first_alternative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_quantified_enclosing_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_tree_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_positive_tree_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_escaped_tree_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_surrogate_tree_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_negative_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_first_negative_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_negative_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_negative_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_optional_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_first_optional_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_positive_optional_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_known_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_optional_known_impossible_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_first_optional_known_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_positive_optional_known_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_established_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_optional_established_negative_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let zero = self.object_work(span, |_, budget| {
                RegExpZeroCountSequenceMatcher::compile_enclosing_optional_established_negative_sequence_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if zero.is_some() {
                enclosing_captures = 0;
            }
            zero.map(RegExpMatcherBody::ZeroCountSequence)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_required_enclosing_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_positive_impossible_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_composite_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_wrapped_composite_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_alternatives_composite_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_enclosing_composite_alternatives_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_nullable_group_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_nullable_atom_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_nullable_sequence_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_nullable_alternatives_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_nullable_positive_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_wrapped_nullable_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_known_alternatives_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_wrapped_known_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_optional_known_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = if matcher.is_none() {
            let impossible = self.object_work(span, |_, budget| {
                RegExpImpossibleMatcher::compile_wrapped_optional_known_negative_validated_with_work(
                    &capture_source,
                    unicode,
                    flags.code_units().contains(&u16::from(b'v')),
                    |work| budget.charge(work),
                )
            })?;
            if impossible.is_some() {
                enclosing_captures = 0;
            }
            impossible.map(RegExpMatcherBody::Impossible)
        } else {
            matcher
        };
        let matcher = matcher.map(|body| RegExpMatcher::new(body, enclosing_captures));
        if let Some(matcher) = &matcher {
            // RegExpBuiltinExec requires the plan's captures to agree with the
            // RegExp Record's validated CapturingGroupsCount (22.2.7.2).
            debug_assert_eq!(matcher.capture_count(), captures as usize);
        }
        self.object_work(span, |objects, _| {
            objects.initialize_regexp(
                &object,
                RegExpData {
                    source,
                    flags,
                    matcher,
                    named_groups,
                },
            )
        })?;
        self.set_property_or_throw(
            &object,
            JsString::from("lastIndex"),
            Value::Number(0.0),
            span,
        )?;
        Ok(Value::Object(object))
    }
}

fn unicode_set_compile_error(
    error: RegExpUnicodeSetCompileError<crate::object::Error>,
) -> crate::object::Error {
    match error {
        RegExpUnicodeSetCompileError::Work(error) => error,
        RegExpUnicodeSetCompileError::Capacity => {
            crate::object::Error::Heap(spite_heap::Error::Capacity)
        }
    }
}
