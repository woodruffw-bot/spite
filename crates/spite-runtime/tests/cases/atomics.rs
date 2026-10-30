//! Atomics, ordered coercion, revalidation and empty waiter lists (25.4).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const INTEGER_KINDS: [(&str, bool); 8] = [
    ("Int8Array", false),
    ("Uint8Array", false),
    ("Int16Array", false),
    ("Uint16Array", false),
    ("Int32Array", false),
    ("Uint32Array", false),
    ("BigInt64Array", true),
    ("BigUint64Array", true),
];

const MEMORY_METHODS: [&str; 9] = [
    "add",
    "and",
    "compareExchange",
    "exchange",
    "load",
    "or",
    "store",
    "sub",
    "xor",
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
fn pause_ignores_receivers_and_extra_values_after_argument_evaluation() {
    check(
        "let pause=Atomics.pause,log='',value=new Proxy({},{get(){throw 1}});pause.call(value,value)===undefined&&pause.call(null,Symbol())===undefined&&pause.call(undefined,NaN)===undefined&&pause((log+='a',value),(log+='b',value))===undefined&&log==='ab'",
    );
}

#[test]
fn all_integer_kinds_support_load_store_and_read_modify_write() {
    for (name, bigint) in INTEGER_KINDS {
        let n = if bigint { "n" } else { "" };
        check(&format!(
            "let a={name}.of(10{n});Atomics.load(a,0)===10{n} && Atomics.add(a,0,3{n})===10{n} && a[0]===13{n} && Atomics.sub(a,0,2{n})===13{n} && a[0]===11{n} && Atomics.and(a,0,6{n})===11{n} && a[0]===2{n} && Atomics.or(a,0,8{n})===2{n} && a[0]===10{n} && Atomics.xor(a,0,3{n})===10{n} && a[0]===9{n} && Atomics.exchange(a,0,7{n})===9{n} && a[0]===7{n} && Atomics.store(a,0,4{n})===4{n} && a[0]===4{n}"
        ));
        check(&format!(
            "let a={name}.of(5{n});Atomics.compareExchange(a,0,4{n},9{n})===5{n} && a[0]===5{n} && Atomics.compareExchange(a,0,5{n},9{n})===5{n} && a[0]===9{n}"
        ));
    }
}

#[test]
fn shared_integer_kinds_use_the_same_bytes_for_all_atomic_methods_and_views() {
    for (name, bigint) in INTEGER_KINDS {
        let n = if bigint { "n" } else { "" };
        check(&format!(
            "let b=new SharedArrayBuffer(32),a=new {name}(b,8,2);a[1]=23{n};a[0]=10{n};Atomics.load(a,0)===10{n}&&Atomics.add(a,0,3{n})===10{n}&&a[0]===13{n}&&Atomics.sub(a,0,2{n})===13{n}&&Atomics.and(a,0,6{n})===11{n}&&Atomics.or(a,0,8{n})===2{n}&&Atomics.xor(a,0,3{n})===10{n}&&Atomics.exchange(a,0,7{n})===9{n}&&Atomics.compareExchange(a,0,8{n},12{n})===7{n}&&a[0]===7{n}&&Atomics.compareExchange(a,0,7{n},12{n})===7{n}&&Atomics.store(a,0,4{n})===4{n}&&new {name}(b,8,2)[0]===4{n}&&a[1]===23{n}"
        ));
    }
}

#[test]
fn shared_atomic_arithmetic_and_comparisons_truncate_at_element_width() {
    for (name, max, min, n) in [
        ("Int8Array", "127", "-128", ""),
        ("Uint8Array", "255", "0", ""),
        ("Int16Array", "32767", "-32768", ""),
        ("Uint16Array", "65535", "0", ""),
        ("Int32Array", "2147483647", "-2147483648", ""),
        ("Uint32Array", "4294967295", "0", ""),
        (
            "BigInt64Array",
            "9223372036854775807",
            "-9223372036854775808",
            "n",
        ),
        ("BigUint64Array", "18446744073709551615", "0", "n"),
    ] {
        check(&format!(
            "let a=new {name}(new SharedArrayBuffer(16));a[0]={max}{n};Atomics.add(a,0,1{n})==={max}{n}&&a[0]==={min}{n}&&Atomics.sub(a,0,1{n})==={min}{n}&&a[0]==={max}{n}"
        ));
    }
    check(
        "let a=new Int8Array(new SharedArrayBuffer(8));a[0]=-1;Atomics.compareExchange(a,0,255,257)===-1&&a[0]===1&&Atomics.store(a,0,258.9)===258&&a[0]===2&&Object.is(Atomics.store(a,0,-0),0)",
    );
    check(
        "let a=new BigUint64Array(new SharedArrayBuffer(8));Atomics.store(a,0,-1n)===-1n&&Atomics.compareExchange(a,0,-1n,18446744073709551617n)===18446744073709551615n&&a[0]===1n",
    );
}

#[test]
fn shared_atomic_coercions_finish_before_reading_and_keep_original_index_bounds() {
    for (name, bigint) in INTEGER_KINDS {
        let n = if bigint { "n" } else { "" };
        check(&format!(
            "let b=new SharedArrayBuffer(16,{{maxByteLength:32}}),a=new {name}(b),log='';let old=Atomics.compareExchange(a,{{valueOf(){{log+='i';b.grow(24);return 0}}}},{{valueOf(){{log+='v';a[0]=10{n};return 20{n}}}}},{{valueOf(){{log+='r';a[0]=20{n};return 7{n}}}}});old===20{n}&&a[0]===7{n}&&log==='ivr'&&b.byteLength===24"
        ));
        check(&format!(
            "let b=new SharedArrayBuffer(16,{{maxByteLength:32}}),a=new {name}(b),length=a.length,calls=0,caught=false;try{{Atomics.store(a,{{valueOf(){{b.grow(32);return length}}}},{{valueOf(){{calls++;return 1{n}}}}})}}catch(e){{caught=e instanceof RangeError}}caught&&calls===0&&b.byteLength===32"
        ));
    }
    check(
        "let a=new Int32Array(new SharedArrayBuffer(8)),reason={},log='',same=false;try{Atomics.compareExchange(a,0,{valueOf(){log+='v';throw reason}},{valueOf(){log+='r';return 1}})}catch(e){same=e===reason}same&&log==='v'&&a[0]===0",
    );
}

#[test]
fn shared_atomic_access_preserves_sparse_safe_integer_sized_buffers() {
    check(
        "let max=2**53-1,b=new SharedArrayBuffer(0,{maxByteLength:max});b.grow(max);let a=new Uint32Array(b,max-7,1);Atomics.load(a,0)===0&&Atomics.store(a,0,4294967295)===4294967295&&a[0]===4294967295&&Atomics.add(a,0,1)===4294967295&&a[0]===0&&b.byteLength===max",
    );
}

#[test]
fn shared_atomic_bigint_return_refusal_leaves_bytes_and_skips_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(4),
        ..Limits::default()
    });
    realm.eval("let b=new SharedArrayBuffer(8),bytes=new Uint8Array(b),a=new BigUint64Array(b),caught=0,finalized=0,disposed=0;bytes[0]=255;let r={[Symbol.dispose](){disposed++}}").unwrap();
    let result = realm.eval(
        "function f(){using x=r;try{Atomics.add(a,0,1n)}catch(e){caught++}finally{finalized++}}f()",
    );
    assert!(matches!(result, Err(Error::Limit { .. })), "{result:?}");
    assert_eq!(
        realm.eval("bytes[0]===255&&caught===0&&finalized===0&&disposed===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn read_modify_write_wraps_at_every_signed_and_unsigned_boundary() {
    for (name, max, next, one) in [
        ("Int8Array", "127", "-128", "1"),
        ("Uint8Array", "255", "0", "1"),
        ("Int16Array", "32767", "-32768", "1"),
        ("Uint16Array", "65535", "0", "1"),
        ("Int32Array", "2147483647", "-2147483648", "1"),
        ("Uint32Array", "4294967295", "0", "1"),
        (
            "BigInt64Array",
            "9223372036854775807n",
            "-9223372036854775808n",
            "1n",
        ),
        ("BigUint64Array", "18446744073709551615n", "0n", "1n"),
    ] {
        check(&format!(
            "let a={name}.of({max});Atomics.add(a,0,{one})==={max} && a[0]==={next} && Atomics.sub(a,0,{one})==={next} && a[0]==={max}"
        ));
    }
    check(
        "let a=BigUint64Array.of(18446744073709551615n);Atomics.and(a,0,9223372036854775808n)===18446744073709551615n && Atomics.xor(a,0,18446744073709551615n)===9223372036854775808n && a[0]===9223372036854775807n",
    );
}

#[test]
fn store_returns_the_converted_value_before_element_width_truncation() {
    for (input, returned, stored) in [
        ("257.9", "257", "1"),
        ("-257.9", "-257", "-1"),
        ("Infinity", "Infinity", "0"),
        ("-Infinity", "-Infinity", "0"),
        ("NaN", "0", "0"),
        ("-0", "0", "0"),
        ("-0.9", "0", "0"),
        ("undefined", "0", "0"),
    ] {
        check(&format!(
            "let a=new Int8Array(1);Object.is(Atomics.store(a,0,{input}),{returned}) && a[0]==={stored}"
        ));
    }
    check(
        "let a=new BigInt64Array(1),v=2n**100n+5n;Atomics.store(a,0,v)===v && a[0]===5n && Atomics.store(a,0,'18446744073709551615')===18446744073709551615n && a[0]===-1n",
    );
}

#[test]
fn operands_are_truncated_before_byte_comparison_and_modular_arithmetic() {
    check(
        "let a=Int8Array.of(-1);Atomics.compareExchange(a,0,255.9,258.9)===-1 && a[0]===2 && Atomics.add(a,0,4294967297.9)===2 && a[0]===3 && Atomics.sub(a,0,Infinity)===3 && a[0]===3",
    );
    check(
        "let a=BigInt64Array.of(-1n);Atomics.compareExchange(a,0,18446744073709551615n,18446744073709551618n)===-1n && a[0]===2n && Atomics.exchange(a,0,true)===2n && a[0]===1n",
    );
}

#[test]
fn internal_slots_and_offsets_ignore_public_properties_and_this() {
    for (name, bigint) in INTEGER_KINDS {
        let n = if bigint { "n" } else { "" };
        check(&format!(
            "let A={name},buffer=new ArrayBuffer(4*A.BYTES_PER_ELEMENT),all=new A(buffer),a=new A(buffer,A.BYTES_PER_ELEMENT,2);for(let key of ['length','buffer','byteOffset','byteLength','constructor'])Object.defineProperty(a,key,{{get(){{throw 7;}}}});Object.preventExtensions(a);Atomics.store.call(null,a,1,11{n})===11{n} && all[2]===11{n} && all[0]===0{n} && all[3]===0{n}"
        ));
    }
}

#[test]
fn wrong_brands_and_element_kinds_reject_before_converting_arguments() {
    for method in MEMORY_METHODS
        .into_iter()
        .chain(["notify", "wait", "waitAsync"])
    {
        for input in [
            "undefined",
            "null",
            "1",
            "{}",
            "[]",
            "new DataView(new ArrayBuffer(8))",
            "Object.create(new Int32Array(2))",
            "new Proxy(new Int32Array(2),{get(){throw 8;}})",
            "new Float16Array(2)",
            "new Float32Array(2)",
            "new Float64Array(2)",
            "new Uint8ClampedArray(2)",
        ] {
            exception(
                &format!(
                    "Atomics.{method}({input},{{valueOf(){{throw 7;}}}},{{valueOf(){{throw 9;}}}})"
                ),
                ExceptionKind::TypeError,
            );
        }
    }
}

#[test]
fn invalid_initial_views_reject_before_index_conversion() {
    for method in MEMORY_METHODS
        .into_iter()
        .chain(["notify", "wait", "waitAsync"])
    {
        for invalidate in ["b.transfer()", "b.resize(0)"] {
            exception(
                &format!(
                    "let b=new ArrayBuffer(8,{{maxByteLength:16}}),a=new Int32Array(b,0,2);{invalidate};Atomics.{method}(a,{{valueOf(){{throw 7;}}}})"
                ),
                ExceptionKind::TypeError,
            );
        }
    }
}

#[test]
fn indices_use_toindex_and_the_original_length() {
    for index in [
        "undefined",
        "NaN",
        "null",
        "-0",
        "-0.9",
        "0.9",
        "false",
        "'0'",
    ] {
        check(&format!("Atomics.load(Int8Array.of(7),{index})===7"));
    }
    for method in MEMORY_METHODS.into_iter().chain(["notify"]) {
        for index in ["-1", "Infinity", "-Infinity", "2", "9007199254740992"] {
            exception(
                &format!("Atomics.{method}(new Int32Array(2),{index},{{valueOf(){{throw 7;}}}})"),
                ExceptionKind::RangeError,
            );
        }
        for index in ["1n", "Symbol()"] {
            exception(
                &format!("Atomics.{method}(new Int32Array(2),{index})"),
                ExceptionKind::TypeError,
            );
        }
    }
    exception(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),a=new Int32Array(b);Atomics.load(a,{valueOf(){b.resize(8);return 1;}})",
        ExceptionKind::RangeError,
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),a=new Int32Array(b);Atomics.store(a,{valueOf(){b.resize(8);return 0;}},7)===7 && a.length===2 && a[0]===7",
    );
}

