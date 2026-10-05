//! Private method/accessor brands and evaluation (15.7.14, 7.3.27–33).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn method_values_are_shared_strict_nonconstructible_and_keep_exact_metadata() {
    check(
        "class C{#m /*comment*/ (v=7){return [this,v,arguments.length,new.target];}get(){return this.#m;}run(){return this.#m(8);}}let a=new C,b=new C,f=a.get(),r=a.run(),caught=false;try{new f;}catch(e){caught=e instanceof TypeError;}caught && f===b.get() && f.name==='#m' && f.length===0 && f.toString()==='#m /*comment*/ (v=7){return [this,v,arguments.length,new.target];}' && !Object.hasOwn(f,'prototype') && f() [0]===undefined && r[0]===a && r[1]===8 && r[2]===1 && r[3]===undefined",
    );
    check(
        "class C{#m(a,b,c=1){return a+b;}run(){return this.#m.name==='#m' && this.#m.length===2 && this.#m(3,4)===7;}}new C().run()",
    );
}

#[test]
fn every_instance_method_and_accessor_is_installed_before_fields() {
    check(
        "class C{a=this.#m();b=this.#x;#m(){return 7;}get #x(){return 8;}has(){return #m in this && #x in this;}}let c=new C;c.a===7 && c.b===8 && c.has()",
    );
    check(
        "class C{a=(this.#x=7,this.#x);set #x(v){this.value=v;}get #x(){return this.value;}}new C().a===7",
    );
    check(
        "let o={},caught=false;class B{constructor(){return o;}}class C extends B{a=(()=>{throw 9;})();#m(){return 7;}get #x(){return 8;}static has(o){return #m in o && #x in o;}static read(o){return o.#m()+o.#x;}}try{new C;}catch(e){caught=e===9;}caught && C.has(o) && C.read(o)===15",
    );
}

#[test]
fn static_methods_and_accessor_pairs_precede_all_static_fields_and_blocks() {
    check(
        "class C{static a=this.#m();static{this.b=this.#x;this.#x=8;}static #m(){return 7;}static set #x(v){this.value=v;}static get #x(){return this.value===undefined?6:this.value;}static has(o){return #m in o && #x in o;}}class D extends C{}C.a===7 && C.b===6 && C.value===8 && C.has(C) && !C.has(D) && !C.has(new C)",
    );
    check(
        "let saved,caught=false;try{class C{static{saved=this;throw 9;}static #m(){return 7;}static read(){return this.#m();}}}catch(e){caught=e===9;}caught && saved.read()===7",
    );
}

#[test]
fn missing_brands_getters_and_setters_and_method_writes_throw_type_errors() {
    check(
        "class C{#m(){return 7;}get #read(){return 8;}set #write(v){this.v=v;}run(){let n=0;for(let f of [()=>this.#m=1,()=>this.#read=1,()=>this.#write]){try{f();}catch(e){if(e instanceof TypeError)n++;}}this.#write=9;return n===3 && this.v===9 && #write in this && #read in this && #m in this;}}new C().run()",
    );
    check(
        "class C{#m(){}get #x(){return 7;}read(o){return o.#x;}call(o){return o.#m();}has(o){return #m in o;}}let c=new C,n=0;for(let o of [{},Object.create(c),null,undefined,1,'x',Symbol(),1n]){for(let f of [()=>c.read(o),()=>c.call(o)]){try{f();}catch(e){if(e instanceof TypeError)n++;}}}n===16 && !c.has(Object.create(c))",
    );
    check(
        "let effects=0;class C{#m(){}run(){try{this.#m=(effects++,1);}catch(e){return e instanceof TypeError;}}}new C().run() && effects===1",
    );
}

#[test]
fn private_accessor_calls_preserve_receiver_and_abrupt_completion_identity() {
    check(
        "class C{#v=1;get #x(){return this.#v;}set #x(v){this.#v=v;}run(){let old=this.#x++;this.#x+=3;this.#x&&=7;this.#x||=9;this.#x??=10;return old===1 && this.#x===7;}}new C().run()",
    );
    check(
        "let thrown={};class C{get #x(){throw thrown;}set #y(v){throw thrown;}run(){let n=0;try{this.#x;}catch(e){if(e===thrown)n++;}try{this.#y=7;}catch(e){if(e===thrown)n++;}return n===2;}}new C().run()",
    );
    check(
        "class C{get #f(){return function(){return this;};}run(o){return o?.#f?.();}}let c=new C;c.run(c)===c && c.run(null)===undefined",
    );
}

#[test]
fn method_and_accessor_brands_use_fresh_names_with_inheritance_and_shadowing() {
    check(
        "function make(){return class{#m(){return 7;}get #x(){return 8;}static has(o){return #m in o && #x in o;}read(o){return o.#m()+o.#x;}};}let A=make(),B=make(),a=new A,b=new B,caught=false;try{a.read(b);}catch(e){caught=e instanceof TypeError;}caught && A.has(a) && !A.has(b) && B.has(b)",
    );
    check(
        "class B{#m(){return 7;}get #x(){return 8;}readB(){return this.#m()+this.#x;}}class D extends B{#m(){return 1;}get #x(){return 2;}readD(){return this.#m()+this.#x;}}let d=new D;d.readB()===15 && d.readD()===3",
    );
    check(
        "class O{#m(){return 7;}make(){return class{read(o){return o.#m();}};}}let o=new O,C=o.make();new C().read(o)===7",
    );
}

