//! Shared capture and caller restoration for resumable source functions.

use crate::{Error, Realm, object::source_context::SourceExecutionContext};
use spite_core::Span;

impl Realm {
    pub(super) fn capture_source_execution_context(
        &mut self,
        base: usize,
        span: Span,
        allocation_message: &'static str,
    ) -> Result<SourceExecutionContext, Error> {
        let count = self.scopes.len() - base + 1;
        self.object_work(span, |_, budget| budget.charge(count))?;
        let mut scopes = Vec::new();
        scopes.try_reserve(count).map_err(|_| Error::Limit {
            span,
            message: allocation_message.into(),
        })?;
        scopes.push(self.scopes.first().expect("global environment").clone());
        scopes.extend(self.scopes[base..].iter().cloned());
        Ok(SourceExecutionContext {
            scopes,
            variable: self.variable_environment.clone(),
            strict: self.strict,
        })
    }

    pub(super) fn with_source_execution_context<T>(
        &mut self,
        context: SourceExecutionContext,
        execute: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let caller = SourceExecutionContext {
            scopes: std::mem::replace(&mut self.scopes, context.scopes),
            variable: std::mem::replace(&mut self.variable_environment, context.variable),
            strict: std::mem::replace(&mut self.strict, context.strict),
        };
        let result = execute(self);
        self.scopes = caller.scopes;
        self.variable_environment = caller.variable;
        self.strict = caller.strict;
        result
    }
}
