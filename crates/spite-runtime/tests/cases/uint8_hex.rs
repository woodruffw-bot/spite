//! Uint8Array hexadecimal decoding, prefix writes, and encoding (23.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

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
        matches!(&result,Err(Error::Exception {kind,..}) if *kind==expected),
        "{source}: {result:?}"
    );
}

#[test]
fn hex_round_trips_every_byte_with_case_insensitive_input_and_lowercase_output() {
    let hex = (0u16..=255)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let upper = hex.to_ascii_uppercase();
    check(&format!(
        "let lower='{hex}',upper='{upper}',a=Uint8Array.fromHex(lower),b=Uint8Array.fromHex(upper),valid=a.length===256 && b.length===256;for(let i=0;i<256;i++)valid=valid && a[i]===i && b[i]===i;valid && a.toHex()===lower && b.toHex()===lower"
    ));
    check(
        "let a=Uint8Array.fromHex('aB01CdEf');a.join()==='171,1,205,239' && a.toHex()==='ab01cdef'",
    );
    check("Uint8Array.of(0,1,9,10,15,16,127,128,255).toHex()==='0001090a0f107f80ff'");
}

#[test]
fn setter_decode_matrix_distinguishes_odd_inputs_partial_errors_and_unexamined_suffixes() {
    let mut rows = Vec::new();
    for (label, input, capacity) in [
        ("empty", "''", 0),
        ("empty", "''", 2),
        ("mixed case", "'00aF'", 3),
        ("invalid suffix", "'00xz'", 1),
        ("invalid suffix", "'00xz'", 2),
        ("invalid first pair", "'xz00'", 2),
        ("odd after valid pair", "'00f'", 0),
        ("odd after valid pair", "'00f'", 1),
        ("odd after valid pair", "'00f'", 2),
        ("even invalid", "'zz'", 0),
        ("odd invalid", "'z'", 0),
        ("valid suffix", "'00010203'", 2),
        ("valid input", "'000102'", 0),
        ("prefix notation", "'0x'", 1),
        ("space in pair", "'00 1'", 2),
        ("surrogate suffix", r#"'00\uD800\uDC00'"#, 1),
        ("surrogate suffix", r#"'00\uD800\uDC00'"#, 2),
        ("odd lone surrogate", r#"'00\uD800'"#, 2),
    ] {
        let result=Realm::default().eval(&format!(
            "let a=new Uint8Array({capacity}),status='ok',read='-',written='-';a.fill(170);try{{let r=a.setFromHex({input});read=r.read;written=r.written;}}catch(e){{if(!(e instanceof SyntaxError))throw e;status='syntax';}}status+' '+read+'/'+written+' ['+a.join()+']'"
        )).unwrap();
        let Value::String(output) = result else {
            panic!("{label}: {result:?}")
        };
        rows.push(format!(
            "{label} cap{capacity}: {}",
            output.to_utf8().unwrap()
        ));
    }
    insta::assert_snapshot!(rows.join("\n"), @r###"
    empty cap0: ok 0/0 []
    empty cap2: ok 0/0 [170,170]
    mixed case cap3: ok 4/2 [0,175,170]
    invalid suffix cap1: ok 2/1 [0]
    invalid suffix cap2: syntax -/- [0,170]
    invalid first pair cap2: syntax -/- [170,170]
    odd after valid pair cap0: syntax -/- []
    odd after valid pair cap1: syntax -/- [170]
    odd after valid pair cap2: syntax -/- [170,170]
    even invalid cap0: ok 0/0 []
    odd invalid cap0: syntax -/- []
    valid suffix cap2: ok 4/2 [0,1]
    valid input cap0: ok 0/0 []
    prefix notation cap1: syntax -/- [170]
    space in pair cap2: syntax -/- [0,170]
    surrogate suffix cap1: ok 2/1 [0]
    surrogate suffix cap2: syntax -/- [0,170]
    odd lone surrogate cap2: syntax -/- [170,170]
    "###);
}

#[test]
fn static_decoder_ignores_this_subclasses_species_and_replaced_global_bindings() {
    check(
        "let A=Uint8Array,from=A.fromHex,prototype=A.prototype;class S extends A{constructor(){throw 7;}}Object.defineProperty(S,Symbol.species,{get(){throw 8;}});let inherited=S.fromHex('fF');let receiver={get prototype(){throw 9;},get [Symbol.species](){throw 10;}};Uint8Array=function(){throw 11;};let a=from.call(receiver,'0010'),b=from.call(null,''),c=from.call(Symbol(),'aB');Object.getPrototypeOf(inherited)===prototype && !(inherited instanceof S) && inherited[0]===255 && Object.getPrototypeOf(a)===prototype && a.join()==='0,16' && a.byteOffset===0 && a.byteLength===2 && a.buffer.byteLength===2 && a.buffer.maxByteLength===2 && !a.buffer.resizable && b.length===0 && b.buffer.byteLength===0 && b.buffer!==a.buffer && c[0]===171 && Object.getPrototypeOf(c)===prototype",
    );
    check(
        "let a=Uint8Array.fromHex(''),b=Uint8Array.fromHex('');a!==b && a.buffer!==b.buffer && !a.buffer.resizable && !b.buffer.resizable",
    );
}

#[test]
fn decoders_require_primitive_strings_without_invoking_any_coercion_hook() {
    for input in [
        "undefined",
        "null",
        "true",
        "7",
        "1n",
        "Symbol()",
        "Object('00')",
        "new String('00')",
        "[]",
        "{}",
    ] {
        exception(
            &format!("Uint8Array.fromHex({input})"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("new Uint8Array(2).setFromHex({input})"),
            ExceptionKind::TypeError,
        );
    }
    for call in ["Uint8Array.fromHex(input)", "a.setFromHex(input)"] {
        check(&format!(
            "let a=new Uint8Array(2),hits=0,caught=false,input={{get [Symbol.toPrimitive](){{hits++;throw 7;}},valueOf(){{hits++;throw 8;}},toString(){{hits++;throw 9;}}}};try{{{call};}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0 && a[0]===0 && a[1]===0"
        ));
    }
}

#[test]
fn instance_methods_require_the_exact_uint8_kind_and_accept_its_subclasses() {
    for name in [
        "Int8Array",
        "Uint8ClampedArray",
        "Int16Array",
        "Uint16Array",
        "Int32Array",
        "Uint32Array",
        "BigInt64Array",
        "BigUint64Array",
        "Float16Array",
        "Float32Array",
        "Float64Array",
    ] {
        for method in ["setFromHex", "toHex"] {
            exception(
                &format!("Uint8Array.prototype.{method}.call(new {name}(0),'')"),
                ExceptionKind::TypeError,
            );
        }
    }
    for receiver in [
        "null",
        "undefined",
        "{}",
        "[]",
        "new ArrayBuffer(2)",
        "new DataView(new ArrayBuffer(2))",
        "Uint8Array.prototype",
        "Object.create(new Uint8Array(2))",
    ] {
        exception(
            &format!("Uint8Array.prototype.setFromHex.call({receiver},'')"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("Uint8Array.prototype.toHex.call({receiver})"),
            ExceptionKind::TypeError,
        );
    }
    check(
        "class S extends Uint8Array{}let a=new S(2),r=a.setFromHex('00Ff');a instanceof S && a[0]===0 && a[1]===255 && a.toHex()==='00ff' && r.read===4 && r.written===2",
    );
    check(
        "let a=new Uint8Array(1),set=Uint8Array.prototype.setFromHex,to=Uint8Array.prototype.toHex;Object.setPrototypeOf(a,Uint8ClampedArray.prototype);let r=set.call(a,'aB');r.written===1 && to.call(a)==='ab'",
    );
    exception(
        "let a=new Uint8ClampedArray(1);Object.defineProperty(a,Symbol.toStringTag,{value:'Uint8Array'});Uint8Array.prototype.toHex.call(a)",
        ExceptionKind::TypeError,
    );
}

#[test]
fn hex_methods_ignore_public_state_iteration_and_numeric_string_conversion_overrides() {
    check(
        "let a=Uint8Array.of(1,2,3);for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT','constructor',Symbol.iterator,Symbol.toStringTag])Object.defineProperty(a,key,{get(){throw 7;}});Number.prototype.toString=function(){throw 8;};String.prototype.toLowerCase=function(){throw 9;};Object.defineProperty(String.prototype,'charCodeAt',{get(){throw 10;}});let r=a.setFromHex('aB00');r.read===4 && r.written===2 && a[0]===171 && a[1]===0 && a[2]===3 && a.toHex()==='ab0003' && Uint8Array.fromHex('fF')[0]===255",
    );
}

#[test]
fn unused_options_are_not_read_but_normal_argument_evaluation_still_precedes_the_call() {
    check(
        "let ignored={get alphabet(){throw 7;},get lastChunkHandling(){throw 8;},get omitPadding(){throw 9;},valueOf(){throw 10;},toString(){throw 11;}};let a=Uint8Array.fromHex('00',ignored),r=a.setFromHex('ff',ignored);r.read===2 && r.written===1 && a.toHex(ignored)==='ff'",
    );
    check(
        "let a=Uint8Array.of(1),log='',caught=false;try{a.toHex((log+='argument',a.buffer.transfer()));}catch(e){caught=e instanceof TypeError;}caught && log==='argument'",
    );
}

#[test]
fn static_hex_rejects_non_ascii_code_units_prefixes_separators_and_whitespace() {
    for input in [
        "'0'",
        "'000'",
        "'0x00'",
        "'0Xff'",
        "'+0'",
        "'-0'",
        "'1_00'",
        "' 0'",
        "'00 '",
        "'00 1'",
        r#"'0\n'"#,
        r#"'0\t'"#,
        r#"'0\r'"#,
        r#"'0\v'"#,
        r#"'\u0000\u0000'"#,
        r#"'\u00A0\u00A0'"#,
        r#"'\u2028\u2029'"#,
        r#"'\uFF10\uFF10'"#,
        r#"'\u0660\u0660'"#,
        r#"'\u00E9\u00E9'"#,
        r#"'\uD800'"#,
        r#"'\uDC00'"#,
        r#"'\uD800\uDC00'"#,
    ] {
        exception(
            &format!("Uint8Array.fromHex({input})"),
            ExceptionKind::SyntaxError,
        );
    }
}

#[test]
fn setters_write_only_complete_valid_pairs_before_a_language_syntax_error() {
    check(
        "let a=Uint8Array.of(170,171,172,173),caught=false;try{a.setFromHex('0102x3');}catch(e){caught=e instanceof SyntaxError;}caught && a[0]===1 && a[1]===2 && a[2]===172 && a[3]===173",
    );
    check(
        "let a=Uint8Array.of(170,171),caught=false;try{a.setFromHex('012');}catch(e){caught=e instanceof SyntaxError;}caught && a[0]===170 && a[1]===171",
    );
    check(
        "let a=Uint8Array.of(170,171),caught=false,finallyRan=false;try{a.setFromHex('01zz');}catch(e){caught=e instanceof SyntaxError;}finally{finallyRan=true;}caught && finallyRan && a[0]===1 && a[1]===171",
    );
}

#[test]
fn result_objects_have_only_ordered_writable_enumerable_configurable_data_properties() {
    check(
        "let a=new Uint8Array(3),r=a.setFromHex('00ff'),read=Object.getOwnPropertyDescriptor(r,'read'),written=Object.getOwnPropertyDescriptor(r,'written');Object.getPrototypeOf(r)===Object.prototype && Reflect.ownKeys(r).join()==='read,written' && read.value===4 && read.writable && read.enumerable && read.configurable && !('get' in read) && !('set' in read) && written.value===2 && written.writable && written.enumerable && written.configurable && !('get' in written) && !('set' in written)",
    );
    check(
        "let hits=0;Object.defineProperty(Object.prototype,'read',{set(){hits++;throw 7;},configurable:true});Object.defineProperty(Object.prototype,'written',{set(){hits++;throw 8;},configurable:true});let a=new Uint8Array(1),r=a.setFromHex('ff'),s=a.setFromHex('');hits===0 && r!==s && r.read===2 && r.written===1 && s.read===0 && s.written===0 && Reflect.ownKeys(r).join()==='read,written' && Object.getOwnPropertyDescriptor(r,'read').value===2 && a[0]===255",
    );
}

#[test]
fn nonzero_offsets_and_partial_errors_preserve_outside_bytes_and_unwritten_tail() {
    check(
        "let bytes=Uint8Array.of(90,91,170,171,172,173,92),a=new Uint8Array(bytes.buffer,2,4),caught=false;try{a.setFromHex('0102xz');}catch(e){caught=e instanceof SyntaxError;}caught && bytes.join()==='90,91,1,2,172,173,92' && a.toHex()==='0102acad'",
    );
    check(
        "let bytes=Uint8Array.of(90,91,170,171,172,173,92),a=new Uint8Array(bytes.buffer,2,4),r=a.setFromHex('00010203zz');r.read===8 && r.written===4 && bytes.join()==='90,91,0,1,2,3,92' && a.toHex()==='00010203'",
    );
}

#[test]
fn detached_and_oob_receivers_fail_even_for_empty_inputs_and_regrow_recovers_fixed_views() {
    for method in ["setFromHex", "toHex"] {
        exception(
            &format!("let a=new Uint8Array(0);a.buffer.transfer();a.{method}('')"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!(
                "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(buffer,1,3);buffer.resize(2);a.{method}('')"
            ),
            ExceptionKind::TypeError,
        );
    }
    check(
        "let buffer=new ArrayBuffer(8,{maxByteLength:12}),bytes=new Uint8Array(buffer),fixed=new Uint8Array(buffer,2,3),auto=new Uint8Array(buffer,2);bytes.set([144,145,18,52,86,120,154,188]);let initial=fixed.toHex()==='123456' && auto.toHex()==='123456789abc';buffer.resize(4);let short=auto.toHex()==='1234',r=auto.setFromHex('ffffabcd');buffer.resize(8);let recovered=fixed.toHex()==='ffff00' && auto.toHex()==='ffff00000000';let s=fixed.setFromHex('0a0b');initial && short && r.read===4 && r.written===2 && recovered && s.read===4 && s.written===2 && fixed.toHex()==='0a0b00' && auto.toHex()==='0a0b00000000' && bytes[0]===144 && bytes[1]===145",
    );
    check(
        "let buffer=new ArrayBuffer(0,{maxByteLength:3}),a=new Uint8Array(buffer),r=a.setFromHex('zz');r.read===0 && r.written===0 && a.toHex()===''",
    );
}

#[test]
fn captured_current_capacity_tracks_rab_growth_and_argument_side_effects() {
    check(
        "let buffer=new ArrayBuffer(2,{maxByteLength:6}),a=new Uint8Array(buffer),r=a.setFromHex((buffer.resize(4),'00010203ff'));r.read===8 && r.written===4 && a.toHex()==='00010203'",
    );
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4);buffer.resize(1);let r=a.setFromHex((buffer.resize(4),'01020304'));r.read===8 && r.written===4 && a.toHex()==='01020304'",
    );
}

#[test]
fn writes_cross_sparse_page_boundaries_and_safe_integer_offsets_without_touching_guards() {
    check(
        "let buffer=new ArrayBuffer(4102),a=new Uint8Array(buffer,4094,6),bytes=new Uint8Array(buffer);bytes[4093]=90;bytes[4100]=91;let r=a.setFromHex('000102030405');r.read===12 && r.written===6 && a.toHex()==='000102030405' && bytes[4093]===90 && bytes[4100]===91",
    );
    check(
        "let maximum=9007199254740991,buffer=new ArrayBuffer(maximum),a=new Uint8Array(buffer,maximum-3,3),guard=new DataView(buffer,maximum-4,1),r=a.setFromHex('0001ff');r.read===6 && r.written===3 && a.toHex()==='0001ff' && a.byteOffset===maximum-3 && buffer.byteLength===maximum && guard.getUint8(0)===0",
    );
}

#[test]
fn decoded_intrinsic_results_survive_collection_without_retaining_input_or_receiver() {
    let mut realm = Realm::default();
    realm.eval("let result=(()=>{let from=Uint8Array.fromHex,input='0010ff';return from.call({},input);})()").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("result.toHex()==='0010ff' && result.length===3 && !result.buffer.resizable"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn decoded_byte_lists_are_not_call_argument_lists() {
    let hex = (0u16..=255)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut realm = Realm::new(Limits {
        max_arguments: Some(1),
        ..Limits::default()
    });
    assert_eq!(realm.eval(&format!("let text='{hex}',a=Uint8Array.fromHex(text),b=new Uint8Array(256),r=b.setFromHex(text);r.read===512 && r.written===256 && a.toHex()===text && b.toHex()===text")),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_output_string_quota_preserves_source_and_aborts_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(8),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let a=new Uint8Array(5),flag=0;a[0]=1;a[4]=255;Uint8Array.fromHex('00010203').toHex()==='00010203'"),Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval("try{a.toHex();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && a[0]===1 && a[4]===255 && a.length===5"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("let huge=new Uint8Array(9007199254740991)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{huge.toHex();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn opted_in_work_abort_is_a_host_failure_without_source_mutation() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    realm.eval("let a=new Uint8Array(100000),flag=0").unwrap();
    realm.eval("a[0]=7").unwrap();
    realm.eval("a[99999]=9").unwrap();
    assert!(matches!(
        realm.eval("try{a.toHex();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && a.length===100000 && a[0]===7 && a[99999]===9"),
        Ok(Value::Boolean(true))
    );
}
