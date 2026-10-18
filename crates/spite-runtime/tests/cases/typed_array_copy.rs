//! TypedArray species copies, shared views, and intrinsic numeric copies (23.2.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const KINDS: [(&str, bool); 12] = [
    ("Int8Array", false),
    ("Uint8Array", false),
    ("Uint8ClampedArray", false),
    ("Int16Array", false),
    ("Uint16Array", false),
    ("Int32Array", false),
    ("Uint32Array", false),
    ("BigInt64Array", true),
    ("BigUint64Array", true),
    ("Float16Array", false),
    ("Float32Array", false),
    ("Float64Array", false),
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
        matches!(&result,Err(Error::Exception { kind,.. }) if *kind==expected),
        "{source}: {result:?}"
    );
}

#[test]
fn all_twelve_kinds_distinguish_shared_views_from_independent_same_kind_copies() {
    for (name, bigint) in KINDS {
        let values = if bigint { "1n,2n,3n,4n" } else { "1,2,3,4" };
        let seven = if bigint { "7n" } else { "7" };
        let nine = if bigint { "9n" } else { "9" };
        check(&format!(
            "let A={name},a=A.of({values}),s=a.slice(1,-1),v=a.subarray(1,3),r=a.toReversed(),w=a.with(-2,{nine});let valid=s instanceof A && v instanceof A && r instanceof A && w instanceof A && s!==a && v!==a && r!==a && w!==a && s.buffer!==a.buffer && r.buffer!==a.buffer && w.buffer!==a.buffer && v.buffer===a.buffer && v.byteOffset===a.BYTES_PER_ELEMENT && s.join()==='2,3' && v.join()==='2,3' && r.join()==='4,3,2,1' && w.join()==='1,2,9,4' && a.join()==='1,2,3,4';v[0]={seven};valid && a.join()==='1,7,3,4' && s.join()==='2,3' && r.join()==='4,3,2,1' && w.join()==='1,2,9,4'"
        ));
        check(&format!(
            "let A={name},a=new A(0),s=a.slice(),v=a.subarray(),r=a.toReversed();s!==a && v!==a && r!==a && s instanceof A && v instanceof A && r instanceof A && s.length===0 && v.length===0 && r.length===0 && v.buffer===a.buffer && s.buffer!==a.buffer && r.buffer!==a.buffer"
        ));
    }
}

#[test]
fn methods_use_internal_layout_and_ignore_public_state_and_iteration() {
    check(
        "let a=Uint16Array.of(1,2,3);a.constructor=undefined;for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT',Symbol.iterator])Object.defineProperty(a,key,{get(){throw 7;}});let s=a.slice(1),v=a.subarray(1),r=a.toReversed(),w=a.with(1,7);s[0]===2 && s[1]===3 && v[0]===2 && v[1]===3 && r[0]===3 && r[2]===1 && w[0]===1 && w[1]===7 && w[2]===3 && a[1]===2",
    );
}

