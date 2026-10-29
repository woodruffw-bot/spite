//! Explicit HostLoadImportedModule requests and ContinueDynamicImport jobs.

use super::*;
use crate::function::{PromiseCapability, async_native::AsyncResumeKind};
use spite_parser::ast::ModuleRequest;
use std::collections::VecDeque;

/// One dynamic import waiting for the embedding to supply a loaded Module.
///
/// The request roots its Promise capability and optional referring Module.
/// Complete it once through [`Realm::finish_module_import`]. Dropping it leaves
/// the JavaScript Promise pending.
#[derive(Debug)]
pub struct ModuleImportRequest {
    request: ModuleRequest,
    referrer: Option<ModuleHandle>,
    state: crate::object::Root,
}

#[cfg(test)]
mod tests;

impl ModuleImportRequest {
    /// The converted specifier and canonical string-valued import attributes.
    pub fn request(&self) -> &ModuleRequest {
        &self.request
    }

    /// The defining Module, or None for Script-originated imports.
    pub fn referrer(&self) -> Option<&ModuleHandle> {
        self.referrer.as_ref()
    }
}

impl Realm {
    /// Enables dynamic-import requests for an explicit embedding loader.
    ///
    /// The embedding must poll [`Self::take_module_import_request`], provide
    /// dependency graphs and call [`Self::finish_module_import`]. Without this
    /// opt-in, valid import calls report Unsupported after operand validation.
    pub fn enable_module_imports(&mut self) {
        self.module_import_requests
            .get_or_insert_with(VecDeque::new);
    }

    /// Takes the oldest pending host request without running JavaScript.
    pub fn take_module_import_request(&mut self) -> Option<ModuleImportRequest> {
        self.module_import_requests.as_mut()?.pop_front()
    }

    pub(crate) fn queue_module_import(
        &mut self,
        request: ModuleRequest,
        capability: PromiseCapability,
        span: Span,
    ) -> Result<(), Error> {
        let Some(queue) = &self.module_import_requests else {
            return Err(Self::unsupported(
                span,
                "dynamic module loading is not implemented",
            ));
        };
        let work = if queue.len() == queue.capacity() {
            queue.len() + 1
        } else {
            1
        };
        self.object_work(span, |_, budget| budget.charge(work))?;
        self.module_import_requests
            .as_mut()
            .expect("enabled loader")
            .try_reserve(1)
            .map_err(|_| Error::Limit {
                span,
                message: "module import request capacity exceeded".into(),
            })?;
        let referrer = if let Some(record) = self.current_module_record(span)? {
            Some(ModuleHandle {
                root: self.object_work(span, |objects, budget| objects.root(&record, budget))?,
            })
        } else {
            None
        };
        let state = self.object_work(span, |objects, budget| {
            let state = objects.create_module_import(capability, budget)?;
            objects.root(&state, budget)
        })?;
        self.module_import_requests
            .as_mut()
            .expect("enabled loader")
            .push_back(ModuleImportRequest {
                request,
                referrer,
                state,
            });
        Ok(())
    }

    /// Completes a host request with a loaded Module or a language rejection.
    ///
    /// Supply dependencies through [`Self::load_module_graph`] before success.
    /// Linking and evaluation begin in a later Promise job. Failure reasons use
    /// the original JavaScript value. Host failures return directly and leave
    /// the import Promise pending. All handles must belong to this Realm.
    pub fn finish_module_import(
        &mut self,
        request: ModuleImportRequest,
        completion: Result<ModuleHandle, Value>,
    ) -> Result<(), Error> {
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let span = Span::new(0, 0);
        let state = request.state.handle();
        // Embedding handles are untrusted at the public boundary; object_work
        // assumes that heap identity and liveness have already been checked.
        self.objects.inspect(state).map_err(Error::InvalidObject)?;
        match &completion {
            Ok(module) => {
                self.objects
                    .module_source(module.root.handle())
                    .map_err(Error::InvalidObject)?;
            }
            Err(Value::Object(reason)) => {
                self.objects.inspect(reason).map_err(Error::InvalidObject)?;
            }
            Err(_) => (),
        }
        match completion {
            Err(reason) => {
                let data = self.object_work(span, |objects, budget| {
                    objects.take_module_import(state, budget)
                })?;
                self.call(data.capability.reject, Value::Undefined, vec![reason], span)?;
            }
            Ok(module) => {
                self.object_work(span, |objects, budget| {
                    objects.set_module_import_target(state, module.root.handle(), budget)
                })?;
                let load = self.intrinsic_promise_capability(span)?;
                let Value::Object(promise) = &load.promise else {
                    unreachable!("intrinsic loading Promise")
                };
                self.react_to_native_promise(
                    promise,
                    state,
                    AsyncResumeKind::ModuleImportLoad,
                    span,
                )?;
                self.call(load.resolve, Value::Undefined, vec![Value::Undefined], span)?;
            }
        }
        Ok(())
    }

    pub(crate) fn resume_module_import(
        &mut self,
        state: ObjectHandle,
        fulfilled: bool,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let data = self.object_work(span, |objects, budget| {
            objects.take_module_import(&state, budget)
        })?;
        if fulfilled {
            self.continue_module_import(
                &data.module.expect("completed host import"),
                data.capability,
                span,
            )?;
        } else {
            self.call(data.capability.reject, Value::Undefined, vec![value], span)?;
        }
        Ok(Value::Undefined)
    }
}
