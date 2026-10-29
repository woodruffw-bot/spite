//! ExecuteAsyncModule, available ancestors and completion reactions (16.2.1.6.2).

use super::*;
use crate::object::ModuleExecutionStatus;
use std::collections::HashSet;

fn capacity_error(span: Span) -> Error {
    Error::Limit {
        span,
        message: "async module scheduling capacity exceeded".into(),
    }
}

fn push<T>(values: &mut Vec<T>, value: T, span: Span) -> Result<(), Error> {
    values.try_reserve(1).map_err(|_| capacity_error(span))?;
    values.push(value);
    Ok(())
}

impl Realm {
    // A started Promise job can exhaust its allowance in Call before the native
    // callback is entered. Release its suspended Module state in that case too.
    pub(crate) fn abort_native_module_reaction(
        &mut self,
        handler: &Value,
        error: &Error,
    ) -> Result<(), Error> {
        use crate::function::{Callable, async_native::AsyncResumeKind};
        let Value::Object(handler) = handler else {
            return Ok(());
        };
        let Some(Callable::AsyncResume { state, kind, .. }) = self
            .objects
            .inspect(handler)
            .map_err(Error::InvalidObject)?
            .callable()
        else {
            return Ok(());
        };
        let kind = *kind;
        let state = state.clone();
        let record = match kind {
            AsyncResumeKind::ModuleExecution => Some(state.clone()),
            AsyncResumeKind::Execution => self
                .objects
                .async_execution_module(&state)
                .map_err(Error::InvalidObject)?,
            _ => None,
        };
        if let Some(record) = record {
            if matches!(kind, AsyncResumeKind::Execution)
                && let Some(data) = self
                    .objects
                    .take_async_execution(&state)
                    .map_err(Error::InvalidObject)?
            {
                for environment in &data
                    .context
                    .as_ref()
                    .expect("suspended source context")
                    .scopes[1..]
                {
                    self.objects
                        .take_environment_disposable_resources(environment)
                        .map_err(Error::InvalidObject)?;
                }
            }
            self.abort_module_evaluation(&record, error)?;
        }
        Ok(())
    }

    pub(super) fn add_async_module_parent(
        &mut self,
        dependency: &ObjectHandle,
        parent: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        self.object_work(span, |objects, budget| {
            budget.charge(2)?;
            let pending = objects
                .module_execution(parent)?
                .expect("evaluating parent")
                .pending_async_dependencies
                .checked_add(1)
                .ok_or(crate::object::Error::WorkLimit)?;
            let dependency = objects.module_execution_mut(dependency)?;
            if dependency.async_parents.len() == dependency.async_parents.capacity() {
                budget.charge(dependency.async_parents.len())?;
            }
            dependency
                .async_parents
                .try_reserve(1)
                .map_err(|_| spite_heap::Error::Capacity)?;
            dependency.async_parents.push(parent.clone());
            objects
                .module_execution_mut(parent)?
                .pending_async_dependencies = pending;
            Ok(())
        })
    }

    pub(super) fn assign_module_async_order(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        let next = self
            .module_async_order
            .checked_add(1)
            .ok_or_else(|| capacity_error(span))?;
        self.object_work(span, |objects, budget| {
            budget.charge(1)?;
            let execution = objects.module_execution_mut(record)?;
            debug_assert!(execution.async_order.is_none());
            execution.async_order = Some(next);
            Ok(())
        })?;
        self.module_async_order = next;
        Ok(())
    }

    pub(super) fn execute_module_record(
        &mut self,
        record: &ObjectHandle,
        asynchronous: bool,
        span: Span,
    ) -> Result<(), Error> {
        let (source, environment) = self.object_work(span, |objects, budget| {
            budget.charge(1)?;
            Ok((
                objects.module_source(record)?.clone(),
                objects.module_environment(record)?.expect("linked module"),
            ))
        })?;
        if asynchronous {
            self.execute_async_module_body(
                source.body(),
                environment,
                module_default_index(&source),
                record,
                span,
            )
        } else {
            self.execute_sync_module_body(
                source.body(),
                environment,
                module_default_index(&source),
                span,
            )
        }
    }

