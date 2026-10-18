//! TypedArray numeric stable sorting and independent sorted copies (23.2.3.29/.33).

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
        matches!(&result,Err(Error::Exception {kind,..}) if *kind==expected),
        "{source}: {result:?}"
    );
}

#[test]
fn all_twelve_kinds_sort_numerically_in_place_and_create_independent_sorted_copies() {
    for (name, bigint) in KINDS {
        let values = if bigint {
            "12n,2n,1n,10n,2n"
        } else {
            "12,2,1,10,2"
        };
        let nine = if bigint { "9n" } else { "9" };
        check(&format!(
            "let A={name},a=new A([{values}]),b=a.toSorted(),c=a.toSorted((x,y)=>x<y?1:x>y?-1:0);let independent=b instanceof A && c instanceof A && b!==a && c!==a && b.buffer!==a.buffer && c.buffer!==a.buffer && b.join()==='1,2,2,10,12' && c.join()==='12,10,2,2,1' && a.join()==='12,2,1,10,2';let same=a.sort()===a && a.join()==='1,2,2,10,12';b[0]={nine};independent && same && a[0]!=={nine} && a.sort((x,y)=>x<y?1:x>y?-1:0)===a && a.join()==='12,10,2,2,1'"
        ));
    }
}

#[test]
fn empty_and_single_element_views_validate_callbacks_but_never_invoke_them() {
    for (name, value) in [("Uint8Array", "7"), ("BigInt64Array", "7n")] {
        for method in ["sort", "toSorted"] {
            check(&format!(
                "let empty=new {name}(0),single=new {name}([{value}]),calls=0;let e=empty.{method}(()=>{{calls++;throw 7;}}),s=single.{method}(()=>{{calls++;throw 8;}});calls===0 && e.length===0 && s.length===1 && s[0]==={value} && e instanceof {name} && s instanceof {name} && {}",
                if method == "sort" {
                    "e===empty && s===single"
                } else {
                    "e!==empty && s!==single && e.buffer!==empty.buffer && s.buffer!==single.buffer"
                }
            ));
            check(&format!(
                "class C{{}}new {name}(0).{method}(C).length===0 && new {name}([{value}]).{method}(C)[0]==={value}"
            ));
        }
    }
}

#[test]
fn comparator_validation_covers_empty_and_invalid_receivers_without_coercing_nonfunctions() {
    for method in ["sort", "toSorted"] {
        for receiver in [
            "null",
            "{}",
            "[]",
            "new DataView(new ArrayBuffer(3))",
            "Object.create(new Uint8Array(3))",
            "new Uint8Array(0)",
        ] {
            for comparator in [
                "null",
                "7",
                "false",
                "'x'",
                "Symbol()",
                "{valueOf(){throw 7;},toString(){throw 8;}}",
            ] {
                exception(
                    &format!("Uint8Array.prototype.{method}.call({receiver},{comparator})"),
                    ExceptionKind::TypeError,
                );
            }
        }
        for receiver in [
            "null",
            "{}",
            "[]",
            "new DataView(new ArrayBuffer(3))",
            "Object.create(new Uint8Array(3))",
        ] {
            exception(
                &format!("Uint8Array.prototype.{method}.call({receiver},()=>0)"),
                ExceptionKind::TypeError,
            );
        }
        exception(
            &format!("class C{{}}Uint8Array.of(2,1).{method}(C)"),
            ExceptionKind::TypeError,
        );
        for invalidation in ["buffer.transfer()", "buffer.resize(1)"] {
            check(&format!(
                "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array(buffer,0,3),calls=0,caught=false;{invalidation};try{{a.{method}(()=>{{calls++;throw 7;}});}}catch(e){{caught=e instanceof TypeError;}}caught && calls===0"
            ));
        }
    }
}

