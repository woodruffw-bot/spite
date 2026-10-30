//! Host transfer of shared backing without transferring JavaScript heap values.

use crate::{Error, ObjectHandle, Realm, Value, object};
use spite_core::{JsString, Span};

/// An owned shared data block that an embedding can transfer between Realms.
///
/// Clones retain the same bytes, growable length and native waiter lists. This
/// handle contains no JavaScript heap values and can cross native threads;
/// each receiving Realm creates its own branded wrapper and intrinsic graph.
#[derive(Clone, Debug)]
pub struct SharedBuffer {
    data: object::SharedArrayBufferData,
}

impl Realm {
    /// Retains a SharedArrayBuffer's native backing without running JavaScript.
    ///
    /// Wrong brands, Proxy wrappers and foreign/stale heap handles are rejected.
    /// The returned backing does not root the exporting JavaScript wrapper.
    pub fn share_array_buffer(&self, buffer: &ObjectHandle) -> Result<SharedBuffer, object::Error> {
        let data = self
            .objects
            .inspect(buffer)?
            .shared_array_buffer_data()
            .ok_or(object::Error::WrongKind)?;
        Ok(SharedBuffer { data: data.clone() })
    }

    /// Defines a global-object data property containing a fresh shared wrapper.
    ///
    /// The wrapper uses this Realm's retained SharedArrayBuffer prototype. Its
    /// backing remains shared with every clone of `buffer`. The property is
    /// writable, enumerable and configurable; inherited setters are bypassed.
    /// Existing global lexical bindings retain their ordinary lookup priority.
    /// The returned handle is unrooted if JavaScript removes the binding; use
    /// [`Self::root_value`] when retaining it across explicit collection.
    pub fn bind_shared_array_buffer(
        &mut self,
        name: impl Into<JsString>,
        buffer: &SharedBuffer,
    ) -> Result<ObjectHandle, Error> {
        self.initialize_realm()?;
        if !self.job_active {
            self.remaining_steps = self.limits.max_steps;
        }
        let span = Span::new(0, 0);
        let name = name.into();
        self.check_string(&Value::String(name.clone()), span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .global_metadata
            .shared_array_buffer_prototype
            .clone();
        let wrapper = self.object_work(span, |objects, budget| {
            objects.create_shared_array_buffer_from_data(&prototype, buffer.data.clone(), budget)
        })?;
        self.define_property_or_throw(
            &self.global_object(),
            name,
            object::DataDescriptor {
                value: Some(Value::Object(wrapper.clone())),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(wrapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn export(realm: &mut Realm, source: &str) -> SharedBuffer {
        let Value::Object(buffer) = realm.eval(source).unwrap() else {
            panic!("buffer")
        };
        realm.share_array_buffer(&buffer).unwrap()
    }

    #[test]
    fn wrappers_use_receiving_intrinsics_and_preserve_shared_data_identity() {
        let mut first = Realm::default();
        let shared = export(&mut first, "new SharedArrayBuffer(8,{maxByteLength:16})");
        let mut second = Realm::default();
        second
            .eval("const Original=SharedArrayBuffer;SharedArrayBuffer=function(){throw 1}")
            .unwrap();
        second.bind_shared_array_buffer("left", &shared).unwrap();
        second.bind_shared_array_buffer("right", &shared).unwrap();
        assert_eq!(second.eval("left!==right&&left instanceof Original&&right instanceof Original&&Object.getPrototypeOf(left)===Original.prototype&&left.constructor===Original"), Ok(Value::Boolean(true)));
        assert_eq!(second.eval("new Uint8Array(left)[0]=19;left.grow(16);new Uint8Array(right)[0]===19&&right.byteLength===16"), Ok(Value::Boolean(true)));
        assert_eq!(second.eval("left.constructor={[Symbol.species]:function(){return right}};let caught=false;try{left.slice(0,4)}catch(e){caught=e instanceof TypeError}caught"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn native_backing_outlives_the_exporting_realm_and_still_grows() {
        let shared = {
            let mut first = Realm::default();
            export(
                &mut first,
                "let b=new SharedArrayBuffer(4,{maxByteLength:16});new Uint8Array(b)[0]=31;b",
            )
        };
        let mut second = Realm::default();
        second.bind_shared_array_buffer("buffer", &shared).unwrap();
        assert_eq!(second.eval("buffer.grow(16);let a=new Uint8Array(buffer);a[0]===31&&a[15]===0&&buffer.byteLength===16"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn exported_backing_does_not_root_the_original_javascript_wrapper() {
        let mut first = Realm::default();
        let shared = export(
            &mut first,
            "let weak;(()=>{let b=new SharedArrayBuffer(8);new Uint8Array(b)[0]=47;weak=new WeakRef(b);return b})()",
        );
        first.clear_kept_objects().unwrap();
        first.collect(usize::MAX).unwrap();
        assert_eq!(
            first.eval("weak.deref()===undefined"),
            Ok(Value::Boolean(true))
        );
        let mut second = Realm::default();
        second.bind_shared_array_buffer("buffer", &shared).unwrap();
        assert_eq!(
            second.eval("new Uint8Array(buffer)[0]===47"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn extraction_rejects_wrong_brands_and_foreign_handles_without_hooks() {
        let mut first = Realm::default();
        for source in [
            "({})",
            "new ArrayBuffer(8)",
            "SharedArrayBuffer.prototype",
            "new Proxy(new SharedArrayBuffer(8),{get(){throw 7}})",
        ] {
            let Value::Object(buffer) = first.eval(source).unwrap() else {
                panic!("object")
            };
            assert_eq!(
                first.share_array_buffer(&buffer).unwrap_err(),
                object::Error::WrongKind
            );
        }
        let Value::Object(buffer) = first.eval("new SharedArrayBuffer(8)").unwrap() else {
            panic!("buffer")
        };
        let second = Realm::default();
        assert!(matches!(
            second.share_array_buffer(&buffer),
            Err(object::Error::Heap(spite_heap::Error::ForeignHandle))
        ));
    }

    #[test]
    fn isolated_realms_share_atomic_updates_across_native_threads() {
        let mut first = Realm::default();
        let shared = export(
            &mut first,
            "let b=new SharedArrayBuffer(8),counter=new Uint32Array(b);b",
        );
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let gate = barrier.clone();
        // Realm requires the documented native stack, including on Windows.
        let thread = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let mut second = Realm::default();
                second.bind_shared_array_buffer("buffer", &shared).unwrap();
                second.eval("let counter=new Uint32Array(buffer)").unwrap();
                gate.wait();
                second
                    .eval("for(let i=0;i<5000;i++)Atomics.add(counter,0,1)")
                    .unwrap();
            })
            .unwrap();
        barrier.wait();
        first
            .eval("for(let i=0;i<5000;i++)Atomics.add(counter,0,1)")
            .unwrap();
        thread.join().unwrap();
        assert_eq!(
            first.eval("counter[0]===10000&&counter[1]===0"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn foreign_notification_resolves_only_at_the_receiving_realms_job_boundary() {
        let mut first = Realm::default();
        let shared = export(
            &mut first,
            "let b=new SharedArrayBuffer(8),a=new Int32Array(b),log='';Atomics.waitAsync(a,0,0).value.then(v=>log=v);b",
        );
        let mut second = Realm::default();
        second.bind_shared_array_buffer("buffer", &shared).unwrap();
        assert_eq!(
            second.eval("Atomics.notify(new Int32Array(buffer),0)"),
            Ok(Value::Number(1.0))
        );
        assert_eq!(
            first.eval("log===''&&Atomics.notify(a,0)===0"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            first.next_atomics_wait_delay(),
            Some(std::time::Duration::ZERO)
        );
        // Receiving a generic notification job queues, but never calls, JS.
        first.run_next_promise_job().unwrap();
        assert_eq!(first.eval("log===''"), Ok(Value::Boolean(true)));
        assert!(first.run_next_atomics_wait_job().unwrap());
        assert_eq!(first.eval("log===''"), Ok(Value::Boolean(true)));
        first.run_next_promise_job().unwrap();
        assert_eq!(first.eval("log==='ok'"), Ok(Value::Boolean(true)));
        assert!(first.next_atomics_wait_delay().is_none());
        assert!(!first.run_next_atomics_wait_job().unwrap());
    }
    #[test]
    fn foreign_jobs_follow_notification_order_across_offsets_and_backings() {
        let mut receiver = Realm::default();
        let first = export(
            &mut receiver,
            "let first=new SharedArrayBuffer(8),a=new Int32Array(first),log='';Atomics.waitAsync(a,0,0).value.then(()=>log+='a');Atomics.waitAsync(a,1,0).value.then(()=>log+='b');first",
        );
        let second = export(
            &mut receiver,
            "let second=new SharedArrayBuffer(8),b=new Int32Array(second);Atomics.waitAsync(b,0,0).value.then(()=>log+='c');second",
        );
        let mut sender = Realm::default();
        sender.bind_shared_array_buffer("first", &first).unwrap();
        sender.bind_shared_array_buffer("second", &second).unwrap();
        assert_eq!(sender.eval("let a=new Int32Array(first),b=new Int32Array(second);Atomics.notify(b,0,1)===1&&Atomics.notify(a,1,1)===1&&Atomics.notify(a,0,1)===1&&Atomics.notify(a,0)===0"), Ok(Value::Boolean(true)));
        for _ in 0..3 {
            assert!(receiver.run_next_atomics_wait_job().unwrap());
        }
        assert!(!receiver.run_next_atomics_wait_job().unwrap());
        assert_eq!(receiver.eval("log===''"), Ok(Value::Boolean(true)));
        for _ in 0..3 {
            receiver.run_next_promise_job().unwrap();
        }
        assert_eq!(receiver.eval("log==='cba'"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn mixed_local_and_foreign_waiters_keep_fifo_counts_and_local_inline_settlement() {
        let mut first = Realm::default();
        let shared = export(
            &mut first,
            "let buffer=new SharedArrayBuffer(8),a=new Int32Array(buffer),log='';Atomics.waitAsync(a,0,0).value.then(()=>log+='a');buffer",
        );
        let mut second = Realm::default();
        second.bind_shared_array_buffer("buffer", &shared).unwrap();
        second.eval("let a=new Int32Array(buffer),log='';Atomics.waitAsync(a,0,0).value.then(()=>log+='b')").unwrap();
        first
            .eval("Atomics.waitAsync(a,0,0).value.then(()=>log+='c')")
            .unwrap();
        assert_eq!(first.eval("Atomics.notify(a,0,2)"), Ok(Value::Number(2.0)));
        assert!(!first.run_next_atomics_wait_job().unwrap());
        first.run_next_promise_job().unwrap();
        assert_eq!(
            first.eval("log==='a'&&Atomics.notify(a,0)===1"),
            Ok(Value::Boolean(true))
        );
        first.run_next_promise_job().unwrap();
        assert_eq!(first.eval("log==='ac'"), Ok(Value::Boolean(true)));
        assert_eq!(
            second.eval("log===''&&Atomics.notify(a,0)===0"),
            Ok(Value::Boolean(true))
        );
        assert!(second.run_next_atomics_wait_job().unwrap());
        second.run_next_promise_job().unwrap();
        assert_eq!(second.eval("log==='b'"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn native_thread_notifications_keep_receiving_captures_rooted_until_reactions_finish() {
        let mut receiver = Realm::default();
        let shared = export(
            &mut receiver,
            "let buffer=new SharedArrayBuffer(8),weak,log='';function keep(x){return ()=>log=x.tag}{let token={tag:'ok'};weak=new WeakRef(token);Atomics.waitAsync(new Int32Array(buffer),0,0).value.then(keep(token))}buffer",
        );
        let thread = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let mut sender = Realm::default();
                sender.bind_shared_array_buffer("buffer", &shared).unwrap();
                assert_eq!(
                    sender.eval("Atomics.notify(new Int32Array(buffer),0)"),
                    Ok(Value::Number(1.0))
                );
            })
            .unwrap();
        receiver.clear_kept_objects().unwrap();
        receiver.collect(usize::MAX).unwrap();
        thread.join().unwrap();
        assert_eq!(
            receiver.eval("weak.deref()!==undefined&&log===''"),
            Ok(Value::Boolean(true))
        );
        receiver.run_next_atomics_wait_job().unwrap();
        receiver.clear_kept_objects().unwrap();
        receiver.collect(usize::MAX).unwrap();
        receiver.run_next_promise_job().unwrap();
        assert_eq!(receiver.eval("log==='ok'"), Ok(Value::Boolean(true)));
        receiver.clear_kept_objects().unwrap();
        receiver.collect(usize::MAX).unwrap();
        assert_eq!(
            receiver.eval("weak.deref()===undefined"),
            Ok(Value::Boolean(true))
        );
    }

    #[test]
    fn refused_foreign_delivery_keeps_the_native_job_and_restores_caller_work() {
        let mut receiver = Realm::default();
        let shared = export(
            &mut receiver,
            "let buffer=new SharedArrayBuffer(8),log='';Atomics.waitAsync(new Int32Array(buffer),0,0).value.then(v=>log=v);buffer",
        );
        let mut sender = Realm::default();
        sender.bind_shared_array_buffer("buffer", &shared).unwrap();
        assert_eq!(
            sender.eval("Atomics.notify(new Int32Array(buffer),0)"),
            Ok(Value::Number(1.0))
        );
        receiver.limits.max_steps = Some(0);
        receiver.remaining_steps = Some(77);
        assert!(matches!(
            receiver.run_next_atomics_wait_job(),
            Err(Error::Limit { .. })
        ));
        assert_eq!(receiver.remaining_steps, Some(77));
        assert!(!receiver.job_active);
        assert_eq!(
            receiver.next_atomics_wait_delay(),
            Some(std::time::Duration::ZERO)
        );
        receiver.limits.max_steps = None;
        assert!(receiver.run_next_atomics_wait_job().unwrap());
        receiver.run_next_promise_job().unwrap();
        assert_eq!(receiver.eval("log==='ok'"), Ok(Value::Boolean(true)));
        assert!(!receiver.run_next_atomics_wait_job().unwrap());
    }

    #[test]
    fn foreign_notification_and_timeout_have_one_winner() {
        use std::time::Duration;
        let mut receiver = Realm::default();
        let shared = export(
            &mut receiver,
            "let buffer=new SharedArrayBuffer(8),a=new Int32Array(buffer),log='';Atomics.waitAsync(a,0,0,1).value.then(v=>log+=v);buffer",
        );
        let mut sender = Realm::default();
        sender.bind_shared_array_buffer("buffer", &shared).unwrap();
        sender.eval("let a=new Int32Array(buffer)").unwrap();
        std::thread::sleep(Duration::from_millis(2));
        assert_eq!(sender.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
        receiver.run_next_atomics_wait_job().unwrap();
        receiver.run_next_promise_job().unwrap();
        assert_eq!(receiver.eval("log==='ok'"), Ok(Value::Boolean(true)));
        receiver
            .eval("Atomics.waitAsync(a,0,0,1).value.then(v=>log+=v)")
            .unwrap();
        std::thread::sleep(Duration::from_millis(2));
        receiver.run_next_atomics_wait_job().unwrap();
        assert_eq!(sender.eval("Atomics.notify(a,0)"), Ok(Value::Number(0.0)));
        receiver.run_next_promise_job().unwrap();
        assert_eq!(
            receiver.eval("log==='oktimed-out'"),
            Ok(Value::Boolean(true))
        );
        assert!(receiver.next_atomics_wait_delay().is_none());
        assert!(!receiver.run_next_atomics_wait_job().unwrap());
    }
}
