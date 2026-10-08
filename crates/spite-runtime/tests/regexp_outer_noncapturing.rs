//! Whole noncapturing wrappers preserve Pattern text and inner matching order.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn repeated_continuations_choices_and_assertions_reuse_complete_body_semantics() {
    check(
        "let a=/(?:a+b)/.exec('xaaab'),b=/(?:(?:a+|b))/.exec('baa'),c=/(?:(?:a+?|aa))/.exec('aaa');a[0]==='aaab'&&a.index===1&&b[0]==='b'&&c[0]==='a'",
    );
    check(
        r"let a=/(?:^a+$)/m.exec('x\naaa\ny'),b=/(?:(?:^a+?aa$))/.exec('aaaa');a.index===2&&a[0]==='aaa'&&b[0]==='aaaa'&&/(?:^a+$)/.exec('aaa\n')===null",
    );
    check(
        "let a=/(?:(?:a)(?:b))/.exec('xab'),b=/(?:(?:a)(b))/.exec('ab');a[0]==='ab'&&b[1]==='b'&&/(?:a)+(?:b)/.exec('aaab')[0]==='aaab'",
    );
}

#[test]
fn captures_indices_and_unselected_alternative_slots_preserve_source_order() {
    check(
        "let m=/(?:(a)+(b))/d.exec('aaab');m.length===3&&m[1]==='a'&&m[2]==='b'&&m.indices[1][0]===2&&m.indices[2][0]===3&&m.groups===undefined",
    );
    check(
        "let m=/(?:(x)|(a)+(b)|(y))/d.exec('aaab');m.length===5&&m[1]===undefined&&m[2]==='a'&&m[3]==='b'&&m[4]===undefined&&Object.hasOwn(m,'1')&&Object.hasOwn(m.indices,'4')&&m.indices[3][1]===4",
    );
    check(
        "let a=/(?:(a)*())/d.exec(''),b=/(?:(a*)())/d.exec('');a[1]===undefined&&a[2]===''&&a.indices[2][0]===0&&b[1]===''&&b.indices[1][1]===0",
    );
}

#[test]
fn global_sticky_state_and_strict_lastindex_writes_remain_ordered() {
    check(
        "let r=/(?:a+b)/g,a=r.exec('aab xaab'),b=r.exec('aab xaab');a.index===0&&b.index===5&&r.lastIndex===8&&r.exec('aab xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(?:(a)+(b))/y;r.lastIndex=1;let a=r.exec('xaab');a.index===1&&a[2]==='b'&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(?:a+b)/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('aab')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn original_source_flags_and_constructor_copying_survive_unwrapping() {
    check(
        "let r=new RegExp('(?:(?:a+b))','gi'),c=new RegExp(r),f=RegExp(r);r.source==='(?:(?:a+b))'&&r.toString()==='/(?:(?:a+b))/gi'&&c.source===r.source&&c!==r&&f===r&&c.exec('AAAB')[0]==='AAAB'",
    );
    check("let r=/(?:)/;r.source==='(?:)'&&r.exec('x')[0]===''&&r.exec('x').length===1");
    check(
        "let calls=[];let pattern={toString(){calls.push('pattern');return '(?:a+b)'}},flags={toString(){calls.push('flags');return 'g'}};let r=new RegExp(pattern,flags);r.source==='(?:a+b)'&&calls.join(',')==='pattern,flags'&&r.test('ab')",
    );
}

#[test]
fn escapes_classes_surrogates_and_original_case_are_preserved() {
    check(
        r"let a=/(?:\(\))/.exec('x()'),b=/(?:[()|]+)/.exec('x(|)'),c=/(?:(?:[[]))/.exec('x[');a[0]==='()'&&b[0]==='(|)'&&c[0]==='['",
    );
    check(
        r"let m=/(?:(.)+(\uD800))/sd.exec('a\n\uD800');m[1]==='\n'&&m[2]==='\uD800'&&m.indices[2][0]===2",
    );
    check(
        "let m=/(?:(µ)+(x))/i.exec('ΜΜX');m[1]==='Μ'&&m[2]==='X'&&/(?:(ſ)+x)/i.exec('ssx')===null",
    );
}

#[test]
fn generic_string_consumers_use_the_same_capture_results() {
    check(
        "'aaab'.replace(/(?:(a)+(b))/,'$1-$2')==='a-b'&&'aabxaab'.split(/(?:(a)+(b))/).join(',')===',a,b,x,a,b,'",
    );
    check(
        "let seen;let s='aaab'.replace(/(?:(a)+(b))/,(whole,a,b,index,input)=>{seen=[whole,a,b,index,input];return 'x'});s==='x'&&seen.join(',')==='aaab,a,b,0,aaab'",
    );
    check(
        "let r=/(?:(a)+(b))/dg,m=[...'aaab aab'.matchAll(r)];m.length===2&&m[1][2]==='b'&&m[1].indices[2][0]===7&&r.lastIndex===0&&'a'.match(/(?:a*())/g).length===2&&'xaaab'.search(/(?:a+b)/)===1",
    );
}

#[test]
fn deeply_nested_wrappers_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(?:'.repeat(100000)+'(a)+(b)'+')'.repeat(100000),'d'),c=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec('aaab');r.source.length===400007&&c.source===r.source&&m[1]==='a'&&m[2]==='b'&&m.indices[2][0]===3"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_host_aborts_and_unsupported_bodies_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/(?:\d+x)/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(?:a+b)/y,s='a'.repeat(8000)+'b',flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    for source in [
        "/(?:(?:a|bc)+)/.test('a')",
        "/(?:(?:(?:(a+[b]+))){2}){2}/.test('ab')",
        "/(?:(?:(?:a+[b]+)){2}){2}/.test('ab')",
        "/(?:a(?=b))/.test('ab')",
        "/(?:a+b)/u.test('ab')",
        "/(?:(?:a+(?<x>b))\\k<x>){2}/.test('ab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}