#[test]
fn index_and_value_hooks_run_once_in_order_before_the_memory_read() {
    for method in ["add", "and", "exchange", "or", "store", "sub", "xor"] {
        check(&format!(
            "let a=Int8Array.of(1),log='';let result=Atomics.{method}(a,{{valueOf(){{log+='i';return 0;}}}},{{valueOf(){{log+='v';a[0]=10;return 3.9;}}}});log==='iv' && result==={returned}",
            returned = if method == "store" { 3 } else { 10 }
        ));
    }
    check(
        "let a=Int8Array.of(1),log='';Atomics.compareExchange(a,{valueOf(){log+='i';return 0;}},{valueOf(){log+='e';return 10;}},{valueOf(){log+='r';a[0]=10;return 9;}})===10 && log==='ier' && a[0]===9",
    );
    check(
        "let a=Int8Array.of(1),log='';Atomics.compareExchange(a,0,2,{valueOf(){log+='r';return 9;}})===1 && log==='r' && a[0]===1",
    );
}

#[test]
fn number_and_bigint_content_types_preserve_their_distinct_conversions() {
    for method in MEMORY_METHODS.into_iter().filter(|m| *m != "load") {
        for value in ["1n", "Symbol()"] {
            exception(
                &format!("Atomics.{method}(new Int8Array(1),0,{value},{value})"),
                ExceptionKind::TypeError,
            );
        }
        for value in ["1", "null", "undefined", "Symbol()"] {
            exception(
                &format!("Atomics.{method}(new BigInt64Array(1),0,{value},1n)"),
                ExceptionKind::TypeError,
            );
        }
        exception(
            &format!("Atomics.{method}(new BigInt64Array(1),0,'1.5',1n)"),
            ExceptionKind::SyntaxError,
        );
    }
    check(
        "let a=new BigInt64Array(1),log='';Atomics.store(a,0,{[Symbol.toPrimitive](hint){log+=hint;return '123';}})===123n && a[0]===123n && log==='number'",
    );
}

