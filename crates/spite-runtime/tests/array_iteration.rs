//! Array iterators observe live lengths and ordered state mutations.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn keys_values_and_entries_are_fresh_iterators_with_dense_results() {
    check(
        "let a=[10,,30],k=a.keys(),v=a.values(),e=a.entries();k!==a.keys() && k.next().value===0 && k.next().value===1 && v.next().value===10 && v.next().value===undefined && v.next().value===30 && v.next().done",
    );
    check(
        "let i=[,7].entries(),a=i.next(),b=i.next();!a.done && Array.isArray(a.value) && a.value.length===2 && Object.hasOwn(a.value,'0') && Object.hasOwn(a.value,'1') && a.value[0]===0 && a.value[1]===undefined && b.value[0]===1 && b.value[1]===7 && i.next().done",
    );
    check(
        "let i=[].values(),a=i.next(),b=i.next();a!==b && a.done===true && a.value===undefined && b.done===true && b.value===undefined && Object.getPrototypeOf(a)===Object.prototype",
    );
    check(
        "let i=[1].values(),r=i.next(),v=Object.getOwnPropertyDescriptor(r,'value'),d=Object.getOwnPropertyDescriptor(r,'done');!r.done && r.value===1 && v.writable && v.enumerable && v.configurable && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn construction_is_lazy_and_each_next_reads_live_length() {
    check(
        "let count=0,o={0:'x',1:'y'},length=1;Object.defineProperty(o,'length',{get:()=>{count++;return length;}});let i=Array.prototype.values.call(o);count===0 && i.next().value==='x' && count===1 && (length=2,i.next().value==='y') && count===2 && i.next().done && count===3 && (length=100,i.next().done) && count===3",
    );
    check(
        "let a=[1],i=a.values();a.push(2);i.next().value===1 && i.next().value===2 && (a.length=0,i.next().done) && (a.push(3),i.next().done)",
    );
    check(
        "let o={0:1,1:2,length:2},i=Array.prototype.values.call(o);i.next().value===1 && (o[1]=7,i.next().value===7)",
    );
    check(
        "let o={length:1};Object.defineProperty(o,'0',{get:()=>{throw 7;}});let k=Array.prototype.keys.call(o);k.next().value===0 && k.next().done",
    );
}

#[test]
fn array_like_iteration_boxes_primitives_and_reads_inherited_elements() {
    check(
        "let o=Object.create({0:'p'});o.length=1;Array.prototype.values.call(o).next().value==='p'",
    );
    check(
        "let i=Array.prototype.values.call('\\uD83D\\uDE00');i.next().value.charCodeAt(0)===0xD83D && i.next().value.charCodeAt(0)===0xDE00 && i.next().done",
    );
    check(
        "Array.prototype.values.call(3).next().done && Array.prototype.keys.call(false).next().done",
    );
    for method in ["keys", "values", "entries"] {
        for receiver in ["undefined", "null"] {
            assert!(matches!(
                Realm::default().eval(&format!("Array.prototype.{method}.call({receiver})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
}

#[test]
fn abrupt_length_conversion_retries_but_abrupt_elements_advance() {
    check(
        "let calls=0,o={0:1,length:{valueOf:()=>{if(calls++===0){throw 7;}return 1;}}},i=Array.prototype.values.call(o),caught=false;try{i.next();}catch(e){caught=e===7;}caught && i.next().value===1 && i.next().done",
    );
    for method in ["values", "entries"] {
        let result = if method == "entries" {
            "r.value[0]===1 && r.value[1]===2"
        } else {
            "r.value===2"
        };
        check(&format!(
            "let o={{1:2,length:2}},i=Array.prototype.{method}.call(o),caught=false;Object.defineProperty(o,'0',{{get:()=>{{throw 7;}}}});try{{i.next();}}catch(e){{caught=e===7;}}let r=i.next();caught && !r.done && {result} && i.next().done"
        ));
    }
}

#[test]
fn reentrant_length_getters_use_the_snapshot_index_and_preserve_inner_completion() {
    check(
        "let calls=0,inner,o={0:'a',1:'b'},i;Object.defineProperty(o,'length',{get:()=>{if(calls++===0){inner=i.next();}return 2;}});i=Array.prototype.values.call(o);let outer=i.next();inner.value==='a' && outer.value==='a' && i.next().value==='b' && i.next().done",
    );
    check(
        "let calls=0,inner,o={0:'a'},i;Object.defineProperty(o,'length',{get:()=>{if(calls++===0){inner=i.next();return 1;}return 0;}});i=Array.prototype.values.call(o);let outer=i.next();inner.done && !outer.done && outer.value==='a' && i.next().done && calls===2",
    );
}

#[test]
fn reentrant_index_getters_observe_the_advanced_index() {
    check(
        "let inner,o={1:'b',length:2},i=Array.prototype.values.call(o);Object.defineProperty(o,'0',{get:()=>{inner=i.next();return 'a';}});let outer=i.next();outer.value==='a' && inner.value==='b' && i.next().done",
    );
    check(
        "let o={length:2},i=Array.prototype.keys.call(o);Object.defineProperty(o,'0',{get:()=>{throw 7;}});i.next({valueOf:()=>{throw 8;}}).value===0 && i.next().value===1",
    );
}

#[test]
fn entries_and_results_use_intrinsic_objects_and_bypass_inherited_setters() {
    check(
        "let i=[7].entries(),a=Array.prototype,o=Object.prototype,getProto=Object.getPrototypeOf;Array=function(){throw 7;};Object=function(){throw 8;};let r=i.next();r.value[0]===0 && r.value[1]===7 && getProto(r)===o && getProto(r.value)===a",
    );
    check(
        "let i=[7].entries();Object.defineProperty(Array.prototype,'0',{set:()=>{throw 7;}});Object.defineProperty(Object.prototype,'value',{set:()=>{throw 8;}});let r=i.next();Object.hasOwn(r,'value') && Object.hasOwn(r.value,'0') && r.value[0]===0 && r.value[1]===7",
    );
}

#[test]
fn next_is_branded_but_does_not_require_its_original_prototype() {
    check(
        "let i=[1].values(),next=i.next;Object.setPrototypeOf(i,null);next.call(i).value===1 && next.call(i).done",
    );
    for value in [
        "undefined",
        "null",
        "1",
        "'x'",
        "{}",
        "[]",
        "Object.getPrototypeOf(i)",
        "Object.create(i)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("let i=[1].values();i.next.call({value})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{value}"
        );
    }
    for expression in [
        "new Array.prototype.values",
        "new Array.prototype.keys",
        "new Array.prototype.entries",
        "let i=[].values();new i.next",
    ] {
        assert!(
            matches!(
                Realm::default().eval(expression),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{expression}"
        );
    }
}
