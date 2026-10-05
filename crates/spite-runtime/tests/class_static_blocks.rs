//! Static initialization order, declaration instantiation, and captured contexts.

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn computed_names_and_methods_precede_ordered_static_fields_and_blocks() {
    check(
        "let log='';class C{static[(log+='K','x')]=(log+='F',1);static{log+='B';this.y=this.m();}static y=(log+='Y',this.y+1);static[(log+='M','m')](){return 6;}static{log+='C';this.z=this.x+this.y;}}log==='KMFBYC' && C.z===8",
    );
    check(
        "let log='';class C{static x=(log+='A',1);static{log+='B';}static y=(log+='C',2);static{log+='D';}}log==='ABCD'",
    );
}

#[test]
fn each_block_has_its_own_hoisted_variable_and_lexical_scope() {
    check(
        "let x=7;class C{static{this.before=x;var x=1;let y=2;this.f=()=>x+y;function f(){return 3;}this.v=f();}static{this.before2=x;var x=4;this.g=()=>x;}}C.before===undefined && C.before2===undefined && C.f()===3 && C.g()===4 && C.v===3 && x===7 && typeof y==='undefined' && typeof f==='undefined'",
    );
    check("class C{static{this.x=f();function f(){return 1;}function f(){return 2;}}}C.x===2");
}

#[test]
fn this_super_and_new_target_use_the_synthetic_function_context() {
    check(
        "class B{static get x(){return this.v+1;}static m(){return this.v+2;}}class C extends B{static v=6;static{this.result=super.x;this.y=super.m();this.target=new.target;this.f=()=>this;}}C.result===7 && C.y===8 && C.target===undefined && C.f()===C",
    );
    check("class C{static{this.f=()=>super.x;}}Object.setPrototypeOf(C,{x:7});C.f()===7");
}

#[test]
fn direct_eval_observes_strictness_and_ordinary_static_function_arguments() {
    check(
        "let x=7;class B{static x=8;}class C extends B{static{this.x=eval('super.x');this.target=eval('new.target');this.args=eval('arguments.length');this.f=()=>eval('arguments.length');eval('var x=9;');this.outer=x;}}C.x===8 && C.target===undefined && C.args===0 && C.f()===0 && C.outer===7 && x===7",
    );
    check(
        "class C{static{this.x=(function(x){return arguments[0];})(7);let caught=false;try{eval('super()');}catch(e){caught=e instanceof SyntaxError;}this.caught=caught;}}C.x===7 && C.caught",
    );
    check(
        "let await=7;class C{static{this.x=eval('await');this.y=(()=>await)();this.z=(()=>{let await=8;return await;})();}}C.x===7 && C.y===7 && C.z===8",
    );
}

#[test]
fn block_control_flow_is_local_and_normal_statement_values_are_discarded() {
    check(
        "class C{static{let n=0;label:for(let i=0;i<4;i++){if(i===1)continue;if(i===3)break label;n+=i;}try{throw 7;}catch(e){n+=e;}finally{n++;}this.n=n;123;}}C.n===10 && typeof C==='function'",
    );
}

#[test]
fn thrown_blocks_preserve_completed_elements_and_the_internal_class_name() {
    check(
        "let saved,log='',caught=false;try{class C{static x=(log+='A',1);static{saved=()=>C;log+='B';throw 7;}static y=(log+='Y',2);static m(){return 3;}}}catch(e){caught=e===7;}let D=saved();caught && log==='AB' && D.x===1 && !Object.hasOwn(D,'y') && D.m()===3",
    );
    check(
        "let outer,saved;try{outer=()=>C;class C{static{saved=()=>C;throw 7;}}}catch(e){}let caught=false;try{outer();}catch(e){caught=e instanceof ReferenceError;}caught && typeof saved()==='function'",
    );
    let mut realm = Realm::default();
    realm
        .eval("try{class C{static{throw 7;}}}catch(e){}")
        .unwrap();
    assert_eq!(
        realm.eval("typeof C"),
        Ok(Value::String("undefined".into()))
    );
}

#[test]
fn instance_fields_are_installed_before_static_blocks_construct_instances() {
    check("class C{static{this.instance=new C;}x=7;}C.instance.x===7 && C.instance instanceof C");
    check(
        "class B{x=1;}class C extends B{static{this.instance=new C;}y=2;}C.instance.x===1 && C.instance.y===2",
    );
}

#[test]
fn class_names_remain_immutable_and_class_source_retains_static_blocks() {
    check(
        "let caught=false;class C{static{try{C=7;}catch(e){caught=e instanceof TypeError;}this.same=this===C;}}caught && C.same",
    );
    check(
        "let source='class C { static /*x*/ { this.x=7; } }',C=eval('('+source+')');C.x===7 && C.toString()===source",
    );
}

#[test]
fn escaped_block_closures_keep_block_bindings_home_objects_and_this_alive() {
    let mut realm = Realm::default();
    realm.eval("let f;(function(){class B{static v=7;}class C extends B{static{let x=1;f=()=>super.v+x+this.extra;this.extra=2;}}})();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(10.0)));
    realm.eval("f=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn static_language_and_host_failures_restore_the_caller() {
    check("let x=7;try{class C{static{throw 9;}}}catch(e){}after=8;x===7 && after===8");
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{class C{static{eval('class P{#x;}');}}}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=8"), Ok(Value::Number(8.0)));
}
