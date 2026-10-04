//! Copying Array methods use intrinsic arrays and never read replaced elements.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn copies_are_fresh_arrays_with_dense_elements_and_preserved_value_identity() {
    check(
        "let a=[1,2,3],b=a.toReversed();a!==b && Array.isArray(b) && b.join()==='3,2,1' && a.join()==='1,2,3'",
    );
    check(
        "let a=[1,2,3],b=a.with(1,7);a!==b && Array.isArray(b) && b.join()==='1,7,3' && a.join()==='1,2,3'",
    );
    check("let a=[];a.toReversed()!==a && a.toReversed().length===0");
    check("let o={},a=[o];a.toReversed()[0]===o && a.with(0,o)[0]===o && a.with(0,o)!==a");
    check(
        "let b=[,,].toReversed();b.length===2 && Object.hasOwn(b,'0') && Object.hasOwn(b,'1') && b[0]===undefined",
    );
    check(
        "let b=[,,,].with(1,7);b.length===3 && Object.hasOwn(b,'0') && Object.hasOwn(b,'2') && b[0]===undefined && b[1]===7",
    );
    check("let b=[1].with();Object.hasOwn(b,'0') && b[0]===undefined");
    check(
        "let b=[1].toReversed(),d=Object.getOwnPropertyDescriptor(b,'0');d.value===1 && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn intrinsic_prototype_and_own_element_creation_ignore_public_constructor_and_setters() {
    for call in ["toReversed()", "with(0,7)"] {
        check(&format!(
            "let a=Object.freeze([1,2]),p=Array.prototype;Array=function(){{throw 7;}};Object.getPrototypeOf(a.{call})===p"
        ));
        check(&format!(
            "let a=[1,2];Object.defineProperty(a,'constructor',{{get:()=>{{throw 7;}}}});Object.getPrototypeOf(a.{call})===Array.prototype"
        ));
        check(&format!(
            "let a=[1,2];Object.defineProperty(Array.prototype,'0',{{get:()=>{{throw 7;}},set:()=>{{throw 8;}}}});let b=a.{call};Object.hasOwn(b,'0') && b.length===2"
        ));
    }
    check("Array.prototype[1]=9;[, ,3].toReversed()[1]===9 && [,,3].with(0,7)[1]===9");
    check(
        "let a=[1,2];Object.setPrototypeOf(a,null);Object.getPrototypeOf(Array.prototype.toReversed.call(a))===Array.prototype",
    );
}

