//! Labelled source statements and direct iteration label chains.

use spite_runtime::{PromiseJobOutcome, Realm, Value};

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

fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn labelled_block_break_unwinds_resources_after_await_and_skips_the_remaining_body() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],later=0,result;
        async function f(){outer:{using r={[Symbol.dispose](){events.push('dispose')}};{await 0;break outer;later++}later++}events.push('after');return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='dispose,after'&&later===0&&result===7",
    );
}

#[test]
fn direct_label_chains_match_while_and_do_while_continues_after_finalizers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],first,second,tests=0;
        async function f(){let i=0;firstLabel:secondLabel:while(await(i<3)){const n=i++;try{await 0;if(n<2)continue firstLabel;break secondLabel}finally{events.push(n);await 0}}return i}
        async function g(){let j=0;a:b:do{await 0;if(++j<3)continue b;break a}while(await(++tests,true));return j}
        f().then(value=>{first=value});g().then(value=>{second=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===3&&second===3&&tests===2&&events.join(',')==='0,1,2'",
    );
}

#[test]
fn labelled_for_of_continue_keeps_the_iterator_open_and_labelled_break_closes_once() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let steps=0,closed=0,result,source={[Symbol.iterator](){return{next(){return{done:++steps>3,value:steps}},return(){closed++;return{}}}}};
        async function f(){outer:for(const value of source){await 0;if(value<3)continue outer;break outer}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "steps===3&&closed===1&&result===7");
}

#[test]
fn continuing_an_outer_for_of_closes_only_inner_iterators_after_their_finalizers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;
        function input(name){return{[Symbol.iterator](){let n=0;return{next(){return{done:++n>2,value:n}},return(){events.push('close-'+name);return{}}}}}}
        let outerSource=input('outer'),innerSource=input('inner');
        async function f(){outer:for(const x of outerSource){for(const y of innerSource){try{await 0;if(x===1)continue outer;break outer}finally{events.push('finally'+x);await 0}}}return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&events.join(',')==='finally1,close-inner,finally2,close-inner,close-outer'",
    );
}

#[test]
fn empty_labelled_blocks_preserve_the_parent_statement_value_without_update_empty() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result;state.first=new Promise(resolve=>{state.releaseFirst=resolve});state.second=new Promise(resolve=>{state.releaseSecond=resolve});
        function make(){const value={x:7};state.weak=new WeakRef(value);return value}
        async function f(){make();label:{}const x=await state.first;0;const y=await state.second;return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn labels_add_no_binding_environment_and_keep_private_eval_this_and_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let alias,result;
        async function f(x){label:var x=await 7;return arguments[0]}
        class C{#x=10;async read(){outer:{await 0;break outer}return eval('this.#x')+arguments.length}}
        f(1).then(value=>{alias=value});new C().read('arg').then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "alias===7&&result===11");
}