#[test]
fn brand_checks_and_initial_witnesses_precede_index_and_value_hooks() {
    for method in ["slice", "subarray", "toReversed", "with"] {
        for receiver in [
            "null",
            "{}",
            "new DataView(new ArrayBuffer(8))",
            "Object.create(new Uint8Array(3))",
        ] {
            exception(
                &format!(
                    "Uint8Array.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}},{{valueOf(){{throw 8;}}}})"
                ),
                ExceptionKind::TypeError,
            );
        }
    }
    for method in ["slice", "toReversed", "with"] {
        for invalidation in ["buffer.transfer()", "buffer.resize(1)"] {
            check(&format!(
                "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array(buffer,0,3),hits=0,caught=false;{invalidation};try{{a.{method}({{valueOf(){{hits++;throw 7;}}}},{{valueOf(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
}

#[test]
fn slice_and_subarray_convert_relative_bounds_before_species_lookup() {
    for method in ["slice", "subarray"] {
        for (arguments, expected) in [
            ("1,-1", "2,3"),
            ("-2,undefined", "3,4"),
            ("NaN,2.9", "1,2"),
            ("-Infinity,Infinity", "1,2,3,4"),
            ("Infinity,0", ""),
            ("0,-Infinity", ""),
        ] {
            check(&format!(
                "Uint8Array.of(1,2,3,4).{method}({arguments}).join()==='{expected}'"
            ));
        }
        check(&format!(
            "let a=Uint8Array.of(1,2,3,4),log='',argc;function C(){{log+='C';argc=arguments.length;return new Uint16Array(3);}}Object.defineProperty(a,'constructor',{{get(){{log+='c';return {{get [Symbol.species](){{log+='s';return C;}}}};}}}});a.{method}({{valueOf(){{log+='b';return 1;}}}},{{valueOf(){{log+='e';return 3;}}}});log==='becsC' && argc==={}",
            if method == "slice" { 1 } else { 3 }
        ));
        for arguments in ["Symbol()", "0,Symbol()", "1n"] {
            exception(
                &format!("new Uint8Array(0).{method}({arguments})"),
                ExceptionKind::TypeError,
            );
        }
        assert_eq!(Realm::default().eval(&format!("let a=Uint8Array.of(1);Object.defineProperty(a,'constructor',{{get(){{throw 8;}}}});a.{method}({{valueOf(){{throw 7;}}}})")),Err(Error::Thrown(Value::Number(7.0))));
        check(&format!(
            "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array(buffer);a.set([1,2,3]);let b=a.{method}({{valueOf(){{buffer.resize(6);a[5]=9;return -1;}}}});b.join()==='{}' && a.length===6",
            if method == "slice" { "3" } else { "3,0,0,9" }
        ));
    }
}

#[test]
fn species_defaults_retain_intrinsic_kind_when_public_constructors_change() {
    for method in ["slice", "subarray"] {
        for constructor in [
            "undefined",
            "{}",
            "{[Symbol.species]:undefined}",
            "{[Symbol.species]:null}",
        ] {
            check(&format!(
                "let A=Uint16Array,a=A.of(1,2,3),prototype=A.prototype;a.constructor={constructor};Uint16Array=function(){{throw 7;}};let b=a.{method}(1);Object.getPrototypeOf(b)===prototype && b.join()==='2,3' && b.BYTES_PER_ELEMENT===2"
            ));
        }
    }
}

#[test]
fn species_results_require_a_live_brand_and_matching_content_even_when_empty() {
    for method in ["slice", "subarray"] {
        for result in [
            "{}",
            "[]",
            "new DataView(new ArrayBuffer(3))",
            "Uint8Array.prototype",
            "Object.create(new Uint8Array(3))",
            "(()=>{let b=new Uint8Array(3);b.buffer.transfer();return b;})()",
            "(()=>{let buffer=new ArrayBuffer(3,{maxByteLength:6}),b=new Uint8Array(buffer,0,3);buffer.resize(1);return b;})()",
            "new BigInt64Array(3)",
        ] {
            exception(
                &format!(
                    "let a=Uint8Array.of(1,2,3);function C(){{return {result};}}a.constructor={{[Symbol.species]:C}};a.{method}(0,2)"
                ),
                ExceptionKind::TypeError,
            );
        }
        for (name, species) in [
            ("Uint8Array", "BigInt64Array"),
            ("BigInt64Array", "Uint8Array"),
        ] {
            exception(
                &format!(
                    "let a=new {name}(0);a.constructor={{[Symbol.species]:{species}}};a.{method}(0,0)"
                ),
                ExceptionKind::TypeError,
            );
        }
        for constructor in [
            "null",
            "7",
            "{[Symbol.species]:7}",
            "{[Symbol.species]:()=>new Uint8Array(3)}",
        ] {
            exception(
                &format!("let a=Uint8Array.of(1,2,3);a.constructor={constructor};a.{method}(0,2)"),
                ExceptionKind::TypeError,
            );
        }
    }
    exception(
        "let a=Uint8Array.of(1,2,3);function C(){return new Uint8Array(1);}a.constructor={[Symbol.species]:C};a.slice(0,2)",
        ExceptionKind::TypeError,
    );
    check(
        "let a=BigInt64Array.of(-1n,2n);a.constructor={[Symbol.species]:BigUint64Array};let b=a.slice();b instanceof BigUint64Array && b[0]===18446744073709551615n && b[1]===2n",
    );
}

#[test]
fn slice_uses_requested_length_before_clipping_and_preserves_custom_target_tails() {
    check(
        "let buffer=new ArrayBuffer(6,{maxByteLength:10}),a=new Uint8Array(buffer),count,target;a.set([1,2,3,4,5,6]);function C(n){count=n;buffer.resize(3);target=new Uint8Array(n+1);target.fill(9);return target;}a.constructor={[Symbol.species]:C};let b=a.slice(1,5);b===target && count===4 && b.join()==='2,3,9,9,9' && a.join()==='1,2,3'",
    );
    check(
        "let buffer=new ArrayBuffer(6,{maxByteLength:10}),a=new Uint8Array(buffer),count;function C(n){count=n;buffer.resize(2);return Uint8Array.of(7,8);}a.constructor={[Symbol.species]:C};let b=a.slice(4,6);count===2 && b.join()==='7,8' && a.length===2",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer),count;a.set([1,2,3]);function C(n){count=n;buffer.resize(6);a[5]=9;return new Uint8Array(n);}a.constructor={[Symbol.species]:C};let b=a.slice(0,Infinity);count===3 && b.join()==='1,2,3' && a.length===6 && a[5]===9",
    );
}

#[test]
fn slice_skips_source_revalidation_for_zero_count_and_validates_species_before_positive_rechecks() {
    for invalidation in ["a.buffer.transfer()", "buffer.resize(0)"] {
        check(&format!(
            "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array(buffer,0,3),count,argc,log='';function C(n){{log+='C';count=n;argc=arguments.length;return Uint8Array.of(7,8);}}a.constructor={{[Symbol.species]:C}};let b=a.slice({{valueOf(){{log+='b';{invalidation};return 2;}}}},{{valueOf(){{log+='e';return 1;}}}});log==='beC' && count===0 && argc===1 && b.join()==='7,8' && a.length===0"
        ));
    }
    check(
        "let a=Uint8Array.of(1,2,3),target,count,caught=false;function C(n){count=n;a.buffer.transfer();target=Uint8Array.of(7,8);return target;}a.constructor={[Symbol.species]:C};try{a.slice(0,2);}catch(e){caught=e instanceof TypeError;}caught && count===2 && target.join()==='7,8'",
    );
    assert_eq!(Realm::default().eval("let a=Uint8Array.of(1,2,3);function C(){throw 7;}a.constructor={[Symbol.species]:C};a.slice({valueOf(){a.buffer.transfer();return 0;}},2)"),Err(Error::Thrown(Value::Number(7.0))));
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer,0,3),target,caught=false;function C(n){buffer.resize(1);target=Uint8Array.of(7,8);return target;}a.constructor={[Symbol.species]:C};try{a.slice(0,2);}catch(e){caught=e instanceof TypeError;}caught && target.join()==='7,8'",
    );
}

#[test]
fn slice_same_kind_aliases_copy_forward_live_bytes_instead_of_a_snapshot() {
    check(
        "let a=Uint8Array.of(1,2,3,4,5);function C(){return new Uint8Array(a.buffer,1,4);}a.constructor={[Symbol.species]:C};let b=a.slice(0,4);b.join()==='1,1,1,1' && a.join()==='1,1,1,1,1'",
    );
    for name in ["Float16Array", "Float32Array", "Float64Array"] {
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(6*size),d=new DataView(buffer),a=new A(buffer,size,4),first=[],valid=true;for(let i=0;i<6*size;i++)d.setUint8(i,90);for(let i=0;i<4*size;i++)d.setUint8(size+i,i%size===(little?0:size-1)?17*(1+Math.floor(i/size)):255);for(let i=0;i<size;i++)first.push(d.getUint8(size+i));function C(){{return new A(buffer,2*size,3);}}a.constructor={{[Symbol.species]:C}};let b=a.slice(0,3);for(let i=0;i<4*size;i++)valid=valid && d.getUint8(size+i)===first[i%size];for(let i=0;i<size;i++)valid=valid && d.getUint8(i)===90 && d.getUint8(5*size+i)===90;valid && Number.isNaN(b[0]) && Number.isNaN(b[2])"
        ));
    }
}