#[test]
fn public_state_iteration_and_constructor_overrides_are_unobserved() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=Uint16Array.of(12,2,1);for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT','constructor',Symbol.iterator])Object.defineProperty(a,key,{{get(){{throw 7;}}}});let b=a.{method}();b[0]===1 && b[1]===2 && b[2]===12 && {}",
            if method == "sort" {
                "b===a"
            } else {
                "b!==a && a[0]===12 && a[1]===2 && a[2]===1"
            }
        ));
    }
    check(
        "let A=Uint16Array,prototype=A.prototype;class S extends A{}let a=new S([12,2,1]);Object.defineProperty(a,'constructor',{get(){throw 7;}});Object.defineProperty(S,Symbol.species,{get(){throw 8;}});Uint16Array=function(){throw 9;};let b=a.toSorted();Object.getPrototypeOf(b)===prototype && !(b instanceof S) && b.join()==='1,2,12' && a.join()==='12,2,1'",
    );
}

#[test]
fn default_floating_order_places_negative_zero_before_positive_zero_and_nans_last() {
    for name in ["Float16Array", "Float32Array", "Float64Array"] {
        for method in ["sort", "toSorted"] {
            check(&format!(
                "let a={name}.of(NaN,0,-0,Infinity,-Infinity,-5,3,NaN),b=a.{method}(undefined);b[0]===-Infinity && b[1]===-5 && Object.is(b[2],-0) && Object.is(b[3],0) && b[4]===3 && b[5]===Infinity && Number.isNaN(b[6]) && Number.isNaN(b[7]) && {}",
                if method == "sort" {
                    "b===a"
                } else {
                    "b!==a && Number.isNaN(a[0]) && Object.is(a[1],0) && Object.is(a[2],-0) && a[3]===Infinity && a[4]===-Infinity"
                }
            ));
        }
    }
}

#[test]
fn default_bigint_order_is_exact_across_binary64_precision_and_signed_boundaries() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=BigInt64Array.of(9007199254740993n,9007199254740992n,-9007199254740993n,9223372036854775807n,-9223372036854775808n,0n,-1n),b=a.{method}();b.join()==='-9223372036854775808,-9007199254740993,-1,0,9007199254740992,9007199254740993,9223372036854775807'"
        ));
        check(&format!(
            "let a=BigUint64Array.of(18446744073709551615n,9007199254740993n,9007199254740992n,0n,1n),b=a.{method}();b.join()==='0,1,9007199254740992,9007199254740993,18446744073709551615'"
        ));
    }
}

#[test]
fn custom_equal_groups_retain_their_original_order_for_number_and_bigint_contents() {
    for (name, values, comparator) in [
        (
            "Uint8Array",
            "21,11,22,12,23,13",
            "Math.floor(x/10)-Math.floor(y/10)",
        ),
        (
            "BigInt64Array",
            "21n,11n,22n,12n,23n,13n",
            "x/10n<y/10n?-1:x/10n>y/10n?1:0",
        ),
    ] {
        for method in ["sort", "toSorted"] {
            check(&format!(
                "let a={name}.of({values}),b=a.{method}((x,y)=>{comparator});b.join()==='11,12,13,21,22,23'"
            ));
        }
    }
}

#[test]
fn custom_zero_negative_zero_and_nan_results_mean_stable_equality() {
    for (name, values) in [
        ("Float64Array", "0,-0,3,1,2"),
        ("BigInt64Array", "3n,1n,2n"),
    ] {
        for method in ["sort", "toSorted"] {
            for result in [
                "0",
                "-0",
                "NaN",
                "undefined",
                "null",
                "false",
                "'not a number'",
            ] {
                check(&format!(
                    "let a={name}.of({values}),b=a.{method}(()=>{result});b.join()===a.join() && {}",
                    if name == "Float64Array" {
                        "Object.is(b[0],0) && Object.is(b[1],-0) && b[2]===3 && b[3]===1 && b[4]===2"
                    } else {
                        "b[0]===3n && b[1]===1n && b[2]===2n"
                    }
                ));
            }
        }
    }
}

