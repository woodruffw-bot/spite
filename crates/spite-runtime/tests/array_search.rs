//! Array searches distinguish SameValueZero, strict equality, and holes.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

const METHODS: [&str; 3] = ["includes", "indexOf", "lastIndexOf"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn equality_handles_nan_zeros_types_and_object_identity_without_coercion() {
    check("[NaN].includes(NaN) && [NaN].indexOf(NaN)===-1 && [NaN].lastIndexOf(NaN)===-1");
    check(
        "[0].includes(-0) && [-0].includes(0) && Object.is([-0].indexOf(0),0) && Object.is([0].lastIndexOf(-0),0)",
    );
    check(
        "![1].includes('1') && ![1].includes(1n) && ![1].includes(true) && ![null].includes(undefined) && [1n].includes(1n)",
    );
    check(
        "[1,2,1].indexOf(1)===0 && [1,2,1].lastIndexOf(1)===2 && [1,2].indexOf(7)===-1 && ![1,2].includes(7)",
    );
    for method in METHODS {
        check(&format!(
            "let o={{valueOf:()=>{{throw 7;}},toString:()=>{{throw 8;}}}};[o].{method}(o)==={} && [o].{method}({{}})==={}",
            if method == "includes" { "true" } else { "0" },
            if method == "includes" { "false" } else { "-1" }
        ));
    }
    check("['\\uD800'].includes('\\uD800') && ['💩'].indexOf('💩')===0 && ['é'].indexOf('é')===-1");
    check("[123456789012345678901234567890n].lastIndexOf(123456789012345678901234567890n)===0");
}

#[test]
fn includes_reads_holes_while_index_searches_skip_absent_properties() {
    check(
        "[,].includes(undefined) && [,].indexOf(undefined)===-1 && [,].lastIndexOf(undefined)===-1",
    );
    check("[,undefined,,].indexOf(undefined)===1 && [,undefined,,].lastIndexOf(undefined)===1");
    check(
        "[].indexOf()===-1 && ![].includes() && [undefined].includes() && [undefined].lastIndexOf()===0",
    );
    check(
        "Array.prototype[1]=7;[,,].includes(7) && [,,].indexOf(7)===1 && [,,].lastIndexOf(7)===1",
    );
    check(
        "let a=[,,];Object.defineProperty(Array.prototype,'1',{get:()=>9});a.includes(9) && a.indexOf(9)===1 && a.lastIndexOf(9)===1",
    );
}

#[test]
fn starting_indices_distinguish_absence_nan_infinities_and_relative_offsets() {
    check(
        "[1,2,1].lastIndexOf(1)===2 && [1,2,1].lastIndexOf(1,undefined)===0 && [1,2,1].lastIndexOf(1,NaN)===0",
    );
    for from in [
        "undefined",
        "NaN",
        "null",
        "false",
        "0",
        "-0",
        "0.9",
        "-0.9",
        "-Infinity",
        "-100",
    ] {
        check(&format!(
            "[7,8].includes(7,{from}) && [7,8].indexOf(7,{from})===0"
        ));
    }
    check(
        "[1,2,3].indexOf(2,'1.9')===1 && [1,2,3].indexOf(3,-1.9)===2 && [1,2,3].lastIndexOf(2,-1.9)===1",
    );
    check(
        "![1,2].includes(1,Infinity) && [1,2].indexOf(1,Infinity)===-1 && [1,2].lastIndexOf(2,Infinity)===1 && [1,2].lastIndexOf(1,-Infinity)===-1",
    );
    check(
        "[1,2,1].lastIndexOf(1,99)===2 && [1,2,1].lastIndexOf(1,-3)===0 && [1,2,1].lastIndexOf(1,-4)===-1",
    );
    check(
        "![1,2].includes(2,2) && [1,2].indexOf(2,2)===-1 && [1,2].includes(2,-1) && ![1,2].includes(1,-1)",
    );
}

#[test]
fn generic_receivers_use_utf16_indices_and_the_full_safe_integer_length_range() {
    check(
        "Array.prototype.includes.call('💩x','\\uDCA9') && Array.prototype.indexOf.call('💩x','x')===2 && Array.prototype.lastIndexOf.call('aba','a')===2",
    );
    check(
        "let o={0:'a',1:'b',2:'c',length:'2.9'};Array.prototype.includes.call(o,'b') && Array.prototype.indexOf.call(o,'c')===-1 && Array.prototype.lastIndexOf.call(o,'b')===1",
    );
    check(
        "let o={9007199254740990:7,length:Infinity};Array.prototype.includes.call(o,7,-1) && Array.prototype.indexOf.call(o,7,9007199254740990)===9007199254740990 && Array.prototype.lastIndexOf.call(o,7)===9007199254740990",
    );
    check(
        "!Array.prototype.includes.call(false,undefined) && Array.prototype.indexOf.call(7,undefined)===-1 && Array.prototype.lastIndexOf.call(7,undefined)===-1",
    );
    for length in ["NaN", "-1", "-Infinity", "0.9"] {
        check(&format!(
            "!Array.prototype.includes.call({{0:7,length:{length}}},7)"
        ));
    }
}

#[test]
fn length_is_snapshotted_before_index_coercion_but_property_values_remain_live() {
    check("let a=[1,2];a.indexOf(9,{valueOf:()=>{a[1]=9;a[2]=9;return 0;}})===1");
    check("let a=[1];!a.includes(9,{valueOf:()=>{a[1]=9;return 0;}})");
    check("let a=[1,2];a.includes(undefined,{valueOf:()=>{a.length=0;return 0;}})");
    check(
        "let a=[1,2];a.lastIndexOf(7,{valueOf:()=>{a.length=0;Array.prototype[1]=7;return 2;}})===1",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{get:()=>{delete a[1];a[2]=9;return 1;}});a.indexOf(9)===2",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'2',{get:()=>{delete a[1];a[0]=9;return 3;}});a.lastIndexOf(9)===0",
    );
    for method in METHODS {
        check(&format!(
            "let log='',o={{}};Object.defineProperty(o,'length',{{get:()=>{{log+='l';return {{valueOf:()=>{{log+='n';return 1;}}}};}}}});Object.defineProperty(o,'0',{{get:()=>{{log+='g';return 7;}}}});Array.prototype.{method}.call(o,7,{{valueOf:()=>{{log+='i';return 0;}}}});log==='lnig'"
        ));
    }
}

