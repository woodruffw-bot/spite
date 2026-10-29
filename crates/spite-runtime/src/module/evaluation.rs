//! Synchronous InnerModuleEvaluation with persistent cycle roots (16.2.1.6.2).

use super::*;
use crate::object::ModuleExecutionStatus;

struct Frame {
    node: usize,
    next: usize,
}

#[cfg(test)]
mod tests;

fn reserved<T>(count: usize, span: Span) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| Error::Limit {
        span,
        message: "module evaluation capacity exceeded".into(),
    })?;
    Ok(values)
}

impl Realm {
    pub(crate) fn abort_module_evaluation(
        &mut self,
        record: &ObjectHandle,
        error: &Error,
    ) -> Result<(), Error> {
        debug_assert!(!error.is_language_exception());
        let execution = self
            .objects
            .module_execution_mut(record)
            .map_err(Error::InvalidObject)?;
        execution.status = ModuleExecutionStatus::Aborted;
        execution.error = None;
        execution.host_failure = Some(error.clone());
        Ok(())
    }

    pub(super) fn evaluate_sync_module_graph(
        &mut self,
        root: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let catalog = self.module_catalog(root, span)?;
        // Async dependency scheduling is a separate boundary. Refuse before
        // allocating evaluation state or entering any graph body.
        for node in &catalog.nodes {
            let asynchronous = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                Ok(objects.module_source(&node.record)?.has_top_level_await())
            })?;
            if asynchronous {
                return Err(Self::unsupported(
                    span,
                    "asynchronous module dependency evaluation is not implemented",
                ));
            }
        }
        let count = catalog.nodes.len();
        let mut indices = reserved(count, span)?;
        indices.resize(count, None);
        let mut ancestors = reserved(count, span)?;
        ancestors.resize(count, 0usize);
        let mut stack = reserved(count, span)?;
        let mut frames = reserved(count, span)?;
        self.object_work(span, |_, budget| {
            budget.charge(
                count
                    .checked_mul(4)
                    .ok_or(crate::object::Error::WorkLimit)?,
            )
        })?;
        let capability = self.intrinsic_promise_capability(span)?;
        let previous = self
            .objects
            .module_execution(root)
            .map_err(Error::InvalidObject)?
            .map(|execution| (execution.status, execution.error.clone()));
        self.object_work(span, |objects, budget| {
            objects.store_module_top_level_capability(root, capability.clone(), budget)
        })?;
        if let Some((status, error)) = previous {
            if status == ModuleExecutionStatus::Evaluated
                && let Err(error) = self.call(
                    if error.is_some() {
                        capability.reject
                    } else {
                        capability.resolve
                    },
                    Value::Undefined,
                    vec![error.unwrap_or(Value::Undefined)],
                    span,
                )
            {
                self.abort_module_evaluation(root, &error)?;
                return Err(error);
            }
            return Ok(capability.promise);
        }
        stack.push(0usize);
        frames.push(Frame { node: 0, next: 0 });
        indices[0] = Some(0);
        let mut next_index = 1;
        let result = (|| {
            while let Some(frame) = frames.last_mut() {
                self.tick(span)?;
                let node = frame.node;
                if let Some(&child) = catalog.nodes[node].dependencies.get(frame.next) {
                    frame.next += 1;
                    let record = &catalog.nodes[child].record;
                    match self
                        .objects
                        .module_execution(record)
                        .map_err(Error::InvalidObject)?
                    {
                        None => {
                            self.object_work(span, |objects, budget| {
                                objects.start_module_execution(record, budget)
                            })?;
                            indices[child] = Some(next_index);
                            ancestors[child] = next_index;
                            next_index += 1;
                            stack.push(child);
                            frames.push(Frame {
                                node: child,
                                next: 0,
                            });
                        }
                        Some(execution)
                            if execution.status == ModuleExecutionStatus::Evaluating =>
                        {
                            ancestors[node] = ancestors[node].min(ancestors[child])
                        }
                        Some(execution) => {
                            let root = execution.cycle_root.as_ref().unwrap_or(record);
                            let execution = self
                                .objects
                                .module_execution(root)
                                .map_err(Error::InvalidObject)?
                                .expect("completed dependency");
                            if let Some(error) = &execution.host_failure {
                                return Err(error.clone());
                            }
                            if let Some(reason) = &execution.error {
                                return Err(Error::Thrown(reason.clone()));
                            }
                        }
                    }
                    continue;
                }
                let record = &catalog.nodes[node].record;
                let source = self
                    .objects
                    .module_source(record)
                    .map_err(Error::InvalidObject)?
                    .clone();
                let environment = self
                    .objects
                    .module_environment(record)
                    .map_err(Error::InvalidObject)?
                    .expect("linked module");
                self.execute_sync_module_body(
                    source.body(),
                    environment,
                    module_default_index(&source),
                    span,
                )?;
                if ancestors[node] == indices[node].expect("visited module") {
                    let start = stack
                        .iter()
                        .rposition(|candidate| *candidate == node)
                        .expect("active evaluation component");
                    self.object_work(span, |objects, budget| {
                        budget.charge(stack.len() - start)?;
                        for member in &stack[start..] {
                            let execution =
                                objects.module_execution_mut(&catalog.nodes[*member].record)?;
                            execution.status = ModuleExecutionStatus::Evaluated;
                            execution.cycle_root = Some(record.clone());
                        }
                        Ok(())
                    })?;
                    stack.truncate(start);
                }
                frames.pop();
                if let Some(parent) = frames.last()
                    && self
                        .objects
                        .module_execution(record)
                        .map_err(Error::InvalidObject)?
                        .expect("visited module")
                        .status
                        == ModuleExecutionStatus::Evaluating
                {
                    ancestors[parent.node] = ancestors[parent.node].min(ancestors[node]);
                }
            }
            Ok(())
        })();
        let settled = match result {
            Ok(()) => self
                .call(
                    capability.resolve,
                    Value::Undefined,
                    vec![Value::Undefined],
                    span,
                )
                .map(|_| ()),
            Err(error) if error.is_language_exception() => {
                match self.language_exception_value(error) {
                    Ok(reason) => {
                        for member in &stack {
                            let execution = self
                                .objects
                                .module_execution_mut(&catalog.nodes[*member].record)
                                .map_err(Error::InvalidObject)?;
                            execution.status = ModuleExecutionStatus::Evaluated;
                            execution.error = Some(reason.clone());
                        }
                        self.call(capability.reject, Value::Undefined, vec![reason], span)
                            .map(|_| ())
                    }
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };
        if let Err(error) = settled {
            // Host aborts must release bookkeeping even with no budget and
            // preserve an unsettled cached Promise without replaying bodies.
            for record in stack
                .iter()
                .map(|node| &catalog.nodes[*node].record)
                .chain(std::iter::once(root))
            {
                self.abort_module_evaluation(record, &error)?;
            }
            return Err(error);
        }
        Ok(capability.promise)
    }
}
