//! Suspended class expressions and NamedEvaluation before static initialization.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value, object::PromiseState};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn bindings_and_pattern_defaults_name_classes_before_static_initialization() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function f(){
            let A=class extends (await Object){static seen=this.name};
            var B=(class{[await 'm'](){}static seen=this.name});
            let [C=class{[await 'm'](){}static seen=this.name}]=[];
            let {q:D=class{[await 'm'](){}static seen=this.name}}={};
            let names=[A.seen,B.seen,C.seen,D.seen];
            for(let [E=class{[await 'm'](){}static seen=this.name}] of [[]])names.push(E.seen);
            try{throw []}catch([F=class{[await 'm'](){}static seen=this.name}]){names.push(F.seen)}
            return names;
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='A,B,C,D,E,F'");
}

#[test]
fn object_keys_name_classes_while_awaited_and_explicitly_named_values_keep_their_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function f(){
            let key=Symbol('key');
            let object={plain:class{[await 'm'](){}static seen=this.name},
                [key]:(class extends (await Object){static seen=this.name}),
                explicit:class Inner{[await 'm'](){}static seen=this.name},
                awaited:await(class{[await 'm'](){}static seen=this.name})};
            let protoLiteral={__proto__:class{[await 'm'](){}static seen=this.name}};
            let protoData={['__proto__']:class{[await 'm'](){}static seen=this.name}};
            return [object.plain.seen,object[key].seen,object.explicit.seen,object.awaited.seen,Object.getPrototypeOf(protoLiteral).seen,protoData['__proto__'].seen];
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result.join(',')==='plain,[key],Inner,,,__proto__'",
    );
}

#[test]
fn identifier_assignment_infers_names_and_property_or_parenthesized_targets_do_not() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function f(){
            let A,B=0,C=null,D,object={};
            A=class{[await 'm'](){}static seen=this.name};
            B||=class extends(await Object){static seen=this.name};
            C??=(class{[await 'm'](){}static seen=this.name});
            (D)=class{[await 'm'](){}static seen=this.name};
            object.x=class{[await 'm'](){}static seen=this.name};
            return [A.seen,B.seen,C.seen,D.seen,object.x.seen];
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='A,B,C,,'");
}

#[test]
fn nested_class_heritage_and_key_expressions_preserve_outer_tasks_and_internal_tdz() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,read,release;let gate=new Promise(r=>release=r);
        async function f(){
            let C=class Outer extends (read=()=>Outer,await gate,await(class Base{[await 'base'](){return 3}})){
                [(await(class Key{static[await 'keyName']='m'})).keyName](){return this.base()+4}
                static seen=this.name;
            };
            let internal=read();C=99;
            return [internal,new internal().m(),internal.seen,C];
        }
        f().then(v=>result=v);
    "#).unwrap();
    check(
        &mut realm,
        "(()=>{try{read()}catch(e){return e instanceof ReferenceError}})()",
    );
    realm.collect(usize::MAX).unwrap();
    realm.eval("release()").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result[1]===7&&result[2]==='Outer'&&result[3]===99&&read()===result[0]",
    );
}

#[test]
fn rejected_nested_expression_classes_unwind_before_enclosing_declaration_contexts() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},events=[];
        async function f(declaration){
            try{
                if(declaration){class Outer{static field=events.push('wrong');[await(class Inner{[await Promise.reject(marker)](){}})](){}}}
                else{let C=class Outer{static field=events.push('wrong');[await(class Inner{[await Promise.reject(marker)](){}})](){}}}
            }catch(e){nestedClassSloppy=7;events.push(e===marker)}finally{events.push(nestedClassSloppy===7)}
        }
        f(true);f(false);
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.length===4&&events.every(Boolean)&&nestedClassSloppy===7",
    );
}

#[test]
fn expression_class_captures_survive_collection_and_host_abort_releases_them() {
    let mut realm = Realm::default();
    realm.eval("let weak,flag=0;async function f(){let key=Symbol();weak=new WeakRef(key);try{let C=class{[key]=1;[(key=null,await 0,import('unavailable'))](){}}}catch{flag=1}finally{flag=2}}let pending=f()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weak.deref()!==undefined&&flag===0");
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
    ));
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weak.deref()===undefined&&flag===0&&(expressionClassSloppy=7)===7",
    );
}

#[test]
fn yielded_expression_classes_preserve_inferred_names_and_handle_return_and_throw() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],marker={};
        function* g(){try{let C=class extends(yield 'base'){[yield 'key'](){return 8}static seen=this.name};return C}
        catch(e){events.push(e===marker)}finally{expressionGeneratorSloppy=4;events.push('finally')}}
        let success=g();if(success.next().value!=='base'||success.next(Object).value!=='key')throw 1;
        let C=success.next('m').value;if(C.seen!=='C'||new C().m()!==8)throw 2;
        let returned=g();returned.next();returned.next(Object);let end=returned.return(99);if(!end.done||end.value!==99)throw 3;
        let thrown=g();thrown.next();thrown.next(Object);thrown.throw(marker);
        events.join(',')==='finally,finally,true,finally'&&expressionGeneratorSloppy===4
    "#,
    );
}

#[test]
fn parenthesized_default_export_names_expression_classes_before_static_elements() {
    let mut realm = Realm::default();
    realm.eval("let observed=[]").unwrap();
    let promise=realm.eval_module("export default (class extends(await Object){[await 'm'](){return 7}static record=observed.push(this.name,new this().m())});observed.push('body')").unwrap();
    let Value::Object(promise) = promise else {
        panic!("evaluation promise")
    };
    check(&mut realm, "observed.length===0");
    drain(&mut realm);
    assert!(matches!(
        realm.inspect_object(&promise).unwrap().promise_state(),
        Some(PromiseState::Fulfilled(Value::Undefined))
    ));
    check(&mut realm, "observed.join(',')==='default,7,body'");
}
