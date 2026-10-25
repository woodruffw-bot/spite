//! Edition-17 global metadata and ordinary global own-key behavior (19, 26–28).

use crate::common::REALM_ENTRIES;
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn deferred_intrinsic_metadata_graph_matches_edition_17() {
    let result = Realm::default().eval(r#"
        let rows=[],graph=[];
        function flags(d){return (d.writable?'w':'-')+(d.enumerable?'e':'-')+(d.configurable?'c':'-');}
        function keys(o){return Object.getOwnPropertyNames(o).sort().concat(Object.getOwnPropertySymbols(o).map(s=>'@@'+s.description)).join(',');}
        function constructor(f){try{Reflect.construct(function(){},[],f);return true;}catch(e){if(!(e instanceof TypeError))throw e;return false;}}
        function member(label,o,key){
            let d=Object.getOwnPropertyDescriptor(o,key),f=d.get===undefined?d.value:d.get;
            graph.push(f);
            rows.push(label+' '+f.name+'/'+f.length+' '+(d.get===undefined?flags(d):'accessor:'+flags(d)+':setter='+('set' in d && d.set!==undefined))+' meta='+flags(Object.getOwnPropertyDescriptor(f,'name'))+'/'+flags(Object.getOwnPropertyDescriptor(f,'length'))+' parent='+(Object.getPrototypeOf(f)===Function.prototype)+' keys='+keys(f)+' construct='+constructor(f));
        }
        for(let name of ['Promise','Proxy','SharedArrayBuffer','WeakRef','FinalizationRegistry']){
            let C=this[name];graph.push(C);
            rows.push(name+' '+C.name+'/'+C.length+' global='+flags(Object.getOwnPropertyDescriptor(this,name))+' meta='+flags(Object.getOwnPropertyDescriptor(C,'name'))+'/'+flags(Object.getOwnPropertyDescriptor(C,'length'))+' parent='+(Object.getPrototypeOf(C)===Function.prototype)+' construct='+constructor(C)+' keys='+keys(C));
            if(Object.hasOwn(C,'prototype')){
                let p=C.prototype,prototype=Object.getOwnPropertyDescriptor(C,'prototype'),tag=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag);graph.push(p);
                if(!Object.hasOwn(prototype,'value') || prototype.value!==p || !Object.hasOwn(tag,'value') || Object.hasOwn(tag,'get'))throw 1;
                rows.push(name+'.prototype '+flags(Object.getOwnPropertyDescriptor(C,'prototype'))+' parent='+(Object.getPrototypeOf(p)===Object.prototype)+' constructor='+(p.constructor===C)+' constructor-descriptor='+flags(Object.getOwnPropertyDescriptor(p,'constructor'))+' keys='+keys(p)+' tag='+p[Symbol.toStringTag]+'/'+flags(Object.getOwnPropertyDescriptor(p,Symbol.toStringTag)));
            }
        }
        for(let name of ['all','allSettled','any','race','reject','resolve','try','withResolvers'])member('Promise.'+name,Promise,name);
        member('Promise.@@species',Promise,Symbol.species);
        for(let name of ['catch','finally','then'])member('Promise.prototype.'+name,Promise.prototype,name);
        member('Proxy.revocable',Proxy,'revocable');
        member('SharedArrayBuffer.@@species',SharedArrayBuffer,Symbol.species);
        for(let name of ['byteLength','grow','growable','maxByteLength','slice'])member('SharedArrayBuffer.prototype.'+name,SharedArrayBuffer.prototype,name);
        member('WeakRef.prototype.deref',WeakRef.prototype,'deref');
        for(let name of ['register','unregister'])member('FinalizationRegistry.prototype.'+name,FinalizationRegistry.prototype,name);
        graph.push(Atomics);
        if(!Object.hasOwn(Object.getOwnPropertyDescriptor(Atomics,Symbol.toStringTag),'value'))throw 2;
        rows.push('Atomics namespace parent='+(Object.getPrototypeOf(Atomics)===Object.prototype)+' global='+flags(Object.getOwnPropertyDescriptor(this,'Atomics'))+' keys='+keys(Atomics)+' tag='+Atomics[Symbol.toStringTag]+'/'+flags(Object.getOwnPropertyDescriptor(Atomics,Symbol.toStringTag)));
        for(let name of ['add','and','compareExchange','exchange','isLockFree','load','or','pause','store','sub','wait','waitAsync','notify','xor'])member('Atomics.'+name,Atomics,name);
        rows.push('graph '+graph.length+' distinct='+new Set(graph).size+' extensible='+graph.every(o=>Object.isExtensible(o)));
        rows.push('absent Proxy.prototype='+Object.hasOwn(Proxy,'prototype')+' FinalizationRegistry.cleanupSome='+('cleanupSome' in FinalizationRegistry.prototype));
        rows.join('\n')
    "#).unwrap();
    let Value::String(output) = result else {
        panic!("metadata graph must produce a string: {result:?}")
    };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @r###"
    Promise Promise/1 global=w-c meta=--c/--c parent=true construct=true keys=all,allSettled,any,length,name,prototype,race,reject,resolve,try,withResolvers,@@Symbol.species
    Promise.prototype --- parent=true constructor=true constructor-descriptor=w-c keys=catch,constructor,finally,then,@@Symbol.toStringTag tag=Promise/--c
    Proxy Proxy/2 global=w-c meta=--c/--c parent=true construct=true keys=length,name,revocable
    SharedArrayBuffer SharedArrayBuffer/1 global=w-c meta=--c/--c parent=true construct=true keys=length,name,prototype,@@Symbol.species
    SharedArrayBuffer.prototype --- parent=true constructor=true constructor-descriptor=w-c keys=byteLength,constructor,grow,growable,maxByteLength,slice,@@Symbol.toStringTag tag=SharedArrayBuffer/--c
    WeakRef WeakRef/1 global=w-c meta=--c/--c parent=true construct=true keys=length,name,prototype
    WeakRef.prototype --- parent=true constructor=true constructor-descriptor=w-c keys=constructor,deref,@@Symbol.toStringTag tag=WeakRef/--c
    FinalizationRegistry FinalizationRegistry/1 global=w-c meta=--c/--c parent=true construct=true keys=length,name,prototype
    FinalizationRegistry.prototype --- parent=true constructor=true constructor-descriptor=w-c keys=constructor,register,unregister,@@Symbol.toStringTag tag=FinalizationRegistry/--c
    Promise.all all/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.allSettled allSettled/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.any any/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.race race/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.reject reject/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.resolve resolve/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.try try/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.withResolvers withResolvers/0 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.@@species get [Symbol.species]/0 accessor:--c:setter=false meta=--c/--c parent=true keys=length,name construct=false
    Promise.prototype.catch catch/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.prototype.finally finally/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Promise.prototype.then then/2 w-c meta=--c/--c parent=true keys=length,name construct=false
    Proxy.revocable revocable/2 w-c meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.@@species get [Symbol.species]/0 accessor:--c:setter=false meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.prototype.byteLength get byteLength/0 accessor:--c:setter=false meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.prototype.grow grow/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.prototype.growable get growable/0 accessor:--c:setter=false meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.prototype.maxByteLength get maxByteLength/0 accessor:--c:setter=false meta=--c/--c parent=true keys=length,name construct=false
    SharedArrayBuffer.prototype.slice slice/2 w-c meta=--c/--c parent=true keys=length,name construct=false
    WeakRef.prototype.deref deref/0 w-c meta=--c/--c parent=true keys=length,name construct=false
    FinalizationRegistry.prototype.register register/2 w-c meta=--c/--c parent=true keys=length,name construct=false
    FinalizationRegistry.prototype.unregister unregister/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics namespace parent=true global=w-c keys=add,and,compareExchange,exchange,isLockFree,load,notify,or,pause,store,sub,wait,waitAsync,xor,@@Symbol.toStringTag tag=Atomics/--c
    Atomics.add add/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.and and/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.compareExchange compareExchange/4 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.exchange exchange/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.isLockFree isLockFree/1 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.load load/2 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.or or/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.pause pause/0 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.store store/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.sub sub/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.wait wait/4 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.waitAsync waitAsync/4 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.notify notify/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    Atomics.xor xor/3 w-c meta=--c/--c parent=true keys=length,name construct=false
    graph 46 distinct=46 extensible=true
    absent Proxy.prototype=false FinalizationRegistry.cleanupSome=false
    "###);
}

