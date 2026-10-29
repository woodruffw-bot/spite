//! Standalone ExecuteAsyncModule and completion reactions (16.2.1.6.2).

use super::*;
use crate::object::ModuleExecutionStatus;

impl Realm {
    pub(super) fn evaluate_async_standalone_module(
        &mut self,
        module: &Module,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let capability = self.intrinsic_promise_capability(span)?;
        self.object_work(span, |objects, budget| {
            objects.store_module_top_level_capability(record, capability.clone(), budget)
        })?;
        let result = (|| {
            self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let execution = objects.module_execution_mut(record)?;
                execution.status = ModuleExecutionStatus::EvaluatingAsync;
                execution.cycle_root = Some(record.clone());
                Ok(())
            })?;
            let environment = self
                .objects
                .module_environment(record)
                .map_err(Error::InvalidObject)?
                .expect("linked module");
            self.execute_async_module_body(
                module.body(),
                environment,
                module_default_index(module),
                record,
                span,
            )
        })();
        if let Err(error) = result {
            self.abort_module_evaluation(record, &error)?;
            return Err(error);
        }
        Ok(capability.promise)
    }

    pub(crate) fn resume_module_execution(
        &mut self,
        record: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let result = self.object_work(span, |objects, budget| {
            budget.charge(1)?;
            let execution = objects.module_execution_mut(&record)?;
            if execution.status == ModuleExecutionStatus::Aborted {
                return Ok(None);
            }
            debug_assert_eq!(execution.status, ModuleExecutionStatus::EvaluatingAsync);
            execution.status = ModuleExecutionStatus::Evaluated;
            execution.error = (!fulfilled).then(|| value.clone());
            Ok(execution.capability.clone())
        });
        let capability = match result {
            Ok(capability) => capability,
            Err(error) => {
                self.abort_module_evaluation(&record, &error)?;
                return Err(error);
            }
        };
        if let Some(capability) = capability {
            let result = self.call(
                if fulfilled {
                    capability.resolve
                } else {
                    capability.reject
                },
                Value::Undefined,
                vec![if fulfilled { Value::Undefined } else { value }],
                span,
            );
            if let Err(error) = result {
                self.abort_module_evaluation(&record, &error)?;
                return Err(error);
            }
        }
        Ok(Value::Undefined)
    }
}

#[cfg(test)]
mod tests;
