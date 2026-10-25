//! Branded Promise state and retained settlement values (27.2.1).

use super::{Budget, Error, Objects, OrdinaryObject};
use crate::Value;
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(crate) enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug)]
pub(super) struct PromiseData {
    state: PromiseState,
}

impl PromiseData {
    pub(super) fn result(&self) -> Option<&Value> {
        match &self.state {
            PromiseState::Pending => None,
            PromiseState::Fulfilled(value) | PromiseState::Rejected(value) => Some(value),
        }
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(
            self.result()
                .and_then(|value| value.trace().next().flatten()),
        )
    }
}

impl OrdinaryObject {
    pub(crate) fn promise_state(&self) -> Option<&PromiseState> {
        self.promise.as_ref().map(|data| &data.state)
    }
}

impl Objects {
    pub(crate) fn create_promise(&mut self, prototype: &Handle) -> Result<Handle, Error> {
        let promise = self.create(Some(prototype))?;
        self.object_mut(&promise)?.promise = Some(Box::new(PromiseData {
            state: PromiseState::Pending,
        }));
        Ok(promise)
    }

    pub(crate) fn settle_promise(
        &mut self,
        promise: &Handle,
        fulfilled: bool,
        result: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        if let Value::Object(value) = &result {
            self.inspect(value)?;
        }
        budget.value(&result)?;
        let data = self
            .object_mut(promise)?
            .promise
            .as_mut()
            .ok_or(Error::WrongKind)?;
        if matches!(data.state, PromiseState::Pending) {
            data.state = if fulfilled {
                PromiseState::Fulfilled(result)
            } else {
                PromiseState::Rejected(result)
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error as RuntimeError, Limits, PromiseJobOutcome, Realm};
    use spite_core::JsString;

    fn promise(realm: &mut Realm, source: &str) -> Handle {
        let Value::Object(value) = realm.eval(source).unwrap() else {
            panic!("Promise object")
        };
        value
    }

    fn fulfilled(realm: &Realm, promise: &Handle, expected: Value) {
        let Some(PromiseState::Fulfilled(value)) =
            realm.objects.inspect(promise).unwrap().promise_state()
        else {
            panic!("fulfilled")
        };
        assert_eq!(value, &expected);
    }

    fn rejected(realm: &Realm, promise: &Handle, expected: Value) {
        let Some(PromiseState::Rejected(value)) =
            realm.objects.inspect(promise).unwrap().promise_state()
        else {
            panic!("rejected")
        };
        assert_eq!(value, &expected);
    }

    #[test]
    fn first_resolving_call_wins_and_later_executor_throws_are_ignored() {
        let mut realm = Realm::default();
        let p = promise(
            &mut realm,
            "new Promise((resolve,reject)=>{resolve();reject(1)})",
        );
        fulfilled(&realm, &p, Value::Undefined);
        let p = promise(
            &mut realm,
            "new Promise((resolve,reject)=>{reject();resolve(1)})",
        );
        rejected(&realm, &p, Value::Undefined);
        let p = promise(
            &mut realm,
            "new Promise((resolve,reject)=>{resolve(7);reject(8);resolve(9);throw 10})",
        );
        fulfilled(&realm, &p, Value::Number(7.0));
        let p = promise(
            &mut realm,
            "new Promise((resolve,reject)=>{reject(8);resolve(9);throw 10})",
        );
        rejected(&realm, &p, Value::Number(8.0));
        let p = promise(&mut realm, "new Promise(()=>{throw 10})");
        rejected(&realm, &p, Value::Number(10.0));
    }

    #[test]
    fn then_lookup_is_synchronous_cached_and_uses_a_fresh_resolving_pair_in_the_job() {
        let mut realm = Realm::default();
        let p = promise(
            &mut realm,
            "let log='',thenable={get then(){log+='g';return function(resolve,reject){if(this!==thenable||arguments.length!==2)throw 1;log+='t';resolve(42);reject(43);throw 44}}};let p=new Promise(resolve=>{resolve(thenable);log+='e'});Object.defineProperty(thenable,'then',{value(){throw 45}});p",
        );
        assert_eq!(realm.eval("log"), Ok(Value::String("ge".into())));
        assert!(matches!(
            realm.objects.inspect(&p).unwrap().promise_state(),
            Some(PromiseState::Pending)
        ));
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        fulfilled(&realm, &p, Value::Number(42.0));
        assert_eq!(realm.eval("log"), Ok(Value::String("get".into())));
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::NoJob)
        ));
    }

    #[test]
    fn getter_and_then_call_errors_reject_while_noncallable_then_fulfills() {
        let mut realm = Realm::default();
        let p = promise(
            &mut realm,
            "new Promise(resolve=>resolve({get then(){throw 17}}))",
        );
        rejected(&realm, &p, Value::Number(17.0));
        let p = promise(
            &mut realm,
            "new Promise(resolve=>resolve({then(){throw 18}}))",
        );
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        rejected(&realm, &p, Value::Number(18.0));
        let p = promise(&mut realm, "let value={then:1};Promise.resolve(value)");
        let value = realm.eval("value").unwrap();
        fulfilled(&realm, &p, value);
    }

    #[test]
    fn self_resolution_rejects_with_an_intrinsic_type_error() {
        let mut realm = Realm::default();
        let p = promise(
            &mut realm,
            "let resolve,p=new Promise(r=>{resolve=r});resolve(p);p",
        );
        let Some(PromiseState::Rejected(value)) =
            realm.objects.inspect(&p).unwrap().promise_state()
        else {
            panic!("rejected")
        };
        let value = value.clone();
        assert_eq!(
            realm.read_property(&value, &JsString::from("name")),
            Ok(Value::String("TypeError".into()))
        );
    }

    #[test]
    fn queued_jobs_and_settled_symbols_survive_collection_without_leaking_cycles() {
        let mut realm = Realm::default();
        realm.eval("let calls=0;(()=>{let thenable={then(resolve){calls++;resolve(1)}};Promise.resolve(thenable)})();").unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        assert_eq!(realm.eval("calls"), Ok(Value::Number(1.0)));
        realm.eval("let holder=(()=>{let symbol=Symbol();return {promise:Promise.resolve(symbol),weak:new WeakRef(symbol)}})();").unwrap();
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("holder.weak.deref()!==undefined"),
            Ok(Value::Boolean(true))
        );
        realm.eval("holder.promise=null").unwrap();
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("holder.weak.deref()===undefined"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn idle_and_preflight_failures_preserve_jobs_and_started_host_failures_are_reported() {
        let mut empty = Realm::default();
        assert!(matches!(
            empty.run_next_promise_job(),
            Ok(PromiseJobOutcome::NoJob)
        ));
        assert!(empty.intrinsics.is_none());
        let mut realm = Realm::default();
        realm
            .eval("Promise.resolve({then(resolve){resolve(1)}})")
            .unwrap();
        realm.job_active = true;
        assert!(matches!(
            realm.run_next_promise_job(),
            Err(RuntimeError::Host { .. })
        ));
        assert_eq!(realm.promise_jobs.len(), 1);
        realm.job_active = false;
        realm.limits.max_steps = Some(0);
        assert!(matches!(
            realm.run_next_promise_job(),
            Err(RuntimeError::Limit { .. })
        ));
        assert_eq!(realm.promise_jobs.len(), 1);
        realm.limits.max_steps = None;
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        realm.eval("let flag=0;Promise.resolve({then(){try{while(true){}}catch{flag=1}finally{flag=2}}})").unwrap();
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Reported(RuntimeError::Limit { .. }))
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
