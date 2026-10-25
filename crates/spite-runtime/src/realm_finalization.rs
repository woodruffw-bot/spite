//! Explicit host cleanup jobs for FinalizationRegistry (9.12, 26.2).

use crate::{Error, Realm, Value, object};
use spite_core::Span;

/// The result of one explicitly scheduled FinalizationRegistry cleanup job.
#[derive(Debug)]
pub enum FinalizationCleanupOutcome {
    /// No cleanup job was queued. The Realm was not initialized or otherwise changed.
    NoJob,
    /// The selected cleanup invocation completed normally.
    Completed,
    /// The host reports a JavaScript throw or host failure from the started job.
    ///
    /// A cell removed before its callback is never retried. Object values in
    /// this error are unrooted after return, as with evaluation errors; retain
    /// them with [`Realm::root_value`] before explicitly collecting.
    Reported(Error),
}

impl Realm {
    /// Runs at most one existing FinalizationRegistry cleanup job at an idle host turn.
    ///
    /// This does not collect or clear WeakRef's kept objects. The embedding must
    /// first finish its whole synchronous sequence, call [`Self::clear_kept_objects`],
    /// and explicitly collect before scheduling a later cleanup turn. Delivery
    /// order and eventual delivery are not guaranteed.
    ///
    /// Active evaluation, a call, or another host job returns a host error
    /// before consuming any work. An empty queue is inert, even on an uninitialized
    /// Realm. Other outer errors are preflight failures that preserve queue and
    /// cells. Once started, the job reports its first abrupt completion through
    /// [`FinalizationCleanupOutcome::Reported`], preserving later pending cells.
    ///
    /// Callbacks receive undefined as this and exactly one held-value argument.
    /// They share one optional work allowance, including host evaluation/property
    /// reentry, and kept objects survive the entire job and later reentry until
    /// the embedding explicitly ends that sequence. No per-callback allowance or
    /// automatic checkpoint is introduced. Realms remain independent execution domains.
    pub fn run_finalization_cleanup_job(&mut self) -> Result<FinalizationCleanupOutcome, Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot run a finalization cleanup job during active execution".into(),
            });
        }
        if !self.objects.has_finalization_cleanup_job() {
            return Ok(FinalizationCleanupOutcome::NoJob);
        }

        // Only this entry initializes the job allowance. All early failures
        // restore the caller's allowance; prepare never removes cells/captures.
        let caller_work = self.remaining_steps;
        self.remaining_steps = self.limits.max_steps;
        let prepared = (|| -> Result<_, Error> {
            let job = self
                .object_work(span, |objects, budget| {
                    objects.prepare_finalization_cleanup_job(budget)
                })?
                .expect("the exclusive Realm retains its queued job");
            let (arguments, cell) = self.prepare_finalization_cleanup_call(&job, span)?;
            Ok((job, arguments, cell))
        })();
        let (job, mut arguments, mut cell) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                self.remaining_steps = caller_work;
                return Err(error);
            }
        };

        // A real queued registry was created in this Realm, so its global
        // environment exists. Retain the existing scope roots throughout the
        // job; ordinary Call restores its own lexical/function context.
        let global = self
            .scopes
            .first()
            .expect("queued registry has a Realm")
            .clone();
        let caller_variable = self.variable_environment.replace(global);
        let caller_strict = std::mem::replace(&mut self.strict, false);
        let caller_scope_depth = self.scopes.len();
        self.job_active = true;
        let outcome = loop {
            let Some(prepared_cell) = cell else {
                break FinalizationCleanupOutcome::Completed;
            };
            // All fallible copying, removal work, root metadata and real
            // argument capacity precede this commit. The queue roots callback
            // and registry; commit installs the held-object root before removal.
            let call = self.objects.commit_finalization_cleanup_cell(prepared_cell);
            arguments.push(call.held);
            let result = self.call(
                Value::Object(call.callback),
                Value::Undefined,
                arguments,
                span,
            );
            // Both normal and abrupt calls retain the transient held root until
            // Call has returned. No object borrow or Symbol gate spans JavaScript.
            self.objects.finish_finalization_cleanup_argument();
            if let Err(error) = result {
                break FinalizationCleanupOutcome::Reported(error);
            }
            // Callback mutations can unregister any remaining cell. Re-select
            // from the current registry; no captured list survives a callback.
            match self.prepare_finalization_cleanup_call(&job, span) {
                Ok((next_arguments, next_cell)) => {
                    arguments = next_arguments;
                    cell = next_cell;
                }
                Err(error) => break FinalizationCleanupOutcome::Reported(error),
            }
        };
        // The pre-reserved capture remains queued when cleared cells remain.
        // This finish is infallible even after the optional allowance is spent.
        self.objects.finish_finalization_cleanup_job(job);
        self.job_active = false;
        self.strict = caller_strict;
        self.variable_environment = caller_variable;
        self.remaining_steps = caller_work;
        debug_assert_eq!(self.scopes.len(), caller_scope_depth);
        debug_assert_eq!((self.call_depth, self.evaluation_depth), (0, 0));
        Ok(outcome)
    }

    fn prepare_finalization_cleanup_call(
        &mut self,
        job: &object::FinalizationCleanupJob,
        span: Span,
    ) -> Result<(Vec<Value>, Option<object::FinalizationCleanupCell>), Error> {
        self.object_work(span, |_, budget| budget.charge(1))?;
        let mut arguments = Vec::new();
        arguments.try_reserve_exact(1).map_err(|_| Error::Limit {
            span,
            message: "finalization cleanup argument capacity exceeded".into(),
        })?;
        let cell = self.object_work(span, |objects, budget| {
            objects.prepare_finalization_cleanup_cell(job, budget)
        })?;
        // An empty pending job performs no Call, so a zero argument allowance
        // need not reject it. A selected cell is still untouched at this point.
        if cell.is_some() {
            self.check_argument_count(1, span)?;
        }
        Ok((arguments, cell))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_job_is_inert_and_active_refusal_preserves_uninitialized_execution_state() {
        let mut realm = Realm {
            remaining_steps: Some(11),
            ..Realm::default()
        };
        assert!(matches!(
            realm.run_finalization_cleanup_job(),
            Ok(FinalizationCleanupOutcome::NoJob)
        ));
        assert!(realm.intrinsics.is_none());
        assert!(realm.scopes.is_empty());
        assert_eq!(realm.remaining_steps, Some(11));
        for (call_depth, evaluation_depth, job_active) in
            [(1, 0, false), (0, 1, false), (0, 0, true)]
        {
            realm.call_depth = call_depth;
            realm.evaluation_depth = evaluation_depth;
            realm.job_active = job_active;
            assert!(matches!(
                realm.run_finalization_cleanup_job(),
                Err(Error::Host { .. })
            ));
            assert!(matches!(
                realm.clear_kept_objects(),
                Err(Error::Host { .. })
            ));
            assert!(realm.intrinsics.is_none());
            assert!(realm.scopes.is_empty());
            assert_eq!(realm.remaining_steps, Some(11));
            assert_eq!(
                (realm.call_depth, realm.evaluation_depth, realm.job_active),
                (call_depth, evaluation_depth, job_active)
            );
        }
    }

    #[test]
    fn cleanup_reentry_keeps_the_job_allowance_for_evaluation_and_host_get() {
        let mut realm = Realm::default();
        let object = realm.eval("({get value(){throw 7;}})").unwrap();
        let key = spite_core::JsString::from("value");
        // Setup is unrestricted. An explicitly exhausted job allowance must not
        // be replaced by the separate, positive allowance on any host entry.
        realm.limits.max_steps = Some(1000);
        realm.remaining_steps = Some(0);
        realm.job_active = true;
        realm.strict = true;
        realm.variable_environment = None;
        assert!(matches!(realm.eval("0"), Err(Error::Limit { .. })));
        assert_eq!(realm.remaining_steps, Some(0));
        assert!(realm.strict);
        assert!(realm.variable_environment.is_none());
        assert!(matches!(
            realm.read_property(&object, &key),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.remaining_steps, Some(0));
        let thrown = Error::Thrown(Value::Number(7.0));
        assert!(matches!(
            realm.exception_value(&thrown),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.remaining_steps, Some(0));
        assert!(realm.strict);
        assert!(realm.variable_environment.is_none());
        assert!(realm.job_active);

        // Idle host entries still receive their ordinary fresh allowance.
        realm.job_active = false;
        assert_eq!(realm.eval("1"), Ok(Value::Number(1.0)));
        assert!(realm.remaining_steps.is_some_and(|remaining| remaining > 0));
        realm.remaining_steps = Some(0);
        assert_eq!(realm.read_property(&object, &key), Err(thrown.clone()));
        assert!(realm.remaining_steps.is_some_and(|remaining| remaining > 0));
        realm.remaining_steps = Some(0);
        assert_eq!(realm.exception_value(&thrown), Ok(Some(Value::Number(7.0))));
        assert!(realm.remaining_steps.is_some_and(|remaining| remaining > 0));
    }
}
