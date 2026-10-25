//! RegExp allocation and initialization (22.2.3.1–3, 22.2.4.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, RegExpData, RegExpMatcher, RegExpNamedGroup},
};
use spite_core::{DiagnosticKind, JsString, RegExpProgram, Span};
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
        self.regexp_initialize_from_strings(object, source, flags, span)
    }

    // Pattern validation and compilation follow observable prototype lookup
    // and conversions, in a separate frame (22.2.4.1).
    #[inline(never)]
    fn regexp_initialize_from_strings(
        &mut self,
        object: ObjectHandle,
        source: JsString,
        flags: JsString,
        span: Span,
    ) -> Result<Value, Error> {
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
        let named_groups = self.object_work(span, |_, budget| {
            budget.charge(metadata.named_captures.len())?;
            let mut groups = Vec::<RegExpNamedGroup>::new();
            let mut names = HashMap::new();
            for capture in metadata.named_captures {
                let group = *names.entry(capture.name.clone()).or_insert_with(|| {
                    let index = groups.len();
                    groups.push(RegExpNamedGroup {
                        name: capture.name,
                        slots: Vec::new(),
                    });
                    index
                });
                groups[group].slots.push((capture.index - 1) as usize);
            }
            Ok(Arc::<[RegExpNamedGroup]>::from(groups))
        })?;
        let program = self
            .object_work(span, |_, budget| {
                budget.charge(source.len().saturating_mul(8))?;
                budget.charge(captures as usize)?;
                Ok(RegExpProgram::compile_validated(&source, &flags))
            })?
            .map_err(|message| {
                if message.contains("too deeply nested")
                    || message.contains("Too many")
                    || message.contains("exceeds backend representation")
                {
                    Error::Limit {
                        span,
                        message: format!("regress implementation limit: {message}"),
                    }
                } else {
                    Self::unsupported(
                        span,
                        format!("regress cannot compile this validated Pattern: {message}"),
                    )
                }
            })?;
        debug_assert_eq!(program.capture_count(), captures as usize);
        let matcher = Some(RegExpMatcher::new(program));
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
