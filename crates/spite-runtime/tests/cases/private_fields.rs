//! Private fields, private references, lexical names, and brands (15.7/7.3.27–33).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn fields_support_reads_updates_assignments_and_private_call_receivers() {
    check(
        "class C{#x;#y=1;#f=function(v){this.#y+=v;return this;};run(){let old=this.#y++;this.#x??=3;this.#x&&=4;this.#x||=9;({v:this.#y}={v:5});return old===1 && this.#x===4 && this.#f(2)===this && this.#y===7;}}new C().run()",
    );
    check(
        "class C{#f=function(){this.value=7;};#tag=function(s,v){return this===c && s[0]==='x' && v===8;};run(){return new this.#f().value===7 && this.#tag`x${8}`;}}let c=new C;c.run()",
    );
    check(
        "class C{#x=7;read(o){return o?.#x;}call(o){return o?.#x.toString();}}let c=new C;c.read(null)===undefined && c.read(c)===7 && c.call(null)===undefined && c.call(c)==='7'",
    );
}

#[test]
fn private_names_use_fresh_identity_and_brands_are_own_only() {
    check(
        "function make(){return class{#x=7;static has(o){return #x in o;}read(o){return o.#x;}};}let A=make(),B=make(),a=new A,b=new B,caught=false;try{a.read(b);}catch(e){caught=e instanceof TypeError;}caught && A.has(a) && !A.has(b) && B.has(b) && !A.has(Object.create(a)) && !A.has(A.prototype)",
    );
    check(
        "class B{#x=1;readB(){return this.#x;}static hasB(o){return #x in o;}}class D extends B{#x=2;readD(){return this.#x;}}let d=new D;d.readB()===1 && d.readD()===2 && B.hasB(d)",
    );
    check(
        "class C{#\\u0078=7;get(){return this.#x;}static has(o){return #\\u0078 in o;}}let c=new C;c.get()===7 && C.has(c)",
    );
}

#[test]
fn private_fields_do_not_appear_in_properties_or_obey_extensibility() {
    check(
        "let o=Object.freeze({});class B{constructor(){return o;}}class C extends B{#x=1;static set(o){o.#x=7;}static read(o){return o.#x;}}new C;C.set(o);C.read(o)===7 && Object.isFrozen(o) && Reflect.ownKeys(o).length===0 && Object.getOwnPropertySymbols(o).length===0 && Object.getOwnPropertyDescriptor(o,'#x')===undefined && JSON.stringify(o)==='{}' && Reflect.ownKeys(Object.assign({},o)).length===0",
    );
    check(
        "class C{#x=7;static has(o){return #x in o;}static read(o){return o.#x;}}let c=new C;c['#x']=9;C.read(c)===7 && C.has(c) && Object.keys(c).join(',')==='#x' && !C.has({'#x':7})",
    );
}

#[test]
fn static_private_fields_and_blocks_follow_source_order() {
    check(
        "let log='';class C{static before=(log+='A',#x in this);static{log+=#x in this?'X':'B';}static #x=(log+='C',7);static{log+=#x in this?'D':'Y';this.read=()=>this.#x;}static after=(log+='E',this.#x);static has(o){return #x in o;}}class D extends C{}log==='ABCDE' && !C.before && C.after===7 && C.read()===7 && C.has(C) && !C.has(D) && !C.has(new C)",
    );
    check(
        "let saved,caught=false;try{class C{static{saved=this;this.#x;}static #x=7;}}catch(e){caught=e instanceof TypeError;}caught && typeof saved==='function'",
    );
    check(
        "let log='';class C{static #x=(log+='I',7);[(log+='K',#x in {})](){}static read(){return this.#x;}}log==='KI' && C.read()===7",
    );
}

#[test]
fn heritage_closures_keep_outer_private_names_and_internal_class_binding() {
    check(
        "class Outer{#x=7;make(){let read,internal;let Inner=class Named extends (read=o=>o.#x,internal=()=>Named,Object){#x=8;read(o){return o.#x;}};return [read,internal,Inner];}}let outer=new Outer,[read,internal,Inner]=outer.make(),inner=new Inner,caught=false;try{read(inner);}catch(e){caught=e instanceof TypeError;}caught && read(outer)===7 && internal()===Inner && inner.read(inner)===8",
    );
    check(
        "class Outer{make(){return class extends (this.#base){#base=8;read(){return this.#base;}};}#base=Object;}let C=new Outer().make();new C().read()===8",
    );
    check(
        "class Outer{#x=7;make(){return class{read(o){return o.#x;}};}}let o=new Outer,C=o.make();new C().read(o)===7",
    );
}

#[test]
fn direct_eval_inherits_names_and_indirect_eval_and_function_do_not() {
    check(
        "class C{#x=7;read(){return eval('this.#x');}run(){let o=this;function f(){return eval('o.#x += 1; #x in o');}return f() && eval('(()=>this.#x)()')===8;}static has(o){return eval('#x in o');}}let c=new C;c.read()===7 && c.run() && C.has(c)",
    );
    check(
        "let effects=0;class C{#x=7;run(){let count=0;for(let source of ['effects=1;this.#missing','effects=1;#missing in this','effects=1;function f(){return this.#missing;}']){try{eval(source);}catch(e){if(e instanceof SyntaxError)count++;}}for(let f of [()=> (0,eval)('this.#x'),()=>Function('return this.#x')]){try{f();}catch(e){if(e instanceof SyntaxError)count++;}}return count===5;}}new C().run() && effects===0",
    );
    check(
        "class C{static #x=7;static{this.f=eval('()=>this.#x');}#x2=eval('this.#x3');#x3=8;}let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught && C.f()===7",
    );
    check(
        "class C{#x=7;make(){return eval('(class D{#x=8;read(o){return o.#x;}outer(o){return eval(\"o.#x\");}})');}}let D=new C().make(),d=new D;d.read(d)===8 && d.outer(d)===8",
    );
}

