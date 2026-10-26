//! Global-scope ordinary string compilation (20.2.1.1, 20.2.1.1.1).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::{
    ast::FunctionKind, parse_dynamic_async_function_utf16, parse_dynamic_function_utf16,
    parse_dynamic_generator_function_utf16,
};

impl Realm {
    pub(super) fn dynamic_function(
        &mut self,
        kind: FunctionKind,
        new_target: Option<ObjectHandle>,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let body = arguments
            .next_back()
            .unwrap_or_else(|| Value::String(JsString::default()));
        let mut parameters = Vec::new();
        parameters
            .try_reserve_exact(arguments.len())
            .map_err(|_| Error::Limit {
                span,
                message: "dynamic parameter capacity exceeded".into(),
            })?;
        // Convert every parameter before the body, and all inputs before parsing
        // or reading newTarget.prototype. Caller strictness never applies.
        for argument in arguments {
            parameters.push(self.string(argument, span)?);
        }
        let body = self.string(body, span)?;
        let overhead = match kind {
            FunctionKind::Async => 32,
            FunctionKind::Ordinary => 26,
            FunctionKind::Generator => 27,
        };
        let parameter_units = parameters
            .iter()
            .enumerate()
            .try_fold(0usize, |length, (index, parameter)| {
                length
                    .checked_add(parameter.len())?
                    .checked_add(usize::from(index != 0))
            })
            .ok_or_else(|| Error::Limit {
                span,
                message: "dynamic source capacity exceeded".into(),
            })?;
        let units = parameter_units
            .checked_add(body.len())
            .and_then(|n| n.checked_add(overhead))
            .ok_or_else(|| Error::Limit {
                span,
                message: "dynamic source capacity exceeded".into(),
            })?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| units > limit)
        {
            return Err(Error::Limit {
                span,
                message: "dynamic source string length limit exceeded".into(),
            });
        }
        // The default host permits string compilation (HostEnsureCanCompileStrings).
        // StringToCodePoints preserves unpaired surrogates (11.1, 20.2.1.1.1).
        let mut joined = Vec::new();
        joined
            .try_reserve_exact(parameter_units)
            .map_err(|_| Error::Limit {
                span,
                message: "dynamic source capacity exceeded".into(),
            })?;
        for (index, parameter) in parameters.iter().enumerate() {
            if index != 0 {
                joined.push(u16::from(b','));
            }
            joined.extend_from_slice(parameter.code_units());
        }
        let joined = JsString::from_code_units(joined);
        let encoded_len = |source: &JsString| {
            char::decode_utf16(source.code_units().iter().copied()).try_fold(0usize, |n, point| {
                n.checked_add(point.map_or(3, char::len_utf8))
            })
        };
        let bytes = encoded_len(&joined)
            .and_then(|n| n.checked_add(encoded_len(&body)?))
            .and_then(|n| n.checked_add(overhead))
            .ok_or_else(|| Error::Limit {
                span,
                message: "dynamic source capacity exceeded".into(),
            })?;
        if self
            .limits
            .max_source_bytes
            .is_some_and(|limit| bytes > limit)
        {
            return Err(Error::Limit {
                span,
                message: "dynamic source size limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(bytes))?;
        let syntax = (match kind {
            FunctionKind::Async => parse_dynamic_async_function_utf16(&joined, &body),
            FunctionKind::Ordinary => parse_dynamic_function_utf16(&joined, &body),
            FunctionKind::Generator => parse_dynamic_generator_function_utf16(&joined, &body),
        })
        .map_err(|diagnostic| match diagnostic.kind {
            DiagnosticKind::Syntax => {
                Self::exception(ExceptionKind::SyntaxError, span, diagnostic.message)
            }
            DiagnosticKind::Unsupported => Self::unsupported(span, diagnostic.message),
            DiagnosticKind::Limit => Error::Limit {
                span,
                message: diagnostic.message,
            },
        })?;
        let intrinsic = self.intrinsics.as_ref().expect("initialized");
        let (constructor, fallback) = match kind {
            FunctionKind::Async => (
                &intrinsic.async_function.constructor,
                &intrinsic.async_function.prototype,
            ),
            FunctionKind::Generator => (
                &intrinsic.generator.constructor,
                &intrinsic.generator.function_prototype,
            ),
            FunctionKind::Ordinary => (
                &intrinsic.function_constructor,
                &intrinsic.function_prototype,
            ),
        };
        let new_target = new_target.unwrap_or_else(|| constructor.clone());
        let fallback = fallback.clone();
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            fallback
        };
        let environment = self.scopes.first().expect("global environment").clone();
        self.allocate_ordinary_function(
            &syntax,
            environment,
            &prototype,
            syntax.body.is_strict(),
            span,
        )
    }
}
