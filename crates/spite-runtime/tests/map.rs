//! Map key identity, constructor ordering, live iteration, and intrinsic grouping.

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
fn keys_use_identity_and_same_value_zero_without_coercion() {
    check(
        "let a={},b={},x=Symbol('x'),y=Symbol('x'),m=new Map();m.set(a,1).set(b,2).set(x,3).set(y,4).set(1,5).set(1n,6).set('1',7).set(undefined,8).set(null,9).set(false,10).set(true,11);m.size===11 && m.get(a)===1 && m.get(b)===2 && m.get(x)===3 && m.get(y)===4 && m.get(1)===5 && m.get(1n)===6 && m.get('1')===7 && m.get(undefined)===8 && m.get(null)===9 && m.get(false)===10 && m.get(true)===11 && !m.has({}) && !m.has(Symbol('x'))",
    );
    check(
        "let m=new Map();m.set(NaN,1).set(0/0,2).set(-0,3).set(+0,4);let keys=[...m.keys()];m.size===2 && m.get(NaN)===2 && m.get(-0)===4 && Number.isNaN(keys[0]) && Object.is(keys[1],0) && m.delete(NaN) && !m.delete(NaN) && m.size===1",
    );
    check(
        "let key={toString(){throw 7;},valueOf(){throw 8;},get [Symbol.toPrimitive](){throw 9;}},m=new Map();m.set(key,undefined);m.has(key) && m.get(key)===undefined && !m.has({}) && m.delete(key)",
    );
    check(
        "let m=new Map();m.set(BigInt('18446744073709551616'),1);m.get(18446744073709551616n)===1 && !m.has(18446744073709551617n) && (m.set('\\ud800',2),m.get('\\ud800')===2)",
    );
}

#[test]
fn insertion_update_deletion_and_reinsertion_preserve_order() {
    check(
        "let m=new Map([[2,'a'],[1,'b'],[2,'c']]);m.set(2,'d');let first=JSON.stringify([...m])==='[[2,\"d\"],[1,\"b\"]]';m.delete(2);m.set(2,'e');first && JSON.stringify([...m])==='[[1,\"b\"],[2,\"e\"]]' && m.clear()===undefined && m.size===0 && m.get(1)===undefined",
    );
    check(
        "let m=new Map();Object.freeze(m);m.set(1,2);m.size===1 && m.delete(1) && m.size===0 && Object.isFrozen(m)",
    );
}

