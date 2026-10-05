//! Global-scope ordinary string compilation (20.2.1.1, 20.2.1.1.1).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{DiagnosticKind, JsString, Span};
use spite_parser::parse_dynamic_function;

impl Realm {
    pub(super) fn dynamic_function(
        &mut self,
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
        let units = parameters
            .iter()
            .enumerate()
            .try_fold(body.len(), |length, (index, parameter)| {
                length
                    .checked_add(parameter.len())?
                    .checked_add(usize::from(index != 0))
            })
            .and_then(|n| n.checked_add(26))
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
        // The existing source representation cannot preserve unpaired surrogates.
        // Reject that gap explicitly, after all observable conversions.
        let body = body.to_utf8().map_err(|_| {
            Self::unsupported(
                span,
                "dynamic source containing unpaired surrogates is not implemented",
            )
        })?;
        let mut joined = String::new();
        for (index, parameter) in parameters.iter().enumerate() {
            let parameter = parameter.to_utf8().map_err(|_| {
                Self::unsupported(
                    span,
                    "dynamic source containing unpaired surrogates is not implemented",
                )
            })?;
            let extra = parameter
                .len()
                .checked_add(usize::from(index != 0))
                .ok_or_else(|| Error::Limit {
                    span,
                    message: "dynamic source capacity exceeded".into(),
                })?;
            joined.try_reserve(extra).map_err(|_| Error::Limit {
                span,
                message: "dynamic source capacity exceeded".into(),
            })?;
            if index != 0 {
                joined.push(',');
            }
            joined.push_str(&parameter);
        }
        let bytes = joined
            .len()
            .checked_add(body.len())
            .and_then(|n| n.checked_add(26))
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
        let syntax =
            parse_dynamic_function(&joined, &body).map_err(|diagnostic| match diagnostic.kind {
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
        let new_target = new_target.unwrap_or_else(|| intrinsic.function_constructor.clone());
        let fallback = intrinsic.function_prototype.clone();
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
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