#[test]
fn same_kind_slice_preserves_nan_bits_source_offset_and_independence() {
    for name in ["Float16Array", "Float32Array", "Float64Array"] {
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(4*size),d=new DataView(buffer),a=new A(buffer,size,3),valid=true;for(let i=0;i<3*size;i++)d.setUint8(size+i,i%size===(little?0:size-1)?17*(1+Math.floor(i/size)):255);let b=a.slice(1,3),copy=new DataView(b.buffer);for(let i=0;i<2*size;i++)valid=valid && copy.getUint8(i)===d.getUint8(2*size+i);d.setUint8(2*size+(little?0:size-1),99);valid && b.buffer!==buffer && b.byteOffset===0 && b.length===2 && copy.getUint8(little?0:size-1)===34 && Number.isNaN(b[0]) && Number.isNaN(b[1])"
        ));
    }
}

#[test]
fn different_kind_slice_uses_live_numeric_reads_when_species_aliases_the_source() {
    check(
        "let little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(8),a=new Uint8Array(buffer,0,4);a[0]=1;a[1]=2;a[2]=3;a[3]=4;function C(){return new Uint16Array(buffer);}a.constructor={[Symbol.species]:C};let b=a.slice(0,4);b.join()===(little?'1,0,0,0':'1,1,0,1')",
    );
    check(
        "let a=Float64Array.of(-1,0.5,1.5,255.5,NaN);function C(n){return new Uint8ClampedArray(n+1);}a.constructor={[Symbol.species]:C};a.slice().join()==='0,0,2,255,0,0'",
    );
}