#[test]
fn brand_errors_follow_expression_order_and_stop_later_effects() {
    check(
        "let log='';class C{#x;run(o){try{(log+='B',o).#x=(log+='R',1);}catch(e){log+=e instanceof TypeError?'E':'X';}try{o.#x+=(log+='Y',2);}catch(e){log+=e instanceof TypeError?'F':'X';}return log;}}new C().run({})==='BREF'",
    );
    check(
        "class C{#x;static has(o){return #x in o;}static get(o){return o.#x;}static put(o){o.#x=1;}}let count=0;for(let o of [null,undefined,1,'x',true,Symbol(),1n]){for(let f of [C.has,C.get,C.put]){try{f(o);}catch(e){if(e instanceof TypeError)count++;}}}count===21",
    );
    check(
        "let effects=0;class C{#x;static run(){try{return #x in (effects++,null);}catch(e){return e instanceof TypeError;}}}C.run() && effects===1",
    );
}

#[test]
fn duplicate_stamping_checks_follow_initializers_and_preserve_existing_fields() {
    check(
        "let o=Object.preventExtensions({}),log='';class B{constructor(){return o;}}class C extends B{#x=(log+='I',1);#y=(log+='J',2);static read(o){return o.#x+o.#y;}}new C;let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught && log==='IJI' && C.read(o)===3",
    );
    check(
        "let o={},saved,caught=false;class B{constructor(){return o;}}class C extends B{#x=7;#y=(()=>{throw 9;})();#z=8;static has(o){return [#x in o,#y in o,#z in o];}static read(o){return o.#x;}}try{new C;}catch(e){caught=e===9;}caught && C.has(o).join(',')==='true,false,false' && C.read(o)===7",
    );
}

#[test]
fn base_and_derived_fields_use_construction_receivers_and_parameter_order() {
    check(
        "let log='';class B{#x=(log+='B',7);constructor(v=(log+='P',this.#x)){this.v=v;}readB(){return this.#x;}}class D extends B{#x=(log+='D',8);constructor(){log+='A';super();log+='Z';}readD(){return this.#x;}}let d=new D;log==='ABPDZ' && d.v===7 && d.readB()===7 && d.readD()===8",
    );
    check(
        "class C{#x=7;constructor(){return {};}static has(o){return #x in o;}}class D extends C{#y=8;static hasY(o){return #y in o;}}let d=new D;!C.has(d) && D.hasY(d)",
    );
    check(
        "let count=0;class B{}class D extends B{#x=(count++,7);constructor(){return {};}static has(o){return #x in o;}}let d=new D;count===0 && !D.has(d)",
    );
    check(
        "class B{#x=7;static has(o){return #x in o;}}let C=B;for(let i=0;i<100;i++){C=class extends C{#x=8;};}let bound=C.bind(null);function N(){}let c=Reflect.construct(bound,[],N);B.has(c) && Object.getPrototypeOf(c)===N.prototype",
    );
}

#[test]
fn initializer_names_and_function_contexts_preserve_private_descriptions() {
    check(
        "class B{get x(){return 7;}}class C extends B{#f=function(){};#a=()=>{};#c=class{static seen=this.name;};#x=super.x;#t=eval('new.target');run(){return this.#f.name==='#f' && this.#a.name==='#a' && this.#c.name==='#c' && this.#c.seen==='#c' && this.#x===7 && this.#t===undefined;}}new C().run()",
    );
    check("class C{#x=7;#y=eval('this.#x');read(){return this.#y;}}new C().read()===7");
}

#[test]
fn tracing_retains_private_values_and_closures_then_reclaims_cycles() {
    let mut realm = Realm::default();
    realm.eval("let f;(function(){class C{#x={n:7};#self=this;read(){return this.#x.n;}make(){return ()=>eval('this.#x.n');}}f=new C().make();})();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(7.0)));
    realm.eval("f=undefined").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 5);
    realm
        .eval("let C=(function(){let n={v:8};return class{#x=n;read(){return this.#x.v;}};})();")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("new C().read()"), Ok(Value::Number(8.0)));
}

#[test]
fn recursive_initializers_and_eval_are_bounded_on_a_two_mebibyte_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for source in [
                "class C{#x=new C;}new C;",
                "class B{}class C extends B{#x=new C;}new C;",
                "function f(){class C{static #x=f();}}f();",
                "class C{#x=()=>eval('this.#x()');run(){return this.#x();}}new C().run();",
            ] {
                assert!(
                    matches!(Realm::default().eval(source), Err(Error::Limit { .. })),
                    "{source}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn language_and_host_aborts_restore_private_scope_and_caller_state() {
    check(
        "let caught=false;try{class C{static #x=(()=>{throw 7;})();}}catch(e){caught=e===7;}after=8;caught && after===8",
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{class C{static #x=eval('function* gap(){}');}}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=8"), Ok(Value::Number(8.0)));
    assert_eq!(
        realm.eval("class C{#x=7;read(){return this.#x;}}new C().read()"),
        Ok(Value::Number(7.0))
    );
}
