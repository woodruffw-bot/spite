//! GetFunctionRealm at constructor fallback and Array species joins (7.3.24).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

// Every constructor uses a distinct ordinary function as newTarget's Proxy
// target. Its writable prototype prevents an unrelated Proxy Get invariant
// from producing the expected TypeError before GetFunctionRealm is reached.
const FAMILIES: &str = r#"
    let bodyCalls=0;
    function Ordinary(){bodyCalls++;}
    class Base{field=(bodyCalls++);constructor(){bodyCalls++;}}
    let families=[
        ['Object',Object,[]],['ordinary',Ordinary,[]],['base class',Base,[]],
        ['Function',Function,[]],['Array',Array,[]],['Boolean',Boolean,[]],
        ['Number',Number,[]],['String',String,[]],['Error',Error,[]],
        ['EvalError',EvalError,[]],['RangeError',RangeError,[]],
        ['ReferenceError',ReferenceError,[]],['SyntaxError',SyntaxError,[]],
        ['TypeError',TypeError,[]],['URIError',URIError,[]],
        ['AggregateError',AggregateError,[[]]],['Date',Date,[0]],
        ['RegExp',RegExp,['x']],['Map',Map,[]],['Set',Set,[]],
        ['WeakMap',WeakMap,[]],['WeakSet',WeakSet,[]],
        ['ArrayBuffer',ArrayBuffer,[0]],['DataView',DataView,[new ArrayBuffer(0)]],
        ['Iterator',Iterator,[]]
    ];
    for(let C of [Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,
        Int32Array,Uint32Array,Float16Array,Float32Array,Float64Array,
        BigInt64Array,BigUint64Array])families.push([C.name,C,[0]]);
"#;

#[test]
fn fallback_invalidated_constructor_proxies_reject_for_every_family() {
    check(&format!(
        "{FAMILIES}{}",
        r#"
        for(let family of families){
            let reads=0,traps=0,pair,plain=function(){};
            if(!Object.getOwnPropertyDescriptor(plain,'prototype').writable)throw 'fixture';
            pair=Proxy.revocable(plain,{
                get(t,k,r){if(k==='prototype'){reads++;pair.revoke();return null;}traps++;throw 1;},
                getPrototypeOf(){traps++;throw 2;},construct(){traps++;throw 3;}
            });
            bodyCalls=0;let caught=false;
            try{Reflect.construct(family[1],family[2],pair.proxy);}
            catch(e){if(!(e instanceof TypeError))throw e;caught=true;}
            if(!caught || reads!==1 || traps!==0 || bodyCalls!==0)throw family[0];
        }
        true
        "#
    ));
}

#[test]
fn object_prototype_results_skip_realm_validation_even_if_revoked() {
    check(&format!(
        "{FAMILIES}{}",
        r#"
        for(let family of families){
            let retired=Proxy.revocable({},{});retired.revoke();
            for(let prototype of [Object.create(null),retired.proxy]){
                let reads=0,traps=0,pair;
                pair=Proxy.revocable(function(){},{
                    get(t,k,r){if(k==='prototype'){reads++;pair.revoke();return prototype;}traps++;throw 1;},
                    getPrototypeOf(){traps++;throw 2;},construct(){traps++;throw 3;}
                });
                let instance=Reflect.construct(family[1],family[2],pair.proxy);
                if(Object.getPrototypeOf(instance)!==prototype || reads!==1 || traps!==0)throw family[0];
            }
        }
        true
        "#
    ));
}

