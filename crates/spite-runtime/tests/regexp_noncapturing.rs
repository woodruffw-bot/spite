//! Unquantified noncapturing groups share native literal matching (22.2.2).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn groups_preserve_original_source_and_produce_no_captures() {
    check(
        "let r=/(?:a(?:b)c)/dig,m=r.exec('xABCy');r.source==='(?:a(?:b)c)' && r.flags==='dgi' && m.length===1 && m[0]==='ABC' && m.index===1 && m.groups===undefined && m.indices.length===1 && m.indices[0][0]===1 && m.indices[0][1]===4 && m.indices.groups===undefined && r.lastIndex===4",
    );
    check(
        "let r=new RegExp('(?:a)(?:b)','y');r.lastIndex=1;let m=r.exec('xab');m[0]==='ab' && m.index===1 && r.lastIndex===3 && r.exec('xab')===null && r.lastIndex===0",
    );
    check(
        r"let r=/(?:\uD83D)(?:\uDCA9)/d,m=r.exec('x💩');m.index===1 && m.indices[0][1]===3 && m[0]==='💩'",
    );
    check(r"/(?:\(\))/.exec('x()')[0]==='()'");
}

#[test]
fn empty_groups_match_without_advancing_native_lastindex() {
    check(
        "let r=/(?:)/dg;r.lastIndex=2;let m=r.exec('ab');m[0]==='' && m.index===2 && m.indices[0][0]===2 && m.indices[0][1]===2 && r.lastIndex===2",
    );
    check("let r=/(?:)/y;r.lastIndex=3;r.exec('ab')===null && r.lastIndex===0");
    check("let r=/(?:a(?:)b)/;r.exec('xab')[0]==='ab' && /(?:(?:))/.exec('')[0]===''");
}

#[test]
fn generic_consumers_advance_empty_group_matches_and_use_grouped_literals() {
    check(
        "'ab'.match(/(?:)/g).length===3 && [...'ab'.matchAll(/(?:)/g)].map(m=>m.index).join(',')==='0,1,2' && 'ab'.replace(/(?:)/g,'-')==='-a-b-' && 'ab'.split(/(?:)/).join(',')==='a,b'",
    );
    check(
        "'xab'.search(/(?:ab)/)===1 && 'abab'.match(/(?:ab)/g).join(',')==='ab,ab' && 'abab'.replaceAll(/(?:ab)/g,'x')==='xx'",
    );
}

#[test]
fn other_group_productions_and_quantified_groups_remain_unsupported() {
    for source in [
        "/(?:a|b)/.test('a')",
        "/(?:ab)*/.test('a')",
        "/(?:ab){1}/.test('a')",
        "/(?i:a)/.test('a')",
        "/(?=a)/.test('a')",
        "/(?<x>a)/.test('a')",
        "/(?:a)/u.test('a')",
        "/(?:a)/v.test('a')",
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
