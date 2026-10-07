//! Fixed ordinary class/dot continuations share one quantified repetition plan.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn fixed_sets_constrain_greedy_lazy_repetitions_at_the_earliest_start() {
    check(
        "let a=/(a)+([ab])/d.exec('aaaab'),b=/(a)+?([ab])/d.exec('aaaab');a[0]==='aaaab'&&a[1]==='a'&&a[2]==='b'&&a.indices[1][0]===3&&b[0]==='aa'&&b[1]==='a'&&b[2]==='a'&&b.indices[2][0]===1",
    );
    check(
        "let a=/([ab])*([ab][ab])/d.exec('ababa'),b=/([ab])*?([ab][ab])/d.exec('ababa');a[0]==='ababa'&&a[1]==='a'&&a[2]==='ba'&&a.indices[1][0]===2&&b[0]==='ab'&&b[1]===undefined&&b[2]==='ab'&&b.indices[1]===undefined",
    );
    check(
        "let a=/(a*)([ab])()/d.exec('a'),b=/(a)*([ab])()/d.exec('a');a[1]===''&&a[2]==='a'&&a[3]===''&&b[1]===undefined&&b[2]==='a'&&b.indices[3][0]===1&&/a+([])/.exec('aa')===null",
    );
}

#[test]
fn fixed_continuation_groups_keep_repeated_and_static_capture_source_order() {
    check(
        "let m=/((a)+)(([b])())/d.exec('xaaab');m.index===1&&m.length===6&&m[1]==='aaa'&&m[2]==='a'&&m[3]==='b'&&m[4]==='b'&&m[5]===''&&m.indices[1][1]===4&&m.indices[2][0]===3&&m.indices[4][0]===4&&m.indices[5][0]===5",
    );
    check(
        "let a=/([x])([ab])*?([ab][ab])()/d.exec('xaabab'),b=/([x])([ab])*([ab][ab])()/d.exec('xaabab');a[0]==='xaa'&&a[1]==='x'&&a[2]===undefined&&a[3]==='aa'&&a.indices[4][0]===3&&b[0]==='xaabab'&&b[2]==='b'&&b[3]==='ab'&&b.indices[2][0]===3&&b.indices[3][0]===4",
    );
}

#[test]
fn global_sticky_failure_resets_and_strict_writes_use_complete_results() {
    check(
        "let r=/(a)+([b])/dg,a=r.exec('xaaab aab'),b=r.exec('xaaab aab');a.index===1&&a[0]==='aaab'&&b.index===6&&b[0]==='aab'&&b.indices[1][0]===7&&r.lastIndex===9&&r.exec('xaaab aab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(a)+([b])/y;r.lastIndex=1;let m=r.exec('xaab');m.index===1&&m[2]==='b'&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/a+[b]/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('aab')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn anchors_branches_and_enclosing_groups_constrain_continuation_endpoints() {
    check(
        "let m=/([x])|(a)+([bc])|([y])/d.exec('aaab');m.length===5&&m[1]===undefined&&m[2]==='a'&&m[3]==='b'&&m[4]===undefined&&m.indices[2][0]===2&&m.indices[3][0]===3",
    );
    check(
        "let m=/^((a)+?([ab]))$/d.exec('aaab');m[1]==='aaab'&&m[2]==='a'&&m[3]==='b'&&m.indices[2][0]===2&&m.indices[3][0]===3",
    );
    check(
        r"let r=/^(a)*?([a])$/dmy;r.lastIndex=2;let m=r.exec('x\naaaa\nx');m.index===2&&m[1]==='a'&&m[2]==='a'&&m.indices[1][0]===4&&m.indices[2][0]===5&&r.lastIndex===6",
    );
}

#[test]
fn continuation_classes_escapes_dot_flags_and_utf16_offsets_keep_pinned_rules() {
    check(
        r"let m=/(a)+(\d)/d.exec('xaa1');m.index===1&&m[1]==='a'&&m[2]==='1'&&m.indices[1][0]===2&&m.indices[2][0]===3",
    );
    check(
        "let a=/a+([µ])/i.exec('AAΜ'),b=/a+([^µ])/i.exec('AΜ');a[1]==='Μ'&&b===null&&/a+([ſ])/i.exec('aas')===null",
    );
    check(
        r"let m=/(a)+([💩])/d.exec('aa💩');m[0].length===3&&m[1]==='a'&&m[2]==='\uD83D'&&m.indices[2][0]===2&&m.indices[2][1]===3",
    );
    check(
        r"let m=/a+(.)/ds.exec('a\n');m[1]==='\n'&&m.indices[1][0]===1&&/a+(.)/.exec('a\n')===null",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_preserve_continuation_slots() {
    check(
        "'aaab'.replace(/(a)+([b])/,'$1-$2')==='a-b'&&'aaabxaaab'.split(/(a)+([b])/).join(',')===',a,b,x,a,b,'",
    );
    check(
        "let seen;let s='aaab'.replace(/(a)+([b])/,(whole,a,b,index,input)=>{seen=[whole,a,b,index,input];return 'z'});s==='z'&&seen.join(',')==='aaab,a,b,0,aaab'",
    );
    check(
        "let r=/(a)+([b])/dg,m=[...'aaab aab'.matchAll(r)];m.length===2&&m[1][1]==='a'&&m[1].indices[2][0]===7&&r.lastIndex===0",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/(a)*([ab])/d.exec('b');calls===0&&m[1]===undefined&&Object.hasOwn(m,'1')&&m.indices[1]===undefined&&Object.hasOwn(m.indices,'1')&&m[2]==='b'",
    );
}

#[test]
fn shared_thousand_capture_continuations_and_copies_survive_collection_without_limits() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(b)+'+'([a])'.repeat(1000),'dg'),copy=new RegExp(r),s='bbb'+'a'.repeat(1000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(s);m.length===1002&&m[1]==='b'&&m[1001]==='a'&&m.indices[1][0]===2&&m.indices[1001][0]===1002&&m.indices[1001][1]===1003&&copy.lastIndex===1003&&r.lastIndex===0&&r.source===copy.source"),Ok(Value::Boolean(true)));
}

#[test]
fn sticky_optional_work_covers_all_continuation_candidates_and_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('a+'+'[b]'.repeat(100),'y'),s='a'.repeat(2000),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/a+(\d)/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for source in [
        "/a+[b]+/.test('ab')",
        "/(a[b]|c)+/.test('ab')",
        "/a+(b|cd)/.test('ab')",
        "/a+(?<n>b)\\k<n>/.test('ab')",
        "/a+[b]/u.test('ab')",
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