#[test]
fn with_coerces_relative_indices_and_never_reads_the_replaced_property() {
    for (index, expected) in [
        ("-1", "1,2,7"),
        ("-3", "7,2,3"),
        ("-0.9", "7,2,3"),
        ("NaN", "7,2,3"),
        ("undefined", "7,2,3"),
        ("'1.9'", "1,7,3"),
    ] {
        check(&format!("[1,2,3].with({index},7).join()==='{expected}'"));
    }
    check(
        "let a=[1,2,3];Object.defineProperty(a,'1',{get:()=>{throw 7;}});a.with(1,9).join()==='1,9,3'",
    );
    check(
        "let n=0,a=[1];Object.defineProperty(a,'0',{get:()=>{n++;throw 7;}});let o={valueOf:()=>{throw 8;}};a.with(0,o)[0]===o && n===0",
    );
    for source in [
        "[].with(0)",
        "[1].with(1)",
        "[1].with(-2)",
        "[1].with(Infinity)",
        "[1].with(-Infinity)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::RangeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    assert!(matches!(
        Realm::default().eval("[1].with(0n,7)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn gets_are_directional_and_live_but_output_length_is_snapshotted() {
    check(
        "let log='',a=[0,1,2];Object.defineProperty(a,'0',{get:()=>{log+='0';return 0;}});Object.defineProperty(a,'1',{get:()=>{log+='1';return 1;}});Object.defineProperty(a,'2',{get:()=>{log+='2';return 2;}});a.toReversed();log==='210'",
    );
    check(
        "let log='',a=[0,1,2];Object.defineProperty(a,'0',{get:()=>{log+='0';return 0;}});Object.defineProperty(a,'1',{get:()=>{throw 7;}});Object.defineProperty(a,'2',{get:()=>{log+='2';return 2;}});a.with(1,7);log==='02'",
    );
    check(
        "let a=[0,1,2,3];Object.defineProperty(a,'3',{get:()=>{a.length=0;return 3;},configurable:true});let b=a.toReversed();b.length===4 && b[0]===3 && b[1]===undefined && Object.hasOwn(b,'3')",
    );
    check(
        "let a=[0,1,2];Object.defineProperty(a,'0',{get:()=>{a.length=1;return 0;},configurable:true});let b=a.with(1,7);b.length===3 && b[0]===0 && b[1]===7 && b[2]===undefined",
    );
    check(
        "let a=[1,2];let b=a.with({valueOf:()=>{a[1]=9;a[2]=3;return 0;}},7);b.join()==='7,9' && b.length===2 && a.length===3",
    );
    check(
        "let a=[1,2];let b=a.with({valueOf:()=>{a.length=0;return 1;}},7);b.length===2 && b[0]===undefined && b[1]===7",
    );
}

#[test]
fn generic_receivers_copy_utf16_units_and_convert_length_before_index() {
    check(
        "let a=Array.prototype.toReversed.call('💩');a.length===2 && a[0]==='\\uDCA9' && a[1]==='\\uD83D'",
    );
    check("Array.prototype.with.call('💩x',1,'y').join()==='\\uD83D,y,x'");
    check(
        "let o={0:1,1:2,2:3,length:'2.9'};Array.prototype.toReversed.call(o).join()==='2,1' && Array.prototype.with.call(o,0,7).join()==='7,2'",
    );
    check(
        "Array.prototype.toReversed.call(false).length===0 && Array.prototype.toReversed.call(7).length===0",
    );
    check(
        "let log='',o={1:2};Object.defineProperty(o,'length',{get:()=>{log+='l';return {valueOf:()=>{log+='n';return 2;}};}});Object.defineProperty(o,'0',{get:()=>{log+='g';return 1;}});Array.prototype.with.call(o,{valueOf:()=>{log+='i';return 1;}},7).join()==='1,7' && log==='lnig'",
    );
}

#[test]
fn array_length_bounds_and_abrupt_conversions_precede_element_reads() {
    for method in ["toReversed", "with"] {
        for receiver in ["null", "undefined", "{length:1n}"] {
            assert!(matches!(
                Realm::default().eval(&format!("Array.prototype.{method}.call({receiver},0,7)")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        for length in ["4294967296", "Infinity"] {
            check(&format!(
                "let n=0,o={{length:{length}}};Object.defineProperty(o,'0',{{get:()=>{{n++;throw 7;}}}});Object.defineProperty(o,'4294967295',{{get:()=>{{n++;throw 8;}}}});try{{Array.prototype.{method}.call(o,0,9);}}catch(e){{if(!(e instanceof RangeError))throw e;}}n===0"
            ));
        }
        assert_eq!(
            Realm::default().eval(&format!(
                "Array.prototype.{method}.call({{length:{{valueOf:()=>{{throw 7;}}}}}},0,7)"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
    }
    assert_eq!(
        Realm::default()
            .eval("Array.prototype.with.call({length:4294967296},{valueOf:()=>{throw 7;}},0)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        Realm::default().eval("[].with({valueOf:()=>{throw 7;}},0)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    for source in [
        "let a=[1];Object.defineProperty(a,'0',{get:()=>{throw 7;}});a.toReversed()",
        "let a=[1,2];Object.defineProperty(a,'1',{get:()=>{throw 7;}});a.with(0,9)",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Err(Error::Thrown(Value::Number(7.0)))
        );
    }
}

#[test]
fn metadata_nonconstructibility_and_collection_are_standard() {
    for (method, length) in [("toReversed", 0), ("with", 2)] {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length==={length} && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Array.prototype.{method}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        let mut realm = Realm::default();
        realm.eval(&format!("let f=Array.prototype.{method};delete Array.prototype.{method};let o={{}},a=[o],b=f.call(a,0,o);delete a[0]")).unwrap();
        realm.collect(10_000).unwrap();
        assert_eq!(realm.eval("b[0]===o && b!==a"), Ok(Value::Boolean(true)));
    }
}