#[test]
fn subarray_passes_exact_buffer_offset_and_two_or_three_arguments_without_a_minimum_length() {
    for (source, arguments, expected_offset, argc, expected_length) in [
        ("new Uint16Array(buffer,4,6)", "-3", 10, 3, 3),
        ("new Uint16Array(buffer,4,6)", "-3,undefined", 10, 3, 3),
        ("new Uint16Array(buffer,4)", "-3", 18, 2, 0),
        ("new Uint16Array(buffer,4)", "-3,undefined", 18, 2, 0),
        ("new Uint16Array(buffer,4)", "2,8", 8, 3, 6),
    ] {
        check(&format!(
            "let buffer=new ArrayBuffer(24,{{maxByteLength:40}}),a={source},valid=false,result=new Float32Array(0);function C(b,offset,length){{valid=arguments.length==={argc} && b===buffer && offset==={expected_offset} && new.target===C && {};return result;}}a.constructor={{[Symbol.species]:C}};a.subarray({arguments})===result && valid && result.length===0",
            if argc == 2 {
                "length===undefined".to_owned()
            } else {
                format!("length==={expected_length}")
            }
        ));
    }
}

#[test]
fn subarray_handles_initial_detachment_or_oob_as_zero_length_with_custom_species() {
    for source in ["new Uint16Array(buffer,4,2)", "new Uint16Array(buffer,4)"] {
        for invalidation in ["buffer.transfer()", "buffer.resize(2)"] {
            check(&format!(
                "let buffer=new ArrayBuffer(12,{{maxByteLength:20}}),a={source},log='',valid=false,result=new Uint8Array(0);a.constructor={{[Symbol.species]:function(b,offset,length){{log+='C';valid=arguments.length===3 && b===buffer && offset===4 && length===0;return result;}}}};{invalidation};let b=a.subarray({{valueOf(){{log+='b';return -1;}}}},{{valueOf(){{log+='e';return Infinity;}}}});b===result && valid && log==='beC'"
            ));
        }
    }
    check(
        "let buffer=new ArrayBuffer(12,{maxByteLength:20}),a=new Uint16Array(buffer,4),valid=false;a.constructor={[Symbol.species]:function(b,offset){valid=arguments.length===2 && b===buffer && offset===4;return new Uint8Array(0);}};buffer.transfer();a.subarray(-1).length===0 && valid",
    );
    check(
        "let a=Uint8Array.of(1,2,3),buffer=a.buffer,log='',valid=false,result=new Uint16Array(0);function C(b,offset,length){log+='C';valid=arguments.length===3 && b===buffer && offset===0 && length===3;return result;}a.constructor={[Symbol.species]:C};let b=a.subarray(0,{valueOf(){log+='e';buffer.transfer();return 3;}});b===result && valid && log==='eC'",
    );
    assert_eq!(
        Realm::default()
            .eval("let a=new Uint8Array(1);a.buffer.transfer();a.subarray({valueOf(){throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    exception(
        "let a=new Uint8Array(1);a.buffer.transfer();a.subarray()",
        ExceptionKind::TypeError,
    );
}

#[test]
fn subarray_uses_captured_zero_bounds_but_default_construction_observes_recovery() {
    check(
        "let buffer=new ArrayBuffer(8,{maxByteLength:16}),a=new Uint16Array(buffer,4);buffer.resize(2);let b=a.subarray({valueOf(){buffer.resize(12);return -1;}});b.buffer===buffer && b.byteOffset===4 && b.length===4",
    );
    check(
        "let buffer=new ArrayBuffer(8,{maxByteLength:16}),a=new Uint16Array(buffer,4,2);buffer.resize(2);let b=a.subarray({valueOf(){buffer.resize(12);return -1;}});b.buffer===buffer && b.byteOffset===4 && b.length===0",
    );
}

#[test]
fn subarray_omitted_and_undefined_end_track_growth_while_explicit_end_is_fixed() {
    check(
        "let buffer=new ArrayBuffer(12,{maxByteLength:24}),a=new Uint16Array(buffer,2),v=a.subarray(1),u=a.subarray(1,undefined),fixed=a.subarray(1,4);a[1]=7;a[2]=8;a[3]=9;let initial=v.length===4 && u.length===4 && fixed.length===3;buffer.resize(16);let grown=v.length===6 && u.length===6 && fixed.length===3;buffer.resize(6);let shrunk=v.length===1 && u.length===1 && fixed.length===0 && fixed.byteOffset===0 && fixed.buffer===buffer && v.byteOffset===4 && v[0]===7;buffer.resize(12);initial && grown && shrunk && v.join()==='7,0,0,0' && u.join()==='7,0,0,0' && fixed.join()==='7,0,0'",
    );
}

#[test]
fn intrinsic_copies_ignore_public_constructors_species_and_subclass_prototypes() {
    check(
        "let A=Uint16Array,prototype=A.prototype;class S extends A{}let a=new S([1,2,3]);Object.defineProperty(a,'constructor',{get(){throw 7;}});Object.defineProperty(S,Symbol.species,{get(){throw 8;}});Uint16Array=function(){throw 9;};let r=a.toReversed(),w=a.with(1,7);Object.getPrototypeOf(r)===prototype && Object.getPrototypeOf(w)===prototype && !(r instanceof S) && !(w instanceof S) && r.join()==='3,2,1' && w.join()==='1,7,3' && a.join()==='1,2,3'",
    );
}

#[test]
fn intrinsic_copies_use_numeric_nan_writes_and_preserve_source_payloads() {
    for (name, setter) in [
        ("Float16Array", "setFloat16"),
        ("Float32Array", "setFloat32"),
        ("Float64Array", "setFloat64"),
    ] {
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(3*size),d=new DataView(buffer),a=new A(buffer),expected=new DataView(new ArrayBuffer(size)),valid=true;for(let i=0;i<3*size;i++)d.setUint8(i,255);expected.{setter}(0,NaN,little);let r=new DataView(a.toReversed().buffer),w=new DataView(a.with(1,NaN).buffer);for(let i=0;i<3*size;i++)valid=valid && r.getUint8(i)===expected.getUint8(i%size) && w.getUint8(i)===expected.getUint8(i%size) && d.getUint8(i)===255;valid"
        ));
    }
    check(
        "let a=Float64Array.of(-0,2,0),r=a.toReversed(),w=a.with(1,-0);Object.is(r[0],0) && Object.is(r[2],-0) && Object.is(w[0],-0) && Object.is(w[1],-0) && Object.is(w[2],0) && a[1]===2",
    );
}

