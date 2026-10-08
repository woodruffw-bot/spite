//! CompileAssertion boundaries and RegExpBuiltinExec (22.2.2.4, 22.2.7.2).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn nonmultiline_assertions_use_exact_input_boundaries() {
    check(
        "/^a/.exec('ba')===null && /a$/.exec('ab')===null && /^a$/.exec('a')[0]==='a' && /^a$/.exec('a\\n')===null && /a$/.exec('a\\r')===null && /$/.exec('abc').index===3 && /^$/.test('') && /^$/.exec('\\n')===null",
    );
    check("/^/.exec('a')[0]==='' && /^/.exec('a').index===0 && /a$/.exec('aba').index===2");
}

#[test]
fn multiline_anchors_use_all_four_line_terminators_and_crlf_positions() {
    check(
        "let values=['\\n','\\r','\\u2028','\\u2029'];values.every(t=>/^a$/m.exec('x'+t+'a'+t+'y').index===2) && /^()$/m.exec('x\\r\\ny').index===2",
    );
    check(
        "[...'x\\r\\ny'.matchAll(/^/mg)].map(m=>m.index).join(',')==='0,2,3' && [...'x\\r\\ny'.matchAll(/$/mg)].map(m=>m.index).join(',')==='1,2,4'",
    );
}

#[test]
fn sticky_offsets_do_not_change_the_meaning_of_beginning_assertions() {
    check("let r=/^a/y;r.lastIndex=1;r.exec('ba')===null && r.lastIndex===0");
    check(
        "let r=/^a$/my;r.lastIndex=2;let m=r.exec('x\\na\\ry');m.index===2 && r.lastIndex===3 && r.exec('x\\na\\ry')===null && r.lastIndex===0",
    );
    check("let r=/a$/y;r.lastIndex=2;r.exec('aba')[0]==='a' && r.lastIndex===3");
}

#[test]
fn captures_case_and_indices_exclude_the_zero_width_assertions() {
    check(
        "let m=/^(a)()$/di.exec('A');m.length===3 && m[0]==='A' && m[1]==='A' && m[2]==='' && m.indices[0].join(',')==='0,1' && m.indices[1].join(',')==='0,1' && m.indices[2].join(',')==='1,1'",
    );
    check(
        r"let m=/^(\uD800)/d.exec('\uD800\uDC00');m[0]==='\uD800' && m[1]==='\uD800' && m.indices[1].join(',')==='0,1'",
    );
}

#[test]
fn escaped_dollars_carets_and_backslash_parity_preserve_literal_meanings() {
    check(
        r"/^a\$/.exec('a$')[0]==='a$' && /\^a$/.exec('x^a')[0]==='^a' && /a\\$/.exec('xa\\')[0]==='a\\'",
    );
}

#[test]
fn rejected_overlapping_matches_and_generic_consumers_continue_correctly() {
    check(
        "/aba$/.exec('ababa').index===2 && 'ba'.match('a$')[0]==='a' && 'ba'.search('a$')===1 && 'a\\nb'.replace(/^/mg,'-')==='-a\\n-b' && 'ab'.split(/$/).join(',')==='ab' && [...'ab'.matchAll('$')][0].index===2",
    );
    check(
        "let r=/a$/g;Object.defineProperty(r,'lastIndex',{writable:false});let threw=false;try{r.exec('ba')}catch(e){threw=e instanceof TypeError}threw && r.lastIndex===0",
    );
}

#[test]
fn copied_long_anchored_plans_survive_collection_without_restarting_searches() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('a'.repeat(60000)+'$','dg'),copy=new RegExp(r),s='a'.repeat(120000);RegExp=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(s);m.index===60000 && m[0].length===60000 && m.indices[0].join(',')==='60000,120000' && copy.lastIndex===120000 && r.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn internal_assertions_alternatives_and_unicode_modes_remain_explicit_gaps() {
    for source in [
        "/(?:(?:a+(^a)+){2}){2}/.test('a')",
        "/(?:(?:^a*[b]+|b$){2}){2}/.test('a')",
        "/(?:(?:^a*[b]+){2}){2}/.test('a')",
        "/^a$/u.test('a')",
        "/^a$/v.test('a')",
        r"/(?:(?:^a+^b+){2}){2}/.test('ab')",
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
