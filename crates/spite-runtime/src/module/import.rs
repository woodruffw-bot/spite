//! Loaded-record namespace completion from ContinueDynamicImport (13.3.10).

use super::*;
use crate::function::{PromiseCapability, async_native::AsyncResumeKind};

impl Realm {
    /// Evaluates a loaded record and resolves a fresh Promise to its namespace.
    ///
    /// The graph must already be supplied through [`Self::load_module_graph`].
    /// Evaluation is cached, while each import has a distinct Promise. Drain
    /// [`Self::run_next_promise_job`] to finish namespace delivery. The usual
    /// Promise resolution rules apply to an exported `then` binding.
    ///
    /// Linking and evaluation exceptions reject the Promise; host failures are
    /// returned directly and bypass JavaScript rejection. Returned object values
    /// are unrooted; use [`Self::root_value`] across explicit collection.
    pub fn import_module_record(&mut self, module: &ModuleHandle) -> Result<Value, Error> {
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let span = Span::new(0, 0);
        let capability = self.intrinsic_promise_capability(span)?;
        self.continue_module_import(module, capability.clone(), span)?;
        Ok(capability.promise)
    }

    fn continue_module_import(
        &mut self,
        module: &ModuleHandle,
        capability: PromiseCapability,
        span: Span,
    ) -> Result<(), Error> {
        let evaluation = match self.evaluate_module_record_handle(module.root.handle()) {
            Ok(evaluation) => evaluation,
            Err(error) => {
                let reason = self.language_exception_value(error)?;
                self.call(capability.reject, Value::Undefined, vec![reason], span)?;
                return Ok(());
            }
        };
        let Value::Object(evaluation) = evaluation else {
            unreachable!("Module evaluation Promise")
        };
        let handler = self.native_promise_handler(
            module.root.handle(),
            AsyncResumeKind::ModuleNamespace,
            true,
            span,
        )?;
        self.perform_promise_then(
            &evaluation,
            handler,
            Value::Undefined,
            Some(capability),
            span,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