#[test]
fn with_converts_index_then_replacement_before_any_live_range_rejection() {
    for (index, expected) in [
        ("undefined", "7,2,3"),
        ("NaN", "7,2,3"),
        ("-0", "7,2,3"),
        ("-0.5", "7,2,3"),
        ("1.9", "1,7,3"),
        ("-1.9", "1,2,7"),
    ] {
        check(&format!(
            "let a=Uint8Array.of(1,2,3),b=a.with({index},7);b.join()==='{expected}' && a.join()==='1,2,3'"
        ));
    }
    for (length, index) in [
        (0, "0"),
        (3, "3"),
        (3, "-4"),
        (3, "Infinity"),
        (3, "-Infinity"),
    ] {
        check(&format!(
            "let a=new Uint8Array({length}),log='',caught=false;try{{a.with({{valueOf(){{log+='i';return {index};}}}},{{valueOf(){{log+='v';return 7;}}}});}}catch(e){{caught=e instanceof RangeError;}}caught && log==='iv'"
        ));
    }
    exception(
        "new Uint8Array(0).with(Infinity,Symbol())",
        ExceptionKind::TypeError,
    );
    exception(
        "new BigInt64Array(0).with(Infinity,1)",
        ExceptionKind::TypeError,
    );
    assert_eq!(
        Realm::default().eval("Uint8Array.of(1).with({valueOf(){throw 7;}},{valueOf(){throw 8;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check("let a=Uint8Array.of(7),b=a.with();b[0]===0 && a[0]===7");
    exception("BigInt64Array.of(7n).with()", ExceptionKind::TypeError);
}

#[test]
fn with_accepts_live_auto_growth_beyond_original_length_but_copies_only_the_original_range() {
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer),hits=0;a.set([1,2,3]);let b=a.with(5,{valueOf(){hits++;buffer.resize(6);a[5]=9;return 7;}});hits===1 && b.length===3 && b.join()==='1,2,3' && a.length===6 && a[5]===9 && b.buffer!==buffer",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer);a.set([1,2,3]);let b=a.with({valueOf(){buffer.resize(6);return -1;}},7);b.join()==='1,2,7' && a.length===6 && a[2]===3",
    );
    check(
        "let buffer=new ArrayBuffer(0,{maxByteLength:3}),a=new Uint8Array(buffer),hits=0;let b=a.with(0,{valueOf(){hits++;buffer.resize(1);return 7;}});hits===1 && b instanceof Uint8Array && b.length===0 && b.buffer!==buffer && a.length===1 && a[0]===0",
    );
}

