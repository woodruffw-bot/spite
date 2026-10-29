//! InnerModuleEvaluation with persistent cycle roots and async dependencies (16.2.1.6.2).

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
        // Abort bookkeeping is unmetered: a refused job must leave no waiting
        // ancestor able to resume its body, even with its work allowance at zero.
        let mut pending = vec![record.clone()];
        let mut seen = std::collections::HashSet::new();
        while let Some(current) = pending.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            let pending_promise = match self
                .objects
                .module_evaluation(&current)
                .map_err(Error::InvalidObject)?
            {
                Some(Value::Object(promise)) => matches!(
                    self.objects
                        .inspect(&promise)
                        .map_err(Error::InvalidObject)?
                        .promise_state(),
                    Some(crate::PromiseState::Pending)
                ),
                _ => false,
            };
            let execution = self
                .objects
                .module_execution_mut(&current)
                .map_err(Error::InvalidObject)?;
            pending.extend(execution.async_parents.iter().cloned());
            if pending_promise || execution.status != ModuleExecutionStatus::Evaluated {
                execution.status = ModuleExecutionStatus::Aborted;
                execution.error = None;
                execution.host_failure = Some(error.clone());
                execution.async_order = None;
            }
        }
        Ok(())
    }

    pub(super) fn evaluate_module_graph(
        &mut self,
        root: &ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        let catalog = self.module_catalog(root, span)?;
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
                    let record = &catalog.nodes[child].record;
                    let previous = self
                        .objects
                        .module_execution(record)
                        .map_err(Error::InvalidObject)?;
                    let dependency = match previous {
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
                            continue;
                        }
                        Some(execution)
                            if execution.status == ModuleExecutionStatus::Evaluating =>
                        {
                            ancestors[node] = ancestors[node].min(ancestors[child]);
                            record.clone()
                        }
                        Some(execution) => execution
                            .cycle_root
                            .clone()
                            .unwrap_or_else(|| record.clone()),
                    };
                    let execution = self
                        .objects
                        .module_execution(&dependency)
                        .map_err(Error::InvalidObject)?
                        .expect("visited dependency");
                    if let Some(error) = &execution.host_failure {
                        return Err(error.clone());
                    }
                    if let Some(reason) = &execution.error {
                        return Err(Error::Thrown(reason.clone()));
                    }
                    if execution.async_order.is_some() {
                        self.add_async_module_parent(
                            &dependency,
                            &catalog.nodes[node].record,
                            span,
                        )?;
                    }
                    frames.last_mut().expect("current evaluation frame").next += 1;
                    continue;
                }
                let record = &catalog.nodes[node].record;
                let (has_tla, pending) = self.object_work(span, |objects, budget| {
                    budget.charge(1)?;
                    Ok((
                        objects.module_source(record)?.has_top_level_await(),
                        objects
                            .module_execution(record)?
                            .expect("visited module")
                            .pending_async_dependencies,
                    ))
                })?;
                if has_tla || pending > 0 {
                    self.assign_module_async_order(record, span)?;
                    if pending == 0 {
                        self.execute_module_record(record, true, span)?;
                    }
                } else {
                    self.execute_module_record(record, false, span)?;
                }
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
                            execution.status = if execution.async_order.is_some() {
                                ModuleExecutionStatus::EvaluatingAsync
                            } else {
                                ModuleExecutionStatus::Evaluated
                            };
                            execution.cycle_root = Some(record.clone());
                        }
                        Ok(())
                    })?;
                    stack.truncate(start);
                }
                frames.pop();
            }
            Ok(())
        })();
        let settled = (|| match result {
            Ok(())
                if self
                    .objects
                    .module_execution(root)
                    .map_err(Error::InvalidObject)?
                    .expect("evaluated root")
                    .status
                    == ModuleExecutionStatus::EvaluatingAsync =>
            {
                Ok(())
            }
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
                            let record = &catalog.nodes[*member].record;
                            if self
                                .objects
                                .module_execution(record)
                                .map_err(Error::InvalidObject)?
                                .expect("active module")
                                .async_order
                                .is_some()
                            {
                                self.reject_async_module_execution(record, reason.clone(), span)?;
                                continue;
                            }
                            let execution = self
                                .objects
                                .module_execution_mut(record)
                                .map_err(Error::InvalidObject)?;
                            execution.status = ModuleExecutionStatus::Evaluated;
                            execution.error = Some(reason.clone());
                            execution.async_order = None;
                        }
                        self.call(capability.reject, Value::Undefined, vec![reason], span)
                            .map(|_| ())
                    }
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        })();
        if let Err(error) = settled {
            // Host aborts must release bookkeeping even with no budget and
            // preserve an unsettled cached Promise without replaying bodies.
            for (index, node) in catalog.nodes.iter().enumerate() {
                if indices[index].is_some()
                    && self
                        .objects
                        .module_execution(&node.record)
                        .map_err(Error::InvalidObject)?
                        .is_some_and(|execution| {
                            execution.status != ModuleExecutionStatus::Evaluated
                        })
                {
                    self.abort_module_evaluation(&node.record, &error)?;
                }
            }
            self.abort_module_evaluation(root, &error)?;
            return Err(error);
        }
        Ok(capability.promise)
    }
}
