//! DataView byte access, conversion order, and resizable-buffer witnesses (25.3).

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const TYPES: [(&str, usize); 11] = [
    ("Int8", 1),
    ("Uint8", 1),
    ("Int16", 2),
    ("Uint16", 2),
    ("Int32", 4),
    ("Uint32", 4),
    ("Float16", 2),
    ("Float32", 4),
    ("Float64", 8),
    ("BigInt64", 8),
    ("BigUint64", 8),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn exception(source: &str, expected: ExceptionKind) {
    let result = Realm::default().eval(source);
    assert!(
        matches!(&result, Err(Error::Exception { kind, .. }) if *kind == expected),
        "{source}: {result:?}"
    );
}

#[test]
fn construction_requires_new_and_an_array_buffer_before_offset_conversion() {
    check(
        "let hits=0,caught=false;try{DataView(new ArrayBuffer(3),{valueOf(){hits++;throw 7;}});}catch(e){caught=e instanceof TypeError;}caught && hits===0",
    );
    for buffer in [
        "undefined",
        "null",
        "false",
        "3",
        "'x'",
        "1n",
        "Symbol()",
        "{}",
        "ArrayBuffer.prototype",
        "Object.create(new ArrayBuffer(3))",
        "{get byteLength(){throw 7;},[Symbol.toPrimitive](){throw 8;}}",
    ] {
        check(&format!(
            "let hits=0,caught=false;try{{new DataView({buffer},{{valueOf(){{hits++;throw 9;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
    check(
        "let b=new ArrayBuffer(5),a=new DataView(b),c=new DataView(b,2),d=new DataView(b,5),e=new DataView(b,1,2);a.buffer===b && a.byteOffset===0 && a.byteLength===5 && c.byteOffset===2 && c.byteLength===3 && d.byteOffset===5 && Object.is(d.byteLength,0) && e.byteOffset===1 && e.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(5),C=DataView.bind(null,b,1);let v=new C(2);v.buffer===b && v.byteOffset===1 && v.byteLength===2 && v instanceof DataView",
    );
}

#[test]
fn constructor_indices_truncate_and_do_not_wrap() {
    for (input, offset) in [
        ("undefined", 0),
        ("null", 0),
        ("false", 0),
        ("true", 1),
        ("NaN", 0),
        ("-0", 0),
        ("-0.9", 0),
        ("2.9", 2),
        ("'3'", 3),
    ] {
        check(&format!(
            "let b=new ArrayBuffer(5),v=new DataView(b,{input},1);Object.is(v.byteOffset,{offset}) && v.byteLength===1 && v.buffer===b"
        ));
    }
    for (input, length) in [
        ("null", 0),
        ("false", 0),
        ("true", 1),
        ("NaN", 0),
        ("-0.9", 0),
        ("2.9", 2),
        ("'3'", 3),
    ] {
        check(&format!(
            "let v=new DataView(new ArrayBuffer(5),1,{input});Object.is(v.byteLength,{length}) && v.byteOffset===1"
        ));
    }
    for input in ["-1", "Infinity", "-Infinity", "9007199254740992", "6"] {
        exception(
            &format!("new DataView(new ArrayBuffer(5),{input})"),
            ExceptionKind::RangeError,
        );
        exception(
            &format!("new DataView(new ArrayBuffer(5),0,{input})"),
            ExceptionKind::RangeError,
        );
    }
    for input in ["1n", "Symbol()"] {
        exception(
            &format!("new DataView(new ArrayBuffer(5),{input})"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("new DataView(new ArrayBuffer(5),0,{input})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn constructor_converts_offset_before_detachment_and_old_bounds_before_length() {
    check(
        "let b=new ArrayBuffer(3);b.transfer();let log='',caught=false;try{new DataView(b,{valueOf(){log+='o';throw 7;}},{valueOf(){log+='l';return 0;}});}catch(e){caught=e===7;}caught && log==='o'",
    );
    check(
        "let b=new ArrayBuffer(3),log='',caught=false;try{new DataView(b,{valueOf(){log+='o';b.transfer();return 0;}},{valueOf(){log+='l';return 0;}});}catch(e){caught=e instanceof TypeError;}caught && log==='o'",
    );
    check(
        "let b=new ArrayBuffer(3),log='',caught=false;try{new DataView(b,{valueOf(){log+='o';return 4;}},{valueOf(){log+='l';throw 7;}});}catch(e){caught=e instanceof RangeError;}caught && log==='o'",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:8}),log='';let v=new DataView(b,{valueOf(){log+='o';b.resize(5);return 3;}},{valueOf(){log+='l';return 2;}});log==='ol' && v.byteOffset===3 && v.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:8}),log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 9;}});try{Reflect.construct(DataView,[b,0,{valueOf(){log+='l';b.resize(4);return 4;}}],N);}catch(e){caught=e instanceof RangeError;}caught && log==='l' && b.byteLength===4",
    );
}

#[test]
fn constructor_converts_before_newtarget_lookup_and_initializes_subclass_elements() {
    check(
        "let log='',p={},b=new ArrayBuffer(5);function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return p;}});let v=Reflect.construct(DataView,[b,{valueOf(){log+='o';return 1;}},{valueOf(){log+='l';return 2;}}],N);log==='olp' && Object.getPrototypeOf(v)===p && Object.getOwnPropertyDescriptor(DataView.prototype,'buffer').get.call(v)===b && Object.getOwnPropertyDescriptor(DataView.prototype,'byteLength').get.call(v)===2",
    );
    check(
        "function C(){}C.prototype=7;let v=Reflect.construct(DataView,[new ArrayBuffer(3)],C);Object.getPrototypeOf(v)===DataView.prototype && v.byteLength===3",
    );
    check(
        "let b=new ArrayBuffer(5);class V extends DataView{#n=this.byteLength;#read(){return this.#n;}read(){return this.#read();}}let v=new V(b,1,3);v instanceof V && v instanceof DataView && v.buffer===b && v.byteOffset===1 && v.read()===3",
    );
}

#[test]
fn constructor_rechecks_current_buffer_after_length_conversion_and_prototype_lookup() {
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return DataView.prototype;}});try{Reflect.construct(DataView,[b,0,{valueOf(){log+='l';b.resize(2);return 4;}}],N);}catch(e){caught=e instanceof RangeError;}caught && log==='lp' && b.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.resize(1);return DataView.prototype;}});try{Reflect.construct(DataView,[b,2],N);}catch(e){caught=e instanceof RangeError;}caught && b.byteLength===1",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.resize(2);return DataView.prototype;}});try{Reflect.construct(DataView,[b,1,3],N);}catch(e){caught=e instanceof RangeError;}caught && b.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8});function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.resize(7);return DataView.prototype;}});let v=Reflect.construct(DataView,[b,1],N);v.byteOffset===1 && v.byteLength===6",
    );
    check(
        "let b=new ArrayBuffer(4),log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 9;}});try{Reflect.construct(DataView,[b,0,{valueOf(){log+='l';b.transfer();return 2;}}],N);}catch(e){caught=e===9;}caught && log==='lp' && b.detached",
    );
    check(
        "let b=new ArrayBuffer(4),caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.transfer();return DataView.prototype;}});try{Reflect.construct(DataView,[b],N);}catch(e){caught=e instanceof TypeError;}caught && b.detached",
    );
}

