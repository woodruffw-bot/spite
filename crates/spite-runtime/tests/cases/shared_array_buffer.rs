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
fn pending_shared_algorithms_remain_host_failures_without_language_cleanup() {
    for operation in [
        "Atomics.load(new Int32Array(b),0)",
        "Atomics.wait(new Int32Array(b),0,0,0)",
        "b.slice(0,1)",
    ] {
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

#[test]
fn shared_views_support_all_element_kinds_and_share_data_view_bytes() {
    check(
        r#"
        for(let C of [Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array,BigInt64Array,BigUint64Array]){
            let b=new SharedArrayBuffer(32),a=new C(b,0,2),zero=typeof a[0]==='bigint'?0n:0,one=typeof zero==='bigint'?1n:1;
            if(a[0]!==zero||a[1]!==zero)throw C.name;
            a[0]=one;
            if(a[0]!==one||a.buffer!==b||a.byteLength!==2*C.BYTES_PER_ELEMENT||!ArrayBuffer.isView(a))throw 1;
        }
        let b=new SharedArrayBuffer(8),a=new Uint8Array(b),d=new DataView(b);
        d.setUint16(0,0x1234);
        if(a[0]!==0x12||a[1]!==0x34||d.getUint16(0,true)!==0x3412)throw 2;
        a[2]=99;
        d.getUint8(2)===99&&d.buffer===b&&ArrayBuffer.isView(d)
    "#,
    );
}

#[test]
fn shared_growth_preserves_bytes_and_updates_auto_views_with_zero_filled_tails() {
    check(
        r#"
        let b=new SharedArrayBuffer(4,{maxByteLength:16}),all=new Uint8Array(b);
        all.set([3,4,5,6]);
        let fixed=new Uint8Array(b,1,2),auto=new Uint8Array(b,1),df=new DataView(b,1,2),da=new DataView(b,1),words=new Uint16Array(b,2);
        b.grow(11);
        if(fixed.length!==2||auto.length!==10||df.byteLength!==2||da.byteLength!==10||words.length!==4)throw 1;
        if(auto[0]!==4||auto[9]!==0||da.getUint8(9)!==0)throw 2;
        b.grow(16);
        if(all.length!==16||auto.length!==15||words.length!==7||da.byteLength!==15)throw 3;
        all[15]=77;
        auto[14]===77&&da.getUint8(14)===77&&Array.from(fixed).join(',')==='4,5'
    "#,
    );
}

#[test]
fn shared_data_view_codecs_preserve_endianness_and_cross_sparse_page_boundaries() {
    check(
        r#"
        let b=new SharedArrayBuffer(8192),d=new DataView(b);
        for(let [kind,value] of [['Int8',-7],['Uint8',255],['Int16',-1234],['Uint16',54321],['Int32',-1234567],['Uint32',3456789],['Float16',1.5],['Float32',1.5],['Float64',1.5],['BigInt64',-7n],['BigUint64',255n]]){
            for(let little of [false,true]){
                d['set'+kind](4093,value,little);
                if(d['get'+kind](4093,little)!==value)throw kind;
            }
        }
        d.setFloat64(4093,-0,true);
        if(!Object.is(d.getFloat64(4093,true),-0))throw 1;
        d.setFloat64(4093,NaN,false);
        Number.isNaN(d.getFloat64(4093,false))
    "#,
    );
}

#[test]
fn shared_typed_arrays_support_iterators_callbacks_raw_moves_and_overlapping_set() {
    check(
        r#"
        let b=new SharedArrayBuffer(16),a=new Uint8Array(b,0,4);a.set([4,1,3,2]);
        if([...a].join(',')!=='4,1,3,2')throw 1;
        if([...a.entries()].map(p=>p.join(':')).join(',')!=='0:4,1:1,2:3,3:2')throw 2;
        a.reverse();a.sort();
        if(a.join(',')!=='1,2,3,4'||a.map(x=>x*2).join(',')!=='2,4,6,8'||a.reduce((s,x)=>s+x,0)!==10)throw 3;
        let sub=a.subarray(1,3);if(sub.buffer!==b||sub.join(',')!=='2,3')throw 4;
        a.copyWithin(1,0,3);if(a.join(',')!=='1,1,2,3')throw 5;
        a.set(a.subarray(0,3),1);
        a.join(',')==='1,1,1,2'&&a.filter(x=>x===1).length===3
    "#,
    );
}

#[test]
fn shared_raw_clones_and_species_copies_preserve_nan_payloads() {
    check(
        r#"
        let b=new SharedArrayBuffer(8),d=new DataView(b),bits=0x7ff8000000000001n;
        d.setBigUint64(0,bits,true);
        let source=new Float64Array(b),clone=new Float64Array(source);
        if(!(clone.buffer instanceof ArrayBuffer)||clone.buffer===b||new DataView(clone.buffer).getBigUint64(0,true)!==bits)throw 1;
        if(new DataView(source.slice().buffer).getBigUint64(0,true)!==bits)throw 2;
        source.constructor={[Symbol.species]:function(length){return new Float64Array(new SharedArrayBuffer(length*8))}};
        let shared=source.slice();
        shared.buffer instanceof SharedArrayBuffer&&shared.buffer!==b&&new DataView(shared.buffer).getBigUint64(0,true)===bits
    "#,
    );
}

#[test]
fn shared_indexed_conversion_observes_growth_and_uint8_text_codecs_use_live_layouts() {
    check(
        r#"
        let b=new SharedArrayBuffer(0,{maxByteLength:8}),a=new Uint8Array(b);
        a[0]={valueOf(){b.grow(4);return 7}};
        if(a[0]!==7||a.length!==4)throw 1;
        b.grow(8);
        let hex=a.setFromHex('0102030405060708');
        if(hex.read!==16||hex.written!==8||a.toHex()!=='0102030405060708')throw 2;
        let base=a.setFromBase64('CQoLDA==');
        base.read===8&&base.written===4&&a.subarray(0,4).toBase64()==='CQoLDA=='&&a.toHex()==='090a0b0c05060708'
    "#,
    );
}

#[test]
fn shared_views_ignore_arraylike_hooks_and_preserve_integer_indexed_integrity_rules() {
    check(
        r#"
        let b=new SharedArrayBuffer(8,{maxByteLength:16}),hits=0;
        Object.defineProperty(b,'length',{get(){hits++;throw 1}});
        Object.defineProperty(b,Symbol.iterator,{get(){hits++;throw 2}});
        let a=new Uint16Array(b),d=new DataView(b),fixed=new Uint8Array(b,0,1);
        if(hits!==0||a.length!==4||d.byteLength!==8||Reflect.preventExtensions(a)||Reflect.preventExtensions(fixed))throw 3;
        b.grow(16);if(a.length!==8||d.byteLength!==16)throw 4;
        let f=new Uint8Array(new SharedArrayBuffer(4));
        if(!Reflect.preventExtensions(f)||Object.isExtensible(f))throw 5;
        f[0]=9;let desc=Object.getOwnPropertyDescriptor(f,'0');
        desc.value===9&&desc.writable&&desc.enumerable&&desc.configurable&&!Reflect.deleteProperty(f,'0')
    "#,
    );
}

#[test]
fn shared_and_ordinary_copy_directions_preserve_independent_values_and_conversion() {
    check(
        r#"
        let source=new Uint8Array(new SharedArrayBuffer(8));source.set([1,2,3,4,5,6,7,8]);
        let ordinary=new Uint8Array(source),target=new Uint8Array(new SharedArrayBuffer(8));
        target.set(ordinary);if(target.join(',')!=='1,2,3,4,5,6,7,8')throw 1;
        source[0]=9;target.set(source);
        if(target[0]!==9||ordinary[0]!==1)throw 2;
        let wider=new Uint16Array(source);
        wider.length===8&&wider[0]===9&&wider.buffer instanceof ArrayBuffer&&wider[7]===8
    "#,
    );
}
