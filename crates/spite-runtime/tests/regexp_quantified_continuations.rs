//! Quantified prefixes preserve continuation-driven repetition ordering.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn greedy_lazy_continuations_choose_the_earliest_start_before_match_length() {
    check(
        "/[ab]*ab/.exec('abab')[0]==='abab'&&/[ab]*?ab/.exec('abab')[0]==='ab'&&/a+aa/.exec('baaaa')[0]==='aaaa'&&/a+?aa/.exec('baaaa')[0]==='aaa'",
    );
    check(
        "let m=/[ab]{2,3}b/.exec('aababb'),n=/[ab]{2,3}?b/.exec('aababb');m.index===0&&m[0]==='aab'&&n.index===0&&n[0]==='aab'&&/a{1,3}a/.exec('aaaaa')[0]==='aaaa'",
    );
    check(
        "/[Nn]?evermore/.exec('Nevermore')[0]==='Nevermore'&&/[Nn]?evermore/.exec('evermore')[0]==='evermore'&&/(?:a)+?(?:aa)/.exec('aaaa')[0]==='aaa'",
    );
}

#[test]
fn empty_and_failed_prefixes_preserve_candidate_positions() {
    check(
        "let m=/[]*x/.exec('aax');m.index===2&&m[0]==='x'&&/[]+x/.exec('aax')===null&&/a{0}a/.exec('baa').index===1&&/a+(?:)/.exec('baaa')[0]==='aaa'",
    );
    check(
        "/[^]*x/.exec('xaxxx')[0]==='xaxxx'&&/[^]*?x/.exec('xaxxx')[0]==='x'&&/a{999999999999999999999999999}x/.exec('aax')===null",
    );
}

#[test]
fn global_sticky_and_strict_lastindex_follow_the_complete_match() {
    check(
        "let r=/\\d+x/g,a=r.exec('12x 3x'),b=r.exec('12x 3x');a[0]==='12x'&&a.index===0&&b[0]==='3x'&&b.index===4&&r.lastIndex===6&&r.exec('12x 3x')===null&&r.lastIndex===0",
    );
    check(
        "let r=/[ab]*ab/y;r.lastIndex=1;let m=r.exec('xabab');m.index===1&&m[0]==='abab'&&r.lastIndex===5&&r.exec('xabab')===null&&r.lastIndex===0",
    );
    check("let r=/a+b/y;r.lastIndex=1;r.exec('xaaacb')===null&&r.lastIndex===0");
    check(
        "let r=/a+b/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aab')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn prefix_sets_suffix_case_dotall_and_utf16_units_use_pinned_semantics() {
    check(
        r"/.+x/s.exec('a\nx')[0]==='a\nx'&&/.*x/.exec('a\nx')[0]==='x'&&/[µ]+x/i.exec('ΜµX')[0]==='ΜµX'&&/[ſ]+x/i.exec('sx')===null",
    );
    check(
        r"let m=/[^]*x/d.exec('\uD800\uDC00x'),n=/\uD800+x/.exec('\uD800\uD800x');m[0].length===3&&m.indices[0][1]===3&&n[0].length===3",
    );
}

#[test]
fn intrinsic_results_have_only_the_whole_match_and_bypass_setters() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(){calls++},configurable:true});let m=/a+b/d.exec('xaab'),p=Object.getOwnPropertyDescriptor(m,'0');calls===0&&m.length===1&&m[0]==='aab'&&m.index===1&&m.input==='xaab'&&m.groups===undefined&&m.indices.length===1&&m.indices[0][0]===1&&m.indices[0][1]===4&&p.writable&&p.enumerable&&p.configurable",
    );
}

#[test]
fn generic_consumers_and_callbacks_share_continuation_ordering() {
    check(
        "'abxab'.match(/[ab]*ab/g).join(',')==='ab,ab'&&'abxab'.replace(/[ab]*ab/g,'X')==='XxX'&&'abxab'.split(/[ab]*ab/).join(',')===',x,'&&'xaab'.search('a+b')===1",
    );
    check(
        "[...'abab'.matchAll(/[ab]*?ab/dg)].length===2&&[...'abab'.matchAll(/[ab]*ab/g)].length===1",
    );
    check(
        "let seen;let s='xaab'.replace(/a+b/,(...args)=>{seen=args;return 'z'});s==='xz'&&seen.length===3&&seen[0]==='aab'&&seen[1]===1&&seen[2]==='xaab'",
    );
}

#[test]
fn long_failed_runs_and_successful_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm
        .eval("let r=/[a]*aaaaab/,copy=new RegExp(r),s='a'.repeat(300000)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("r.exec(s)===null&&copy.exec(s)===null&&/[a]*aaaaa/.exec(s)[0].length===300000&&/[a]*?aaaaa/.exec(s)[0].length===5"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_construction_and_sticky_search_aborts_stay_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/\d+x/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/a+ab/y,flag=0,s='a'.repeat(8000)")
        .unwrap();
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
fn captures_sets_multiple_quantifiers_assertions_and_choices_remain_explicit_gaps() {
    for source in [
        "/a+([b]+)/.test('ab')",
        "/a+[b]+/.test('ab')",
        "/a+b+/.test('ab')",
        "/(a+[b]+)/.test('ab')",
        "/^a+b[c]+$/.test('ab')",
        "/a+[b]+|b/.test('b')",
        "/a+b/u.test('ab')",
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