#[test]
fn descriptor_and_resizable_state_matrix() {
    let Value::String(output) = Realm::default()
        .eval(
            r"let rows=[];for(let name of ['buffer','byteLength','byteOffset']){let d=Object.getOwnPropertyDescriptor(DataView.prototype,name);rows.push(name+' '+[d.get.name,d.get.length,d.set===undefined,d.enumerable,d.configurable,Object.hasOwn(d.get,'prototype')].join(','));}for(let type of ['Int8','Uint8','Int16','Uint16','Int32','Uint32','Float16','Float32','Float64','BigInt64','BigUint64']){let g=Object.getOwnPropertyDescriptor(DataView.prototype,'get'+type),s=Object.getOwnPropertyDescriptor(DataView.prototype,'set'+type);rows.push(type+' '+[g.value.name,g.value.length,s.value.name,s.value.length,g.writable,s.writable,g.enumerable,s.enumerable,g.configurable,s.configurable,Object.hasOwn(g.value,'prototype'),Object.hasOwn(s.value,'prototype')].join(','));}let b=new ArrayBuffer(6,{maxByteLength:10}),fixed=new DataView(b,2,3),auto=new DataView(b,2);function state(name,v){let size,offset;try{size=v.byteLength;}catch(e){size=e instanceof TypeError?'TypeError':'other';}try{offset=v.byteOffset;}catch(e){offset=e instanceof TypeError?'TypeError':'other';}rows.push(name+' '+[size,offset,v.buffer===b,ArrayBuffer.isView(v)].join(','));}state('fixed',fixed);state('auto',auto);b.resize(3);state('fixed-small',fixed);state('auto-small',auto);b.resize(2);state('auto-empty',auto);b.resize(1);state('auto-outside',auto);b.resize(8);state('fixed-regrown',fixed);state('auto-regrown',auto);b.transfer();state('fixed-detached',fixed);state('auto-detached',auto);rows.join('\n')",
        )
        .unwrap()
    else {
        panic!("DataView metadata and states must be a string")
    };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @"
    buffer get buffer,0,true,false,true,false
    byteLength get byteLength,0,true,false,true,false
    byteOffset get byteOffset,0,true,false,true,false
    Int8 getInt8,1,setInt8,2,true,true,false,false,true,true,false,false
    Uint8 getUint8,1,setUint8,2,true,true,false,false,true,true,false,false
    Int16 getInt16,1,setInt16,2,true,true,false,false,true,true,false,false
    Uint16 getUint16,1,setUint16,2,true,true,false,false,true,true,false,false
    Int32 getInt32,1,setInt32,2,true,true,false,false,true,true,false,false
    Uint32 getUint32,1,setUint32,2,true,true,false,false,true,true,false,false
    Float16 getFloat16,1,setFloat16,2,true,true,false,false,true,true,false,false
    Float32 getFloat32,1,setFloat32,2,true,true,false,false,true,true,false,false
    Float64 getFloat64,1,setFloat64,2,true,true,false,false,true,true,false,false
    BigInt64 getBigInt64,1,setBigInt64,2,true,true,false,false,true,true,false,false
    BigUint64 getBigUint64,1,setBigUint64,2,true,true,false,false,true,true,false,false
    fixed 3,2,true,true
    auto 4,2,true,true
    fixed-small TypeError,TypeError,true,true
    auto-small 1,2,true,true
    auto-empty 0,2,true,true
    auto-outside TypeError,TypeError,true,true
    fixed-regrown 3,2,true,true
    auto-regrown 6,2,true,true
    fixed-detached TypeError,TypeError,true,true
    auto-detached TypeError,TypeError,true,true
    ");
}

