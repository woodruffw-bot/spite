//! Shared buffer construction, growable lengths and remaining access boundaries.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructor_brands_fixed_and_growable_buffers_and_requires_new_before_conversion() {
    check(
        r#"
        let hits=0,threw=false;
        try{SharedArrayBuffer({valueOf(){hits++;return 4}})}catch(e){threw=e instanceof TypeError}
        let fixed=new SharedArrayBuffer(4),growable=new SharedArrayBuffer(2,{maxByteLength:8});
        threw&&hits===0&&fixed.byteLength===4&&fixed.maxByteLength===4&&!fixed.growable&&
        growable.byteLength===2&&growable.maxByteLength===8&&growable.growable&&
        fixed instanceof SharedArrayBuffer&&Object.prototype.toString.call(fixed)==='[object SharedArrayBuffer]'&&
        !ArrayBuffer.isView(fixed)
    "#,
    );
    for name in ["byteLength", "maxByteLength", "growable"] {
        check(&format!(
            "let getter=Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'{name}').get;let count=0;for(let value of [undefined,null,1,{{}},new ArrayBuffer(0),SharedArrayBuffer.prototype,new Proxy(new SharedArrayBuffer(0),{{}})]){{try{{getter.call(value)}}catch(e){{if(e instanceof TypeError)count++}}}}count===7"
        ));
    }
}

#[test]
fn constructor_orders_length_options_range_checks_and_prototype_lookup() {
    check(
        r#"
        let log='',proto={};function F(){}let target=F.bind(null);
        Object.defineProperty(target,'prototype',{get(){log+='p';return proto}});
        let length={valueOf(){log+='l';return 4}},options={get maxByteLength(){log+='m';return{valueOf(){log+='v';return 8}}}};
        let b=Reflect.construct(SharedArrayBuffer,[length,options],target);
        if(log!=='lmvp'||Object.getPrototypeOf(b)!==proto)throw 1;
        let getter=Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'byteLength').get;
        if(getter.call(b)!==4)throw 2;
        log='';let threw=false;
        try{Reflect.construct(SharedArrayBuffer,[length,{get maxByteLength(){log+='m';return 3}}],target)}catch(e){threw=e instanceof RangeError}
        if(!threw||log!=='lm')throw 3;
        log='';try{Reflect.construct(SharedArrayBuffer,[-1,options],target)}catch(e){threw=e instanceof RangeError}
        threw&&log===''
    "#,
    );
    check(
        r#"
        let hits=0;
        Object.defineProperty(Number.prototype,'maxByteLength',{get(){hits++;throw 1}});
        Object.defineProperty(String.prototype,'maxByteLength',{get(){hits++;throw 2}});
        let a=new SharedArrayBuffer(1,7),b=new SharedArrayBuffer(1,'x');
        hits===0&&!a.growable&&!b.growable
    "#,
    );
}

#[test]
fn growth_rechecks_current_length_after_coercion_and_never_shrinks() {
    check(
        r#"
        let b=new SharedArrayBuffer(4,{maxByteLength:16}),threw=false;
        try{b.grow({valueOf(){b.grow(12);return 8}})}catch(e){threw=e instanceof RangeError}
        if(!threw||b.byteLength!==12)throw 1;
        if(b.grow(12)!==undefined||b.byteLength!==12)throw 2;
        if(b.grow(16)!==undefined||b.byteLength!==16)throw 3;
        threw=false;try{b.grow(17)}catch(e){threw=e instanceof RangeError}
        if(!threw||b.byteLength!==16)throw 4;
        let fixed=new SharedArrayBuffer(4),hits=0;
        try{fixed.grow({valueOf(){hits++;return 4}})}catch(e){threw=e instanceof TypeError}
        if(!threw||hits!==0)throw 5;
        let token={},caught;
        try{b.grow({valueOf(){throw token}})}catch(e){caught=e}
        caught===token&&b.byteLength===16
    "#,
    );
}

#[test]
fn lengths_follow_the_safe_index_domain_without_eager_large_allocations() {
    check(
        r#"
        let max=2**53-1,b=new SharedArrayBuffer(0,{maxByteLength:max});
        b.grow(max);
        if(b.byteLength!==max||b.maxByteLength!==max||new SharedArrayBuffer(max).byteLength!==max)throw 1;
        if(new SharedArrayBuffer(-0.5).byteLength!==0||new SharedArrayBuffer(3.9).byteLength!==3)throw 2;
        let count=0;
        for(let value of [-1,Infinity,2**53]){try{new SharedArrayBuffer(value)}catch(e){if(e instanceof RangeError)count++}}
        for(let value of [1n,Symbol()]){try{new SharedArrayBuffer(value)}catch(e){if(e instanceof TypeError)count++}}
        count===5
    "#,
    );
}

#[test]
fn subclasses_and_primitive_newtarget_prototypes_use_the_captured_intrinsic_fallback() {
    check(
        r#"
        class Derived extends SharedArrayBuffer{}
        let a=new Derived(2,{maxByteLength:4});a.grow(4);
        if(!(a instanceof Derived)||a.byteLength!==4||Object.getPrototypeOf(a)!==Derived.prototype)throw 1;
        let saved=SharedArrayBuffer,proto=SharedArrayBuffer.prototype;function F(){}let target=F.bind(null);
        Object.defineProperty(target,'prototype',{value:0});
        SharedArrayBuffer=null;
        let b=Reflect.construct(saved,[3],target);
        Object.getPrototypeOf(b)===proto&&b.byteLength===3
    "#,
    );
    check(
        "let b=new SharedArrayBuffer(0,{maxByteLength:8});Object.freeze(b);b.grow(8);Object.isFrozen(b)&&b.byteLength===8",
    );
}

#[test]
fn shared_views_remain_host_failures_before_arraylike_fallbacks_or_language_cleanup() {
    for operation in ["new Uint8Array(b)", "new DataView(b)", "b.slice(0,1)"] {
        let mut realm = Realm::default();
        realm.eval("let b=new SharedArrayBuffer(4),flag=0,hits=0;Object.defineProperty(b,'length',{get(){hits++;throw 1}});Object.defineProperty(b,Symbol.iterator,{get(){hits++;throw 2}})").unwrap();
        let result = realm.eval(&format!(
            "try{{{operation};flag=3}}catch{{flag=1}}finally{{flag=2}}"
        ));
        assert!(
            matches!(result, Err(Error::Unsupported { .. })),
            "{operation}: {result:?}"
        );
        assert_eq!(realm.eval("flag===0&&hits===0"), Ok(Value::Boolean(true)));
    }
    assert!(matches!(
        Realm::default().eval("SharedArrayBuffer.prototype.grow.call({},0)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}