#[test]
fn constructor_checks_new_target_and_orders_prototype_adder_and_iterator() {
    type_error("Map()");
    check(
        "let trace='',p=Object.create(Map.prototype);Object.defineProperty(p,'set',{get(){trace+='s';return function(k,v){trace+='a';Map.prototype.set.call(this,k,v);};}});function C(){}C.prototype=p;let items={[Symbol.iterator](){trace+='i';let used=false;return {get next(){trace+='n';return function(){if(used)return {done:true};used=true;return {value:{get 0(){trace+='k';return 1;},get 1(){trace+='v';return 2;}}};};}};}};let m=Reflect.construct(Map,[items],C);trace==='sinkva' && Object.getPrototypeOf(m)===p && m.get(1)===2",
    );
    check(
        "let p=Object.create(Map.prototype);Object.defineProperty(p,'set',{get(){throw 7;}});function C(){}C.prototype=p;Reflect.construct(Map,[null],C).size===0 && Reflect.construct(Map,[undefined],C).size===0",
    );
    check(
        "let touched=false,p=Object.create(Map.prototype);p.set=null;function C(){}C.prototype=p;let items={get [Symbol.iterator](){touched=true;throw 7;}},caught=false;try{Reflect.construct(Map,[items],C);}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
    check(
        "function C(){}C.prototype=1;Object.getPrototypeOf(Reflect.construct(Map,[],C))===Map.prototype && new (Map.bind(null,[[1,2]]))().get(1)===2",
    );
    check(
        "let count=0,set=Map.prototype.set;Map.prototype.set=function(k,v){count++;if(count===1)Map.prototype.set=function(){throw 7;};return set.call(this,k,v);};let m=new Map([[1,2],[3,4]]);count===2 && m.size===2",
    );
}

#[test]
fn entry_failures_close_but_step_failures_do_not_and_throw_precedence_is_exact() {
    for entry in [
        "7",
        "{get 0(){throw marker;}}",
        "{0:1,get 1(){throw marker;}}",
    ] {
        let expected = if entry == "7" {
            "e instanceof TypeError"
        } else {
            "e===marker"
        };
        check(&format!(
            "let marker={{}},closed=0,i={{next(){{return {{value:{entry}}};}},return(){{closed++;throw 9;}}}},caught=false;try{{new Map({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught={expected};}}caught && closed===1"
        ));
    }
    for step in [
        "throw marker;",
        "return {get done(){throw marker;}};",
        "return {get value(){throw marker;}};",
    ] {
        check(&format!(
            "let marker={{}},closed=0,i={{next(){{{step}}},return(){{closed++;return {{}};}}}},caught=false;try{{new Map({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===marker;}}caught && closed===0"
        ));
    }
    check(
        "let marker={},closed=0;Map.prototype.set=function(){throw marker;};let i={next(){return {value:[1,2]};},return(){closed++;return 7;}},caught=false;try{new Map({[Symbol.iterator](){return i;}});}catch(e){caught=e===marker;}caught && closed===1",
    );
}

#[test]
fn brands_are_own_internal_slots_and_public_properties_do_not_control_storage() {
    for receiver in [
        "undefined",
        "null",
        "1",
        "1n",
        "'x'",
        "Symbol()",
        "{}",
        "Map.prototype",
        "Object.create(new Map())",
    ] {
        for method in [
            "get",
            "set",
            "has",
            "delete",
            "clear",
            "forEach",
            "getOrInsert",
            "getOrInsertComputed",
            "keys",
            "values",
            "entries",
        ] {
            type_error(&format!("Map.prototype.{method}.call({receiver},1,()=>2)"));
        }
        type_error(&format!(
            "Object.getOwnPropertyDescriptor(Map.prototype,'size').get.call({receiver})"
        ));
    }
    check(
        "let m=new Map(),set=m.set,get=m.get;Object.setPrototypeOf(m,null);Object.freeze(m);set.call(m,1,2)===m && get.call(m,1)===2",
    );
}

#[test]
fn iterators_observe_live_mutation_and_stay_completed() {
    check(
        "let m=new Map([[1,'a'],[2,'b']]),i=m.entries();let a=i.next();m.set(2,'B');m.delete(1);m.set(3,'c');let b=i.next();m.clear();m.set(4,'d');let c=i.next(),end=i.next();m.set(5,'e');a.value[0]===1 && a.value[1]==='a' && b.value[0]===2 && b.value[1]==='B' && c.value[0]===4 && c.value[1]==='d' && end.done && i.next().done && Reflect.ownKeys(a).join(',')==='value,done' && Object.getPrototypeOf(a.value)===Array.prototype",
    );
    check(
        "let m=new Map([[1,2]]),i=m.keys();m.clear();m.set(3,4);i.next().value===3 && i.next().done && m[Symbol.iterator]===m.entries && Iterator.from(m.values()).toArray()[0]===4",
    );
    check(
        "let m=new Map([[1,2]]),i=m.keys();let first=i.next().value;m.delete(1);m.set(1,3);first===1 && i.next().value===1 && i.next().done",
    );
    for receiver in [
        "{}",
        "Object.create(new Map().keys())",
        "[].keys()",
        "Iterator.prototype",
        "Map.prototype",
    ] {
        type_error(&format!("new Map().keys().next.call({receiver})"));
    }
}

#[test]
fn foreach_has_exact_arguments_this_and_live_order_with_nested_calls() {
    check(
        "let m=new Map([[1,'a'],[2,'b'],[3,'c']]),seen=[],context={};let result=m.forEach(function(v,k,o){'use strict';if(this!==context || o!==m || arguments.length!==3)throw 7;seen.push(k+v);if(k===1){m.delete(2);m.set(3,'C');m.set(4,'d');}if(k===3){m.delete(1);m.set(1,'A');}},context);result===undefined && seen.join(',')==='1a,3C,4d,1A'",
    );
    check(
        "let m=new Map([[1,2]]),seen=[];m.forEach(function(v,k){'use strict';if(this!==undefined)throw 7;seen.push(k);if(seen.length===1){m.clear();m.set(3,4);}});seen.join(',')==='1,3'",
    );
    check(
        "let m=new Map([[1,2],[3,4]]),count=0,marker={},caught=false;try{m.forEach(()=>{count++;throw marker;});}catch(e){caught=e===marker;}caught && count===1",
    );
    check(
        "let m=new Map([[1,2],[3,4]]),seen=[];m.forEach((v,k)=>{m.forEach((w,j)=>seen.push(k,j));});seen.join(',')==='1,1,1,3,3,1,3,3'",
    );
}

#[test]
fn get_or_insert_computed_validates_before_lookup_and_rechecks_after_mutation() {
    check(
        "let m=new Map([[1,undefined]]),called=false;m.getOrInsert(1,2)===undefined && m.getOrInsertComputed(1,()=>{called=true;return 3;})===undefined && !called && m.getOrInsert(2,4)===4 && m.get(2)===4",
    );
    type_error("new Map([[1,2]]).getOrInsertComputed(1,null)");
    check(
        "let m=new Map(),key,argc,receiver;let value=m.getOrInsertComputed(-0,function(k){'use strict';key=k;argc=arguments.length;receiver=this;m.set(k,3);m.set(2,4);return 5;});value===5 && m.get(0)===5 && [...m.keys()].join(',')==='0,2' && Object.is(key,0) && argc===1 && receiver===undefined",
    );
    check(
        "let m=new Map([[1,2]]);m.getOrInsertComputed(3,()=>{m.clear();m.set(4,5);m.set(3,6);return 7;})===7 && JSON.stringify([...m])==='[[4,5],[3,7]]'",
    );
    check(
        "let m=new Map(),marker={},caught=false;try{m.getOrInsertComputed(1,()=>{m.set(2,3);throw marker;});}catch(e){caught=e===marker;}caught && !m.has(1) && m.get(2)===3",
    );
}

#[test]
fn grouping_uses_collection_keys_and_intrinsic_map_and_arrays() {
    check(
        "let key={},a=Symbol('x'),b=Symbol('x'),items=[key,key,a,b,NaN,NaN,-0,+0,1n,1,'1'];let m=Map.groupBy(items,function(v,i){'use strict';if(this!==undefined || arguments.length!==2 || items[i]!==v && !Number.isNaN(v))throw 7;return v;});m.size===8 && m.get(key).length===2 && m.get(a).length===1 && m.get(b).length===1 && m.get(NaN).length===2 && m.get(0).length===2 && Object.is([...m.keys()][4],0) && m.get(1n)[0]===1n && m.get(1)[0]===1 && m.get('1')[0]==='1'",
    );
    check(
        "let groupBy=Map.groupBy,M=Map,A=Array;Map.prototype.set=function(){throw 7;};globalThis.Map=function(){throw 8;};globalThis.Array=function(){throw 9;};let m=groupBy.call(null,[1,2,3],v=>v%2);Object.getPrototypeOf(m)===M.prototype && Object.getPrototypeOf(m.get(1))===A.prototype && m.get(1).join(',')==='1,3' && [...m.keys()].join(',')==='1,0'",
    );
    check(
        "let touched=false,items={get [Symbol.iterator](){touched=true;throw 7;}},caught=false;try{Map.groupBy(items,null);}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
    check(
        "let marker={},closed=0,i={next(){return {value:7};},return(){closed++;throw 8;}},caught=false;try{Map.groupBy({[Symbol.iterator](){return i;}},()=>{throw marker;});}catch(e){caught=e===marker;}caught && closed===1",
    );
    check(
        "let marker={},closed=0,i={next(){return {get value(){throw marker;}};},return(){closed++;return {};}},caught=false;try{Map.groupBy({[Symbol.iterator](){return i;}},v=>v);}catch(e){caught=e===marker;}caught && closed===0",
    );
}

#[test]
fn full_metadata_and_species_and_iterator_prototypes_are_standard() {
    check(
        "Map.name==='Map' && Map.length===0 && Object.getPrototypeOf(Map)===Function.prototype && Object.getPrototypeOf(Map.prototype)===Object.prototype && Map.prototype.constructor===Map && Object.keys(Map.prototype).length===0 && Object.getOwnPropertyNames(Map.prototype).sort().join(',')==='clear,constructor,delete,entries,forEach,get,getOrInsert,getOrInsertComputed,has,keys,set,size,values' && Object.getOwnPropertySymbols(Map.prototype).length===2 && Object.getOwnPropertyNames(Map).sort().join(',')==='groupBy,length,name,prototype'",
    );
    check(
        "let size=Object.getOwnPropertyDescriptor(Map.prototype,'size'),species=Object.getOwnPropertyDescriptor(Map,Symbol.species),x={};size.get.name==='get size' && size.get.length===0 && size.set===undefined && !size.enumerable && size.configurable && species.get.name==='get [Symbol.species]' && species.get.call(x)===x && species.get.call(null)===null && species.get.call(7)===7 && Map[Symbol.species]===Map",
    );
    check(
        "let i=new Map().entries(),p=Object.getPrototypeOf(i),d=Object.getOwnPropertyDescriptor(p,'next');Object.getPrototypeOf(p)===Iterator.prototype && Reflect.ownKeys(i).length===0 && Object.getOwnPropertyNames(p).join(',')==='next' && d.value.name==='next' && d.value.length===0 && d.writable && !d.enumerable && d.configurable && Object.prototype.toString.call(i)==='[object Map Iterator]' && Object.prototype.toString.call(new Map())==='[object Map]'",
    );
    for (name, length) in [
        ("clear", 0),
        ("delete", 1),
        ("entries", 0),
        ("forEach", 1),
        ("get", 1),
        ("getOrInsert", 2),
        ("getOrInsertComputed", 2),
        ("has", 1),
        ("keys", 0),
        ("set", 2),
        ("values", 0),
    ] {
        check(&format!(
            "let f=Map.prototype.{name},d=Object.getOwnPropertyDescriptor(Map.prototype,'{name}');f.name==='{name}' && f.length==={length} && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(&format!("new Map.prototype.{name}()"));
    }
    type_error("new Map.groupBy()");
}

#[test]
fn maps_and_paused_iterators_keep_keys_and_values_alive_after_public_deletion() {
    let mut realm = Realm::default();
    realm.eval("let m=new Map(),key={},value={x:7};m.set(key,value);let i=m.entries(),get=m.get;delete globalThis.Map;delete m.get;key=null;value=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let entry=i.next().value;get.call(m,entry[0])===entry[1] && entry[1].x===7"),
        Ok(Value::Boolean(true))
    );
    realm.eval("m=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("i.next().done"), Ok(Value::Boolean(true)));
}

#[test]
fn default_large_maps_and_opted_in_abort_keep_language_and_host_errors_separate() {
    check(
        "let m=new Map();for(let i=0;i<20000;i++)m.set(i,i*2);let ok=m.size===20000;for(let i=0;i<20000;i++)ok=ok && m.get(i)===i*2;ok",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let flag=0,m=new Map([[1,2]]);").unwrap();
    assert!(matches!(
        realm.eval(
            "try{m.forEach((v,k)=>{m.delete(k);m.set(k,v);});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
