//! Explicit collection requests serviced only between JavaScript executions.

use super::PromiseCapability;
use crate::{Error, Realm, Value, object};
use spite_core::Span;

#[derive(Debug)]
pub(crate) struct GcRequest {
    capability: PromiseCapability,
    _roots: [object::Root; 3],
}

impl Realm {
    pub(super) fn request_host_gc(&mut self, span: Span) -> Result<Value, Error> {
        let capacity_work = if self.gc_requests.len() == self.gc_requests.capacity() {
            self.gc_requests.len() + 1
        } else {
            1
        };
        self.object_work(span, |_, budget| budget.charge(capacity_work))?;
        self.gc_requests.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "collection request capacity exceeded".into(),
        })?;
        let capability = self.intrinsic_promise_capability(span)?;
        let roots = [
            self.root_gc_value(&capability.promise, span)?,
            self.root_gc_value(&capability.resolve, span)?,
            self.root_gc_value(&capability.reject, span)?,
        ];
        let promise = capability.promise.clone();
        self.gc_requests.push_back(GcRequest {
            capability,
            _roots: roots,
        });
        Ok(promise)
    }

    fn root_gc_value(&mut self, value: &Value, span: Span) -> Result<object::Root, Error> {
        let Value::Object(handle) = value else {
            unreachable!("intrinsic capability objects")
        };
        self.object_work(span, |objects, budget| objects.root(handle, budget))
    }

    /// Reports pending collection requests from the explicitly installed host.
    pub fn has_requested_gc(&self) -> bool {
        !self.gc_requests.is_empty()
    }

    /// Services pending host collection requests between JavaScript executions.
    ///
    /// This maintenance job clears kept objects, collects, and then fulfills the
    /// request Promises. Retain embedding values with [`Self::root_value`] before
    /// calling, as for [`Self::collect`]. Active execution is rejected. Evaluation
    /// and allocation never collect automatically.
    ///
    /// `max_work` bounds collection; the whole maintenance job also uses the
    /// configured optional job allowance. Collection refusal preserves pending
    /// requests. Later host failures can leave response Promises unsettled and
    /// never become JavaScript rejections. An empty queue returns false.
    pub fn run_requested_gc(&mut self, max_work: usize) -> Result<bool, Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot service collection during active execution".into(),
            });
        }
        if self.gc_requests.is_empty() {
            return Ok(false);
        }
        self.clear_kept_objects()?;
        let caller_work = self.remaining_steps;
        self.remaining_steps = self.limits.max_steps;
        self.job_active = true;
        let result = (|| {
            let budget = self.remaining_steps.unwrap_or(usize::MAX).min(max_work);
            let collected = self.collect(budget).map_err(|error| match error {
                spite_heap::Error::Capacity | spite_heap::Error::Limit => Error::Limit {
                    span,
                    message: error.to_string(),
                },
                _ => Error::Host {
                    span,
                    message: error.to_string(),
                },
            })?;
            if let Some(remaining) = &mut self.remaining_steps {
                *remaining -= collected.work_used;
            }
            // Root guards remain live throughout collection and settlement.
            // Resolving with undefined queues reactions without calling user code.
            let requests = std::mem::take(&mut self.gc_requests);
            for request in requests {
                self.call(
                    request.capability.resolve,
                    Value::Undefined,
                    vec![Value::Undefined],
                    span,
                )?;
            }
            Ok(true)
        })();
        self.job_active = false;
        self.remaining_steps = caller_work;
        result
    }
}