#[test]
fn bound_and_mixed_targets_validate_without_public_traps() {
    check(
        r#"
        for(let mixed of [false,true]){
            for(let objectResult of [false,true]){
                let armed=false,traps=0,reads=0,pair,prototype=Object.create(null);
                let handler={get(t,k,r){if(armed){traps++;throw 1;}return Reflect.get(t,k,r);},
                    getPrototypeOf(t){if(armed){traps++;throw 2;}return Reflect.getPrototypeOf(t);},
                    construct(){traps++;throw 3;}};
                pair=Proxy.revocable(function(){},handler);
                let N=pair.proxy.bind(null).bind(null);
                if(mixed)N=new Proxy(N,handler).bind(null);
                Object.defineProperty(N,'prototype',{get(){reads++;pair.revoke();armed=true;return objectResult?prototype:0;}});
                let caught=false,instance;
                try{instance=Reflect.construct(Object,[],N);}
                catch(e){if(!(e instanceof TypeError))throw e;caught=true;}
                if(reads!==1 || traps!==0 || caught===objectResult)throw 4;
                if(objectResult && Object.getPrototypeOf(instance)!==prototype)throw 5;
            }
        }
        let marker={},reads=0,pair=Proxy.revocable(function(){},{}),N=pair.proxy.bind(null);
        Object.defineProperty(N,'prototype',{get(){reads++;throw marker;}});
        pair.revoke();let caught=false;
        try{Reflect.construct(Object,[],N);}catch(e){caught=e===marker;}
        caught && reads===1
        "#,
    );
}

#[test]
fn array_species_validates_constructor_realm_before_reading_species() {
    check(
        r#"
        let operations=[
            ['map',a=>a.map(x=>x*2),3,'2,4,6'],
            ['filter',a=>a.filter(x=>x>1),0,'2,3'],
            ['slice',a=>a.slice(1),2,'2,3']
        ];
        for(let operation of operations){
            for(let bound of [false,true]){
                let speciesReads=0,constructorGets=0,callbackCalls=0;
                function Species(){throw 'species must not be constructed';}
                let inner=Proxy.revocable(function(){},{}),C;
                if(bound){
                    C=inner.proxy.bind(null);
                    Object.defineProperty(C,Symbol.species,{get(){speciesReads++;return Species;}});
                }else{
                    C=new Proxy(inner.proxy,{get(t,k,r){if(k===Symbol.species){speciesReads++;return Species;}throw 'unexpected Get';}});
                }
                inner.revoke();
                let a=[1,2,3];Object.defineProperty(a,'constructor',{get(){constructorGets++;return C;}});
                let caught=false;
                try{
                    if(operation[0]==='map')a.map(x=>{callbackCalls++;return x;});
                    else if(operation[0]==='filter')a.filter(x=>{callbackCalls++;return true;});
                    else a.slice(1);
                }catch(e){if(!(e instanceof TypeError))throw e;caught=true;}
                if(!caught || constructorGets!==1 || speciesReads!==0 || callbackCalls!==0)throw operation[0];
            }
            // IsConstructor(C) controls the realm query, not whether a species
            // property is read: a nonconstructor object can supply a constructor.
            for(let constructor of [{},()=>{}]){
                let speciesReads=0,calls=0,length;
                function Species(n){calls++;length=n;return [];}
                Object.defineProperty(constructor,Symbol.species,{get(){speciesReads++;return Species;}});
                let a=[1,2,3];a.constructor=constructor;
                let result=operation[1](a);
                if(speciesReads!==1 || calls!==1 || length!==operation[2] || result.join(',')!==operation[3])throw 1;
            }
            let speciesReads=0;
            function C(){throw 'constructor must not be called';}
            Object.defineProperty(C,Symbol.species,{get(){speciesReads++;return Array;}});
            let a=[1,2,3];a.constructor=C;
            if(operation[1](a).join(',')!==operation[3] || speciesReads!==1)throw 2;
        }
        true
        "#,
    );
}

