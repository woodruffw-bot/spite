//! Array.prototype.toSpliced preserves argument presence and skips removed reads.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn omission_and_undefined_have_distinct_deletion_ranges() {
    for (args, expected) in [
        ("", "1,2,3"),
        ("undefined", ""),
        ("undefined,undefined", "1,2,3"),
        ("1", "1"),
        ("1,undefined", "1,2,3"),
        ("1,0,7,8", "1,7,8,2,3"),
        ("1,1,7", "1,7,3"),
        ("0,Infinity", ""),
        ("1,-Infinity,7", "1,7,2,3"),
        ("1,NaN,7", "1,7,2,3"),
        ("1,99,7", "1,7"),
        ("1.9,1.9,7", "1,7,3"),
        ("-1,1,7", "1,2,7"),
        ("-0.9,1,7", "7,2,3"),
        ("-Infinity,1,7", "7,2,3"),
        ("Infinity,1,7", "1,2,3,7"),
    ] {
        check(&format!(
            "let a=[1,2,3],b=a.toSpliced({args});a!==b && Array.isArray(b) && b.join()==='{expected}' && a.join()==='1,2,3'"
        ));
    }
    check("[].toSpliced(0,0,7)[0]===7 && [].toSpliced().length===0");
    check("let o={valueOf:()=>{throw 7;}},a=[o],b=a.toSpliced(1,0,o);b[0]===o && b[1]===o");
}

#[test]
fn copies_are_dense_intrinsic_arrays_and_bypass_inherited_setters() {
    check(
        "let a=[,,,],b=a.toSpliced(1,1);b.length===2 && Object.hasOwn(b,'0') && Object.hasOwn(b,'1') && b[0]===undefined",
    );
    check("Array.prototype[2]=9;[,,,].toSpliced(1,1)[1]===9");
    check(
        "let a=Object.freeze([1,2]),p=Array.prototype;Array=function(){throw 7;};let b=a.toSpliced(1,1,9);b.join()==='1,9' && Object.getPrototypeOf(b)===p",
    );
    check(
        "let a=[1,2];Object.defineProperty(a,'constructor',{get:()=>{throw 7;}});Object.defineProperty(Array.prototype,'0',{set:()=>{throw 8;}});let b=a.toSpliced(),d=Object.getOwnPropertyDescriptor(b,'0');d.value===1 && d.writable && d.enumerable && d.configurable && Object.getPrototypeOf(b)===Array.prototype",
    );
}

#[test]
fn removed_elements_are_never_read_and_retained_gets_are_ordered_and_live() {
    check(
        "let a=[0,1,2,3],log='';Object.defineProperty(a,'0',{get:()=>{log+='0';a[3]=9;return 0;}});Object.defineProperty(a,'1',{get:()=>{throw 7;}});Object.defineProperty(a,'2',{get:()=>{throw 8;}});let b=a.toSpliced(1,2,7);b.join()==='0,7,9' && log==='0'",
    );
    check(
        "let a=[0,1,2,3],log='';Object.defineProperty(a,'0',{get:()=>{log+='0';return 0;}});Object.defineProperty(a,'2',{get:()=>{log+='2';a.length=3;return 2;},configurable:true});let b=a.toSpliced(1,1);b.length===3 && b[0]===0 && b[1]===2 && b[2]===undefined && Object.hasOwn(b,'2') && log==='02'",
    );
    check(
        "let a=[0,1,2];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.toSpliced(0,Infinity,9).join()==='9'",
    );
    assert_eq!(
        Realm::default().eval(
            "let a=[0,1,2];Object.defineProperty(a,'2',{get:()=>{throw 7;}});a.toSpliced(1,1)"
        ),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn length_start_and_skip_conversions_precede_reads_and_use_captured_length() {
    check(
        "let log='',o={0:0,1:1,2:2};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 3;}};}});Object.defineProperty(o,'0',{get:()=>{log+='g';return 0;}});let a=Array.prototype.toSpliced.call(o,{valueOf:()=>{log+='s';return 1;}},{valueOf:()=>{log+='d';return 1;}},7);a.join()==='0,7,2' && log==='lnsdg'",
    );
    check(
        "let a=[0,1,2];let b=a.toSpliced({valueOf:()=>{a.length=1;return 1;}},{valueOf:()=>{a[2]=9;a[3]=7;return 1;}});b.length===2 && b.join()==='0,9' && a.length===4",
    );
    check(
        "let log='';[].toSpliced({valueOf:()=>{log+='s';return 0;}},{valueOf:()=>{log+='d';return 0;}});log==='sd'",
    );
    for source in [
        "[].toSpliced({valueOf:()=>{throw 7;}})",
        "[].toSpliced(0,{valueOf:()=>{throw 7;}})",
        "Array.prototype.toSpliced.call({length:4294967296},{valueOf:()=>{throw 7;}})",
        "Array.prototype.toSpliced.call({length:{valueOf:()=>{throw 7;}}},0,Infinity)",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0))),
            "{source}"
        );
    }
}

#[test]
fn generic_receivers_use_utf16_and_full_safe_integer_source_indices() {
    check("Array.prototype.toSpliced.call('💩x',1,1,'y').join()==='\\uD83D,y,x'");
    check("Array.prototype.toSpliced.call({0:1,1:2,length:'2.9'},1).join()==='1'");
    check(
        "Array.prototype.toSpliced.call(false).length===0 && Array.prototype.toSpliced.call(7,0,0,1)[0]===1",
    );
    check(
        "let o={9007199254740989:7,9007199254740990:8,length:Infinity};let a=Array.prototype.toSpliced.call(o,0,9007199254740989);a.length===2 && a.join()==='7,8'",
    );
    check("Array.prototype.toSpliced.call({length:4294967296},0,Infinity,7).join()==='7'");
}

#[test]
fn type_errors_and_output_length_bounds_precede_element_reads() {
    for source in [
        "Array.prototype.toSpliced.call(null)",
        "Array.prototype.toSpliced.call(undefined)",
        "Array.prototype.toSpliced.call({length:1n})",
        "[].toSpliced(0n)",
        "[].toSpliced(0,0n)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    for (length, args, error) in [
        ("4294967296", "", "RangeError"),
        ("4294967295", "0,0,1", "RangeError"),
        ("Infinity", "0,0,1", "TypeError"),
        ("9007199254740990", "0,0,1,2", "TypeError"),
    ] {
        check(&format!(
            "let n=0,caught=false,o={{length:{length}}};Object.defineProperty(o,'0',{{get:()=>{{n++;throw 7;}}}});try{{Array.prototype.toSpliced.call(o{comma}{args});}}catch(e){{caught=e instanceof {error};}}caught && n===0",
            comma = if args.is_empty() { "" } else { "," }
        ));
    }
}

#[test]
fn metadata_and_gc_retain_intrinsics_and_copied_values() {
    check(
        "let f=Array.prototype.toSpliced,d=Object.getOwnPropertyDescriptor(Array.prototype,'toSpliced');f.name==='toSpliced' && f.length===2 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.toSpliced"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm.eval("let f=Array.prototype.toSpliced;delete Array.prototype.toSpliced;let o={},a=[o],b=f.call(a,1,0,o);delete a[0]").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("b[0]===o && b[1]===o && b!==a && f.call(b).length===2"),
        Ok(Value::Boolean(true))
    );
}
