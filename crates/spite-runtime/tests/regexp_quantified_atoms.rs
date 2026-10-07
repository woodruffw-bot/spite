//! Whole-pattern single consuming atoms with greedy/lazy quantifiers.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn greedy_lazy_and_bounded_runs_select_the_earliest_position() {
    check(
        "/a*/.exec('aaab')[0]==='aaa'&&/a*?/.exec('aaab')[0]===''&&/a+/.exec('baaab')[0]==='aaa'&&/a+?/.exec('baaab')[0]==='a'&&/a?/.exec('aa')[0]==='a'&&/a??/.exec('aa')[0]===''",
    );
    check(
        "let m=/[ab]{2,3}/.exec('aXbbaba'),n=/[ab]{2,3}?/.exec('aXbbaba');m.index===2&&m[0]==='bba'&&n.index===2&&n[0]==='bb'&&/[a]{2}/.exec('aba')===null&&/[a]{2,}/.exec('xaaa')[0]==='aaa'",
    );
    check(
        "/a{00,02}/.exec('aaa')[0]==='aa'&&/[]{0}/.exec('a')[0]===''&&/[]+/.exec('a')===null&&/[^]{2}/.exec('ab')[0]==='ab'",
    );
}

#[test]
fn lastindex_global_sticky_and_strict_writes_follow_complete_runs() {
    check(
        "let r=/[ab]+/g,a=r.exec('aabb xa'),b=r.exec('aabb xa');a.index===0&&a[0]==='aabb'&&b.index===6&&b[0]==='a'&&r.lastIndex===7&&r.exec('aabb xa')===null&&r.lastIndex===0",
    );
    check(
        "let r=/a+/y;r.lastIndex=1;let m=r.exec('baaa');m.index===1&&m[0]==='aaa'&&r.lastIndex===4&&r.exec('baaa')===null&&r.lastIndex===0",
    );
    check("let r=/a*/gy;r.lastIndex=2;let m=r.exec('ba');m[0]===''&&m.index===2&&r.lastIndex===2");
    check(
        "let r=/a+/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aa')}catch(e){ok=e instanceof TypeError}ok&&r.lastIndex===0",
    );
}

#[test]
fn dotall_ignorecase_escapes_and_utf16_units_use_existing_membership() {
    check(
        r"/.+/.exec('a\nb')[0]==='a'&&/.+/s.exec('a\nb')[0]==='a\nb'&&/[^a]+/i.exec('aaBBa')[0]==='BB'&&/[µ]+/i.exec('µΜμ')[0]==='µΜμ'&&/[ſ]+/i.exec('s')===null",
    );
    check(
        r"/\d+/.exec('x123')[0]==='123'&&/\w+?/.exec('abc')[0]==='a'&&/\s+/.exec(' \n')[0]===' \n'&&/\++/.exec('x+++')[0]==='+++'",
    );
    check(
        r"let m=/[^]{2}/d.exec('\uD800\uDC00'),n=/\uD800+/.exec('\uD800\uD800\uDC00');m[0].length===2&&m.indices[0][1]===2&&n[0].length===2",
    );
}

#[test]
fn intrinsic_results_have_no_capture_slots_and_bypass_prototype_setters() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(){calls++},configurable:true});let m=/a+/d.exec('xaa'),p=Object.getOwnPropertyDescriptor(m,'0');calls===0&&m.length===1&&m[0]==='aa'&&m.index===1&&m.input==='xaa'&&m.groups===undefined&&m.indices.length===1&&m.indices[0][0]===1&&m.indices[0][1]===3&&m.indices.groups===undefined&&p.writable&&p.enumerable&&p.configurable",
    );
}

#[test]
fn generic_consumers_keep_lazy_and_empty_match_advancement() {
    check(
        "'aa xbbb'.match(/[ab]+/g).join(',')==='aa,bbb'&&'aa'.match(/a*?/g).length===3&&'aa'.replace(/a+?/g,'x')==='xx'&&'aabb'.replace(/[ab]+/,'x')==='x'",
    );
    check(
        "'aabbx'.split(/[ab]+/).join(',')===',x'&&'xaab'.search('a+')===1&&[...'aa'.matchAll(/a*?/g)].length===3",
    );
    check(
        "let seen;let s='xaa'.replace(/a+/,(...args)=>{seen=args;return 'z'});s==='xz'&&seen.length===3&&seen[0]==='aa'&&seen[1]===1&&seen[2]==='xaa'",
    );
}

#[test]
fn enormous_counts_are_compact_and_shared_copies_survive_collection() {
    check(
        "/a{999999999999999999999999999999}/.exec('aaa')===null&&/a{0,999999999999999999999999999999}/.exec('aaa')[0]==='aaa'&&/a{0,999999999999999999999999999999}?/.exec('aaa')[0]===''",
    );
    let mut realm = Realm::default();
    realm
        .eval("let r=/[a]+/d,copy=new RegExp(r),s='a'.repeat(100000)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec(s),n=copy.exec(s);m[0].length===100000&&n[0].length===100000&&m.indices[0][1]===100000"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_constructor_and_sticky_search_aborts_remain_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/\d+/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let r=/a+/y,flag=0,s='a'.repeat(8000)").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn groups_concatenation_assertions_alternatives_and_unicode_remain_explicit_gaps() {
    for source in [
        "/(ab)+/.test('a')",
        "/(?:ab)+/.test('a')",
        "/a+[b]+/.test('ab')",
        "/a+[b]+|b/.test('b')",
        "/^a+[b]+$/.test('a')",
        "/a+/u.test('a')",
        "/[a]+/v.test('a')",
        r"/(a)\1+/.test('a')",
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
