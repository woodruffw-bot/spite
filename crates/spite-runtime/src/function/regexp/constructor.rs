//! RegExp allocation and initialization (22.2.3.1–3, 22.2.4.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, RegExpData, RegExpMatcher, RegExpMatcherBody, RegExpNamedGroup},
};
use spite_core::{
    DiagnosticKind, JsString, RegExpAnchoredMatcher, RegExpBackreferenceMatcher,
    RegExpCharacterMatcher, RegExpDisjunctionMatcher, RegExpLiteralMatcher, RegExpPrefixedMatcher,
    RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher, RegExpRepeatedCaptureMatcher,
    RegExpRepeatedContinuationMatcher, RegExpRepeatedLiteralMatcher, RegExpRepeatedPrefixedMatcher,
    RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher, Span, regexp_outer_group_body,
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
                    if let Some(matcher) =
                        RegExpLiteralMatcher::compile_bmp_unicode(&capture_source)
                    {
                        Some(RegExpMatcherBody::Literal(matcher))
                    } else {
                        budget.charge(capture_source.len())?;
                        budget.charge(capture_source.len())?;
                        budget.charge(capture_source.len())?;
                        RegExpLiteralMatcher::compile_unicode_scalars(&capture_source)
                            .map(RegExpMatcherBody::UnicodeScalars)
                    }
                } else {
                    None
                };
                if literal.is_some() {
                    Ok(literal)
                } else {
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    budget.charge(capture_source.len())?;
                    Ok(RegExpLiteralMatcher::compile_unicode_empty(&capture_source)
                        .map(RegExpMatcherBody::UnicodeEmpty))
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
        // A proved u/v atom can reuse ordinary membership and matching only
        // when every member is a nonsurrogate BMP character (22.2.2.7.1).
        // UnicodeSets operators and ignore-case folding remain separate.
        let matcher =
            if matcher.is_none() && unicode && !flags.code_units().contains(&u16::from(b'i')) {
                self.object_work(span, |_, budget| {
                    budget.charge(capture_source.len())?;
                    let body = if flags.code_units().contains(&u16::from(b'v')) {
                        RegExpCharacterMatcher::compile_bmp_unicode_sets_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )
                    } else {
                        RegExpCharacterMatcher::compile_bmp_unicode_with_work(
                            &capture_source,
                            |work| budget.charge(work),
                        )
                    }?;
                    Ok(body.map(RegExpMatcherBody::Character))
                })?
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
