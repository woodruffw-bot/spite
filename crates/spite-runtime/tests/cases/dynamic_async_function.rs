//! Dynamic AsyncFunction conversion, prototype selection, scope and source.

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}
fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn calls_and_construction_produce_async_functions_with_standard_source_and_fresh_promises() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let AF=(async function(){}).constructor,values=[],f=AF('x','y=1','return x+y'),g=new AF('return 7');f(2).then(v=>{values.push(v)});g().then(v=>{values.push(v)});f.name==='anonymous'&&f.length===1&&!Object.hasOwn(f,'prototype')&&Object.getPrototypeOf(f)===AF.prototype&&f.toString()==='async function anonymous(x,y=1\\n) {\\nreturn x+y\\n}'"),Ok(Value::Boolean(true)));
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("3,7".into()))
    );
    realm
        .eval(
            "let empty=AF(),undefinedResult=false;empty().then(v=>{undefinedResult=v===undefined})",
        )
        .unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("undefinedResult"), Ok(Value::Boolean(true)));
}

#[test]
fn conversions_finish_before_parsing_and_newtarget_prototype_lookup() {
    check(
        r#"
        let AF=(async function(){}).constructor,log='',proto={},target=(function(){}).bind(null);
        Object.defineProperty(target,'prototype',{get(){log+='p';return proto}});
        let f=Reflect.construct(AF,[{toString(){log+='a';return'x'}},{toString(){log+='b';return'y'}},{toString(){log+='c';return'return x+y'}}],target);
        let correct=log==='abcp'&&Object.getPrototypeOf(f)===proto;
        log='';let caught=false;
        try{Reflect.construct(AF,[{toString(){log+='a';return'x)'}},{toString(){log+='b';return'return 1'}}],target)}catch(e){caught=e instanceof SyntaxError}
        correct&&caught&&log==='ab'
    "#,
    );
}

#[test]
fn subclass_and_primitive_prototype_fallback_preserve_internal_function_kind() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let AF=(async function(){}).constructor,values=[],target=(function(){}).bind(null);target.prototype=null;let fallback=Reflect.construct(AF,['return 3'],target);class C extends AF{}let subclass=new C('return 4');let asyncOrdinaryProto=Reflect.construct(AF,['return 5'],Function),ordinaryAsyncProto=Reflect.construct(Function,['return 6'],AF);fallback().then(v=>{values.push(v)});subclass().then(v=>{values.push(v)});asyncOrdinaryProto().then(v=>{values.push(v)});Object.getPrototypeOf(fallback)===AF.prototype&&subclass instanceof C&&Object.getPrototypeOf(asyncOrdinaryProto)===Function.prototype&&Object.getPrototypeOf(ordinaryAsyncProto)===AF.prototype&&ordinaryAsyncProto()===6&&Object.hasOwn(ordinaryAsyncProto,'prototype')&&!Object.hasOwn(asyncOrdinaryProto,'prototype')"),Ok(Value::Boolean(true)));
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')"),
        Ok(Value::String("3,4,5".into()))
    );
}

#[test]
fn dynamic_scope_and_strictness_are_independent_of_the_constructor_caller() {
    let mut realm = Realm::default();
    realm.eval("let AF=(async function(){}).constructor,globalValue=9,anonymous=8,values=[];function build(){'use strict';let globalValue=1;return AF('a,a','return[globalValue,a,anonymous,this===globalThis]')}let f=build();f(1,2).then(a=>{values=a});let strict=AF('\"use strict\";return this'),strictResult=false;strict.call(7).then(v=>{strictResult=v===7})").unwrap();
    realm.collect(usize::MAX).unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("values.join(',')==='9,2,8,true'&&strictResult"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn generated_source_preserves_utf16_literals_and_comment_code_points() {
    let mut realm = Realm::default();
    realm.eval("let AF=(async function(){}).constructor,lone=String.fromCharCode(0xd800),body=\"return '\"+lone+\"'\",f=AF('x/*'+lone+'*/',body),result;f().then(v=>{result=v});if(f.toString().split(lone).length!==3)throw 1").unwrap();
    realm.collect(usize::MAX).unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result.length===1&&result.charCodeAt(0)===0xd800&&f.toString().startsWith('async function anonymous(')"),Ok(Value::Boolean(true)));
}

#[test]
fn syntax_errors_throw_synchronously_and_async_source_overhead_obeys_opt_in_limits() {
    check(
        "let AF=(async function(){}).constructor,caught=0;for(const args of [['await','return 1'],['a,a','\"use strict\"'],['a=1','\"use strict\"'],['a) {}','return 1'],['a','return a; } evil(); {']]){try{AF(...args)}catch(e){if(e instanceof SyntaxError)caught++}}caught===5",
    );
    for limits in [
        Limits {
            max_source_bytes: Some(100),
            ..Limits::default()
        },
        Limits {
            max_string_units: Some(100),
            ..Limits::default()
        },
    ] {
        let mut realm = Realm::new(limits);
        realm
            .eval("let AF=(async function(){}).constructor,flag=0")
            .unwrap();
        let result = realm.eval("try{AF('return 1'.repeat(9))}catch(e){flag=1}finally{flag=2}");
        assert!(
            matches!(result, Err(Error::Limit { .. })),
            "{limits:?}: {result:?}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