    fn finish_async_module(
        &mut self,
        record: &ObjectHandle,
        reason: Option<Value>,
        span: Span,
    ) -> Result<(), Error> {
        let result = (|| {
            let capability = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let execution = objects.module_execution_mut(record)?;
                execution.status = ModuleExecutionStatus::Evaluated;
                execution.async_order = None;
                execution.error = reason.clone();
                Ok(execution.capability.clone())
            })?;
            if let Some(capability) = capability {
                self.call(
                    if reason.is_some() {
                        capability.reject
                    } else {
                        capability.resolve
                    },
                    Value::Undefined,
                    vec![reason.unwrap_or(Value::Undefined)],
                    span,
                )?;
            }
            Ok(())
        })();
        if let Err(error) = &result {
            self.abort_module_evaluation(record, error)?;
        }
        result
    }

    pub(super) fn reject_async_module_execution(
        &mut self,
        record: &ObjectHandle,
        reason: Value,
        span: Span,
    ) -> Result<(), Error> {
        let mut pending = Vec::new();
        push(&mut pending, record.clone(), span)?;
        while let Some(record) = pending.pop() {
            let parents = self.object_work(span, |objects, budget| {
                let execution = objects
                    .module_execution(&record)?
                    .expect("evaluating module");
                budget.charge(execution.async_parents.len() + 1)?;
                if matches!(
                    execution.status,
                    ModuleExecutionStatus::Evaluated | ModuleExecutionStatus::Aborted
                ) {
                    return Ok(None);
                }
                Ok(Some(execution.async_parents.clone()))
            })?;
            let Some(parents) = parents else { continue };
            self.finish_async_module(&record, Some(reason.clone()), span)?;
            for parent in parents.into_iter().rev() {
                push(&mut pending, parent, span)?;
            }
        }
        Ok(())
    }

    // GatherAvailableAncestors uses an explicit work list; the resulting ready
    // set is sorted by AsyncEvaluationOrder before any ancestor body executes.
    fn available_async_module_ancestors(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<Vec<(usize, ObjectHandle)>, Error> {
        let mut pending = Vec::new();
        push(&mut pending, record.clone(), span)?;
        let mut ready = Vec::new();
        let mut seen = HashSet::new();
        while let Some(record) = pending.pop() {
            let parents = self.object_work(span, |objects, budget| {
                let execution = objects
                    .module_execution(&record)?
                    .expect("completed dependency");
                budget.charge(execution.async_parents.len() + 1)?;
                Ok(execution.async_parents.clone())
            })?;
            for parent in parents {
                self.tick(span)?;
                if seen.contains(&parent) {
                    continue;
                }
                let available = self.object_work(span, |objects, budget| {
                    budget.charge(2)?;
                    let execution = objects.module_execution(&parent)?.expect("async parent");
                    let root = execution.cycle_root.as_ref().unwrap_or(&parent);
                    let root = objects.module_execution(root)?.expect("async cycle root");
                    if root.error.is_some() || root.host_failure.is_some() {
                        return Ok(None);
                    }
                    let has_tla = objects.module_source(&parent)?.has_top_level_await();
                    let execution = objects.module_execution_mut(&parent)?;
                    debug_assert_eq!(execution.status, ModuleExecutionStatus::EvaluatingAsync);
                    debug_assert!(execution.pending_async_dependencies > 0);
                    execution.pending_async_dependencies -= 1;
                    Ok((execution.pending_async_dependencies == 0).then(|| {
                        (
                            execution.async_order.expect("async evaluation order"),
                            has_tla,
                        )
                    }))
                })?;
                if let Some((order, has_tla)) = available {
                    seen.try_reserve(1).map_err(|_| capacity_error(span))?;
                    seen.insert(parent.clone());
                    push(&mut ready, (order, parent.clone()), span)?;
                    if !has_tla {
                        push(&mut pending, parent, span)?;
                    }
                }
            }
        }
        ready.sort_unstable_by_key(|(order, _)| *order);
        Ok(ready)
    }

    fn fulfill_async_module_execution(
        &mut self,
        record: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        self.finish_async_module(record, None, span)?;
        let ready = self.available_async_module_ancestors(record, span)?;
        for (_, parent) in ready {
            let active = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let execution = objects.module_execution(&parent)?.expect("ready ancestor");
                Ok((
                    execution.status == ModuleExecutionStatus::EvaluatingAsync,
                    objects.module_source(&parent)?.has_top_level_await(),
                ))
            })?;
            if !active.0 {
                continue;
            }
            match self.execute_module_record(&parent, active.1, span) {
                Ok(()) if active.1 => (),
                Ok(()) => self.finish_async_module(&parent, None, span)?,
                Err(error) if error.is_language_exception() => {
                    let reason = self.language_exception_value(error)?;
                    self.reject_async_module_execution(&parent, reason, span)?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    pub(crate) fn resume_module_execution(
        &mut self,
        record: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let result = (|| {
            let active = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                Ok(objects
                    .module_execution(&record)?
                    .expect("async module")
                    .status
                    == ModuleExecutionStatus::EvaluatingAsync)
            })?;
            if !active {
                return Ok(());
            }
            if fulfilled {
                self.fulfill_async_module_execution(&record, span)
            } else {
                self.reject_async_module_execution(&record, value, span)
            }
        })();
        if let Err(error) = result {
            self.abort_module_evaluation(&record, &error)?;
            return Err(error);
        }
        Ok(Value::Undefined)
    }
}

#[cfg(test)]
mod tests;
