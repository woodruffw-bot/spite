//! Set identity, live cursors, set-like records, and combination mutation order.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
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

#[test]
fn canonical_values_preserve_primitive_types_object_and_symbol_identity() {
    check(
        "let a={},b={},x=Symbol('x'),y=Symbol('x'),s=new Set([a,b,x,y,1,1n,'1',undefined,null,false,true,NaN,0/0,-0,+0]);s.size===13 && s.has(a) && s.has(b) && s.has(x) && s.has(y) && s.has(1) && s.has(1n) && s.has('1') && s.has(undefined) && s.has(null) && s.has(false) && s.has(true) && s.has(NaN) && s.has(0) && !s.has({}) && !s.has(Symbol('x')) && Object.is([...s][12],0)",
    );
    check(
        "let key={toString(){throw 7;},valueOf(){throw 8;},get [Symbol.toPrimitive](){throw 9;}},s=new Set();s.add(key)===s && s.add(key)===s && s.size===1 && s.has(key) && s.delete(key) && !s.delete(key)",
    );
    check(
        "let s=new Set([18446744073709551616n]);s.has(BigInt('18446744073709551616')) && !s.has(18446744073709551617n) && (s.add('\\ud800'),s.has('\\ud800'))",
    );
}

#[test]
fn updates_delete_reinsert_clear_and_frozen_instances_have_ordered_storage() {
    check(
        "let s=new Set([2,1,2]);s.add(2);let first=[...s].join(',')==='2,1';s.delete(2);s.add(2);first && [...s].join(',')==='1,2' && s.clear()===undefined && s.size===0 && !s.has(1)",
    );
    check(
        "let s=new Set();Object.freeze(s);s.add(1);s.size===1 && s.delete(1) && Object.isFrozen(s)",
    );
}

