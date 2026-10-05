//! Public field evaluation and construction ordering (15.7.10/14, 7.3.32/33).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn fields_create_own_enumerable_data_properties_without_inherited_setters() {
    check(
        "let calls=0;class B{set x(v){calls++;}}class C extends B{x=1;y;__proto__=7;x=2;}let c=new C,d=Object.getOwnPropertyDescriptor(c,'x');calls===0 && c.x===2 && c.y===undefined && Object.hasOwn(c,'y') && c.__proto__===7 && Object.getPrototypeOf(c)===C.prototype && d.writable && d.enumerable && d.configurable",
    );
    check(
        "class C{static x=1;static x(){return 2;}static x=3;}let d=Object.getOwnPropertyDescriptor(C,'x');C.x===3 && d.enumerable && d.writable && d.configurable",
    );
    check(
        "class C{2=2;1=1;z=3;['constructor']=4;}Object.keys(new C).join(',')==='1,2,z,constructor'",
    );
}

#[test]
fn computed_names_and_methods_finish_before_static_initializers() {
    check(
        "let log='',n=0;class C{[(log+='A',++n)]=(log+='I',1);static [(log+='B',++n)]=(log+='S',2);[(log+='M','m')](){}static y=(log+='T',C);}let a=new C,b=new C;log==='ABMSTII' && n===2 && C[2]===2 && C.y===C && a[1]===1 && b[1]===1",
    );
    check(
        "let caught=false;try{class C{[C]=1;}}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    check("let C=class{static x=this.name;};C.x==='C'");
    check("class C{static x=C.name;static m(){return C;}}C.x==='C' && C.m()===C");
}

#[test]
fn base_fields_precede_parameter_defaults_and_capture_definition_scope() {
    check(
        "let log='',x=7;class C{v=(log+='F',x);constructor(x=(log+='P',this.v)){log+='B';this.param=x;}}let c=new C;log==='FPB' && c.v===7 && c.param===7",
    );
    check(
        "let x=7;class C{value=()=>x;constructor(x=9){this.parameter=()=>x;}}let c=new C;c.value()===7 && c.parameter()===9",
    );
}

#[test]
fn derived_fields_follow_bind_this_and_precede_the_rest_of_the_body() {
    check(
        "let log='';class B{x=(log+='b',1);constructor(){log+='B';}}class D extends B{x=(log+='d',2);constructor(){log+='A';super();log+='D';}}let d=new D;log==='AbBdD' && d.x===2",
    );
    check(
        "let fields=0,constructs=0;class B{constructor(){constructs++;}}class D extends B{x=++fields;constructor(){super();try{super();}catch(e){if(!(e instanceof ReferenceError))throw e;}}}let d=new D;fields===1 && constructs===2 && d.x===1",
    );
    check(
        "let log='';class D extends Object{a=(log+='A',1);b=(function(){throw 7;})();c=(log+='C',3);constructor(){try{super();}catch(e){if(e!==7)throw e;this.saved=this.a;}}}let d=new D;log==='A' && d.a===1 && d.saved===1 && !Object.hasOwn(d,'b') && !Object.hasOwn(d,'c')",
    );
}

#[test]
fn constructor_object_returns_observe_the_correct_initialization_receiver() {
    check(
        "let receiver,o={};class B{x=1;constructor(){receiver=this;return o;}}new B===o && receiver.x===1 && !Object.hasOwn(o,'x')",
    );
    check(
        "let n=0,o={};class D extends Object{x=++n;constructor(){return o;}}new D===o && n===0 && !Object.hasOwn(o,'x')",
    );
    check("let o={};class B{constructor(){return o;}}class D extends B{x=7;}new D===o && o.x===7");
}

#[test]
fn default_forwarding_initializes_fields_from_base_to_derived_without_iteration() {
    check(
        "let log='';class A{x=(log+='A',1);}class B extends A{x=(log+='B',2);}class C extends B{x=(log+='C',3);}Array.prototype[Symbol.iterator]=function(){throw 7;};let c=new C;log==='ABC' && c.x===3",
    );
    check(
        "let n=0,C=class{x=++n;};for(let i=0;i<100;i++){C=class extends C{x=++n;};}new C().x===101 && n===101",
    );
    check(
        "class B{x=7;}class D extends B{y=8;}let F=D.bind(null);function N(){}let d=Reflect.construct(F,[],N);d.x===7 && d.y===8 && Object.getPrototypeOf(d)===N.prototype",
    );
}

#[test]
fn initializer_this_super_and_new_target_have_their_own_function_context() {
    check(
        "class B{get x(){return this.value+1;}static x=8;}class D extends B{value=6;y=super.x;target=new.target;arrow=()=>this;static y=super.x;static target=new.target;static arrow=()=>this;}let d=new D;d.y===7 && d.target===undefined && d.arrow()===d && D.y===8 && D.target===undefined && D.arrow()===D",
    );
    check(
        "class B{get x(){return this.v;}}class D extends B{v=7;x=eval('super.x');t=eval('new.target');static t=eval('new.target');}let d=new D;d.x===7 && d.t===undefined && D.t===undefined",
    );
    check("function f(k){return class {[arguments[0]]=1;};}Object.hasOwn(new (f('x')),'x')");
}

#[test]
fn direct_eval_rejects_arguments_across_arrows_but_respects_ordinary_boundaries() {
    for source in [
        "arguments",
        "()=>arguments",
        "({arguments})",
        "({[arguments](){}})",
    ] {
        check(&format!(
            "let effects=0;class C{{x=(()=>{{try{{eval(\"effects=1;{source}\");}}catch(e){{return e instanceof SyntaxError;}}}})();}}new C().x && effects===0"
        ));
    }
    check(
        "class C{x=function(){return arguments[0];};y=({m(){return arguments[0];}});z=eval('(function(){return arguments.length;})()');}let c=new C;c.x(7)===7 && c.y.m(8)===8 && c.z===0",
    );
    check(
        "class C{x=()=>eval('arguments');}let f=new C().x,caught=false;try{f();}catch(e){caught=e instanceof SyntaxError;}caught",
    );
    check(
        "class C{x=(()=>{let caught=false;try{eval('super()');}catch(e){caught=e instanceof SyntaxError;}return caught;})();}new C().x",
    );
}

#[test]
fn initializer_named_evaluation_uses_exact_string_and_symbol_names() {
    check(
        "let s=Symbol('key');class C{f=function(){};a=()=>{};c=class{static nameAtCreation=this.name;};[s]=()=>{};static f=function(){};}let c=new C;c.f.name==='f' && c.a.name==='a' && c.c.name==='c' && c.c.nameAtCreation==='c' && c[s].name==='[key]' && C.f.name==='f'",
    );
    check("let key=String.fromCharCode(55296);class C{[key]=()=>{};}new C()[key].name===key");
}

#[test]
fn field_definition_errors_follow_initializer_effects_and_stop_later_fields() {
    check(
        "let log='',o=Object.preventExtensions({});class B{constructor(){return o;}}class D extends B{x=(log+='I',1);y=(log+='Y',2);}let caught=false;try{new D;}catch(e){caught=e instanceof TypeError;}caught && log==='I'",
    );
    check(
        "let log='',o={};Object.defineProperty(o,'x',{value:0});class B{constructor(){return o;}}class D extends B{x=(log+='I',1);}let caught=false;try{new D;}catch(e){caught=e instanceof TypeError;}caught && log==='I' && o.x===0",
    );
    check(
        "let log='',saved,caught=false;try{class C{static [(saved=()=>C,'prototype')]=(log+='I',1);static y=(log+='Y',2);}}catch(e){caught=e instanceof TypeError;}caught && log==='I' && typeof saved()==='function'",
    );
    check(
        "class C extends Array{length=2;}let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn contextual_names_and_automatic_semicolons_remain_fields_or_methods() {
    check(
        "class C{get\nx;set\ny;async\nm(){return 7;}static\nz=8;}let c=new C;Object.keys(c).join(',')==='get,x,set,y,async' && c.m()===7 && C.z===8",
    );
    check("class C{get\nx(){return 7;}set\nx(v){this.v=v;}}let c=new C;c.x=8;c.x===7 && c.v===8");
}

#[test]
fn field_initializer_captures_and_escaped_arrows_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let C=(function(){let value={n:7};return class {x=value;f=()=>this.x.n;};})(),c=new C,f=c.f;C=undefined;c=undefined;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(7.0)));
    realm.eval("f=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm
        .eval("let D=(function(){let value={n:8};return class{x=value;};})();")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("new D().x.n"), Ok(Value::Number(8.0)));
}

#[test]
fn language_and_host_failures_restore_the_callers_context() {
    check(
        "let x=7,caught=false;try{class C{static x=(()=>{throw 9;})();}}catch(e){caught=e===9;}after=8;caught && x===7 && after===8",
    );
    let mut realm = Realm::default();
    realm
        .eval("let flag=0;class C{x=eval('class P{#x;}');}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{new C;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=8"), Ok(Value::Number(8.0)));
}