#[test]
fn deeply_nested_bound_proxy_targets_use_the_normal_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            check(
                r#"
                let tail=Proxy.revocable(function(){},{}),N=tail.proxy;
                for(let i=0;i<4096;i++){
                    N=Function.prototype.bind.call(N,null);
                    Object.defineProperty(N,'name',{value:''});
                    N=new Proxy(N,{});
                }
                // This own value makes the public Get shallow. Object is the
                // distinct construction target, so Construct does not forward
                // through N; only GetFunctionRealm traverses its target chain.
                Object.defineProperty(N,'prototype',{value:null});
                let live=Reflect.construct(Object,[],N);
                if(Object.getPrototypeOf(live)!==Object.prototype)throw 1;
                tail.revoke();let caught=false;
                try{Reflect.construct(Object,[],N);}catch(e){caught=e instanceof TypeError;}
                caught
                "#,
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn constructor_realm_validation_preserves_conversion_and_error_order() {
    check(
        r#"
        let log='',marker={};
        function newTarget(){
            let pair;
            pair=Proxy.revocable(function(){},{get(t,k,r){if(k==='prototype'){log+='p';pair.revoke();return null;}throw 1;}});
            return pair.proxy;
        }
        // Array selects a prototype before validating its numeric length.
        let caught=false;try{Reflect.construct(Array,[1.5],newTarget());}catch(e){caught=e instanceof TypeError;}
        if(!caught || log!=='p')throw 2;
        // Number, String and Date perform their argument conversions first.
        for(let C of [Number,String,Date]){
            log='';let argument={[Symbol.toPrimitive](){log+='c';throw marker;}};
            caught=false;try{Reflect.construct(C,[argument],newTarget());}catch(e){caught=e===marker;}
            if(!caught || log!=='c')throw 3;
        }
        // Error allocation precedes message/cause and AggregateError iteration.
        for(let C of [Error,AggregateError]){
            log='';let message={toString(){log+='m';throw marker;}},options={get cause(){log+='c';throw marker;}};
            let errors={get [Symbol.iterator](){log+='i';throw marker;}};
            let args=C===AggregateError?[errors,message,options]:[message,options];
            caught=false;try{Reflect.construct(C,args,newTarget());}catch(e){caught=e instanceof TypeError;}
            if(!caught || log!=='p')throw 4;
        }
        log='';caught=false;
        try{Reflect.construct(Function,[')'],newTarget());}catch(e){caught=e instanceof SyntaxError;}
        if(!caught || log!=='')throw 5;
        // RegExp's source/flags Gets precede allocation; coercions follow it.
        log='';let text={toString(){log+='c';throw marker;}};
        let pattern={get [Symbol.match](){log+='m';return true;},get source(){log+='s';return text;},get flags(){log+='f';return text;}};
        caught=false;try{Reflect.construct(RegExp,[pattern],newTarget());}catch(e){caught=e instanceof TypeError;}
        if(!caught || log!=='msfp')throw 6;
        // ArrayBuffer and DataView retain their earlier argument failures.
        log='';caught=false;
        let length={valueOf(){log+='l';return 1;}},options={get maxByteLength(){log+='m';return {valueOf(){log+='v';return 0;}};}};
        try{Reflect.construct(ArrayBuffer,[length,options],newTarget());}catch(e){caught=e instanceof RangeError;}
        if(!caught || log!=='lmv')throw 7;
        log='';caught=false;
        try{Reflect.construct(DataView,[new ArrayBuffer(4),-1],newTarget());}catch(e){caught=e instanceof RangeError;}
        if(!caught || log!=='')throw 8;
        log='';caught=false;
        try{Reflect.construct(Uint8Array,[-1],newTarget());}catch(e){caught=e instanceof RangeError;}
        if(!caught || log!=='')throw 9;
        // An object prototype skips the realm check but not DataView's final
        // buffer-state check after the prototype getter.
        log='';let buffer=new ArrayBuffer(4),N=function(){}.bind(null);
        Object.defineProperty(N,'prototype',{get(){log+='p';buffer.transfer();return {};}});
        let offset={valueOf(){log+='o';return 0;}},size={valueOf(){log+='l';return 4;}};
        caught=false;try{Reflect.construct(DataView,[buffer,offset,size],N);}catch(e){caught=e instanceof TypeError;}
        if(!caught || log!=='olp')throw 10;
        function fail(){log+='b';throw marker;}
        function Ordinary(x=fail()){log+='c';}
        class Base{field=fail();constructor(){log+='c';}}
        for(let C of [Ordinary,Base]){
            log='';caught=false;
            try{Reflect.construct(C,[],newTarget());}catch(e){caught=e instanceof TypeError;}
            if(!caught || log!=='p')throw 11;
        }
        true
        "#,
    );
}
