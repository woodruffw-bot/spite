//! Unquantified noncapturing groups share native literal matching (22.2.2).

use spite_runtime::Realm;

use crate::common::assert_true;

#[test]
fn groups_preserve_original_source_and_produce_no_captures() {
    assert_true(
        r#"
            let r = /(?:a(?:b)c)/dig,
                m = r.exec('xABCy');
            r.source === '(?:a(?:b)c)' && r.flags === 'dgi' && m.length === 1 && m[0] === 'ABC' && m
                .index === 1 && m.groups === undefined && m.indices.length === 1 && m.indices[0][0] ===
                1 && m.indices[0][1] === 4 && m.indices.groups === undefined && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = new RegExp('(?:a)(?:b)', 'y');
            r.lastIndex = 1;
            let m = r.exec('xab');
            m[0] === 'ab' && m.index === 1 && r.lastIndex === 3 && r.exec('xab') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r"let r=/(?:\uD83D)(?:\uDCA9)/d,m=r.exec('x💩');m.index===1 && m.indices[0][1]===3 && m[0]==='💩'",
    );
    assert_true(r"/(?:\(\))/.exec('x()')[0]==='()'");
}

#[test]
fn empty_groups_match_without_advancing_native_lastindex() {
    assert_true(
        r#"
            let r = /(?:)/dg;
            r.lastIndex = 2;
            let m = r.exec('ab');
            m[0] === '' && m.index === 2 && m.indices[0][0] === 2 && m.indices[0][1] === 2 && r
                .lastIndex === 2
        "#,
    );
    assert_true("let r=/(?:)/y;r.lastIndex=3;r.exec('ab')===null && r.lastIndex===0");
    assert_true("let r=/(?:a(?:)b)/;r.exec('xab')[0]==='ab' && /(?:(?:))/.exec('')[0]===''");
}

#[test]
fn generic_consumers_advance_empty_group_matches_and_use_grouped_literals() {
    assert_true(
        r#"
            'ab'.match(/(?:)/g).length === 3 && [...'ab'.matchAll(/(?:)/g)].map(m => m.index).join(',') ===
                '0,1,2' && 'ab'.replace(/(?:)/g, '-') === '-a-b-' && 'ab'.split(/(?:)/).join(',') === 'a,b'
        "#,
    );
    assert_true(
        "'xab'.search(/(?:ab)/)===1 && 'abab'.match(/(?:ab)/g).join(',')==='ab,ab' && 'abab'.replaceAll(/(?:ab)/g,'x')==='xx'",
    );
}

#[test]
fn other_group_productions_and_quantified_groups_match_reference_results() {
    for source in [
        r"/(?:(?:((?:(?:a(?:a|bc)){2}){2}){2})|){2,3}/.test('a')",
        "/(?:(?:ab|c)*){2}/.test('a')",
        r"/(?:(?:((?:(?:a[b]|c){1}){2}){2})|){2,3}/.test('a')",
        "/(?i:a)/.test('a')",
        r"/(?<=(a+))/.test('a')",
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('a')",
        "/(?:\\u{D800}\\u{DC00})/u.test('a')",
        "/(?:\\u{D800}\\u{DC00})/v.test('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