#[test]
fn all_argument_coercions_precede_post_conversion_detachment_checks() {
    for method in [
        "add",
        "and",
        "compareExchange",
        "exchange",
        "or",
        "store",
        "sub",
        "xor",
    ] {
        check(&format!(
            "let a=new Int32Array(1),log='',caught=false;try{{Atomics.{method}(a,{{valueOf(){{log+='i';a.buffer.transfer();return 0;}}}},{{valueOf(){{log+='v';return 1;}}}},{{valueOf(){{log+='r';return 2;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && log==='{log}'",
            log = if method == "compareExchange" {
                "ivr"
            } else {
                "iv"
            }
        ));
        assert_eq!(Realm::default().eval(&format!("let a=new Int32Array(1);Atomics.{method}(a,{{valueOf(){{a.buffer.transfer();return 0;}}}},{{valueOf(){{throw 7;}}}})")), Err(Error::Thrown(Value::Number(7.0))));
    }
    exception(
        "let a=new Int32Array(1);Atomics.load(a,{valueOf(){a.buffer.transfer();return 0;}})",
        ExceptionKind::TypeError,
    );
    assert_eq!(Realm::default().eval("let a=new Int32Array(1);Atomics.compareExchange(a,0,{valueOf(){a.buffer.transfer();return 1;}},{valueOf(){throw 7;}})"), Err(Error::Thrown(Value::Number(7.0))));
}