#[test]
fn super_new_target_and_direct_eval_use_captured_private_method_contexts() {
    check(
        "class B{m(){return this.v+1;}static m(){return this.v+2;}}class C extends B{v=6;#m(){return super.m();}get #x(){return eval('super.m()');}read(){return this.#m()===7 && this.#x===7 && eval('this.#m()')===7;}static v=6;static #sm(){return super.m();}static read(){return eval('this.#sm()')===8;}}new C().read() && C.read()",
    );
    check(
        "class C{#m(){return new.target;}get #x(){return new.target;}run(){let o=this;function f(){return eval('o.#m()');}return f()===undefined && eval('this.#x')===undefined && eval('#m in this');}}new C().run()",
    );
    check(
        "class C{#m(){return 7;}get #x(){return 8;}run(){return eval('(()=>this.#m()+this.#x)()');}}new C().run()===15",
    );
}

#[test]
fn duplicate_method_stamping_rejects_before_any_field_initializer() {
    check(
        "let o=Object.freeze({}),effects=0;class B{constructor(){return o;}}class C extends B{x=(effects++,7);#m(){return 8;}get #y(){return 9;}static read(o){return o.#m()+o.#y;}}let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught && effects===1 && C.read(o)===17",
    );
    check(
        "let o={},effects=0;class B{constructor(){return o;}}class C extends B{#x=(effects++,7);#m(){return 8;}get #y(){return 9;}static read(o){return o.#x+o.#m()+o.#y;}}new C;let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught && effects===1 && C.read(o)===24",
    );
}

#[test]
fn methods_only_default_derived_forwarding_and_parameter_timing_install_brands() {
    check(
        "class B{#m(){return 7;}constructor(v=this.#m()){this.v=v;}static has(o){return #m in o;}}let C=B;for(let i=0;i<100;i++){C=class extends C{#m(){return 8;}hasOwn(){return #m in this;}};}function N(){}let bound=C.bind(null),c=Reflect.construct(bound,[],N);B.has(c) && c.v===7 && C.prototype.hasOwn.call(c) && Object.getPrototypeOf(c)===N.prototype",
    );
    check(
        "class B{}class C extends B{#m(){return 7;}constructor(){super();this.v=this.#m();}static has(o){return #m in o;}}let c=new C;c.v===7 && C.has(c)",
    );
    check(
        "class B{}class C extends B{#m(){return 7;}constructor(){return {};}static has(o){return #m in o;}}!C.has(new C)",
    );
}

#[test]
fn methods_and_accessors_are_hidden_and_work_on_frozen_receivers() {
    check(
        "let o=Object.freeze({});class B{constructor(){return o;}}class C extends B{#m(){return 7;}get #x(){return this.#m();}static read(o){return o.#x;}}new C;C.read(o)===7 && Reflect.ownKeys(o).length===0 && JSON.stringify(o)==='{}'",
    );
    check(
        "class C{#m(){return 7;}get #x(){return 8;}}Reflect.ownKeys(new C).length===0 && Reflect.ownKeys(C.prototype).join(',')==='constructor'",
    );
}

#[test]
fn accessor_destructuring_and_loop_writes_follow_property_evaluation_order() {
    check(
        "let log='';class C{#v;get #x(){return this.#v;}set #x(v){log+='S';this.#v=v;}run(){({a:this.#x}={get a(){log+='G';return 7;}});for(this.#x of [8]){}return this.#x===8;}}new C().run() && log==='GSS'",
    );
    check(
        "let token={};class C{set #x(v){}run(o){({a:o.#x}={get a(){throw token;}});}}let caught=false;try{new C().run({});}catch(e){caught=e===token;}caught",
    );
}

#[test]
fn tracing_retains_methods_accessor_functions_homes_and_private_cycles() {
    let mut realm = Realm::default();
    realm.eval("let f,g,C;(function(){let value={n:7};class B{get x(){return this.#unused;}#unused;}C=class extends B{#m(){return value.n;}get #x(){return this.#m();}set #x(v){value.n=v;}get(){return this.#m;}read(){return this.#x;}write(v){this.#x=v;}};let c=new C;f=c.get();g=()=>c.read();})();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()+g()"), Ok(Value::Number(14.0)));
    realm.eval("new C().write(8)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()+g()"), Ok(Value::Number(16.0)));
    realm.eval("f=undefined;g=undefined;C=undefined;").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 8);
}

#[test]
fn recursive_private_calls_and_accessors_are_bounded_on_two_mebibyte_stacks() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for source in [
                "class C{#m(){return this.#m();}run(){return this.#m();}}new C().run();",
                "class C{get #x(){return this.#x;}run(){return this.#x;}}new C().run();",
                "class C{set #x(v){this.#x=v;}run(){this.#x=7;}}new C().run();",
                "class C{#m(){return eval('this.#m()');}run(){return this.#m();}}new C().run();",
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