#[test]
fn empty_ranges_skip_index_conversion_and_out_of_range_indices_skip_getters() {
    for method in METHODS {
        check(&format!(
            "let n=0;Array.prototype.{method}.call({{length:0}},7,{{valueOf:()=>{{n++;throw 7;}}}});n===0"
        ));
        check(&format!(
            "[].{method}(7,1n)==={}",
            if method == "includes" { "false" } else { "-1" }
        ));
        let from = if method == "lastIndexOf" {
            "-Infinity"
        } else {
            "Infinity"
        };
        check(&format!(
            "let n=0,o={{length:1}};Object.defineProperty(o,'0',{{get:()=>{{n++;throw 7;}}}});Array.prototype.{method}.call(o,7,{from});n===0"
        ));
        for receiver in ["null", "undefined", "{length:1n}"] {
            assert!(matches!(
                Realm::default().eval(&format!("Array.prototype.{method}.call({receiver},7)")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        assert!(matches!(
            Realm::default().eval(&format!("[7].{method}(7,1n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn reads_are_ordered_and_stop_after_success_or_an_abrupt_completion() {
    for method in METHODS {
        let expected = if method == "lastIndexOf" { "10" } else { "01" };
        check(&format!(
            "let log='',o={{length:2}};Object.defineProperty(o,'0',{{get:()=>{{log+='0';return 0;}}}});Object.defineProperty(o,'1',{{get:()=>{{log+='1';return 1;}}}});Array.prototype.{method}.call(o,9);log==='{expected}'"
        ));
        check(&format!(
            "let n=0,a=[7,7];Object.defineProperty(a,'{}',{{get:()=>{{n++;throw 8;}}}});a.{method}(7);n===0",
            if method == "lastIndexOf" { "0" } else { "1" }
        ));
        for source in [
            format!("Array.prototype.{method}.call({{length:{{valueOf:()=>{{throw 7;}}}}}},7)"),
            format!("[1].{method}(7,{{valueOf:()=>{{throw 7;}}}})"),
            format!(
                "let a=[1];Object.defineProperty(a,'0',{{get:()=>{{throw 7;}}}});a.{method}(7)"
            ),
        ] {
            assert_eq!(
                Realm::default().eval(&source),
                Err(Error::Thrown(Value::Number(7.0)))
            );
        }
    }
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    for method in METHODS {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length===1 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Array.prototype.{method}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let f=Array.prototype.{method};delete Array.prototype.{method}"
            ))
            .unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("f.call([7],7)"),
            Ok(if method == "includes" {
                Value::Boolean(true)
            } else {
                Value::Number(0.0)
            })
        );
    }
}
