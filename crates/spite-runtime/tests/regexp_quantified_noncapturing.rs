//! Transparent noncapturing groups preserve a single consuming atom's repetition.

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn quantifier_placement_preserves_greedy_lazy_and_bounded_runs() {
    check(
        "/(?:a)+/.exec('baaab')[0]==='aaa'&&/(?:a+)/.exec('baaab')[0]==='aaa'&&/(?:(?:a)+?)/.exec('baaab')[0]==='a'&&/(?:(?:a+?))/.exec('baaab')[0]==='a'",
    );
    check(
        "/(?:[ab]){2,3}/.exec('xbbaba')[0]==='bba'&&/(?:[ab]{2,3}?)/.exec('xbbaba')[0]==='bb'&&/(?:[])*/.exec('a')[0]===''",
    );
    check(
        r"/(?:[^]{2})/.exec('\uD800\uDC00')[0].length===2&&/(?:.)+/s.exec('a\nb')[0]==='a\nb'&&/(?:[µ])+/i.exec('Μµμ')[0]==='Μµμ'",
    );
}

#[test]
fn nested_groups_keep_no_capture_slots_and_original_state() {
    check(
        "let r=/(?:(?:[ab])+)/dg,a=r.exec('aab xbb'),b=r.exec('aab xbb');a.length===1&&a.indices.length===1&&a.index===0&&a[0]==='aab'&&b.index===5&&b.indices[0][1]===7&&r.lastIndex===7&&r.exec('aab xbb')===null&&r.lastIndex===0&&r.source==='(?:(?:[ab])+)'&&r.flags==='dg'",
    );
    check(
        "let r=/(?:[ab])+?/y;r.lastIndex=1;let m=r.exec('xaba');m[0]==='a'&&m.index===1&&r.lastIndex===2",
    );
}

#[test]
fn generic_consumers_advance_empty_runs_and_share_grouped_plans() {
    check(
        "'aa'.match(/(?:a*?)/g).length===3&&'aa'.replace(/(?:a)+?/g,'x')==='xx'&&'xaab'.search('(?:a)+')===1&&'xaab'.split(/(?:a)+/).join(',')==='x,b'&&[...'aa'.matchAll(/(?:a*?)/g)].length===3",
    );
}

#[test]
fn deeply_nested_plans_and_copies_survive_collection_with_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval("let source='(?:'.repeat(100000)+'[ab]+?'+')'.repeat(100000),r=new RegExp(source,'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('aab'),n=copy.exec('ba');m.length===1&&m[0]==='a'&&m.indices[0][1]===1&&n[0]==='b'&&copy.source===source"),Ok(Value::Boolean(true)));
}

#[test]
fn multiple_quantifiers_captures_choices_and_multi_atom_groups_remain_explicit_gaps() {
    for source in [
        "/(?:(?:a)+)+/.test('a')",
        "/(?:a*)?/.test('a')",
        "/(?:ab)+/.test('ab')",
        "/(?:a|b)+/.test('a')",
        "/((?:a)+)/.test('a')",
        "/(?<x>a)+/.test('a')",
        "/(?:a)+/u.test('a')",
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
