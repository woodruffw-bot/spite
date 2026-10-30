//! Explicit Test262 workers and their native report channel.

use super::super::Builtin;
use crate::{AgentControl, Error, ObjectHandle, Realm, Value, object};
use spite_core::Span;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

mod host;
pub(crate) use host::AgentContext;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Method {
    Start,
    Broadcast,
    ReceiveBroadcast,
    GetReport,
    Report,
    Leaving,
    Sleep,
    MonotonicNow,
}

impl Method {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Broadcast => "broadcast",
            Self::ReceiveBroadcast => "receiveBroadcast",
            Self::GetReport => "getReport",
            Self::Report => "report",
            Self::Leaving => "leaving",
            Self::Sleep => "sleep",
            Self::MonotonicNow => "monotonicNow",
        }
    }
    pub(crate) fn length(self) -> f64 {
        match self {
            Self::Broadcast => 2.0,
            Self::Start | Self::Report | Self::Sleep | Self::ReceiveBroadcast => 1.0,
            _ => 0.0,
        }
    }
}

impl Realm {
    pub(super) fn test262_agent_object(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let worker = self
            .test262_agents
            .as_ref()
            .is_some_and(|context| context.worker.is_some());
        let agent = self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let methods = if worker {
            [
                Method::ReceiveBroadcast,
                Method::Report,
                Method::Leaving,
                Method::Sleep,
                Method::MonotonicNow,
            ]
        } else {
            [
                Method::Start,
                Method::Broadcast,
                Method::GetReport,
                Method::Sleep,
                Method::MonotonicNow,
            ]
        };
        for method in methods {
            let function =
                self.new_builtin(function_prototype, Builtin::Test262Agent(method), span)?;
            self.define_builtin_property(
                &agent,
                method.name(),
                Value::Object(function),
                true,
                span,
            )?;
        }
        Ok(agent)
    }

    fn agent_hub(&mut self) -> Arc<host::Hub> {
        if self.test262_agents.is_none() {
            let control = self.agent_control();
            self.test262_agents = Some(AgentContext::main(control));
        }
        self.test262_agents
            .as_ref()
            .expect("agent context")
            .hub
            .clone()
    }

    pub(in crate::function) fn test262_agent_method(
        &mut self,
        method: Method,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        match method {
            Method::Start => {
                let Value::String(source) = arguments.next().unwrap_or(Value::Undefined) else {
                    return Err(Self::exception(
                        crate::ExceptionKind::TypeError,
                        span,
                        "agent.start requires a String",
                    ));
                };
                let hub = self.agent_hub();
                let id = self.object_work(span, |_, budget| Ok(hub.add_worker(budget, span)))??;
                hub.spawn(id, source, self.limits, span)?;
                Ok(Value::Undefined)
            }
            Method::Broadcast => {
                let buffer = arguments.next().unwrap_or(Value::Undefined);
                let data = if let Value::Object(buffer) = buffer {
                    self.object_work(span, |objects, _| {
                        Ok(objects
                            .inspect(&buffer)?
                            .shared_array_buffer_data()
                            .cloned())
                    })?
                } else {
                    None
                };
                let Some(data) = data else {
                    return Err(Self::exception(
                        crate::ExceptionKind::TypeError,
                        span,
                        "broadcast requires a SharedArrayBuffer",
                    ));
                };
                let value = self.primitive(
                    arguments.next().unwrap_or(Value::Undefined),
                    crate::realm_object::Hint::Number,
                    span,
                )?;
                let value = if let Value::BigInt(value) = value {
                    host::BroadcastValue::BigInt(value)
                } else {
                    host::BroadcastValue::Int(
                        crate::value::to_uint32(self.number(value, span)?) as i32
                    )
                };
                let hub = self.agent_hub();
                self.object_work(span, |_, budget| {
                    Ok(hub.broadcast(data, value, budget, span))
                })??;
                Ok(Value::Undefined)
            }
            Method::ReceiveBroadcast => {
                let callback = arguments.next().unwrap_or(Value::Undefined);
                if !self.is_callable(&callback, span)? {
                    return Err(Self::exception(
                        crate::ExceptionKind::TypeError,
                        span,
                        "receiveBroadcast requires a callable",
                    ));
                }
                let context = self.test262_agents.as_ref().expect("worker context");
                let id = context.worker.expect("worker receiver");
                let hub = context.hub.clone();
                let control = self.agent_control();
                let message = self.object_work(span, |_, budget| {
                    Ok(hub.receive(id, &control, budget, span))
                })??;
                // Retrieval is acknowledged before entering user code. Only
                // native bytes and numeric values crossed; make a local wrapper.
                let prototype = self
                    .intrinsics
                    .as_ref()
                    .expect("initialized")
                    .global_metadata
                    .shared_array_buffer_prototype
                    .clone();
                let buffer = self.object_work(span, |objects, budget| {
                    objects.create_shared_array_buffer_from_data(
                        &prototype,
                        message.data.clone(),
                        budget,
                    )
                })?;
                let value = match &message.value {
                    host::BroadcastValue::Int(value) => Value::Number(f64::from(*value)),
                    host::BroadcastValue::BigInt(value) => {
                        Value::BigInt(self.integer_work(span, |budget| {
                            value.add(&spite_bigint::BigInt::default(), budget)
                        })?)
                    }
                };
                self.call(
                    callback,
                    Value::Undefined,
                    vec![Value::Object(buffer), value],
                    span,
                )?;
                Ok(Value::Undefined)
            }
            Method::GetReport => {
                let Some(context) = &self.test262_agents else {
                    return Ok(Value::Null);
                };
                let hub = context.hub.clone();
                let message =
                    self.object_work(span, |_, budget| Ok(hub.report_next(budget, span)))??;
                Ok(message.map_or(Value::Null, Value::String))
            }
            Method::Report => {
                let message = self.string(arguments.next().unwrap_or(Value::Undefined), span)?;
                let hub = self.agent_hub();
                self.object_work(span, |_, budget| Ok(hub.report(message, budget, span)))??;
                Ok(Value::Undefined)
            }
            Method::Leaving => {
                self.agent_control().terminate();
                Ok(Value::Undefined)
            }
            Method::Sleep => {
                let millis = self.number(arguments.next().unwrap_or(Value::Undefined), span)?;
                let millis = if millis.is_nan() {
                    0.0
                } else {
                    millis.max(0.0)
                };
                let timeout = millis
                    .is_finite()
                    .then(|| object::WaitTimeout::new(millis, Instant::now()));
                let owner = self.atomic_waits.get_or_insert_default().owner().clone();
                self.object_work(span, |_, budget| owner.sleep(timeout, budget))?;
                Ok(Value::Undefined)
            }
            Method::MonotonicNow => {
                static ORIGIN: OnceLock<Instant> = OnceLock::new();
                Ok(Value::Number(
                    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0,
                ))
            }
        }
    }