#[test]
fn intrinsic_function_properties_and_constructibility_follow_the_specification() {
    check(
        "let D=DataView,p=D.prototype,g=Object.getOwnPropertyDescriptor(globalThis,'DataView'),d=Object.getOwnPropertyDescriptor(D,'prototype'),c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag);g.value===D && g.writable && !g.enumerable && g.configurable && D.name==='DataView' && D.length===1 && d.value===p && !d.writable && !d.enumerable && !d.configurable && c.value===D && c.writable && !c.enumerable && c.configurable && Object.getPrototypeOf(D)===Function.prototype && Object.getPrototypeOf(p)===Object.prototype && t.value==='DataView' && !t.writable && !t.enumerable && t.configurable && Object.prototype.toString.call(new D(new ArrayBuffer(0)))==='[object DataView]' && Object.prototype.toString.call(p)==='[object DataView]' && Reflect.ownKeys(new D(new ArrayBuffer(0))).length===0 && !Object.hasOwn(D,Symbol.species)",
    );
    for (suffix, _) in TYPES {
        for prefix in ["get", "set"] {
            let name = format!("{prefix}{suffix}");
            check(&format!(
                "let f=DataView.prototype.{name},n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');n.value==='{name}' && !n.writable && !n.enumerable && n.configurable && !l.writable && !l.enumerable && l.configurable"
            ));
            exception(
                &format!("new DataView.prototype.{name}"),
                ExceptionKind::TypeError,
            );
        }
    }
    for name in ["buffer", "byteLength", "byteOffset"] {
        exception(
            &format!("new (Object.getOwnPropertyDescriptor(DataView.prototype,'{name}').get)"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn getters_and_access_methods_require_internal_slots_before_argument_conversion() {
    for receiver in [
        "undefined",
        "null",
        "3",
        "'x'",
        "1n",
        "Symbol()",
        "{}",
        "new ArrayBuffer(8)",
        "DataView.prototype",
        "Object.create(new DataView(new ArrayBuffer(8)))",
    ] {
        for name in ["buffer", "byteLength", "byteOffset"] {
            exception(
                &format!(
                    "Object.getOwnPropertyDescriptor(DataView.prototype,'{name}').get.call({receiver})"
                ),
                ExceptionKind::TypeError,
            );
        }
        // Every numeric member shares the same DataView brand requirement.
        for (suffix, _) in TYPES {
            check(&format!(
                "let hits=0,caught=false;try{{DataView.prototype.get{suffix}.call({receiver},{{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
            check(&format!(
                "let hits=0,caught=false;try{{DataView.prototype.set{suffix}.call({receiver},{{valueOf(){{hits++;throw 7;}}}},{{valueOf(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
}

#[test]
fn is_view_remains_true_for_detached_and_out_of_bounds_data_views() {
    check(
        "let b=new ArrayBuffer(5,{maxByteLength:8}),v=new DataView(b,2,3),inherited=Object.create(v);ArrayBuffer.isView(v) && !ArrayBuffer.isView(inherited) && !ArrayBuffer.isView(DataView.prototype) && !ArrayBuffer.isView(b) && !ArrayBuffer.isView({buffer:b,byteOffset:2,byteLength:3})",
    );
    check(
        "let b=new ArrayBuffer(5,{maxByteLength:8}),v=new DataView(b,2,3);b.resize(1);let outside=ArrayBuffer.isView(v) && v.buffer===b;b.transfer();outside && ArrayBuffer.isView(v) && v.buffer===b",
    );
}

#[test]
fn byte_storage_is_zero_initialized_shared_and_relative_to_each_view_offset() {
    check(
        "let b=new ArrayBuffer(12),a=new DataView(b),v=new DataView(b,2,8);a.getBigUint64(0)===0n && a.getUint32(8)===0 && v.setUint32(1,16909060)===undefined && a.getUint8(2)===0 && a.getUint8(3)===1 && a.getUint8(4)===2 && a.getUint8(5)===3 && a.getUint8(6)===4 && a.getUint8(7)===0 && v.getUint32(1)===16909060 && v.getUint32(1,true)===67305985",
    );
    check(
        "let v=new DataView(new ArrayBuffer(10),1,8);v.setFloat64(0,-0);Object.is(v.getFloat64(0),-0) && v.getUint8(0)===128 && v.getUint8(7)===0",
    );
}

#[test]
fn safe_integer_offsets_address_sparse_bytes_without_machine_width_wrapping() {
    check(
        "let b=new ArrayBuffer(9007199254740991),whole=new DataView(b),high=new DataView(b,4294967296,16),tail=new DataView(b,9007199254740983,8);high.setUint32(8,16909060);tail.setBigUint64(0,72623859790382856n);whole.getUint32(4294967304)===16909060 && whole.getBigUint64(9007199254740983)===72623859790382856n && whole.getUint8(9007199254740990)===8 && new DataView(b,9007199254740991).byteLength===0",
    );
    exception(
        "new DataView(new ArrayBuffer(9007199254740991)).getUint16(9007199254740990)",
        ExceptionKind::RangeError,
    );
    exception(
        "new DataView(new ArrayBuffer(9007199254740991),9007199254740991,1)",
        ExceptionKind::RangeError,
    );
}

#[test]
fn integer_setters_wrap_truncate_and_reinterpret_signed_byte_patterns() {
    for (suffix, input, expected) in [
        ("Int8", "257.9", "1"),
        ("Int8", "-129", "127"),
        ("Uint8", "-1", "255"),
        ("Uint8", "Infinity", "0"),
        ("Int16", "65535", "-1"),
        ("Int16", "-65537", "-1"),
        ("Uint16", "65537.9", "1"),
        ("Uint16", "-1", "65535"),
        ("Int32", "4294967295", "-1"),
        ("Int32", "4294967297.9", "1"),
        ("Uint32", "-1", "4294967295"),
        ("Uint32", "-4294967297", "4294967295"),
    ] {
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8));v.set{suffix}(0,{input});v.get{suffix}(0)==={expected}"
        ));
    }
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint32(1,2147483648);v.getInt32(1)===-2147483648 && v.getUint8(1)===128 && v.getUint8(2)===0 && v.getUint8(3)===0 && v.getUint8(4)===0",
    );
    for (suffix, _) in TYPES {
        if suffix.starts_with("Big") {
            continue;
        }
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),calls=0,hint;v.set{suffix}(0,{{[Symbol.toPrimitive](h){{calls++;hint=h;return '1';}}}});calls===1 && hint==='number' && v.get{suffix}(0)===1"
        ));
    }
}

#[test]
fn endian_arguments_use_to_boolean_without_invoking_hooks() {
    for suffix in ["Int16", "Uint16", "Int32", "Uint32"] {
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),endian={{[Symbol.toPrimitive](){{throw 7;}},valueOf(){{throw 8;}}}};v.set{suffix}(0,258,endian);v.getUint8(0)===2 && v.getUint8(1)===1 && v.get{suffix}(0,endian)===258 && v.get{suffix}(0,true)===258"
        ));
    }
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint16(0,258);v.setUint16(2,258,false);v.setUint16(4,258,0);v.setUint16(6,258,'');v.getUint8(0)===1 && v.getUint8(1)===2 && v.getUint8(2)===1 && v.getUint8(3)===2 && v.getUint8(4)===1 && v.getUint8(5)===2 && v.getUint8(6)===1 && v.getUint8(7)===2",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint16(0,258,'false');v.setUint16(2,258,1n);v.setInt8(4,258,{valueOf(){throw 7;}});v.getUint16(0,true)===258 && v.getUint16(2,true)===258 && v.getInt8(4,{valueOf(){throw 8;}})===2",
    );
}

