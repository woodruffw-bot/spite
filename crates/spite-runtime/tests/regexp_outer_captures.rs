//! Enclosing ordinary captures span the complete selected body match (22.2.2.3).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn whole_match_captures_precede_inner_repetition_and_suffix_captures() {
    check(
        "let a=/(a+b)/d.exec('xaaab'),b=/(((a)+(b)))/d.exec('aaab');a[1]==='aaab'&&a.indices[1][0]===1&&a.indices[1][1]===5&&b.length===5&&b[1]==='aaab'&&b[2]==='aaab'&&b[3]==='a'&&b[4]==='b'&&b.indices[3][0]===2&&b.indices[4][0]===3",
    );
    check(
        "let a=/((?:a+?aa))/.exec('aaaa'),b=/(?:(a+b))/.exec('aaab');a[1]==='aaa'&&b[1]==='aaab'&&/((a)+b)/.exec('aaab')[2]==='a'",
    );
    check(
        r"let m=/(^((a)+)(b)$)/md.exec('x\naab\ny');m.index===2&&m[1]==='aab'&&m[2]==='aa'&&m[3]==='a'&&m[4]==='b'&&m.indices[1][0]===2&&m.indices[4][0]===4",
    );
}

#[test]
fn choices_preserve_source_order_and_own_undefined_inner_slots() {
    check(
        "let m=/((x)|(a)+(b)|(y))/d.exec('aaab');m.length===6&&m[1]==='aaab'&&m[2]===undefined&&m[3]==='a'&&m[4]==='b'&&m[5]===undefined&&Object.hasOwn(m,'2')&&Object.hasOwn(m.indices,'5')&&m.indices[4][1]===4",
    );
    check(
        "let a=/(a+|b)/.exec('baa'),b=/((a+?)|(aa))/.exec('aaa');a[1]==='b'&&b[1]==='a'&&b[2]==='a'&&b[3]===undefined",
    );
    check(
        "let m=/((a)|(b))/.exec('b');m[1]==='b'&&m[2]===undefined&&m[3]==='b'&&m.groups===undefined",
    );
}

#[test]
fn zero_repetitions_empty_groups_and_final_iterations_remain_distinct() {
    check(
        "let a=/((a)*())/d.exec(''),b=/((a*)())/d.exec('');a[1]===''&&a[2]===undefined&&a[3]===''&&a.indices[1][1]===0&&a.indices[2]===undefined&&b[1]===''&&b[2]===''&&b[3]===''",
    );
    check(
        "let a=/((a)*)/d.exec('aaa'),b=/(a)*/d.exec('aaa');a[1]==='aaa'&&a[2]==='a'&&a.indices[2][0]===2&&b[1]==='a'&&b.indices[1][0]===2&&/((a)*)/.exec('')[2]===undefined",
    );
    check(
        "let a=/((?:))/.exec('x'),b=/((a)*?())/.exec('aaa');a[1]===''&&b[1]===''&&b[2]===undefined&&b[3]===''",
    );
}

#[test]
fn global_sticky_state_and_strict_lastindex_writes_preserve_complete_ranges() {
    check(
        "let r=/(a+b)/g,a=r.exec('aab xaab'),b=r.exec('aab xaab');a[1]==='aab'&&b.index===5&&b[1]==='aab'&&r.lastIndex===8&&r.exec('aab xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/((a)+(b))/y;r.lastIndex=1;let m=r.exec('xaab');m.index===1&&m[1]==='aab'&&m[2]==='a'&&m[3]==='b'&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(a+b)/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('aab')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn original_source_flags_pinned_case_and_utf16_captures_are_preserved() {
    check(
        "let r=new RegExp('(?:(a+b))','gi'),c=new RegExp(r);r.source==='(?:(a+b))'&&r.toString()==='/(?:(a+b))/gi'&&c.source===r.source&&c!==r&&RegExp(r)===r&&c.exec('AAAB')[1]==='AAAB'",
    );
    check(
        r"let m=/((.)+\uD800)/sd.exec('a\n\uD800');m[1]==='a\n\uD800'&&m[2]==='\n'&&m.indices[1][1]===3&&m.indices[2][0]===1",
    );
    check(
        "let m=/((µ)+(x))/i.exec('ΜΜX');m[1]==='ΜΜX'&&m[2]==='Μ'&&m[3]==='X'&&/(ſ+x)/i.exec('ssx')===null",
    );
    check(r"let m=/([()|]+)/.exec('x(|)');m[1]==='(|)'&&/(\(\))/.exec('x()')[1]==='()'");
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_share_capture_layout() {
    check(
        "'aaab'.replace(/((a)+(b))/,'$1-$2-$3')==='aaab-a-b'&&'aabxaab'.split(/((a)+(b))/).join(',')===',aab,a,b,x,aab,a,b,'",
    );
    check(
        "let seen;let s='aaab'.replace(/((a)+(b))/,(whole,outer,a,b,index,input)=>{seen=[whole,outer,a,b,index,input];return 'x'});s==='x'&&seen.join(',')==='aaab,aaab,a,b,0,aaab'",
    );
    check(
        "let r=/((a)+(b))/dg,m=[...'aaab aab'.matchAll(r)];m.length===2&&m[1][1]==='aab'&&m[1][3]==='b'&&m[1].indices[3][0]===7&&r.lastIndex===0&&'a'.match(/(a*)/g).length===2",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/((x)|(a)+(b))/d.exec('aaab'),p=Object.getOwnPropertyDescriptor(m,'1');calls===0&&m[1]==='aaab'&&m[2]===undefined&&m.indices[2]===undefined&&Object.hasOwn(m,'2')&&p.writable&&p.enumerable&&p.configurable",
    );
}

#[test]
fn deeply_nested_captures_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let deep=new RegExp('('.repeat(100000)+'a+b'+')'.repeat(100000)),r=new RegExp('('.repeat(1000)+'(a)+(b)'+')'.repeat(1000),'d'),c=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec('aaab');deep.source.length===200003&&m.length===1003&&m[1]==='aaab'&&m[1000]==='aaab'&&m[1001]==='a'&&m[1002]==='b'&&m.indices[1000][1]===4&&m.indices[1002][0]===3"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_aborts_and_unsupported_bodies_remain_separate() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/(\d+x)/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(a+b)/y,s='a'.repeat(8000)+'b',flag=0")
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
        "/(a+b)*/.test('ab')",
        "/(a+[b]+)/.test('ab')",
        "/a(a|b)/.test('ab')",
        "/((a|b)+)/.test('ab')",
        "/(a+b)/u.test('ab')",
        "/((?<x>a))/.test('a')",
        r"/(\1a)/.test('a')",
        r"/((a)\1)/.test('aa')",
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