    pub(crate) fn agent_termination(&self, span: Span) -> Error {
        self.test262_agents
            .as_ref()
            .and_then(|context| context.failure(span))
            .unwrap_or_else(|| Error::Host {
                span,
                message: "agent terminated by host".into(),
            })
    }

    /// Stops and joins this Test262 host's workers, reporting native worker errors.
    ///
    /// Call at an idle boundary before accepting a test result. Realm retirement
    /// also stops its workers. Their heap values remain isolated throughout.
    pub fn finish_test262_agents(&mut self) -> Result<(), Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot retire Test262 workers during active execution".into(),
            });
        }
        if self
            .test262_agents
            .as_ref()
            .is_some_and(|context| context.worker.is_some())
        {
            return Err(Error::Host {
                span,
                message: "only the parent can retire Test262 workers".into(),
            });
        }
        self.test262_agents
            .take()
            .map_or(Ok(()), |context| context.finish(span))
    }
}

fn run_worker(
    realm: &mut Realm,
    control: &AgentControl,
    script: &spite_parser::ast::Script,
) -> Result<(), Error> {
    realm.evaluate(script)?;
    loop {
        if control.is_terminated() {
            return Err(realm.agent_termination(Span::new(0, 0)));
        }
        if realm.run_requested_gc(usize::MAX)? || realm.run_next_atomics_wait_job()? {
            continue;
        }
        match realm.run_finalization_cleanup_job()? {
            crate::FinalizationCleanupOutcome::NoJob => (),
            crate::FinalizationCleanupOutcome::Completed => continue,
            crate::FinalizationCleanupOutcome::Reported(error) => return Err(error),
        }
        match realm.run_next_promise_job()? {
            crate::PromiseJobOutcome::Completed => continue,
            crate::PromiseJobOutcome::Reported(error) => return Err(error),
            crate::PromiseJobOutcome::NoJob => (),
        }
        if !realm
            .atomic_waits
            .as_ref()
            .is_some_and(|waits| waits.has_pending())
        {
            return Ok(());
        }
        let delay = realm
            .next_atomics_wait_delay()
            .unwrap_or(Duration::from_millis(10));
        std::thread::sleep(delay.min(Duration::from_millis(10)));
    }
}