#[test]
fn constructor_orders_adder_before_iterator_and_caches_methods() {
    type_error("Set()");
    check(
        "let trace='',p=Object.create(Set.prototype);Object.defineProperty(p,'add',{get(){trace+='a';return function(v){if(arguments.length!==1)throw 7;trace+='c';Set.prototype.add.call(this,v);};}});function C(){}C.prototype=p;let items={[Symbol.iterator](){trace+='i';let n=0;return {get next(){trace+='n';return function(){return n++===0?{value:1}:{done:true};};}};}};let s=Reflect.construct(Set,[items],C);trace==='ainc' && Object.getPrototypeOf(s)===p && s.has(1)",
    );
    check(
        "let p=Object.create(Set.prototype);Object.defineProperty(p,'add',{get(){throw 7;}});function C(){}C.prototype=p;Reflect.construct(Set,[undefined],C).size===0 && Reflect.construct(Set,[null],C).size===0",
    );
    check(
        "let touched=false,p=Object.create(Set.prototype);p.add=null;function C(){}C.prototype=p;let items={get [Symbol.iterator](){touched=true;throw 7;}},caught=false;try{Reflect.construct(Set,[items],C);}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
    check(
        "let calls=0,add=Set.prototype.add;Set.prototype.add=function(v){calls++;Set.prototype.add=null;return add.call(this,v);};let s=new Set([1,2]);calls===2 && s.size===2",
    );
    check(
        "function C(){}C.prototype=1;Object.getPrototypeOf(Reflect.construct(Set,[],C))===Set.prototype && new (Set.bind(null,[1,2]))().size===2",
    );
}

#[test]
fn constructor_closes_adder_throws_and_preserves_step_error_precedence() {
    check(
        "let marker={},closed=0;Set.prototype.add=function(){throw marker;};let i={next(){return {value:1};},return(){closed++;throw 7;}},caught=false;try{new Set({[Symbol.iterator](){return i;}});}catch(e){caught=e===marker;}caught && closed===1",
    );
    for step in [
        "throw marker;",
        "return {get done(){throw marker;}};",
        "return {get value(){throw marker;}};",
    ] {
        check(&format!(
            "let marker={{}},closed=0,i={{next(){{{step}}},return(){{closed++;return {{}};}}}},caught=false;try{{new Set({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===marker;}}caught && closed===0"
        ));
    }
}

#[test]
fn own_set_and_iterator_brands_reject_maps_and_inherited_slots() {
    for receiver in [
        "undefined",
        "null",
        "1",
        "1n",
        "'x'",
        "Symbol()",
        "{}",
        "Set.prototype",
        "new Map()",
        "Object.create(new Set())",
    ] {
        for method in [
            "add",
            "has",
            "delete",
            "clear",
            "forEach",
            "values",
            "entries",
            "difference",
            "intersection",
            "union",
            "symmetricDifference",
            "isDisjointFrom",
            "isSubsetOf",
            "isSupersetOf",
        ] {
            type_error(&format!(
                "Set.prototype.{method}.call({receiver},new Set())"
            ));
        }
        type_error(&format!(
            "Object.getOwnPropertyDescriptor(Set.prototype,'size').get.call({receiver})"
        ));
    }
    for receiver in [
        "{}",
        "new Set()",
        "Object.create(new Set().values())",
        "new Map().values()",
        "[].values()",
    ] {
        type_error(&format!("new Set().values().next.call({receiver})"));
    }
    check(
        "let s=new Set(),add=s.add,has=s.has;Object.setPrototypeOf(s,null);Object.freeze(s);add.call(s,1)===s && has.call(s,1)",
    );
}

#[test]
fn live_iterators_and_callbacks_observe_appends_and_reinsertions() {
    check(
        "let s=new Set([1,2]),i=s.values();let a=i.next();s.delete(2);s.add(3);let b=i.next();s.clear();s.add(4);let c=i.next(),end=i.next();s.add(5);a.value===1 && b.value===3 && c.value===4 && end.done && i.next().done && s.keys===s.values && s[Symbol.iterator]===s.values",
    );
    check(
        "let s=new Set([1]),i=s.entries(),a=i.next();s.delete(1);s.add(1);a.value[0]===1 && a.value[1]===1 && Object.getPrototypeOf(a.value)===Array.prototype && i.next().value[0]===1 && i.next().done",
    );
    check(
        "let s=new Set([1,2,3]),seen=[],context={};let result=s.forEach(function(v,k,o){'use strict';if(this!==context || v!==k || o!==s || arguments.length!==3)throw 7;seen.push(v);if(v===1 && seen.length===1){s.delete(2);s.delete(1);s.add(4);s.add(1);}},context);result===undefined && seen.join(',')==='1,3,4,1'",
    );
    check(
        "let s=new Set([1]),seen=[];s.forEach(function(v){'use strict';if(this!==undefined)throw 7;seen.push(v);if(v===1){s.clear();s.add(2);}});seen.join(',')==='1,2'",
    );
}

#[test]
fn all_combination_and_predicate_results_are_ordered_and_deduplicated() {
    check(
        "let a=new Set([3,1,2]),b=new Set([2,4,1]);[...a.union(b)].join(',')==='3,1,2,4' && [...a.intersection(b)].join(',')==='1,2' && [...a.difference(b)].join(',')==='3' && [...a.symmetricDifference(b)].join(',')==='3,4' && !a.isSubsetOf(b) && !a.isSupersetOf(b) && !a.isDisjointFrom(b) && a.isSupersetOf(new Set([2])) && new Set([2]).isSubsetOf(a) && a.isDisjointFrom(new Set([4]))",
    );
    check(
        "let a=new Set([3,1,2]),b=new Map([[2,9],[1,8]]);[...a.intersection(b)].join(',')==='2,1' && [...a.difference(b)].join(',')==='3' && a.isSupersetOf(b)",
    );
    check(
        "let a=new Set([NaN,-0,1n]),b=new Set([NaN,+0,1]);[...a.intersection(b)].length===2 && Object.is([...a.intersection(b)][1],0) && a.union(b).size===4 && a.symmetricDifference(b).size===2",
    );
    check(
        "let other={size:0,has(){throw 7;},keys(){return [2,2,3,3].values();}},s=new Set([1,2]);[...s.union(other)].join(',')==='1,2,3' && [...s.intersection(other)].join(',')==='2' && [...s.symmetricDifference(other)].join(',')==='1,3'",
    );
}

#[test]
fn set_like_records_convert_size_and_read_has_then_keys_before_branching() {
    check(
        "let log='',other={get size(){log+='s';return {valueOf(){log+='n';return 0;}};},get has(){log+='h';return function(){throw 7;};},get keys(){log+='k';return function(){log+='i';return [2].values();};},get [Symbol.iterator](){throw 8;}};let r=new Set([1]).union(other);log==='snhki' && [...r].join(',')==='1,2'",
    );
    for method in ["isSubsetOf", "isSupersetOf"] {
        let size = if method == "isSubsetOf" { "0" } else { "2" };
        check(&format!(
            "let log='',other={{get size(){{log+='s';return {size};}},get has(){{log+='h';return ()=>false;}},get keys(){{log+='k';return ()=>{{throw 7;}};}}}};!new Set([1]).{method}(other) && log==='shk'"
        ));
    }
    for size in ["undefined", "NaN", "1n", "Symbol()"] {
        type_error(&format!(
            "new Set().union({{size:{size},has(){{}},keys(){{}}}})"
        ));
    }
    for size in ["-1", "-Infinity"] {
        check(&format!(
            "let touched=false,caught=false;try{{new Set().union({{size:{size},get has(){{touched=true;throw 7;}}}});}}catch(e){{caught=e instanceof RangeError;}}caught && !touched"
        ));
    }
    check(
        "let s=new Set([1]),other={size:-0.5,has(){return false;},keys(){return [].values();}};s.union(other).size===1 && !s.isSubsetOf(other)",
    );
    check(
        "let s=new Set([1]),other={size:Infinity,has(){return true;},keys(){throw 7;}};s.isSubsetOf(other) && s.intersection(other).size===1 && s.difference(other).size===0",
    );
    for other in [
        "null",
        "1",
        "'x'",
        "[]",
        "{size:0,has:1,keys(){}}",
        "{size:0,has(){},keys:1}",
    ] {
        type_error(&format!("new Set().union({other})"));
    }
}

#[test]
fn difference_uses_snapshot_while_intersection_and_predicates_resume_live_data() {
    check(
        "let s=new Set([1,2]),seen=[],other={size:100,has(v){seen.push(v);if(v===1){s.delete(2);s.add(3);}return false;},keys(){throw 7;}};[...s.difference(other)].join(',')==='1,2' && seen.join(',')==='1,2' && [...s].join(',')==='1,3'",
    );
    for method in ["intersection", "isSubsetOf", "isDisjointFrom"] {
        let returns = if method == "isDisjointFrom" {
            "false"
        } else {
            "true"
        };
        let result = if method == "intersection" {
            "[...result].join(',')==='1,3'"
        } else {
            "result===true"
        };
        check(&format!(
            "let s=new Set([1,2]),seen=[],other={{size:100,has(v){{seen.push(v);if(v===1){{s.delete(2);s.add(3);}}return {returns};}},keys(){{throw 7;}}}};let result=s.{method}(other);{result} && seen.join(',')==='1,3'"
        ));
    }
    check(
        "let s=new Set([1,2]),seen=[],other={size:100,has(v){seen.push(v);if(seen.length===1){s.delete(1);s.add(1);}return true;},keys(){throw 7;}};[...s.intersection(other)].join(',')==='1,2' && seen.join(',')==='1,2,1'",
    );
}

#[test]
fn union_and_symmetric_snapshots_follow_iterator_and_next_acquisition() {
    for method in ["union", "symmetricDifference"] {
        check(&format!(
            "let s=new Set([1]),other={{size:0,has(){{throw 7;}},keys(){{s.add(2);let used=false;return {{get next(){{s.add(3);return function(){{s.add(4);if(used)return {{done:true}};used=true;return {{value:5}};}};}}}};}}}};[...s.{method}(other)].join(',')==='1,2,3,5' && [...s].join(',')==='1,2,3,4'"
        ));
    }
    check(
        "let s=new Set([1,2]),n=0,other={size:0,has(){throw 7;},keys(){return {next(){if(n++===0){s.delete(2);return {value:2};}return {done:true};}};}};[...s.symmetricDifference(other)].join(',')==='1,2'",
    );
    check(
        "let s=new Set([1]),n=0,other={size:0,has(){throw 7;},keys(){return {next(){if(n===2)return {done:true};n++;if(n===2)s.add(2);return {value:2};}};}};[...s.symmetricDifference(other)].join(',')==='1'",
    );
}

#[test]
fn predicates_close_short_circuit_iterators_and_propagate_normal_close_errors() {
    for method in ["isDisjointFrom", "isSupersetOf"] {
        let value = if method == "isDisjointFrom" { "1" } else { "2" };
        check(&format!(
            "let closed=0,calls=0,i={{next(){{calls++;return {{value:{value}}};}},return(){{'use strict';if(this!==i || arguments.length!==0)throw 7;closed++;return {{}};}}}},other={{size:0,has(){{throw 8;}},keys(){{return i;}}}};!new Set([1]).{method}(other) && closed===1 && calls===1"
        ));
        check(&format!(
            "let marker={{}},caught=false,other={{size:0,has(){{}},keys(){{return {{next(){{return {{value:{value}}};}},return(){{throw marker;}}}};}}}};try{{new Set([1]).{method}(other);}}catch(e){{caught=e===marker;}}caught"
        ));
        type_error(&format!(
            "new Set([1]).{method}({{size:0,has(){{}},keys(){{return {{next(){{return {{value:{value}}};}},return(){{return 1;}}}};}}}})"
        ));
    }
}

#[test]
fn combination_step_failures_do_not_close_and_getter_errors_keep_identity() {
    for method in [
        "union",
        "symmetricDifference",
        "intersection",
        "difference",
        "isDisjointFrom",
        "isSupersetOf",
    ] {
        for step in [
            "throw marker;",
            "return {get done(){throw marker;}};",
            "return {get value(){throw marker;}};",
        ] {
            check(&format!(
                "let marker={{}},closed=0,other={{size:0,has(){{}},keys(){{return {{next(){{{step}}},return(){{closed++;return {{}};}}}};}}}},caught=false;try{{new Set([1]).{method}(other);}}catch(e){{caught=e===marker;}}caught && closed===0"
            ));
        }
    }
    for property in ["size", "has", "keys"] {
        check(&format!(
            "let marker={{}},other={{size:0,has(){{}},keys(){{}}}},caught=false;Object.defineProperty(other,'{property}',{{get(){{throw marker;}}}});try{{new Set().union(other);}}catch(e){{caught=e===marker;}}caught"
        ));
    }
}

#[test]
fn combination_results_bypass_public_constructors_add_and_species() {
    for method in ["union", "symmetricDifference", "intersection", "difference"] {
        check(&format!(
            "let S=Set,s=new S([1,2]),other=new S([2,3]);Object.defineProperty(s,'constructor',{{get(){{throw 7;}}}});S.prototype.add=function(){{throw 8;}};globalThis.Set=function(){{throw 9;}};let result=s.{method}(other);Object.getPrototypeOf(result)===S.prototype && result!==s && result!==other && [...s].join(',')==='1,2'"
        ));
    }
}

#[test]
fn full_metadata_aliases_species_and_iterator_prototypes_are_standard() {
    check(
        "Set.name==='Set' && Set.length===0 && Object.getPrototypeOf(Set)===Function.prototype && Object.getPrototypeOf(Set.prototype)===Object.prototype && Set.prototype.constructor===Set && Set.prototype.keys===Set.prototype.values && Set.prototype[Symbol.iterator]===Set.prototype.values && Object.keys(Set.prototype).length===0 && Object.getOwnPropertyNames(Set.prototype).sort().join(',')==='add,clear,constructor,delete,difference,entries,forEach,has,intersection,isDisjointFrom,isSubsetOf,isSupersetOf,keys,size,symmetricDifference,union,values' && Object.getOwnPropertySymbols(Set.prototype).length===2 && Object.getOwnPropertyNames(Set).sort().join(',')==='length,name,prototype'",
    );
    check(
        "let size=Object.getOwnPropertyDescriptor(Set.prototype,'size'),species=Object.getOwnPropertyDescriptor(Set,Symbol.species),x={};size.get.name==='get size' && size.get.length===0 && size.set===undefined && !size.enumerable && size.configurable && species.get.name==='get [Symbol.species]' && species.get.call(x)===x && species.get.call(null)===null && species.get.call(7)===7 && Set[Symbol.species]===Set",
    );
    check(
        "let i=new Set().values(),p=Object.getPrototypeOf(i);Object.getPrototypeOf(p)===Iterator.prototype && Reflect.ownKeys(i).length===0 && Object.getOwnPropertyNames(p).join(',')==='next' && p.next.name==='next' && p.next.length===0 && Object.prototype.toString.call(i)==='[object Set Iterator]' && Object.prototype.toString.call(new Set())==='[object Set]'",
    );
    for (name, length) in [
        ("add", 1),
        ("clear", 0),
        ("delete", 1),
        ("entries", 0),
        ("forEach", 1),
        ("has", 1),
        ("values", 0),
        ("difference", 1),
        ("intersection", 1),
        ("union", 1),
        ("symmetricDifference", 1),
        ("isDisjointFrom", 1),
        ("isSubsetOf", 1),
        ("isSupersetOf", 1),
    ] {
        check(&format!(
            "let f=Set.prototype.{name},d=Object.getOwnPropertyDescriptor(Set.prototype,'{name}');f.name==='{name}' && f.length==={length} && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(&format!("new Set.prototype.{name}()"));
    }
}

#[test]
fn live_set_values_and_saved_methods_survive_collection_and_default_large_inputs() {
    let mut realm = Realm::default();
    realm
        .eval("let s=new Set([{x:7}]),i=s.values(),has=s.has;delete globalThis.Set;s=null;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("i.next().value.x===7 && i.next().done"),
        Ok(Value::Boolean(true))
    );
    check(
        "let s=new Set();for(let i=0;i<20000;i++)s.add(i);let ok=s.size===20000;for(let i=0;i<20000;i++)ok=ok && s.has(i);ok",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let flag=0,s=new Set([1]);").unwrap();
    assert!(matches!(
        realm.eval("try{s.forEach(v=>{s.delete(v);s.add(v);});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