#[test]
fn custom_comparators_receive_primitive_values_exactly_two_arguments_and_undefined_this() {
    for (name, values, scalar_type) in [
        ("Uint8Array", "3,1,2", "number"),
        ("BigInt64Array", "3n,1n,2n", "bigint"),
    ] {
        for method in ["sort", "toSorted"] {
            check(&format!(
                "let a={name}.of({values}),calls=0,valid=true;let b=a.{method}(function(x,y){{'use strict';calls++;valid=valid && this===undefined && arguments.length===2 && typeof x==='{scalar_type}' && typeof y==='{scalar_type}';return x<y?-1:x>y?1:0;}});valid && calls>0 && b.join()==='1,2,3'"
            ));
        }
    }
}

#[test]
fn custom_comparison_runs_for_nan_inputs_and_infinite_results_supply_normal_ordering() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=Float64Array.of(3,NaN,1,NaN,2),sawNaN=false;let b=a.{method}((x,y)=>{{let nx=Number.isNaN(x),ny=Number.isNaN(y);if(nx || ny)sawNaN=true;return nx?(ny?0:-1):ny?1:x-y;}});sawNaN && Number.isNaN(b[0]) && Number.isNaN(b[1]) && b[2]===1 && b[3]===2 && b[4]===3"
        ));
        check(&format!(
            "Uint8Array.of(3,1,2).{method}((x,y)=>x<y?-Infinity:x>y?Infinity:NaN).join()==='1,2,3'"
        ));
    }
}

#[test]
fn each_comparator_result_is_converted_once_with_number_hint_before_the_next_comparison() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=Uint8Array.of(3,1,2),calls=0,conversions=0,valid=true,pending=false;let b=a.{method}((x,y)=>{{valid=valid && !pending;pending=true;calls++;return {{[Symbol.toPrimitive](hint){{valid=valid && pending && hint==='number';pending=false;conversions++;return x-y;}}}};}});valid && !pending && calls>0 && conversions===calls && b.join()==='1,2,3'"
        ));
        check(&format!(
            "let calls=0,values=0,strings=0;let b=Uint8Array.of(3,1,2).{method}((x,y)=>{{calls++;return {{valueOf(){{values++;return {{}};}},toString(){{strings++;return String(x-y);}}}};}});calls>0 && calls===values && values===strings && b.join()==='1,2,3'"
        ));
    }
}

#[test]
fn bigint_and_symbol_comparator_results_throw_type_error_instead_of_becoming_numbers() {
    for method in ["sort", "toSorted"] {
        for (name, values) in [("Uint8Array", "2,1"), ("BigInt64Array", "2n,1n")] {
            for result in ["0n", "1n", "Symbol()", "{valueOf(){return 1n;}}"] {
                exception(
                    &format!("{name}.of({values}).{method}(()=>({result}))"),
                    ExceptionKind::TypeError,
                );
            }
        }
    }
}

#[test]
fn abrupt_comparisons_stop_without_writeback_and_preserve_their_own_prior_effects() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=Uint8Array.of(3,1,2),calls=0,caught=false;try{{a.{method}(()=>{{calls++;a[0]=99;throw 7;}});}}catch(e){{caught=e===7;}}caught && calls===1 && a.join()==='99,1,2'"
        ));
        check(&format!(
            "let a=Uint8Array.of(3,1,2),calls=0,conversions=0,caught=false;try{{a.{method}(()=>{{calls++;a[0]=99;return {{valueOf(){{conversions++;a[1]=88;throw 8;}}}};}});}}catch(e){{caught=e===8;}}caught && calls===1 && conversions===1 && a.join()==='99,88,2'"
        ));
        assert_eq!(
            Realm::default().eval(&format!(
                "Uint8Array.of(2,1).{method}(()=>({{get [Symbol.toPrimitive](){{throw 9;}}}}))"
            )),
            Err(Error::Thrown(Value::Number(9.0)))
        );
    }
}

