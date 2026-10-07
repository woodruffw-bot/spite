//! Literal continuation captures resolve after the quantified prefix is selected.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prefix_and_suffix_ranges_follow_greedy_lazy_and_source_order() {
    check(
        "let a=/(a)+(b)/d.exec('aaab'),b=/(a+)(b)/d.exec('aaab');a[0]==='aaab'&&a[1]==='a'&&a[2]==='b'&&a.indices[1][0]===2&&a.indices[2][0]===3&&b[1]==='aaa'&&b.indices[1][1]===3",
    );
    check(
        "let a=/([ab])*?(ab)/d.exec('abab'),b=/([ab])*(ab)/d.exec('abab');a[0]==='ab'&&a[1]===undefined&&a[2]==='ab'&&a.indices[2][0]===0&&b[0]==='abab'&&b[1]==='b'&&b[2]==='ab'&&b.indices[1][0]===1&&b.indices[2][0]===2",
    );
    check(
        "let m=/((a)+)((b)())/d.exec('aaab');m.length===6&&m[1]==='aaa'&&m[2]==='a'&&m[3]==='b'&&m[4]==='b'&&m[5]===''&&m.indices[5][0]===4&&m.indices[5][1]===4",
    );
}

#[test]
fn empty_suffixes_and_zero_runs_distinguish_absent_and_empty_captures() {
    check(
        "let a=/(a)*()/d.exec(''),b=/(a*)()/d.exec('');a[1]===undefined&&a[2]===''&&a.indices[1]===undefined&&a.indices[2][0]===0&&b[1]===''&&b[2]===''&&b.indices[1][1]===0&&b.indices[2][1]===0",
    );
    check(
        "let m=/a+((?:))()/d.exec('aaa');m[0]==='aaa'&&m[1]===''&&m[2]===''&&m.indices[1][0]===3&&m.indices[2][1]===3&&/[]*(x)/.exec('x')[1]==='x'",
    );
}

#[test]
fn anchors_choices_and_global_sticky_state_preserve_all_capture_slots() {
    check(
        r"let m=/^((a)+?)(b)$/md.exec('x\naab\ny');m.index===2&&m[1]==='aa'&&m[2]==='a'&&m[3]==='b'&&m.indices[3][0]===4",
    );
    check(
        "let r=/([x])|(a)+(b)|([y])/dg,a=r.exec('aaby'),b=r.exec('aaby');a.length===5&&a[1]===undefined&&a[2]==='a'&&a[3]==='b'&&a[4]===undefined&&a.indices[3][0]===2&&b[4]==='y'&&b[3]===undefined&&r.lastIndex===4&&r.exec('aaby')===null&&r.lastIndex===0",
    );
    check(
        "let r=/(a)+(b)/y;r.lastIndex=1;let m=r.exec('xaab');m.index===1&&m[1]==='a'&&m[2]==='b'&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/a+(b)/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aab')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn fixed_suffixes_capture_original_case_surrogates_and_empty_groups() {
    check("let m=/a+(µ)/i.exec('AAΜ');m[0]==='AAΜ'&&m[1]==='Μ'&&/a+(ſ)/i.exec('aas')===null");
    check(
        r"let m=/(.)+(\uD800)/sd.exec('a\n\uD800');m[1]==='\n'&&m[2]==='\uD800'&&m.indices[1][0]===1&&m.indices[2][0]===2",
    );
}

#[test]
fn generic_consumers_replacements_split_and_matchall_resolve_suffix_captures() {
    check(
        "'aaab'.replace(/(a)+(b)/,'$1-$2')==='a-b'&&'aabxaab'.split(/(a)+(b)/).join(',')===',a,b,x,a,b,'",
    );
    check(
        "let seen;let s='aaab'.replace(/(a)+(b)/,(whole,prefix,suffix,index,input)=>{seen=[whole,prefix,suffix,index,input];return 'x'});s==='x'&&seen.join(',')==='aaab,a,b,0,aaab'",
    );
    check(
        "let r=/(a)+(b)/dg,m=[...'aaab aab'.matchAll(r)];m.length===2&&m[0][2]==='b'&&m[0].indices[2][0]===3&&m[1].indices[2][0]===7&&r.lastIndex===0&&'a'.match(/a*()/g).length===2",
    );
}

#[test]
fn intrinsic_arrays_and_undefined_alternative_slots_bypass_setters() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/a+(b)/d.exec('aaab'),n=/([x])|a+(b)/d.exec('aaab'),p=Object.getOwnPropertyDescriptor(m,'1');calls===0&&m[1]==='b'&&p.writable&&p.enumerable&&p.configurable&&n[1]===undefined&&n[2]==='b'&&Object.hasOwn(n.indices,'1')&&n.indices[1]===undefined&&m.groups===undefined",
    );
}

#[test]
fn deeply_nested_suffix_captures_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let deep=new RegExp('a+'+'('.repeat(100000)+'b'+')'.repeat(100000)),r=new RegExp('((a)+)'+'('.repeat(1000)+'b'+')'.repeat(1000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec('aaab');deep.source.length===200003&&m.length===1003&&m[1]==='aaa'&&m[2]==='a'&&m[1002]==='b'&&m.indices[1002][0]===3&&m.indices[1002][1]===4"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_host_aborts_and_unsupported_suffixes_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/\d+(x)/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(a)+(b)/y,s='a'.repeat(8000)+'b',flag=0")
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
        "/a+([b])/.test('ab')",
        "/a+(b|c)/.test('ab')",
        "/(a+[b])/.test('ab')",
        "/a+(b+)/.test('ab')",
        "/a+(?<x>b)/.test('ab')",
        "/a+(b)/u.test('ab')",
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