#[test]
fn with_uses_final_live_index_validity_and_retains_conversion_side_effects() {
    for invalidation in ["buffer.transfer()", "buffer.resize(1)"] {
        check(&format!(
            "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array(buffer,0,3),log='',caught=false;try{{a.with({{valueOf(){{log+='i';return 0;}}}},{{valueOf(){{log+='v';{invalidation};return 7;}}}});}}catch(e){{caught=e instanceof RangeError;}}caught && log==='iv' && a.length===0"
        ));
    }
    assert_eq!(Realm::default().eval("let a=Uint8Array.of(1,2,3);a.with({valueOf(){a.buffer.transfer();return 0;}},{valueOf(){throw 7;}})"),Err(Error::Thrown(Value::Number(7.0))));
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer,0,3);a.set([1,2,3]);let b=a.with({valueOf(){buffer.resize(0);return 1;}},{valueOf(){buffer.resize(3);return 7;}});b.join()==='0,7,0' && a.join()==='0,0,0'",
    );
}

#[test]
fn with_copies_current_invalidated_tail_values_after_a_still_valid_replacement_index() {
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer);a.set([1,2,3]);let b=a.with(0,{valueOf(){buffer.resize(1);return 7;}});b.join()==='7,0,0' && a.join()==='1'",
    );
    check(
        "let buffer=new ArrayBuffer(24,{maxByteLength:48}),a=new Float64Array(buffer);a.set([1,2,3]);let b=a.with(0,{valueOf(){buffer.resize(8);return 7;}});b.length===3 && b[0]===7 && Number.isNaN(b[1]) && Number.isNaN(b[2]) && a.length===1 && a[0]===1",
    );
    check(
        "let buffer=new ArrayBuffer(24,{maxByteLength:48}),a=new BigInt64Array(buffer),caught=false;a.set([1n,2n,3n]);try{a.with(0,{valueOf(){buffer.resize(8);return 7n;}});}catch(e){caught=e instanceof TypeError;}caught && a.length===1 && a[0]===1n",
    );
}