#[test]
fn comparisons_use_the_complete_captured_list_and_algorithmic_writeback_waits_until_they_finish() {
    // Source mutation makes comparison ordering implementation-defined. Assert
    // captured membership and effects without fixing a comparison sequence/order.
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let a=Uint8Array.of(4,1,3,2),first=true,valid=true,calls=0;let b=a.{method}((x,y)=>{{calls++;valid=valid && x>=1 && x<=4 && y>=1 && y<=4;if(first){{first=false;a.fill(99);}}valid=valid && a[0]===99 && a[1]===99 && a[2]===99 && a[3]===99;return x-y;}});valid && calls>0 && b.length===4 && b.includes(1) && b.includes(2) && b.includes(3) && b.includes(4) && {}",
            if method == "sort" {
                "b===a"
            } else {
                "b!==a && a.join()==='99,99,99,99'"
            }
        ));
    }
}

#[test]
fn detachment_during_comparison_or_result_conversion_keeps_the_captured_list_and_skips_invalid_writes()
 {
    for method in ["sort", "toSorted"] {
        for conversion_hook in [false, true] {
            let result = if conversion_hook {
                "return {valueOf(){if(!moved)moved=buffer.transfer();return x-y;}};"
            } else {
                "if(!moved)moved=buffer.transfer();return x-y;"
            };
            check(&format!(
                "let a=Uint8Array.of(3,1,2),buffer=a.buffer,moved,valid=true,calls=0;let b=a.{method}((x,y)=>{{calls++;valid=valid && x>=1 && x<=3 && y>=1 && y<=3;{result}}});valid && calls>0 && a.length===0 && new Uint8Array(moved).join()==='3,1,2' && {}",
                if method == "sort" {
                    "b===a && b.length===0"
                } else {
                    "b!==a && b.buffer!==buffer && b.length===3 && b.includes(1) && b.includes(2) && b.includes(3)"
                }
            ));
        }
    }
}

#[test]
fn fixed_view_shrink_during_result_conversion_does_not_cause_a_second_validation() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(buffer,0,4),changed=false,valid=true;a.set([4,1,3,2]);let b=a.{method}((x,y)=>{{valid=valid && x>=1 && x<=4 && y>=1 && y<=4;return {{valueOf(){{if(!changed){{changed=true;buffer.resize(2);}}return x-y;}}}};}});changed && valid && a.length===0 && new Uint8Array(buffer).join()==='4,1' && {}",
            if method == "sort" {
                "b===a"
            } else {
                "b!==a && b.length===4 && b.includes(1) && b.includes(2) && b.includes(3) && b.includes(4)"
            }
        ));
    }
}

#[test]
fn growth_keeps_new_tail_elements_outside_the_captured_range_and_sorted_copy() {
    for method in ["sort", "toSorted"] {
        check(&format!(
            "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(buffer),changed=false,valid=true;a.set([4,1,3,2]);let b=a.{method}((x,y)=>{{valid=valid && x>=1 && x<=4 && y>=1 && y<=4;if(!changed){{changed=true;buffer.resize(8);a[4]=99;a[7]=98;}}return x-y;}});changed && valid && a.length===8 && a[4]===99 && a[5]===0 && a[6]===0 && a[7]===98 && {}",
            if method == "sort" {
                "b===a && a[0]>=1 && a[0]<=4 && a[1]>=1 && a[1]<=4 && a[2]>=1 && a[2]<=4 && a[3]>=1 && a[3]<=4 && a.includes(1) && a.includes(2) && a.includes(3) && a.includes(4)"
            } else {
                "b!==a && b.length===4 && b.includes(1) && b.includes(2) && b.includes(3) && b.includes(4) && a[0]===4 && a[1]===1 && a[2]===3 && a[3]===2"
            }
        ));
    }
}