#[test]
fn revalidation_distinguishes_fixed_view_bounds_from_tracking_index_bounds() {
    for method in MEMORY_METHODS {
        let suffix = if method == "load" { "" } else { ",1,2" };
        exception(
            &format!(
                "let b=new ArrayBuffer(8,{{maxByteLength:16}}),a=new Int32Array(b,0,2);Atomics.{method}(a,{{valueOf(){{b.resize(4);return 0;}}}}{suffix})"
            ),
            ExceptionKind::TypeError,
        );
        exception(
            &format!(
                "let b=new ArrayBuffer(8,{{maxByteLength:16}}),a=new Int32Array(b);Atomics.{method}(a,{{valueOf(){{b.resize(4);return 1;}}}}{suffix})"
            ),
            ExceptionKind::RangeError,
        );
    }
    check(
        "let b=new ArrayBuffer(8,{maxByteLength:16}),a=new Int32Array(b,0,2);Atomics.store(a,{valueOf(){b.resize(0);return 1;}},{valueOf(){b.resize(8);return 7;}})===7 && a[1]===7",
    );
    check(
        "let b=new ArrayBuffer(8,{maxByteLength:16}),a=new Int32Array(b);Atomics.store(a,{valueOf(){b.resize(4);return 0;}},9)===9 && a[0]===9",
    );
}