#[test]
fn floating_members_preserve_signed_zero_infinities_nan_and_explicit_byte_order() {
    for (suffix, width, one_bytes) in [
        ("Float16", 2, "60,0"),
        ("Float32", 4, "63,128,0,0"),
        ("Float64", 8, "63,240,0,0,0,0,0,0"),
    ] {
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),a=[];v.set{suffix}(0,1);for(let i=0;i<{width};i++)a.push(v.getUint8(i));a.join()==='{one_bytes}' && v.get{suffix}(0)===1 && v.set{suffix}(0,-0)===undefined && Object.is(v.get{suffix}(0),-0) && v.set{suffix}(0,Infinity)===undefined && v.get{suffix}(0)===Infinity && v.set{suffix}(0,-Infinity)===undefined && v.get{suffix}(0)===-Infinity && v.set{suffix}(0,NaN)===undefined && Number.isNaN(v.get{suffix}(0))"
        ));
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),a=[];v.set{suffix}(0,1,true);for(let i={width}-1;i>=0;i--)a.push(v.getUint8(i));a.join()==='{one_bytes}' && v.get{suffix}(0,true)===1"
        ));
    }
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint16(0,32769);Number.isNaN(v.getFloat16(0))===false && v.getFloat16(0)===-(2**-24) && v.setUint16(0,32257)===undefined && Number.isNaN(v.getFloat16(0))",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint32(0,1);v.getFloat32(0)===2**-149 && v.setBigUint64(0,1n)===undefined && v.getFloat64(0)===Number.MIN_VALUE",
    );
}

