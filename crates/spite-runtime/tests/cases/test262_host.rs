//! Explicit host capabilities invoke native detachment, without public hooks.

use spite_runtime::{Error, ExceptionKind, Limits, PromiseJobOutcome, Realm, Value};

fn host() -> Realm {
    let mut realm = Realm::default();
    realm.install_test262_host().unwrap();
    realm
}

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn host_installation_is_explicit_with_the_current_global_and_native_method_metadata() {
    let mut realm = Realm::default();
    check(&mut realm, "typeof $262==='undefined'");
    realm.install_test262_host().unwrap();
    check(
        &mut realm,
        r#"
        let method=$262.detachArrayBuffer,nonconstructor=false;
        try{new method(new ArrayBuffer(1))}catch(e){nonconstructor=e instanceof TypeError}
        $262.global===globalThis&&method.name==='detachArrayBuffer'&&method.length===1&&
        !Object.getOwnPropertyDescriptor(globalThis,'$262').enumerable&&nonconstructor&&
        Function.prototype.toString.call(method)==='function detachArrayBuffer() { [native code] }'
    "#,
    );
}

#[test]
fn detachment_invalidates_fixed_and_tracking_views_without_copying_or_public_transfer_hooks() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let b=new ArrayBuffer(16,{maxByteLength:32}),tracking=new Uint8Array(b),fixed=new Uint8Array(b,0,8),view=new DataView(b);
        tracking[0]=7;let calls=0;
        b.transfer=()=>{calls++;throw 1};b.constructor={[Symbol.species](){calls++;throw 2}};
        let returned=$262.detachArrayBuffer(b),typed=false;
        try{view.getInt8(0)}catch(e){typed=e instanceof TypeError}
        returned===undefined&&b.detached&&b.byteLength===0&&tracking.length===0&&fixed.length===0&&tracking[0]===undefined&&typed&&calls===0
    "#,
    );
}

#[test]
fn buffer_brands_and_detach_keys_are_checked_without_coercion_or_traps() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let calls=0,errors=0,b=new ArrayBuffer(8),key={[Symbol.toPrimitive](){calls++;return undefined}};
        let proxy=new Proxy(b,{get(){calls++;throw 1}});
        for(const value of [undefined,null,1,{},proxy,new SharedArrayBuffer(8),new Uint8Array(b)]){
            try{$262.detachArrayBuffer(value,key)}catch(e){if(e instanceof TypeError)errors++}
        }
        for(const value of [null,0,'',key]){try{$262.detachArrayBuffer(b,value)}catch(e){if(e instanceof TypeError)errors++}}
        calls===0&&errors===11&&!b.detached
    "#,
    );
}

#[test]
fn repeated_detachment_ignores_this_and_extra_arguments_but_checks_keys_each_time() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let b=new ArrayBuffer(8),detach=$262.detachArrayBuffer,calls=0,typed=false;
        let first=detach.call(7,b,undefined,{valueOf(){calls++;throw 1}}),second=detach(b);
        try{detach(b,0)}catch(e){typed=e instanceof TypeError}
        first===undefined&&second===undefined&&b.detached&&typed&&calls===0
    "#,
    );
}

#[test]
fn unavailable_facilities_report_host_errors_on_access_and_bypass_language_cleanup() {
    for member in ["createRealm", "IsHTMLDDA", "AbstractModuleSource"] {
        let mut realm = host();
        realm
            .eval(
                "let caught=0,finalized=0,disposed=0;let resource={[Symbol.dispose](){disposed++}}",
            )
            .unwrap();
        let error=realm.eval(&format!("function f(){{using r=resource;try{{typeof $262.{member}}}catch(e){{caught++}}finally{{finalized++}}}}f()")).unwrap_err();
        assert!(
            matches!(error, Error::Unsupported { .. }),
            "{member}: {error}"
        );
        check(&mut realm, "caught===0&&finalized===0&&disposed===0");
    }
}

#[test]
fn saved_native_methods_do_not_retain_the_host_object_after_global_deletion() {
    let mut realm = host();
    realm
        .eval("let weakHost=new WeakRef($262),detach=$262.detachArrayBuffer;delete globalThis.$262")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "let b=new ArrayBuffer(8);detach(b);weakHost.deref()===undefined&&b.detached",
    );
}

#[test]
fn installing_over_a_restricted_global_returns_a_language_error() {
    let mut realm = Realm::default();
    realm
        .eval("Object.defineProperty(globalThis,'$262',{value:7})")
        .unwrap();
    let error = realm.install_test262_host().unwrap_err();
    assert!(
        matches!(
            error,
            Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            }
        ),
        "{error}"
    );
    check(&mut realm, "$262===7");
}

