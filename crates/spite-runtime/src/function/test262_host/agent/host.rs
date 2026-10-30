//! Native worker lifetime and string reports, without foreign heap references.

use crate::{AgentControl, Error, Limits, Realm, object};
use spite_core::{DiagnosticKind, JsString, Span};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

#[derive(Clone, Debug)]
enum Failure {
    Unsupported(String),
    Limit(String),
    Host(String),
}

impl Failure {
    fn from_error(error: &Error) -> Self {
        let message = format!("Test262 worker: {error}");
        match error {
            Error::Unsupported { .. } => Self::Unsupported(message),
            Error::Limit { .. } => Self::Limit(message),
            Error::Parse(error) if error.kind == DiagnosticKind::Unsupported => {
                Self::Unsupported(message)
            }
            Error::Parse(error) if error.kind == DiagnosticKind::Limit => Self::Limit(message),
            _ => Self::Host(message),
        }
    }

    fn error(&self, span: Span) -> Error {
        match self {
            Self::Unsupported(message) => Realm::unsupported(span, message.clone()),
            Self::Limit(message) => Error::Limit {
                span,
                message: message.clone(),
            },
            Self::Host(message) => Error::Host {
                span,
                message: message.clone(),
            },
        }
    }
}

#[derive(Debug)]
struct Worker {
    control: Option<AgentControl>,
    finished: bool,
}

#[derive(Debug, Default)]
struct State {
    workers: Vec<Worker>,
    threads: Vec<JoinHandle<()>>,
    reports: VecDeque<JsString>,
    failure: Option<Failure>,
    closed: bool,
}

#[derive(Debug)]
pub(super) struct Hub {
    state: Mutex<State>,
    parent: AgentControl,
}

#[derive(Debug)]
pub(crate) struct AgentContext {
    pub(super) hub: Arc<Hub>,
    pub(super) worker: Option<usize>,
}

impl AgentContext {
    pub(super) fn main(control: AgentControl) -> Self {
        Self {
            hub: Arc::new(Hub {
                state: Mutex::new(State::default()),
                parent: control,
            }),
            worker: None,
        }
    }

    pub(crate) fn failure(&self, span: Span) -> Option<Error> {
        self.hub.failure(span)
    }

    pub(crate) fn finish(self, span: Span) -> Result<(), Error> {
        self.hub.shutdown();
        self.failure(span).map_or(Ok(()), Err)
    }
}

impl Drop for AgentContext {
    fn drop(&mut self) {
        if let Some(id) = self.worker {
            if std::thread::panicking() {
                self.hub
                    .fail(Failure::Host("Test262 worker panicked".into()));
            }
            self.hub.recover().workers[id].finished = true;
        } else {
            self.hub.shutdown();
        }
    }
}

impl Hub {
    fn lock(&self, span: Span) -> Result<MutexGuard<'_, State>, Error> {
        self.state.lock().map_err(|_| Error::Host {
            span,
            message: "Test262 agent host lock is poisoned".into(),
        })
    }

    // Recovery only records failure or retires threads; JS never resumes from it.
    fn recover(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn failure(&self, span: Span) -> Option<Error> {
        self.recover()
            .failure
            .as_ref()
            .map(|failure| failure.error(span))
    }

    fn fail(&self, failure: Failure) {
        self.recover().failure.get_or_insert(failure);
        self.parent.terminate();
    }

    pub(super) fn add_worker(
        &self,
        budget: &mut object::Budget,
        span: Span,
    ) -> Result<usize, Error> {
        let mut state = self.lock(span)?;
        if state.closed {
            return Err(Error::Host {
                span,
                message: "Test262 agent host is closed".into(),
            });
        }
        let work = state.workers.len() + state.threads.len() + 2;
        charge(budget, work, span)?;
        state.workers.try_reserve(1).map_err(|_| capacity(span))?;
        state.threads.try_reserve(1).map_err(|_| capacity(span))?;
        let id = state.workers.len();
        state.workers.push(Worker {
            control: None,
            finished: false,
        });
        Ok(id)
    }

    pub(super) fn spawn(
        self: &Arc<Self>,
        id: usize,
        source: JsString,
        limits: Limits,
        span: Span,
    ) -> Result<(), Error> {
        let (ready, started) = std::sync::mpsc::channel();
        let hub = self.clone();
        let thread = std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(move || {
            let mut realm = Realm::new_suspending(limits);
            let control = realm.agent_control();
            hub.recover().workers[id].control = Some(control.clone());
            realm.test262_agents = Some(AgentContext {hub: hub.clone(), worker: Some(id)});
            if let Err(error) = realm.install_test262_host() {
                hub.fail(Failure::from_error(&error));
                return;
            }
            realm.remaining_steps = realm.limits.max_steps;
            let script = match realm.prepare_host_script(&source, Span::new(0, 0)) {
                Ok(script) => script,
                Err(error) => {
                    hub.fail(Failure::from_error(&error));
                    return;
                }
            };
            if ready.send(()).is_err() {
                control.terminate();
            }
            if let Err(error) = super::run_worker(&mut realm, &control, &script)
                && !matches!(&error, Error::Host {message, ..} if message == "agent terminated by host") {
                    hub.fail(Failure::from_error(&error));
                }
        }).map_err(|error| {
            self.recover().workers[id].finished = true;
            Error::Host {span, message: format!("cannot start Test262 worker: {error}")}
        })?;
        // Space was reserved before spawning. Retain every handle even if a
        // worker panicked and poisoned the state before this insertion.
        self.recover().threads.push(thread);
        started.recv().map_err(|_| {
            self.failure(span).unwrap_or_else(|| Error::Host {
                span,
                message: "Test262 worker failed before startup".into(),
            })
        })
    }

    pub(super) fn report(
        &self,
        message: JsString,
        budget: &mut object::Budget,
        span: Span,
    ) -> Result<(), Error> {
        let mut state = self.lock(span)?;
        charge(
            budget,
            if state.reports.len() == state.reports.capacity() {
                state.reports.len() + 1
            } else {
                1
            },
            span,
        )?;
        state.reports.try_reserve(1).map_err(|_| capacity(span))?;
        state.reports.push_back(message);
        Ok(())
    }

    pub(super) fn report_next(
        &self,
        budget: &mut object::Budget,
        span: Span,
    ) -> Result<Option<JsString>, Error> {
        charge(budget, 1, span)?;
        Ok(self.lock(span)?.reports.pop_front())
    }

    fn shutdown(&self) {
        let threads = {
            let mut state = self.recover();
            state.closed = true;
            for worker in &state.workers {
                if !worker.finished
                    && let Some(control) = &worker.control
                {
                    control.terminate()
                }
            }
            std::mem::take(&mut state.threads)
        };
        for thread in threads {
            if thread.join().is_err() {
                self.fail(Failure::Host("Test262 worker panicked".into()));
            }
        }
    }
}

fn capacity(span: Span) -> Error {
    Error::Limit {
        span,
        message: "Test262 agent host capacity exceeded".into(),
    }
}

fn charge(budget: &mut object::Budget, amount: usize, span: Span) -> Result<(), Error> {
    budget.charge(amount).map_err(|error| Error::Limit {
        span,
        message: error.to_string(),
    })
}