#[test]
fn floating_stores_round_once_to_the_requested_format() {
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setFloat16(0,1+2**-11);v.getFloat16(0)===1 && v.setFloat16(0,1+2**-11+Number.EPSILON)===undefined && v.getFloat16(0)===1+2**-10 && v.setFloat16(0,1+3*2**-11)===undefined && v.getFloat16(0)===1+2**-9",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setFloat16(0,2**-25);Object.is(v.getFloat16(0),0) && v.setFloat16(0,-(2**-25))===undefined && Object.is(v.getFloat16(0),-0) && v.setFloat16(0,3*2**-25)===undefined && v.getFloat16(0)===2**-23 && v.setFloat16(0,65520)===undefined && v.getFloat16(0)===Infinity",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setFloat32(0,1+2**-24);v.getFloat32(0)===1 && v.setFloat32(0,1+3*2**-24)===undefined && v.getFloat32(0)===1+2**-22 && v.setFloat32(0,2**-150)===undefined && Object.is(v.getFloat32(0),0) && v.setFloat64(0,1+Number.EPSILON)===undefined && v.getFloat64(0)===1+Number.EPSILON",
    );
}

#[test]
fn bigint_members_wrap_modulo_64_bits_and_preserve_endianness() {
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setBigUint64(0,72623859790382856n);let a=[];for(let i=0;i<8;i++)a.push(v.getUint8(i));a.join()==='1,2,3,4,5,6,7,8' && v.getBigUint64(0)===72623859790382856n && v.getBigUint64(0,true)===578437695752307201n",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setBigInt64(0,-1n);v.getBigInt64(0)===-1n && v.getBigUint64(0)===18446744073709551615n && v.setBigInt64(0,9223372036854775808n)===undefined && v.getBigInt64(0)===-9223372036854775808n",
    );
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setBigUint64(0,18446744073709551617n,true);v.getBigUint64(0,true)===1n && v.getUint8(0)===1 && v.getUint8(7)===0 && v.setBigInt64(0,-18446744073709551617n)===undefined && v.getBigInt64(0)===-1n",
    );
}