#[test]
fn scripts_use_the_global_environment_and_restore_strict_callers_and_local_bindings() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        var count=1;
        function f(){'use strict';let count=99;
            let result=$262.evalScript('var count=7;let added=11;function readAdded(){return added};globalThis.scriptThis=this;count+added');
            let strict=false;try{unresolvableAfterScript=1}catch(e){strict=e instanceof ReferenceError}
            return count===99&&result===18&&this===undefined&&strict;
        }
        f()&&count===7&&added===11&&readAdded()===11&&scriptThis===globalThis
    "#,
    );
}

#[test]
fn script_parse_and_runtime_errors_are_catchable_without_inheriting_eval_privileges() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let marker={},same=false,errors=0;globalThis.eval=()=>{throw 'public eval'};
        try{$262.evalScript('throw marker')}catch(e){same=e===marker}
        for(const code of ['return 1','new.target','super.x','this.#x']){
            try{$262.evalScript(code)}catch(e){if(e instanceof SyntaxError)errors++}
        }
        $262.evalScript('let retained=7');
        try{$262.evalScript('let retained=8')}catch(e){if(e instanceof SyntaxError)errors++}
        function strictCaller(){'use strict';return $262.evalScript('with({value:3}){value}')}
        same&&errors===5&&retained===7&&strictCaller()===3&&$262.evalScript('({value:4})').value===4
    "#,
    );
}

#[test]
fn script_source_preserves_utf16_and_requires_a_string_without_coercion() {
    let mut realm = host();
    check(
        &mut realm,
        r#"
        let lone=String.fromCharCode(0xd800),source="(function(){return '"+lone+"'})",method=$262.evalScript(source),calls=0,errors=0;
        for(const value of [undefined,null,1,{},new String('4'),{toString(){calls++;return '4'}}]){
            try{$262.evalScript(value)}catch(e){if(e instanceof TypeError)errors++}
        }
        method()===lone&&Function.prototype.toString.call(method).includes(lone)&&calls===0&&errors===6
    "#,
    );
    // The caller supplies a validated AST; the host must independently bound
    // the newly compiled Script using its lossless encoded byte length.
    let mut limited = Realm::new(Limits {
        max_source_bytes: Some(4),
        ..Limits::default()
    });
    limited.install_test262_host().unwrap();
    let large =
        spite_parser::parse_script(r#"$262.evalScript('"'+String.fromCharCode(0xd800)+'"')"#)
            .unwrap();
    assert!(matches!(limited.evaluate(&large), Err(Error::Limit { .. })));
    let small = spite_parser::parse_script(r#"$262.evalScript('"a"')"#).unwrap();
    assert_eq!(
        limited.evaluate(&small),
        Ok(Value::String(spite_core::JsString::from("a")))
    );
}

#[test]
fn nested_scripts_and_modules_preserve_their_distinct_import_referrers() {
    let mut realm = host();
    realm.enable_module_imports();
    let record=realm.create_module("$262.evalScript('globalThis.scriptImport=function(){return import(\"later\")};import(\"script\")');import('module');export const x=7").unwrap();
    realm.evaluate_module_record(&record).unwrap();
    let from_script = realm.take_module_import_request().unwrap();
    assert!(from_script.referrer().is_none());
    assert_eq!(
        from_script.request().specifier,
        spite_core::JsString::from("script")
    );
    let from_module = realm.take_module_import_request().unwrap();
    assert_eq!(from_module.referrer(), Some(&record));
    realm.eval("scriptImport()").unwrap();
    assert!(
        realm
            .take_module_import_request()
            .unwrap()
            .referrer()
            .is_none()
    );
}

#[test]
fn jobs_created_by_global_scripts_keep_their_context_after_caller_restoration() {
    let mut realm = host();
    realm.eval(r#"
        let results=[];
        async function caller(){let local=7;
            $262.evalScript('async function scriptJob(){await 0;return [this===globalThis,typeof local]}scriptJob().then(v=>results.push(v.join(",")))');
            await 0;results.push(local);
        }
        caller();
    "#).unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    check(
        &mut realm,
        "results.includes('true,undefined')&&results.includes(7)",
    );
}

#[test]
fn script_host_failures_restore_context_and_bypass_caller_cleanup() {
    let mut realm = host();
    realm.eval("let caught=0,finalized=0,disposed=0;var entered=0;let resource={[Symbol.dispose](){disposed++}}").unwrap();
    let error=realm.eval("function f(){using r=resource;let entered=99;try{$262.evalScript('entered++;$262.createRealm')}catch(e){caught++}finally{finalized++}}f()").unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }), "{error}");
    check(
        &mut realm,
        "entered===1&&caught===0&&finalized===0&&disposed===0",
    );
}