#[test]
fn copy_and_view_offsets_remain_exact_at_the_safe_integer_boundary() {
    check(
        "let maximum=9007199254740991,buffer=new ArrayBuffer(maximum),a=new Uint8Array(buffer,maximum-3,3);a[0]=7;a[1]=8;a[2]=9;let s=a.slice(1),v=a.subarray(1),r=a.toReversed(),w=a.with(1,1);s.join()==='8,9' && v.join()==='8,9' && v.byteOffset===maximum-2 && v.buffer===buffer && r.join()==='9,8,7' && w.join()==='7,1,9' && a.join()==='7,8,9'",
    );
}

#[test]
fn subarray_keeps_its_buffer_alive_and_intrinsic_copies_survive_source_collection() {
    let mut realm = Realm::default();
    realm.eval("let retained=(()=>{let a=Uint16Array.of(1,2,3);return {view:a.subarray(1),copy:a.toReversed()};})()").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "retained.view[0]=7;retained.view.join()==='7,3' && retained.copy.join()==='3,2,1'"
        ),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn explicit_bigint_limits_distinguish_raw_slice_from_numeric_copies_and_skipped_reads() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(8),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,bytes=new Uint8Array(16);for(let i=0;i<8;i++)bytes[i]=255;let data=new DataView(bytes.buffer);data.setUint32(little?8:12,7,little);let a=new BigUint64Array(bytes.buffer),raw=a.slice(),r=new Uint8Array(raw.buffer),w=a.with(0,1n);r[0]===255 && r[7]===255 && w[0]===1n && w[1]===7n"),Ok(Value::Boolean(true)));
    realm.eval("let flag=0").unwrap();
    for expression in ["a.toReversed()", "a.with(1,1n)", "a.with(Infinity,'256')"] {
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Limit { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("bytes[0]===255 && bytes[7]===255 && a.length===2"),
            Ok(Value::Boolean(true))
        );
    }
    assert!(matches!(realm.eval("a.constructor={[Symbol.species]:BigInt64Array};try{a.slice();}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn subarray_preserves_actual_species_argument_counts_under_an_explicit_call_quota() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(2),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let buffer=new ArrayBuffer(3,{maxByteLength:6}),auto=new Uint8Array(buffer),fixed=new Uint8Array([1,2,3]),flag=0;auto.subarray(0).length===3 && auto.slice().length===3 && fixed.toReversed().join()==='3,2,1' && fixed.with(1,7).join()==='1,7,3'"),Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval("try{fixed.subarray(0);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("fixed.join()==='1,2,3'"),
        Ok(Value::Boolean(true))
    );
}