#[test]
fn bigint_and_number_setters_use_distinct_primitive_conversions() {
    check(
        "let v=new DataView(new ArrayBuffer(8));v.setUint8(0)===undefined && v.getUint8()===0 && v.setFloat64(0)===undefined && Number.isNaN(v.getFloat64(0)) && v.setFloat64(0,null)===undefined && Object.is(v.getFloat64(0),0)",
    );
    for suffix in ["BigInt64", "BigUint64"] {
        for value in ["true", "'1'", "Object(1n)", "{valueOf(){return 1n;}}"] {
            check(&format!(
                "let v=new DataView(new ArrayBuffer(8));v.set{suffix}(0,{value});v.get{suffix}(0)===1n"
            ));
        }
        for value in ["undefined", "null", "1", "Object(1)", "Symbol()"] {
            exception(
                &format!("new DataView(new ArrayBuffer(8)).set{suffix}(0,{value})"),
                ExceptionKind::TypeError,
            );
        }
        exception(
            &format!("new DataView(new ArrayBuffer(8)).set{suffix}(0,'1.5')"),
            ExceptionKind::SyntaxError,
        );
    }
    for (suffix, _) in TYPES {
        if suffix.starts_with("Big") {
            continue;
        }
        for value in ["1n", "Object(1n)", "Symbol()"] {
            exception(
                &format!("new DataView(new ArrayBuffer(8)).set{suffix}(0,{value})"),
                ExceptionKind::TypeError,
            );
        }
    }
}

#[test]
fn access_indices_truncate_without_wrapping_and_check_the_entire_element() {
    for (suffix, width) in TYPES {
        let value = if suffix.starts_with("Big") { "1n" } else { "1" };
        check(&format!(
            "let v=new DataView(new ArrayBuffer(10));v.set{suffix}(2.9,{value});v.get{suffix}(2)==={value} && v.get{suffix}('2')==={value}"
        ));
        check(&format!(
            "let v=new DataView(new ArrayBuffer({width}));v.set{suffix}(undefined,{value});v.get{suffix}(NaN)==={value} && v.get{suffix}(-0.9)==={value}"
        ));
        exception(
            &format!("new DataView(new ArrayBuffer({width})).get{suffix}(1)"),
            ExceptionKind::RangeError,
        );
        exception(
            &format!("new DataView(new ArrayBuffer({width})).set{suffix}(1,{value})"),
            ExceptionKind::RangeError,
        );
        for input in ["-1", "Infinity", "9007199254740992"] {
            exception(
                &format!("new DataView(new ArrayBuffer(8)).get{suffix}({input})"),
                ExceptionKind::RangeError,
            );
        }
        for input in ["1n", "Symbol()"] {
            exception(
                &format!("new DataView(new ArrayBuffer(8)).get{suffix}({input})"),
                ExceptionKind::TypeError,
            );
        }
    }
}

#[test]
fn explicit_lengths_stay_fixed_while_omitted_lengths_follow_buffer_resize() {
    check(
        "let b=new ArrayBuffer(6,{maxByteLength:10}),fixed=new DataView(b,2,3),auto=new DataView(b,2),zero=new DataView(b,6,0);b.resize(10);fixed.byteLength===3 && fixed.byteOffset===2 && auto.byteLength===8 && auto.byteOffset===2 && zero.byteLength===0 && zero.byteOffset===6",
    );
    check(
        "let b=new ArrayBuffer(6,{maxByteLength:10}),auto=new DataView(b,2);b.resize(2);let caught=false;try{auto.getUint8(0);}catch(e){caught=e instanceof RangeError;}caught && auto.byteLength===0 && auto.byteOffset===2",
    );
    check(
        "let b=new ArrayBuffer(6,{maxByteLength:10}),fixed=new DataView(b,2,3),auto=new DataView(b,2);b.resize(3);let caught=false;try{fixed.getUint8(0);}catch(e){caught=e instanceof TypeError;}caught && auto.byteLength===1 && auto.getUint8(0)===0 && fixed.buffer===b",
    );
    check(
        "let b=new ArrayBuffer(6,{maxByteLength:10}),fixed=new DataView(b,2,3),auto=new DataView(b,2);b.resize(1);b.resize(8);fixed.byteLength===3 && fixed.byteOffset===2 && auto.byteLength===6 && auto.byteOffset===2 && fixed.getUint8(0)===0",
    );
}

