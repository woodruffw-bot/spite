//! Explicit host capabilities invoke native detachment, without public hooks.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

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
    for member in ["createRealm", "evalScript", "gc", "agent", "IsHTMLDDA"] {
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