#[test]
fn species_getters_are_distinct_generic_identity_functions_without_construct() {
    check(
        r#"
        let a=Object.getOwnPropertyDescriptor(Promise,Symbol.species).get;
        let b=Object.getOwnPropertyDescriptor(SharedArrayBuffer,Symbol.species).get;
        let symbol=Symbol('receiver'),object={},fn=function(){};
        if(a===b || Promise[Symbol.species]!==Promise || SharedArrayBuffer[Symbol.species]!==SharedArrayBuffer)throw 1;
        for(let receiver of [undefined,null,false,0,-0,NaN,1n,'text',symbol,object,fn]){
            for(let getter of [a,b]){
                if(!Object.is(getter.call(receiver),receiver) || !Object.is(Reflect.apply(getter,receiver,[]),receiver) || !Object.is(getter.bind(receiver)(),receiver))throw 2;
            }
            if(!Object.is(Reflect.get(Promise,Symbol.species,receiver),receiver) || !Object.is(Reflect.get(SharedArrayBuffer,Symbol.species,receiver),receiver))throw 3;
        }
        let inherited=Object.create(Promise);
        if(inherited[Symbol.species]!==inherited)throw 4;
        for(let getter of [a,b]){
            let caught=false;try{new getter();}catch(e){caught=e instanceof TypeError;}
            if(!caught)throw 5;
        }
        Object.defineProperty(Promise,Symbol.species,{value:7});
        let own=Promise[Symbol.species]===7 && delete Promise[Symbol.species] && !(Symbol.species in Promise);
        own && a.call(object)===object && b.call(object)===object
    "#,
    );
}