#[test]
fn nested_script_calls_share_the_current_work_allowance() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Limits::default()
    });
    realm.install_test262_host().unwrap();
    realm
        .eval("let entered=false,caught=0,finalized=0")
        .unwrap();
    let error=realm.eval("function f(){entered=true;try{for(let i=0;i<1000;i++)$262.evalScript('0')}catch(e){caught++}finally{finalized++}}f()").unwrap_err();
    assert!(matches!(error, Error::Limit { .. }), "{error}");
    check(&mut realm, "entered&&caught===0&&finalized===0");
}

fn next_agent_report(realm: &mut Realm) -> Result<spite_core::JsString, Error> {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match realm.eval("$262.agent.getReport()")? {
            Value::String(message) => return Ok(message),
            Value::Null => (),
            value => panic!("unexpected report: {value:?}"),
        }
        assert!(
            Instant::now() < deadline,
            "worker neither reported nor failed"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn isolated_workers_report_lossless_strings_and_run_gc_and_promise_jobs() {
    use spite_core::JsString;
    let mut realm = host();
    check(
        &mut realm,
        "$262.agent.getReport()===null&&typeof $262.agent==='object'&&$262.agent.start.length===1",
    );
    realm
        .eval("let parentOnly=11;Object.prototype.parentMarker=99")
        .unwrap();
    let source = r#"
        $262.agent.report(typeof parentOnly+'|'+typeof Object.prototype.parentMarker+'|'+($262.global===globalThis));
        $262.agent.report('');
        $262.agent.report('\uD800');
        $262.agent.report({toString(){$262.agent.report('inner');return 'outer'}});
        (async function(){await $262.gc();$262.agent.report('jobs');$262.agent.leaving()})();
    "#;
    assert_eq!(
        realm.eval(&format!("$262.agent.start({source:?})")),
        Ok(Value::Undefined)
    );
    let reports = (0..6)
        .map(|_| next_agent_report(&mut realm).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(reports[0], JsString::from("undefined|undefined|true"));
    assert_eq!(reports[1].len(), 0);
    assert_eq!(reports[2].code_units(), &[0xd800]);
    assert_eq!(reports[3], JsString::from("inner"));
    assert_eq!(reports[4], JsString::from("outer"));
    assert_eq!(reports[5], JsString::from("jobs"));
    realm.finish_test262_agents().unwrap();
    check(
        &mut realm,
        "parentOnly===11&&Object.prototype.parentMarker===99&&$262.agent.getReport()===null",
    );
}

#[test]
fn worker_retirement_stops_program_loops_sleeps_and_infinite_waits() {
    let mut realm = host();
    for body in [
        "while(true){}",
        "$262.agent.sleep(Infinity)",
        "Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0)",
    ] {
        let source = format!("$262.agent.report('ready');{body}");
        realm
            .eval(&format!("$262.agent.start({source:?})"))
            .unwrap();
    }
    for _ in 0..3 {
        assert_eq!(next_agent_report(&mut realm).unwrap(), "ready".into());
    }
    realm.finish_test262_agents().unwrap();
    realm.finish_test262_agents().unwrap();
    check(
        &mut realm,
        "let now=$262.agent.monotonicNow(),calls=0;$262.agent.sleep({valueOf(){calls++;return 0}});calls===1&&$262.agent.monotonicNow()>=now",
    );
}

#[test]
fn worker_failures_keep_host_unsupported_and_limit_categories() {
    for (limits, source, expected) in [
        (Limits::default(), "throw new TypeError('worker')", "host"),
        (Limits::default(), "typeof $262.createRealm", "unsupported"),
        (
            Limits {
                max_steps: Some(2000),
                ..Limits::default()
            },
            "while(true){}",
            "limit",
        ),
    ] {
        let mut realm = Realm::new(limits);
        realm.install_test262_host().unwrap();
        let error = match realm.eval(&format!("$262.agent.start({source:?})")) {
            Ok(_) => next_agent_report(&mut realm).unwrap_err(),
            Err(error) => error,
        };
        let kind = match error {
            Error::Host { .. } => "host",
            Error::Unsupported { .. } => "unsupported",
            Error::Limit { .. } => "limit",
            _ => panic!("unexpected worker error: {error}"),
        };
        assert_eq!(kind, expected, "{source}");
        assert!(realm.finish_test262_agents().is_err());
    }
}

#[test]
fn broadcasts_share_live_backing_with_receiving_intrinsics_and_full_bigint_values() {
    let mut realm = host();
    realm.eval("let buffer=new SharedArrayBuffer(8,{maxByteLength:16}),a=new Int32Array(buffer),calls=0").unwrap();
    let source = "let Original=SharedArrayBuffer;SharedArrayBuffer=function(){throw 1};$262.agent.receiveBroadcast(function(buffer,value){if(Object.getPrototypeOf(buffer)!==Original.prototype||!(buffer instanceof Original))throw 2;Atomics.add(new Int32Array(buffer),0,1);buffer.grow(16);$262.agent.report(value.toString()+':'+buffer.byteLength);$262.agent.leaving()})";
    for _ in 0..2 {
        realm
            .eval(&format!("$262.agent.start({source:?})"))
            .unwrap();
    }
    check(
        &mut realm,
        "$262.agent.broadcast(buffer,{valueOf(){calls++;return 2n**80n+7n}})===undefined&&calls===1",
    );
    for _ in 0..2 {
        assert_eq!(
            next_agent_report(&mut realm).unwrap(),
            "1208925819614629174706183:16".into()
        );
    }
    realm.finish_test262_agents().unwrap();
    check(&mut realm, "buffer.byteLength===16&&a[0]===2");
}

#[test]
fn broadcast_retrieval_acknowledges_before_callbacks_and_preserves_repeated_delivery() {
    let mut realm = host();
    realm
        .eval("let buffer=new SharedArrayBuffer(4),a=new Int32Array(buffer)")
        .unwrap();
    let source = "$262.agent.receiveBroadcast(function(first,x){$262.agent.report('first:'+x);$262.agent.receiveBroadcast(function(second,y){if(first===second)throw 1;new Int32Array(second)[0]=37;if(new Int32Array(first)[0]!==37)throw 2;$262.agent.report('second:'+y);$262.agent.leaving()})})";
    realm
        .eval(&format!("$262.agent.start({source:?})"))
        .unwrap();
    realm
        .eval("$262.agent.broadcast(buffer,4294967297);$262.agent.broadcast(buffer,-4294967298)")
        .unwrap();
    assert_eq!(next_agent_report(&mut realm).unwrap(), "first:1".into());
    assert_eq!(next_agent_report(&mut realm).unwrap(), "second:-2".into());
    realm.finish_test262_agents().unwrap();
    check(&mut realm, "a[0]===37");
}

#[test]
fn broadcast_validation_precedes_value_hooks_and_rejects_proxy_brands() {
    let mut realm = host();
    check(
        &mut realm,
        "let buffer=new SharedArrayBuffer(8),calls=0,traps=0,caught=0;for(const invalid of [{},new ArrayBuffer(8),SharedArrayBuffer.prototype,new Proxy(buffer,{get(){traps++;throw 1}})]){try{$262.agent.broadcast(invalid,{valueOf(){calls++;return 0}})}catch(e){if(e instanceof TypeError)caught++}}caught===4&&calls===0&&traps===0",
    );
    check(
        &mut realm,
        "let reason={},same=false;try{$262.agent.broadcast(buffer,{valueOf(){throw reason}})}catch(e){same=e===reason}same&&$262.agent.broadcast(buffer)===undefined",
    );
    realm.finish_test262_agents().unwrap();
}

#[test]
fn broadcast_workers_keep_gc_roots_and_resume_foreign_async_notifications() {
    let mut realm = host();
    realm
        .eval("let buffer=new SharedArrayBuffer(8),a=new Int32Array(buffer)")
        .unwrap();
    let source = "$262.agent.receiveBroadcast(async function(buffer){let a=new Int32Array(buffer),wait=Atomics.waitAsync(a,0,0);$262.agent.report('waiting');await $262.gc();let value=await wait.value;$262.agent.report(value);$262.agent.leaving()})";
    realm
        .eval(&format!("$262.agent.start({source:?})"))
        .unwrap();
    realm.eval("$262.agent.broadcast(buffer)").unwrap();
    assert_eq!(next_agent_report(&mut realm).unwrap(), "waiting".into());
    assert_eq!(realm.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
    assert_eq!(next_agent_report(&mut realm).unwrap(), "ok".into());
    realm.finish_test262_agents().unwrap();
    check(&mut realm, "Atomics.notify(a,0)===0");
}