#[test]
fn shrinking_discards_tail_bytes_and_regrowth_zero_fills_across_sparse_pages() {
    check(
        "let b=new ArrayBuffer(8200,{maxByteLength:12300}),v=new DataView(b);v.setUint32(4094,16909060);v.setUint8(8199,9);b.resize(4096);b.resize(8200);v.getUint8(4094)===1 && v.getUint8(4095)===2 && v.getUint8(4096)===0 && v.getUint8(4097)===0 && v.getUint8(8199)===0",
    );
    check(
        "let b=new ArrayBuffer(8,{maxByteLength:12}),v=new DataView(b);v.setBigUint64(0,18446744073709551615n);b.resize(0);b.resize(12);v.byteLength===12 && v.getBigUint64(0)===0n && v.getUint32(8)===0",
    );
}

#[test]
fn getters_convert_indices_before_bounds_checks_and_can_repair_a_view() {
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,0,4);b.resize(1);let log='',caught=false;try{v.getUint32({valueOf(){log+='i';throw 7;}});}catch(e){caught=e===7;}caught && log==='i'",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,0,4);b.resize(1);v.getUint32({valueOf(){b.resize(4);return 0;}})===0",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b);v.getUint32({valueOf(){b.resize(8);return 4;}})===0 && v.byteLength===8",
    );
    check(
        "let b=new ArrayBuffer(4),v=new DataView(b);b.transfer();let caught=false;try{v.getUint8(-1);}catch(e){caught=e instanceof RangeError;}caught",
    );
    check(
        "let b=new ArrayBuffer(4),v=new DataView(b),caught=false;try{v.getUint8({valueOf(){b.transfer();return 0;}});}catch(e){caught=e instanceof TypeError;}caught && b.detached",
    );
}

#[test]
fn setters_convert_index_then_value_before_bounds_without_converting_endian_objects() {
    for (suffix, _) in TYPES {
        let value = if suffix.starts_with("Big") { "1n" } else { "1" };
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),log='';v.set{suffix}({{valueOf(){{log+='i';return 0;}}}},{{[Symbol.toPrimitive](hint){{log+=hint;return {value};}}}},{{[Symbol.toPrimitive](){{throw 7;}}}});log==='inumber' && v.get{suffix}(0,true)==={value}"
        ));
        check(&format!(
            "let v=new DataView(new ArrayBuffer(8)),hits=0,caught=false;try{{v.set{suffix}(-1,{{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e instanceof RangeError;}}caught && hits===0"
        ));
        check(&format!(
            "let v=new DataView(new ArrayBuffer(0)),hits=0,caught=false;try{{v.set{suffix}(0,{{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e===7;}}caught && hits===1"
        ));
        check(&format!(
            "let b=new ArrayBuffer(8),v=new DataView(b);b.transfer();let hits=0,caught=false;try{{v.set{suffix}(0,{{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e===7;}}caught && hits===1"
        ));
    }
}