#[test]
fn deferred_constructors_require_new_with_ordinary_type_error_completion() {
    check(
        r#"
        let hits=0,catches=0,finalizers=0;
        for(let C of [Promise,Proxy,SharedArrayBuffer,WeakRef,FinalizationRegistry]){
            try{C({valueOf(){hits++;return 0;}},()=>{hits++;});throw 1;}
            catch(e){if(!(e instanceof TypeError))throw e;catches++;}
            finally{finalizers++;}
        }
        hits===0 && catches===5 && finalizers===5
    "#,
    );
}

#[test]
fn deferred_algorithms_remain_host_unsupported_and_skip_language_handlers() {
    // Each operation reaches a deferred algorithm or intrinsic constructor.
    // Reading its metadata must not stand in for executing that dependency.
    let operations = [
        "new SharedArrayBuffer({valueOf(){hits++;return 0;}})",
        "Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'byteLength').get.call({})",
        "Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'growable').get.call({})",
        "Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'maxByteLength').get.call({})",
        "SharedArrayBuffer.prototype.grow.call({},0)",
        "SharedArrayBuffer.prototype.slice.call({},0,0)",
    ];
    let mut realm = Realm::default();
    realm.eval("let flag=0,hits=0,order=0;function NewTarget(){}let bound=NewTarget.bind(null);Object.defineProperty(bound,'prototype',{get(){order+=100;throw 7;}})").unwrap();
    for operation in operations {
        let result = realm.eval(&format!(
            "try{{{operation};flag=3;}}catch{{flag=1;}}finally{{flag=2;}}"
        ));
        assert!(
            matches!(result, Err(Error::Unsupported { .. })),
            "{operation}: {result:?}"
        );
        assert_eq!(
            realm.eval("flag===0 && hits===0"),
            Ok(Value::Boolean(true)),
            "{operation}"
        );
    }
    // The deferred body cannot inspect newTarget.prototype or create a fake
    // instance. Ordinary evaluation of arguments still precedes construction.
    realm.eval("order=0").unwrap();
    let result=realm.eval("try{Reflect.construct(SharedArrayBuffer,[(order++,0)],bound);}catch{flag=1;}finally{flag=2;}");
    assert!(
        matches!(&result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(
        realm.eval("order===1 && flag===0 && hits===0"),
        Ok(Value::Boolean(true))
    );
    // Proxy is implemented now: construction itself does not invoke handler
    // traps, while ordinary completion runs finally and later Get uses the trap.
    realm
        .eval("let proxyTarget={x:7},proxyHandler={get(t,k){hits++;return t[k];}},madeProxy;")
        .unwrap();
    for operation in [
        "new Proxy(proxyTarget,proxyHandler)",
        "Proxy.revocable(proxyTarget,proxyHandler).proxy",
    ] {
        realm.eval("flag=0;hits=0;madeProxy=undefined").unwrap();
        assert_eq!(
            realm.eval(&format!("try{{madeProxy={operation};flag=3;}}catch{{flag=1;}}finally{{flag=2;}}madeProxy!==proxyTarget && flag===2 && hits===0 && madeProxy.x===7 && hits===1")),
            Ok(Value::Boolean(true)),
            "{operation}"
        );
    }
    // The actual ProxyCreate operation ignores newTarget.prototype; retain the
    // existing throwing bound getter and the argument-order observation.
    realm
        .eval("order=0;flag=0;hits=0;madeProxy=undefined")
        .unwrap();
    assert_eq!(
        realm.eval("try{madeProxy=Reflect.construct(Proxy,[(order++,proxyTarget),proxyHandler],bound);flag=3;}catch{flag=1;}finally{flag=2;}order===1 && flag===2 && hits===0 && Object.getPrototypeOf(madeProxy)===Object.prototype && madeProxy.x===7 && hits===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn additional_metadata_functions_reject_construction_before_deferred_bodies() {
    check(
        r#"
        let functions=[Proxy.revocable,WeakRef.prototype.deref,FinalizationRegistry.prototype.register,FinalizationRegistry.prototype.unregister];
        for(let name of ['all','allSettled','any','race','reject','resolve','try','withResolvers'])functions.push(Promise[name]);
        for(let name of ['catch','finally','then'])functions.push(Promise.prototype[name]);
        for(let name of ['byteLength','grow','growable','maxByteLength','slice']){
            let d=Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,name);functions.push(d.get===undefined?d.value:d.get);
        }
        for(let name of Object.getOwnPropertyNames(Atomics))functions.push(Atomics[name]);
        functions.push(Object.getOwnPropertyDescriptor(Promise,Symbol.species).get,Object.getOwnPropertyDescriptor(SharedArrayBuffer,Symbol.species).get);
        for(let f of functions){
            let caught=false;try{Reflect.construct(f,[]);}catch(e){caught=e instanceof TypeError;}
            if(!caught || Object.hasOwn(f,'prototype'))throw 1;
        }
        let namespace=0;try{new Atomics();}catch(e){if(e instanceof TypeError)namespace++;}
        try{Atomics();}catch(e){if(e instanceof TypeError)namespace++;}
        functions.length===36 && namespace===2
    "#,
    );
}

#[test]
fn global_inventory_has_exact_standard_names_and_data_descriptors() {
    let value = Realm::default()
        .eval("Object.getOwnPropertyNames(this).sort().join(',')")
        .unwrap();
    let Value::String(names) = value else {
        panic!("global names must be a string: {value:?}")
    };
    insta::assert_snapshot!(names.to_utf8().unwrap(), @"AggregateError,Array,ArrayBuffer,AsyncDisposableStack,Atomics,BigInt,BigInt64Array,BigUint64Array,Boolean,DataView,Date,DisposableStack,Error,EvalError,FinalizationRegistry,Float16Array,Float32Array,Float64Array,Function,Infinity,Int16Array,Int32Array,Int8Array,Iterator,JSON,Map,Math,NaN,Number,Object,Promise,Proxy,RangeError,ReferenceError,Reflect,RegExp,Set,SharedArrayBuffer,String,SuppressedError,Symbol,SyntaxError,TypeError,URIError,Uint16Array,Uint32Array,Uint8Array,Uint8ClampedArray,WeakMap,WeakRef,WeakSet,decodeURI,decodeURIComponent,encodeURI,encodeURIComponent,eval,globalThis,isFinite,isNaN,parseFloat,parseInt,undefined");
    check(
        r#"
        let names=Object.getOwnPropertyNames(this),keys=Reflect.ownKeys(this),descriptors=Object.getOwnPropertyDescriptors(this);
        if(names.length!==62 || keys.length!==62 || Object.getOwnPropertySymbols(this).length!==0 || Object.keys(this).length!==0 || Object.getPrototypeOf(this)!==Object.prototype)throw 1;
        for(let i=0;i<names.length;i++){
            let name=names[i],d=descriptors[name],fixed=name==='undefined'||name==='NaN'||name==='Infinity';
            if(keys[i]!==name || !Object.hasOwn(d,'value') || Object.hasOwn(d,'get') || d.enumerable || d.writable===fixed || d.configurable===fixed || !Object.is(d.value,this[name]))throw 2;
        }
        !Object.hasOwn(this,'$262') && !('$262' in this) && typeof $262==='undefined' && !Object.hasOwn(this,'console') && !Object.hasOwn(this,'global') && !Object.hasOwn(this,'process')
    "#,
    );
}

#[test]
fn global_own_keys_order_indices_strings_symbols_and_lone_surrogates() {
    check(
        r#"
        let g=this,base=Object.getOwnPropertyNames(g),first=Symbol('same'),second=Symbol('same');
        g[first]=1;g['10']=10;g['2']=2;g['4294967295']=1;g['01']=1;g['-0']=1;g['0']=0;g['4294967294']=1;g['\uD800']=1;g[second]=2;g[first]=3;
        let keys=Reflect.ownKeys(g),names=Object.getOwnPropertyNames(g),symbols=Object.getOwnPropertySymbols(g);
        if(keys.slice(0,4).join(',')!=='0,2,10,4294967294' || keys.length!==base.length+10)throw 1;
        for(let i=0;i<base.length;i++)if(keys[i+4]!==base[i])throw 2;
        let tail=4+base.length;
        let initialOrder=keys[tail]==='4294967295' && keys[tail+1]==='01' && keys[tail+2]==='-0' && keys[tail+3]==='\uD800' && names[names.length-1].charCodeAt(0)===0xD800 && keys[tail+4]===first && keys[tail+5]===second && symbols.length===2 && symbols[0]===first && symbols[1]===second && g[first]===3 && Object.keys(g).join(',')==='0,2,10,4294967294,4294967295,01,-0,\uD800';
        delete g[first];g[first]=9;
        let reordered=Object.getOwnPropertySymbols(g);
        initialOrder && reordered.length===2 && reordered[0]===second && reordered[1]===first && Object.getOwnPropertyNames(g).every((name,index)=>name===names[index])
    "#,
    );
}

#[test]
fn declarations_remain_separate_and_keep_ordinary_global_descriptors() {
    let mut realm = Realm::default();
    realm
        .eval("let initial=Object.getOwnPropertyNames(this)")
        .unwrap();
    assert_eq!(
        realm.eval(r#"
            let lexicalMarker=1;const constantMarker=2;class ClassMarker{}
            var declaredMarker=3;function functionMarker(){}sloppyMarker=4;
            let g=this,d=Object.getOwnPropertyDescriptor(g,'declaredMarker'),f=Object.getOwnPropertyDescriptor(g,'functionMarker'),s=Object.getOwnPropertyDescriptor(g,'sloppyMarker');
            !Object.hasOwn(g,'initial') && !Object.hasOwn(g,'lexicalMarker') && !Object.hasOwn(g,'constantMarker') && !Object.hasOwn(g,'ClassMarker') && Object.getOwnPropertyNames(g).length===initial.length+3 && Object.keys(g).sort().join(',')==='declaredMarker,functionMarker,sloppyMarker' && d.value===3 && d.writable && d.enumerable && !d.configurable && f.value===functionMarker && f.writable && f.enumerable && !f.configurable && s.value===4 && s.writable && s.enumerable && s.configurable && !delete g.declaredMarker && !delete g.functionMarker && delete g.sloppyMarker && !Object.hasOwn(g,'sloppyMarker')
        "#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn materialized_globals_can_be_deleted_recreated_and_redeclared_without_reservations() {
    check(
        r#"
        let g=this,O=Object,run=eval,initial=O.getOwnPropertyNames(g);
        for(let name of ['Promise','Proxy','SharedArrayBuffer','WeakRef','FinalizationRegistry','Atomics']){
            let old=O.getOwnPropertyDescriptor(g,name),position=O.getOwnPropertyNames(g).indexOf(name);
            g[name]=7;
            if(O.getOwnPropertyNames(g).indexOf(name)!==position || g[name]!==7)throw 1;
            if(!delete g[name] || name in g || O.hasOwn(g,name) || O.getOwnPropertyDescriptor(g,name)!==undefined || run('typeof '+name)!=='undefined')throw 2;
            let caught=false;try{run(name);}catch(e){caught=e instanceof ReferenceError;}
            if(!caught)throw 3;
            g[name]=9;
            let d=O.getOwnPropertyDescriptor(g,name),now=O.getOwnPropertyNames(g);
            if(now[now.length-1]!==name || d.value!==9 || !d.writable || !d.enumerable || !d.configurable)throw 4;
            if(!delete g[name])throw 5;
            O.defineProperty(g,name,old);
        }
        delete g.Promise;delete g.Proxy;
        run('var Promise;function Proxy(){return 7;}');
        let d=O.getOwnPropertyDescriptor(g,'Promise'),f=O.getOwnPropertyDescriptor(g,'Proxy');
        d.value===undefined && d.writable && d.enumerable && d.configurable && f.value()===7 && f.writable && f.enumerable && f.configurable && O.getOwnPropertyNames(g).length===initial.length
    "#,
    );
    check(
        "delete this.Proxy;let Proxy=7;Proxy===7 && !Object.hasOwn(this,'Proxy') && !('Proxy' in this)",
    );
}

#[test]
fn unreserved_absent_names_are_ordinary_global_properties() {
    for name in ["AsyncFunction", "$262", "unconfiguredHost"] {
        check(&format!(
            "let g=this;typeof {name}==='undefined' && !('{name}' in g) && !Object.hasOwn(g,'{name}') && Object.getOwnPropertyDescriptor(g,'{name}')===undefined && delete g.{name} && (g.{name}=7)===7 && {name}===7 && delete g.{name} && typeof {name}==='undefined'"
        ));
        let result = Realm::default().eval(name);
        assert!(
            matches!(
                result,
                Err(Error::Exception {
                    kind: ExceptionKind::ReferenceError,
                    ..
                })
            ),
            "{name}: {result:?}"
        );
    }
}

#[test]
fn configured_host_reservations_remain_explicit_and_do_not_install_properties() {
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("missingHost");
    realm.reserve_unsupported_global("Proxy");
    assert_eq!(
        realm.eval("typeof Proxy==='function' && 'missingHost' in this && !Object.hasOwn(Object.getOwnPropertyDescriptors(this),'missingHost') && !Reflect.ownKeys(this).includes('missingHost')"),
        Ok(Value::Boolean(true))
    );
    for source in [
        "missingHost",
        "typeof missingHost",
        "this.missingHost",
        "this.missingHost=7",
        "delete this.missingHost",
        "Object.getOwnPropertyDescriptor(this,'missingHost')",
    ] {
        let result = realm.eval(source);
        assert!(
            matches!(result, Err(Error::Unsupported { .. })),
            "{source}: {result:?}"
        );
    }
    assert_eq!(
        realm.eval("{let missingHost=8;missingHost===8}"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("delete this.Proxy"), Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval("typeof Proxy"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("!Reflect.ownKeys(this).includes('Proxy')"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn the_internal_global_identity_survives_global_this_replacement_and_deletion() {
    check(
        r#"
        let g=this,base=Reflect.ownKeys(g),index=base.indexOf('globalThis');
        globalThis={user:7};
        if(this!==g || g.globalThis.user!==7 || Reflect.ownKeys(g)[index]!=='globalThis' || Object.getOwnPropertyDescriptor(g,'globalThis').value!==globalThis)throw 1;
        if(!delete g.globalThis || 'globalThis' in g || typeof globalThis!=='undefined' || Reflect.ownKeys(g).length!==base.length-1)throw 2;
        g.recreated=8;g.globalThis=g;
        let names=Object.getOwnPropertyNames(g);
        names[names.length-2]==='recreated' && names[names.length-1]==='globalThis' && g.globalThis===g && this===g
    "#,
    );
}

#[test]
fn inherited_globals_and_non_enumerable_shadows_use_ordinary_lookup() {
    check(
        r#"
        let g=this,proto={Proxy:17,ownBlocked:18},seen=[];
        delete g.Proxy;
        Object.setPrototypeOf(g,proto);
        if(Proxy!==17 || !('Proxy' in g) || Object.hasOwn(g,'Proxy') || Reflect.ownKeys(g).includes('Proxy'))throw 1;
        Object.defineProperty(g,'ownBlocked',{value:7,enumerable:false});
        for(let key in g)seen.push(key);
        seen.join(',')==='Proxy' && delete proto.Proxy && typeof Proxy==='undefined' && !('Proxy' in g)
    "#,
    );
}

#[test]
fn global_for_in_is_getter_inert_and_rechecks_live_descriptors_and_prototypes() {
    check(
        r#"
        let g=this,hits=0,seen=[],first=Symbol('ignored'),proto={doomed:1,hidden:2,later:3,protoTail:4};
        Object.setPrototypeOf(g,proto);
        g.first=1;g.doomed=2;
        Object.defineProperty(g,'hidden',{value:3,enumerable:false});
        g.later=4;
        Object.defineProperty(g,'probe',{enumerable:true,get(){hits++;throw 7;}});
        g[first]=5;
        for(let key in g){
            seen.push(key);
            if(key==='first'){
                delete g.doomed;
                Object.defineProperty(g,'later',{enumerable:false});
                g.added=6;proto.newInherited=7;
            }
        }
        seen.join(',')==='first,probe,doomed,protoTail,newInherited' && hits===0 && !seen.includes('hidden') && !seen.includes('later') && !seen.includes('added') && Reflect.ownKeys(g).includes('added') && Object.getOwnPropertySymbols(g)[0]===first
    "#,
    );
}

#[test]
fn global_descriptor_enumerability_checks_run_without_harness_shortcuts() {
    // The ordinary for-in probe used by upstream propertyHelper must finish
    // even when all built-in global properties are non-enumerable.
    check(
        r#"
        let names=['WeakMap','WeakSet','ArrayBuffer','DataView','Promise','Proxy','SharedArrayBuffer','WeakRef','FinalizationRegistry','Atomics'],seen=[];
        for(let key in this)seen.push(key);
        for(let name of names){
            let d=Object.getOwnPropertyDescriptor(this,name);
            if(!Object.hasOwn(this,name) || !d.writable || d.enumerable || !d.configurable || seen.includes(name) || Object.prototype.propertyIsEnumerable.call(this,name))throw 1;
        }
        seen.length===0
    "#,
    );
}

#[test]
fn deferred_array_constructor_keys_remain_an_explicit_separate_gap() {
    for source in [
        "Reflect.ownKeys(Array)",
        "Object.getOwnPropertyNames(Array)",
        "Object.getOwnPropertySymbols(Array)",
        "Array.fromAsync",
        "Object.getOwnPropertyDescriptor(Array,'fromAsync')",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        let result = realm.eval(&format!(
            "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
        ));
        assert!(
            matches!(result, Err(Error::Unsupported { .. })),
            "{source}: {result:?}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn deleted_global_bindings_do_not_unroot_any_new_intrinsic_metadata_object() {
    let mut realm = Realm::default();
    // Drop all configurable public edges before collection. The temporary
    // array and its local environment do not become persistent JavaScript roots.
    let objects = realm.eval(r#"(()=>{let values=[
        AsyncDisposableStack,AsyncDisposableStack.prototype,AsyncDisposableStack.prototype.adopt,AsyncDisposableStack.prototype.defer,
        AsyncDisposableStack.prototype.disposeAsync,AsyncDisposableStack.prototype.move,AsyncDisposableStack.prototype.use,Object.getOwnPropertyDescriptor(AsyncDisposableStack.prototype,'disposed').get,
        Promise,Proxy,SharedArrayBuffer,WeakRef,FinalizationRegistry,Atomics,
        Promise.prototype,SharedArrayBuffer.prototype,WeakRef.prototype,FinalizationRegistry.prototype,
        Promise.all,Promise.allSettled,Promise.any,Promise.race,Promise.reject,Promise.resolve,Promise.try,Promise.withResolvers,
        Object.getOwnPropertyDescriptor(Promise,Symbol.species).get,Promise.prototype.catch,Promise.prototype.finally,Promise.prototype.then,
        Proxy.revocable,Object.getOwnPropertyDescriptor(SharedArrayBuffer,Symbol.species).get,
        Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'byteLength').get,SharedArrayBuffer.prototype.grow,
        Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'growable').get,
        Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype,'maxByteLength').get,SharedArrayBuffer.prototype.slice,
        WeakRef.prototype.deref,FinalizationRegistry.prototype.register,FinalizationRegistry.prototype.unregister,
        Atomics.add,Atomics.and,Atomics.compareExchange,Atomics.exchange,Atomics.isLockFree,Atomics.load,Atomics.or,Atomics.pause,Atomics.store,Atomics.sub,Atomics.wait,Atomics.waitAsync,Atomics.notify,Atomics.xor
    ];
    for(let object of values){
        for(let key of Reflect.ownKeys(object)){
            if(Object.getOwnPropertyDescriptor(object,key).configurable && !delete object[key])throw 1;
        }
    }
    return values;})()"#).unwrap();
    assert_eq!(
        realm.read_property(&objects, &JsString::from("length")),
        Ok(Value::Number(54.0))
    );
    let handles: Vec<_> = (0..54)
        .map(|index| {
            let value = realm
                .read_property(&objects, &JsString::from(index.to_string().as_str()))
                .unwrap();
            let Value::Object(handle) = value else {
                panic!("intrinsic {index} must be an object: {value:?}")
            };
            handle
        })
        .collect();
    let global = realm.eval("this").unwrap();
    realm.eval("delete this.AsyncDisposableStack;delete this.Promise;delete this.Proxy;delete this.SharedArrayBuffer;delete this.WeakRef;delete this.FinalizationRegistry;delete this.Atomics;delete this.globalThis").unwrap();
    drop(objects);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    for (index, handle) in handles.iter().enumerate() {
        assert!(
            realm.inspect_object(handle).is_ok(),
            "deleted global lost intrinsic {index}"
        );
    }
    assert_eq!(realm.eval("this"), Ok(global));
    assert_eq!(
        realm.eval("Object.getOwnPropertyNames(this).length===54 && typeof Promise==='undefined' && typeof Proxy==='undefined' && typeof globalThis==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
