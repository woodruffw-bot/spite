//! Literal alternatives preserve leftmost/source-order choices (22.2.2.3).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn source_order_wins_equal_start_ties_without_preferring_longest_matches() {
    check(
        "/a|ab/.exec('ab')[0]==='a' && /ab|a/.exec('ab')[0]==='ab' && /ll|l/.exec('null')[0]==='ll'",
    );
    check(
        "let r=/a|ab/dg,m=r.exec('ab');m.length===1 && m[0]==='a' && m.index===0 && m.groups===undefined && m.indices.length===1 && m.indices[0].join(',')==='0,1' && m.indices.groups===undefined && r.lastIndex===1",
    );
}

#[test]
fn an_earlier_start_wins_over_an_earlier_source_alternative() {
    check(
        "let m=/2|12/.exec(1.012);m[0]==='12' && m.index===3 && m.input==='1.012' && /z|a/.exec('az')[0]==='a'",
    );
    check(
        "let r=/b|a/y;r.lastIndex=1;let m=r.exec('xab');m[0]==='a' && m.index===1 && r.lastIndex===2 && r.exec('xab')[0]==='b' && r.lastIndex===3 && r.exec('xab')===null && r.lastIndex===0",
    );
}

#[test]
fn empty_alternatives_follow_source_order_and_generic_empty_match_advancement() {
    check(
        "/|a/.exec('a')[0]==='' && /a|/.exec('a')[0]==='a' && /a|/.exec('ba').index===0 && /a|/.exec('ba')[0]===''",
    );
    check(
        "let r=/a|/dg,m=r.exec('a');let n=r.exec('a');m[0]==='a' && n[0]==='' && n.index===1 && n.indices[0].join(',')==='1,1' && r.lastIndex===1",
    );
    check(
        "'ab'.match(/|a/g).length===3 && [...'ab'.matchAll(/|a/g)].map(m=>m.index).join(',')==='0,1,2' && 'ab'.replace(/|a/g,'-')==='-a-b-'",
    );
}

#[test]
fn noncapturing_branches_escaped_delimiters_and_surrogate_halves_keep_utf16_ranges() {
    check(
        "let r=/(?:ab)|(?:c)/d,m=r.exec('xc');r.source==='(?:ab)|(?:c)' && m.length===1 && m.index===1 && m[0]==='c' && m.indices[0].join(',')==='1,2'",
    );
    check(r"/a\|b|c/.exec('xa|b')[0]==='a|b'");
    check(
        r"let m=/\uD800|\uDC00/d.exec('x\uD800\uDC00');m[0]==='\uD800' && m.index===1 && m.indices[0].join(',')==='1,2'",
    );
}

#[test]
fn ignore_case_uses_pinned_ordinary_rules_across_all_alternatives() {
    check(
        "let r=/x|σ/i;let m=r.exec('ς');m[0]==='ς' && m.index===0 && /µ|z/i.test('Μ') && /s|x/i.exec('ſ')===null && /k|x/i.exec('K')===null",
    );
}

#[test]
fn all_generic_consumers_and_string_fallbacks_use_literal_choices() {
    check(
        "'aba'.match(/a|b/g).join(',')==='a,b,a' && 'aba'.search(/b|z/)===1 && 'aba'.replaceAll(/a|b/g,'x')==='xxx' && 'a,b;c'.split(/,|;/).join('|')==='a|b|c'",
    );
    check(
        "[...'aba'.matchAll('a|b')].map(m=>m[0]+m.index).join(',')==='a0,b1,a2' && 'aba'.match('b|z')[0]==='b' && 'aba'.search('b|z')===1",
    );
}

#[test]
fn copies_many_flat_alternatives_and_original_plans_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('a|'.repeat(1000)+'b','dg'),copy=new RegExp(r);RegExp=null;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('xb'),n=copy.exec('a');m[0]==='b' && m.index===1 && m.indices[0].join(',')==='1,2' && n[0]==='a' && copy.lastIndex===1 && r.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn unsupported_alternatives_captures_nested_choices_and_unicode_remain_host_gaps() {
    for source in [
        "/a|[b]/.test('a')",
        "/a|b*/.test('a')",
        "/(a|b)/.test('a')",
        "/(?:a|b)|c/.test('a')",
        "/a|b/u.test('a')",
        "/a|b/v.test('a')",
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

#[test]
fn opted_in_work_accounts_for_all_searches_and_aborts_outside_javascript() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm
        .eval("let r=/a|b/g,s='c'.repeat(10000),flag=0;")
        .unwrap();
    assert_eq!(realm.eval("/a/.test(s)"), Ok(Value::Boolean(false)));
    assert!(matches!(
        realm.eval("try{r.exec(s);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("r=new RegExp('a|'.repeat(1000)+'b','g');")
        .unwrap();
    assert!(matches!(
        realm.eval("try{for(let i=0;i<100;i++){r.exec('');}}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
}