#[test]
fn setter_conversion_reentrancy_uses_current_bounds_and_retains_prior_effects() {
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b),log='';v.setUint32({valueOf(){log+='i';b.resize(6);return 2;}},{valueOf(){log+='v';b.resize(8);return 16909060;}});log==='iv' && v.byteLength===8 && v.getUint32(2)===16909060",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,0,4);b.resize(1);v.setUint32(0,{valueOf(){b.resize(4);return 16909060;}});v.getUint32(0)===16909060",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b,0,4),caught=false;try{v.setUint32(0,{valueOf(){b.resize(2);return 16909060;}});}catch(e){caught=e instanceof TypeError;}b.resize(4);caught && v.getUint32(0)===0",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),v=new DataView(b),caught=false;try{v.setUint32(0,{valueOf(){b.resize(2);return 16909060;}});}catch(e){caught=e instanceof RangeError;}caught && v.byteLength===2 && v.getUint16(0)===0",
    );
    check(
        "let b=new ArrayBuffer(8),v=new DataView(b),moved,caught=false;v.setUint8(0,9);try{v.setBigUint64(0,{valueOf(){moved=b.transfer();return 1n;}});}catch(e){caught=e instanceof TypeError;}caught && b.detached && new DataView(moved).getUint8(0)===9",
    );
    check(
        "let b=new ArrayBuffer(4),v=new DataView(b),caught=false;try{v.setUint32(0,{valueOf(){v.setUint8(0,9);throw 7;}});}catch(e){caught=e===7;}caught && v.getUint8(0)===9 && v.getUint8(1)===0",
    );
}

#[test]
fn transfer_detaches_every_existing_view_and_preserves_the_moved_bytes() {
    for method in ["transfer", "transferToFixedLength"] {
        check(&format!(
            "let b=new ArrayBuffer(8,{{maxByteLength:12}}),a=new DataView(b),c=new DataView(b,2,4);a.setUint32(2,16909060);let moved=b.{method}(10),v=new DataView(moved),caught=0;for(let old of [a,c]){{try{{old.byteLength;}}catch(e){{if(e instanceof TypeError)caught++;}}try{{old.byteOffset;}}catch(e){{if(e instanceof TypeError)caught++;}}try{{old.getUint8(0);}}catch(e){{if(e instanceof TypeError)caught++;}}}}caught===6 && a.buffer===b && c.buffer===b && ArrayBuffer.isView(a) && ArrayBuffer.isView(c) && v.getUint32(2)===16909060 && v.getUint16(8)===0"
        ));
    }
}

#[test]
fn array_buffer_slice_copies_live_bytes_and_preserves_unwritten_species_bytes() {
    check(
        "let b=new ArrayBuffer(8),v=new DataView(b);for(let i=0;i<8;i++)v.setUint8(i,i+1);let copy=b.slice(2,6),c=new DataView(copy);c.byteLength===4 && c.getUint32(0)===50595078 && c.setUint8(0,9)===undefined && v.getUint8(2)===3",
    );
    check(
        "let b=new ArrayBuffer(6,{maxByteLength:10}),v=new DataView(b),target=new ArrayBuffer(6),t=new DataView(target);v.setUint32(0,16909060);for(let i=0;i<6;i++)t.setUint8(i,9);function C(n){b.resize(2);return target;}b.constructor={[Symbol.species]:C};b.slice(1,5)===target && t.getUint8(0)===2 && t.getUint8(1)===9 && t.getUint8(2)===9 && t.getUint8(3)===9 && t.getUint8(4)===9 && t.getUint8(5)===9",
    );
    check(
        "let b=new ArrayBuffer(8),target=new ArrayBuffer(8),t=new DataView(target);for(let i=0;i<8;i++)t.setUint8(i,9);new DataView(b).setUint8(3,7);function C(){return target;}b.constructor={[Symbol.species]:C};b.slice(1,6)===target && t.getUint8(0)===0 && t.getUint8(1)===0 && t.getUint8(2)===7 && t.getUint8(3)===0 && t.getUint8(4)===0 && t.getUint8(5)===9 && t.getUint8(7)===9",
    );
}

#[test]
fn views_retain_their_buffer_and_subclass_captures_through_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let view,read,get,set;(()=>{let capture={n:7},b=new ArrayBuffer(12,{maxByteLength:16});class V extends DataView{#capture=capture;read(){return this.#capture.n;}}view=new V(b,2,8);view.setBigUint64(0,72623859790382856n);read=view.read;get=view.getBigUint64;set=view.setUint8;})();",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "read.call(view)===7 && get.call(view,0)===72623859790382856n && view.buffer.byteLength===12 && view.byteOffset===2 && set.call(view,7,9)===undefined && get.call(view,0)===72623859790382857n",
        ),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("view.buffer.resize(3);view.buffer.resize(12)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "read.call(view)===7 && view.byteLength===8 && view.byteOffset===2 && get.call(view,0)===72057594037927936n",
        ),
        Ok(Value::Boolean(true))
    );
}