#[test]
fn resize_recovery_between_comparator_and_result_conversion_uses_captured_values_for_live_writes() {
    for method in ["sort", "toSorted"] {
        for constructor_arguments in ["buffer", "buffer,0,4"] {
            check(&format!(
                "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array({constructor_arguments}),first=true,valid=true;a.set([4,1,3,2]);let b=a.{method}((x,y)=>{{valid=valid && x>=1 && x<=4 && y>=1 && y<=4;let recover=first;if(first){{first=false;buffer.resize(2);}}return {{valueOf(){{if(recover)buffer.resize(4);return x-y;}}}};}});valid && a.length===4 && b.length===4 && b.includes(1) && b.includes(2) && b.includes(3) && b.includes(4) && {}",
                if method == "sort" {
                    "b===a"
                } else {
                    "b!==a && a.join()==='4,1,0,0'"
                }
            ));
        }
    }
}

#[test]
fn numeric_sort_writes_use_existing_nan_encoding_while_sorted_copies_preserve_source_bits() {
    for (name, setter) in [
        ("Float16Array", "setFloat16"),
        ("Float32Array", "setFloat32"),
        ("Float64Array", "setFloat64"),
    ] {
        for method in ["sort", "toSorted"] {
            check(&format!(
                "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(3*size),data=new DataView(buffer),a=new A(buffer),expected=new DataView(new ArrayBuffer(size)),valid=true;for(let i=0;i<size;i++)data.setUint8(i,255);data.{setter}(size,1,little);data.{setter}(2*size,0,little);expected.{setter}(0,NaN,little);let b=a.{method}(),output=new DataView(b.buffer);for(let i=0;i<size;i++)valid=valid && output.getUint8(2*size+i)===expected.getUint8(i) && {};valid && b[0]===0 && b[1]===1 && Number.isNaN(b[2]) && {}",
                if method == "sort" {
                    "true"
                } else {
                    "data.getUint8(i)===255"
                },
                if method == "sort" {
                    "b===a"
                } else {
                    "b!==a && Number.isNaN(a[0]) && a[1]===1 && a[2]===0"
                }
            ));
        }
    }
}

#[test]
fn sorted_copies_remain_reachable_after_their_original_source_is_collected() {
    let mut realm = Realm::default();
    realm
        .eval("let sorted=(()=>{let a=Uint16Array.of(12,2,1);return a.toSorted();})()")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("sorted.join()==='1,2,12'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn explicit_bigint_limit_aborts_complete_list_collection_before_any_comparison() {
    for method in ["sort", "toSorted"] {
        let mut realm = Realm::new(Limits {
            max_bigint_bits: Some(8),
            ..Limits::default()
        });
        realm.eval("let little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,bytes=new Uint8Array(24),data=new DataView(bytes.buffer);data.setUint32(little?0:4,1,little);data.setUint32(little?8:12,2,little);data.setUint32(little?16:20,256,little);let a=new BigUint64Array(bytes.buffer),calls=0,flag=0").unwrap();
        assert!(matches!(realm.eval(&format!("try{{a.{method}((x,y)=>{{calls++;return x<y?-1:x>y?1:0;}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit { .. })));
        assert_eq!(realm.eval("calls===0 && flag===0 && data.getUint32(little?0:4,little)===1 && data.getUint32(little?8:12,little)===2 && data.getUint32(little?16:20,little)===256"),Ok(Value::Boolean(true)));
    }
}

#[test]
fn explicit_argument_limit_applies_to_the_real_comparator_call_not_the_scratch_list() {
    for method in ["sort", "toSorted"] {
        let mut realm = Realm::new(Limits {
            max_arguments: Some(1),
            ..Limits::default()
        });
        assert_eq!(
            realm.eval(&format!(
                "let a=new Uint8Array([3,1,2]),calls=0,flag=0;a.{method}().join()==='1,2,3'"
            )),
            Ok(Value::Boolean(true))
        );
        realm.eval("a[0]=3;a[1]=1;a[2]=2").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{a.{method}(()=>{{calls++;return 0;}});}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("calls===0 && flag===0 && a.join()==='3,1,2'"),
            Ok(Value::Boolean(true))
        );
    }
}
