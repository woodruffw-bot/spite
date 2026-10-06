//! RegExp allocation and initialization (22.2.3.1–3, 22.2.4.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, RegExpData, RegExpMatcher},
};
use spite_core::{DiagnosticKind, JsString, RegExpDisjunctionMatcher, RegExpLiteralMatcher, Span};
use spite_parser::validate_regexp_pattern;

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
        let captures =
            validate_regexp_pattern(&source, &flags, span).map_err(
                |diagnostic| match diagnostic.kind {
                    DiagnosticKind::Syntax => {
                        Self::exception(ExceptionKind::SyntaxError, span, diagnostic.message)
                    }
                    DiagnosticKind::Unsupported => Self::unsupported(span, diagnostic.message),
                    DiagnosticKind::Limit => Error::Limit {
                        span,
                        message: diagnostic.message,
                    },
                },
            )?;
        let matcher = if flags
            .code_units()
            .iter()
            .any(|&unit| matches!(unit, 0x75 | 0x76))
        {
            None
        } else {
            self.object_work(span, |_, budget| {
                budget.charge(source.len())?;
                budget.charge(source.len())
            })?;
            let ignore_case = flags.code_units().contains(&u16::from(b'i'));
            if let Some(matcher) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                Some(RegExpMatcher::Literal(matcher))
            } else {
                // Cover the top-level scan and remaining per-branch compilation
                // passes. Accounting remains optional, as for literal plans.
                self.object_work(span, |_, budget| {
                    budget.charge(source.len())?;
                    budget.charge(source.len())
                })?;
                RegExpDisjunctionMatcher::compile(&source, ignore_case)
                    .map(RegExpMatcher::Disjunction)
            }
        };
        if let Some(matcher) = &matcher {
            // RegExpBuiltinExec requires the plan's captures to agree with the
            // RegExp Record's validated CapturingGroupsCount (22.2.7.2).
            debug_assert_eq!(matcher.capture_ranges().len(), captures as usize);
        }
        self.object_work(span, |objects, _| {
            objects.initialize_regexp(
                &object,
                RegExpData {
                    source,
                    flags,
                    matcher,
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