#[test]
fn ordinary_buffer_notify_converts_count_and_skips_revalidation() {
    for name in ["Int32Array", "BigInt64Array"] {
        for count in [
            "undefined",
            "NaN",
            "-1",
            "0",
            "1.9",
            "Infinity",
            "-Infinity",
        ] {
            check(&format!(
                "Object.is(Atomics.notify(new {name}(1),0,{count}),0)"
            ));
        }
        check(&format!(
            "let a=new {name}(1),log='';Atomics.notify(a,{{valueOf(){{log+='i';a.buffer.transfer();return 0;}}}},{{valueOf(){{log+='c';return 1;}}}})===0 && log==='ic'"
        ));
        for count in ["Symbol()", "1n"] {
            exception(
                &format!("Atomics.notify(new {name}(1),0,{count})"),
                ExceptionKind::TypeError,
            );
        }
    }
    for name in [
        "Int8Array",
        "Uint8Array",
        "Int16Array",
        "Uint16Array",
        "Uint32Array",
        "BigUint64Array",
    ] {
        exception(
            &format!("Atomics.notify(new {name}(1),0,0)"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn shared_notify_returns_positive_zero_for_empty_fixed_and_growable_waiter_lists() {
    for name in ["Int32Array", "BigInt64Array"] {
        for options in ["", ",{maxByteLength:32}"] {
            for count in [
                "undefined",
                "NaN",
                "-1",
                "0",
                "1.9",
                "Infinity",
                "-Infinity",
            ] {
                check(&format!(
                    "let buffer=new SharedArrayBuffer(16{options}),view=new {name}(buffer);Object.is(Atomics.notify(view,0,{count}),0)&&Object.is(Atomics.notify(view,1),0)"
                ));
            }
        }
        for count in ["Symbol()", "1n"] {
            exception(
                &format!("Atomics.notify(new {name}(new SharedArrayBuffer(16)),0,{count})"),
                ExceptionKind::TypeError,
            );
        }
    }
}

#[test]
fn shared_notify_snapshots_length_before_index_and_converts_count_after_validating_it() {
    for name in ["Int32Array", "BigInt64Array"] {
        check(&format!(
            "let buffer=new SharedArrayBuffer(8,{{maxByteLength:32}}),view=new {name}(buffer),log='';Atomics.notify(view,{{valueOf(){{log+='i';buffer.grow(16);return 0}}}},{{valueOf(){{log+='c';buffer.grow(24);return 1}}}})===0&&log==='ic'&&buffer.byteLength===24"
        ));
        check(&format!(
            "let buffer=new SharedArrayBuffer(8,{{maxByteLength:32}}),view=new {name}(buffer),length=view.length,count=0,caught=false;try{{Atomics.notify(view,{{valueOf(){{buffer.grow(16);return length}}}},{{valueOf(){{count++;return 1}}}})}}catch(e){{caught=e instanceof RangeError}}caught&&count===0&&buffer.byteLength===16"
        ));
        check(&format!(
            "let view=new {name}(new SharedArrayBuffer(16)),reason={{}},same=false,log='';try{{Atomics.notify(view,{{valueOf(){{log+='i';return 0}}}},{{valueOf(){{log+='c';throw reason}}}})}}catch(e){{same=e===reason}}same&&log==='ic'"
        ));
    }
    for name in [
        "Int8Array",
        "Uint8Array",
        "Int16Array",
        "Uint16Array",
        "Uint32Array",
        "BigUint64Array",
        "Float16Array",
        "Float32Array",
        "Float64Array",
    ] {
        check(&format!(
            "let calls=0,index={{valueOf(){{calls++;return 0}}}},caught=false;try{{Atomics.notify(new {name}(new SharedArrayBuffer(16)),index,index)}}catch(e){{caught=e instanceof TypeError}}caught&&calls===0"
        ));
    }
}

#[test]
fn shared_notify_host_failures_bypass_handlers_and_resource_disposal() {
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("unavailableHost");
    realm.eval("let disposed=0,caught=0,finalized=0;let resource={[Symbol.dispose](){disposed++}},view=new Int32Array(new SharedArrayBuffer(8));").unwrap();
    let error=realm.eval("function f(){using r=resource;try{Atomics.notify(view,0,{valueOf(){return unavailableHost()}})}catch(e){caught++}finally{finalized++}}f()").unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }), "{error}");
    assert_eq!(
        realm.eval("disposed===0&&caught===0&&finalized===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn ordinary_buffer_wait_and_wait_async_reject_before_any_conversion() {
    for method in ["wait", "waitAsync"] {
        for name in ["Int32Array", "BigInt64Array"] {
            check(&format!(
                "let hits=0,x={{valueOf(){{hits++;throw 7;}}}},caught=false;try{{Atomics.{method}(new {name}(1),x,x,x);}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
}

#[test]
fn nonsuspending_wait_converts_index_value_and_timeout_before_rejecting() {
    for (name, suffix) in [("Int32Array", ""), ("BigInt64Array", "n")] {
        for initial in ["0", "1"] {
            for timeout in ["undefined", "0", "NaN", "Infinity"] {
                let source = r#"let a=new $TYPE(new SharedArrayBuffer(16)),log='',caught=false;a[0]=$INITIAL$N;
                    try{Atomics.wait(a,{valueOf(){log+='i';return 0}},{valueOf(){log+='v';return 0$N}},{valueOf(){log+='t';return $TIMEOUT}})}catch(e){caught=e instanceof TypeError}
                    caught&&log==='ivt'"#;
                check(
                    &source
                        .replace("$TYPE", name)
                        .replace("$INITIAL", initial)
                        .replace("$TIMEOUT", timeout)
                        .replace("$N", suffix),
                );
            }
        }
        let source = r#"let a=new $TYPE(new SharedArrayBuffer(16)),reason={},same=false;
            try{Atomics.wait(a,0,0$N,{valueOf(){throw reason}})}catch(e){same=e===reason}same"#;
        check(&source.replace("$TYPE", name).replace("$N", suffix));
    }
}

#[test]
fn wait_async_immediate_results_preserve_comparison_and_timeout_precedence() {
    for (name, suffix) in [("Int32Array", ""), ("BigInt64Array", "n")] {
        for timeout in ["undefined", "NaN", "Infinity", "0", "-Infinity"] {
            let source = "let a=new $TYPE(new SharedArrayBuffer(16));a[0]=1$N;let r=Atomics.waitAsync(a,0,0$N,$TIMEOUT);r.async===false&&r.value==='not-equal'";
            check(
                &source
                    .replace("$TYPE", name)
                    .replace("$TIMEOUT", timeout)
                    .replace("$N", suffix),
            );
        }
        for timeout in ["0", "-0", "-0.1", "-Infinity", "false", "null"] {
            let source = "let a=new $TYPE(new SharedArrayBuffer(16)),r=Atomics.waitAsync(a,0,0$N,$TIMEOUT);r.async===false&&r.value==='timed-out'";
            check(
                &source
                    .replace("$TYPE", name)
                    .replace("$TIMEOUT", timeout)
                    .replace("$N", suffix),
            );
        }
    }
    check(
        r#"let O=Object,p=O.prototype,a=new Int32Array(new SharedArrayBuffer(8));
        O.defineProperty(p,'async',{set(){throw 1},configurable:true});O.defineProperty(p,'value',{set(){throw 2},configurable:true});
        Promise=new Proxy(function(){},{construct(){throw 3},get(){throw 4}});Object=function(){throw 5};
        let r=Atomics.waitAsync(a,0,0,0),s=Atomics.waitAsync(a,0,0,0),d=O.getOwnPropertyDescriptors(r);
        r!==s&&O.getPrototypeOf(r)===p&&O.keys(r).join(',')==='async,value'&&r.async===false&&r.value==='timed-out'&&
        d.async.writable&&d.async.enumerable&&d.async.configurable&&d.value.writable&&d.value.enumerable&&d.value.configurable"#,
    );
}

#[test]
fn wait_async_uses_original_bounds_but_reads_after_value_and_timeout_effects() {
    for (name, suffix) in [("Int32Array", ""), ("BigInt64Array", "n")] {
        let source = r#"let b=new SharedArrayBuffer(16,{maxByteLength:32}),a=new $TYPE(b),length=a.length,hits=0,caught=false;
            try{Atomics.waitAsync(a,{valueOf(){b.grow(32);return length}},{valueOf(){hits++;return 0$N}},0)}catch(e){caught=e instanceof RangeError}
            caught&&hits===0&&b.byteLength===32"#;
        check(&source.replace("$TYPE", name).replace("$N", suffix));
        let source = r#"let a=new $TYPE(new SharedArrayBuffer(16)),log='';
            let r=Atomics.waitAsync(a,{valueOf(){log+='i';return 0}},{valueOf(){log+='v';a[0]=0$N;return 0$N}},{valueOf(){log+='t';a[0]=1$N;return 0}});
            log==='ivt'&&r.async===false&&r.value==='not-equal'&&a[0]===1$N"#;
        check(&source.replace("$TYPE", name).replace("$N", suffix));
    }
}

#[test]
fn finite_wait_async_timers_settle_only_when_the_idle_host_services_them() {
    for (name, suffix) in [("Int32Array", ""), ("BigInt64Array", "n")] {
        for timeout in ["1", "2.5"] {
            let mut realm = Realm::default();
            let source = r#"let a=new $TYPE(new SharedArrayBuffer(16)),log='',caught=0,finalized=0,disposed=0,r={[Symbol.dispose](){disposed++}};
                function f(){using x=r;try{return Atomics.waitAsync(a,0,0$N,$TIMEOUT)}catch(e){caught++}finally{finalized++}}
                let result=f();result.value.then(v=>log=v);result.async===true&&result.value instanceof Promise&&log===''&&caught===0&&finalized===1&&disposed===1"#;
            assert_eq!(
                realm.eval(
                    &source
                        .replace("$TYPE", name)
                        .replace("$N", suffix)
                        .replace("$TIMEOUT", timeout)
                ),
                Ok(Value::Boolean(true))
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
            assert!(realm.run_next_atomics_wait_job().unwrap());
            assert!(realm.next_atomics_wait_delay().is_none());
            assert_eq!(
                realm.eval("log===''&&Atomics.notify(a,0)===0"),
                Ok(Value::Boolean(true))
            );
            realm.run_next_promise_job().unwrap();
            assert_eq!(realm.eval("log==='timed-out'"), Ok(Value::Boolean(true)));
            assert!(!realm.run_next_atomics_wait_job().unwrap());
        }
    }
}

#[test]
fn wait_operand_host_failures_precede_policy_rejection_and_skip_language_cleanup() {
    for method in ["wait", "waitAsync"] {
        let mut realm = Realm::default();
        realm.reserve_unsupported_global("unavailableHost");
        realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),caught=0,finalized=0,disposed=0,r={[Symbol.dispose](){disposed++}};").unwrap();
        let source = format!(
            "function f(){{using x=r;try{{Atomics.{method}(a,0,{{valueOf(){{return unavailableHost()}}}},0)}}catch(e){{caught++}}finally{{finalized++}}}}f()"
        );
        let result = realm.eval(&source);
        assert!(
            matches!(result, Err(Error::Unsupported { .. })),
            "{result:?}"
        );
        assert_eq!(
            realm.eval("caught===0&&finalized===0&&disposed===0"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn infinite_wait_async_returns_a_real_promise_and_notify_settles_before_returning() {
    for (name, suffix) in [("Int32Array", ""), ("BigInt64Array", "n")] {
        for timeout in ["undefined", "Infinity", "NaN"] {
            let mut realm = Realm::default();
            let source = "let a=new $TYPE(new SharedArrayBuffer(16)),r=Atomics.waitAsync(a,0,0$N,$TIMEOUT),log='';r.value.then(x=>log+=x);r.async===true&&r.value instanceof Promise&&Atomics.notify(a,0,1)===1&&log===''";
            assert_eq!(
                realm.eval(
                    &source
                        .replace("$TYPE", name)
                        .replace("$N", suffix)
                        .replace("$TIMEOUT", timeout)
                ),
                Ok(Value::Boolean(true))
            );
            // Same-agent resolution has already queued this reaction. There is
            // no extra fulfillment job and no user callback inside notify.
            realm.run_next_promise_job().unwrap();
            assert_eq!(
                realm.eval("log==='ok'&&Atomics.notify(a,0)===0"),
                Ok(Value::Boolean(true))
            );
        }
    }
}

#[test]
fn notify_uses_waiter_fifo_counts_byte_positions_and_backing_identity() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"let b=new SharedArrayBuffer(16,{maxByteLength:32}),a=new Int32Array(b),big=new BigInt64Array(b),other=new Int32Array(new SharedArrayBuffer(16)),log='';
        for(let i=0;i<3;i++){Atomics.waitAsync(a,0,0).value.then(()=>log+=i)}
        Atomics.waitAsync(a,2,0).value.then(()=>log+='x');
        b.grow(32);
        Atomics.notify(other,0)===0&&Atomics.notify(a,1)===0&&Atomics.notify(a,0,-1)===0&&Atomics.notify(a,0,NaN)===0&&Atomics.notify(big,0,1.9)===1&&Atomics.notify(a,0,Infinity)===2&&Atomics.notify(big,1)===1"#), Ok(Value::Boolean(true)));
    for _ in 0..4 {
        realm.run_next_promise_job().unwrap();
    }
    assert_eq!(
        realm.eval("log==='012x'&&Atomics.notify(a,0)===0&&Atomics.notify(a,2)===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn wait_queue_registration_during_notify_count_conversion_is_observed() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),log='',p;let count=Atomics.notify(a,0,{valueOf(){p=Atomics.waitAsync(a,0,0).value;p.then(x=>log=x);return 1}});count===1&&log==='';"), Ok(Value::Boolean(true)));
    realm.run_next_promise_job().unwrap();
    assert_eq!(realm.eval("log==='ok'"), Ok(Value::Boolean(true)));
}

#[test]
fn notify_fulfills_wait_promises_before_later_promise_jobs_are_enqueued() {
    let mut realm = Realm::default();
    realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),log='';Atomics.waitAsync(a,0,0).value.then(()=>log+='w');Promise.resolve().then(()=>log+='a');Atomics.notify(a,0);Promise.resolve().then(()=>log+='b');").unwrap();
    for _ in 0..3 {
        realm.run_next_promise_job().unwrap();
    }
    assert_eq!(realm.eval("log==='awb'"), Ok(Value::Boolean(true)));
}

#[test]
fn huge_finite_timeouts_do_not_overflow_or_expire_before_notification() {
    for timeout in ["Number.MAX_VALUE", "2**64", "Number.MAX_SAFE_INTEGER"] {
        let mut realm = Realm::default();
        assert_eq!(realm.eval(&format!("let a=new Int32Array(new SharedArrayBuffer(8)),r=Atomics.waitAsync(a,0,0,{timeout}),log='';r.value.then(v=>log=v);r.async===true&&r.value instanceof Promise")), Ok(Value::Boolean(true)));
        assert!(realm.next_atomics_wait_delay().is_some());
        assert!(!realm.run_next_atomics_wait_job().unwrap());
        assert_eq!(realm.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
        assert!(realm.next_atomics_wait_delay().is_none());
        realm.run_next_promise_job().unwrap();
        assert_eq!(realm.eval("log==='ok'"), Ok(Value::Boolean(true)));
    }
}

#[test]
fn notification_can_win_after_the_deadline_before_the_timeout_job_runs() {
    let mut realm = Realm::default();
    realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),log='';Atomics.waitAsync(a,0,0,1).value.then(v=>log=v);").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    assert_eq!(realm.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
    assert!(!realm.run_next_atomics_wait_job().unwrap());
    realm.run_next_promise_job().unwrap();
    assert_eq!(realm.eval("log==='ok'"), Ok(Value::Boolean(true)));
}

#[test]
fn is_lock_free_converts_once_and_reports_consistent_agent_flags() {
    for size in [
        "undefined",
        "null",
        "NaN",
        "0",
        "-0",
        "-1",
        "3",
        "5",
        "16",
        "Infinity",
        "-Infinity",
    ] {
        check(&format!("Atomics.isLockFree({size})===false"));
    }
    check(
        "Atomics.isLockFree(4) && Atomics.isLockFree(4.9) && Atomics.isLockFree('4') && Atomics.isLockFree(1)===Atomics.isLockFree(1.9) && typeof Atomics.isLockFree(2)==='boolean' && typeof Atomics.isLockFree(8)==='boolean'",
    );
    check(
        "let log='';Atomics.isLockFree.call(null,{[Symbol.toPrimitive](hint){log+=hint;return 4;}}) && log==='number'",
    );
    for value in ["1n", "Symbol()"] {
        exception(
            &format!("Atomics.isLockFree({value})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn host_limits_bypass_catch_and_store_does_not_decode_the_old_bigint() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(8),
        ..Limits::default()
    });
    realm
        .eval("let a=new BigUint64Array(1);new Uint8Array(a.buffer).fill(255)")
        .unwrap();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{Atomics.add(a,0,1n)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && new Uint8Array(a.buffer).every(x=>x===255)"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("Atomics.store(a,0,1n)===1n"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    assert!(matches!(realm.eval("let a=Int32Array.of(7),flag=0;try{Atomics.add(a,0,{valueOf(){while(true){}}})}catch{flag=1}finally{flag=2}"), Err(Error::Limit {..})));
    assert_eq!(realm.eval("a[0]===7 && flag===0"), Ok(Value::Boolean(true)));
}

#[test]
fn large_sparse_offsets_and_page_boundaries_preserve_unrelated_bytes() {
    for (name, bigint) in INTEGER_KINDS {
        let n = if bigint { "n" } else { "" };
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,start=2**40+4096,b=new ArrayBuffer(start+4096),a=new A(b,start-size,2),d=new DataView(b);d.setUint8(start-2*size,123);d.setUint8(start+size,124);Atomics.store(a,0,5{n})===5{n} && Atomics.add(a,1,7{n})===0{n} && Atomics.load(a,0)===5{n} && a[1]===7{n} && d.getUint8(start-2*size)===123 && d.getUint8(start+size)===124"
        ));
    }
}

#[test]
fn retained_methods_and_views_survive_collection_and_global_deletion() {
    let mut realm = Realm::default();
    realm.eval("let load=Atomics.load,store=Atomics.store,add=Atomics.add,a=Int32Array.of(7);delete this.Atomics").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("store(a,0,9)===9 && add(a,0,2)===9 && load(a,0)===11"),
        Ok(Value::Boolean(true))
    );
}
