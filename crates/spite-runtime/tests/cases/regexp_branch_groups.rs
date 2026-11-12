//! Complete branch wrappers keep branch order and whole-match capture prefixes.

use spite_runtime::Realm;

use crate::common::assert_true;

#[test]
fn wrapped_branches_preserve_earliest_starts_source_order_and_repetition_order() {
    assert_true(
        r#"
            let a = /(a+b)|x/.exec('xaaab'),
                b = /x|(a+b)/.exec('aaabx');
            a[0] === 'x' && a[1] === undefined && b[0] === 'aaab' && b[1] === 'aaab'
        "#,
    );
    assert_true(
        r#"
            let a = /(a+?aa)|(a+aa)/.exec('aaaa'),
                b = /(a+aa)|(a+?aa)/.exec('aaaa');
            a[0] === 'aaa' && a[1] === 'aaa' && a[2] === undefined && b[0] === 'aaaa' && b[1] === 'aaaa' &&
                b[2] === undefined
        "#,
    );
    assert_true(
        r#"
            let m = /((a)+b)|(b+)/d.exec('aaab');
            m.length === 4 && m[1] === 'aaab' && m[2] === 'a' && m[3] === undefined && m.indices[1][1] ===
                4 && m.indices[2][0] === 2
        "#,
    );
}

#[test]
fn assertions_constrain_lengths_and_multiline_boundaries_inside_branches() {
    assert_true(
        r#"
            let m = /(^((a)+)(b)$)|(x)/md.exec('y\naaab\nz');
            m.index === 2 && m[1] === 'aaab' && m[2] === 'aaa' && m[3] === 'a' && m[4] === 'b' && m[5] ===
                undefined && m.indices[4][0] === 5
        "#,
    );
    assert_true(
        r#"
            let a = /(^a+?aa$)|(b+)/.exec('aaaa'),
                b = /(a$)|(^b)/.exec('ba');
            a[1] === 'aaaa' && a[2] === undefined && b[1] === undefined && b[2] === 'b' && /(^a+$)|x/.exec(
                'aaa\n') === null
        "#,
    );
}

#[test]
fn global_sticky_state_and_strict_writes_use_complete_branch_matches() {
    assert_true(
        r#"
            let r = /([x])|((a)+(b))|([y])/dg,
                a = r.exec('aaaby'),
                b = r.exec('aaaby');
            a.length === 6 && a[1] === undefined && a[2] === 'aaab' && a[3] === 'a' && a[4] === 'b' && a[
                5] === undefined && b[5] === 'y' && b[2] === undefined && r.lastIndex === 5 && r.exec(
                'aaaby') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a+b)|x/y;
            r.lastIndex = 1;
            let m = r.exec('xaaab');
            m.index === 1 && m[1] === 'aaab' && r.lastIndex === 5 && r.exec('xaaab') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a+b)|x/g;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let failed = false;
            try {
                r.exec('aaab')
            } catch (e) {
                failed = e instanceof TypeError
            }
            failed
        "#,
    );
}

#[test]
fn empty_branches_groups_and_unselected_captures_keep_participation() {
    assert_true(
        r#"
            let a = /((a)*())|x/d.exec(''),
                b = /x|((a*)())/d.exec('');
            a[1] === '' && a[2] === undefined && a[3] === '' && a.indices[1][0] === 0 && a.indices[2] ===
                undefined && b[1] === '' && b[2] === '' && b[3] === ''
        "#,
    );
    assert_true(
        r#"
            let a = /()|(a+b)/.exec('aaab'),
                b = /(a+b)|()/.exec('aaab');
            a[0] === '' && a[1] === '' && a[2] === undefined && b[0] === 'aaab' && b[1] === 'aaab' && b[
                2] === undefined
        "#,
    );
}

#[test]
fn escapes_pinned_flags_and_original_utf16_captures_survive_wrappers() {
    assert_true(
        "let m=/(µ+x)|([a])/i.exec('ΜΜX');m[1]==='ΜΜX'&&m[2]===undefined&&/(ſ+x)|a/i.exec('ssx')===null",
    );
    assert_true(
        r#"
            let m = /((.)+\uD800)|([x])/sd.exec('a\n\uD800');
            m[1] === 'a\n\uD800' && m[2] === '\n' && m[3] === undefined && m.indices[2][0] === 1
        "#,
    );
    assert_true(
        r#"
            let m = /(?:a+b)|([x])/.exec('aaab');
            m.length === 2 && m[1] === undefined && /(?:(a+b))|x/.exec('aaab')[1] === 'aaab'
        "#,
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_share_branch_layout() {
    assert_true(
        "'aaab'.replace(/(a+b)|x/,'$1')==='aaab'&&'aabxaab'.split(/((a)+b)|x/).join(',')===',aab,a,,,,,aab,a,'",
    );
    assert_true(
        r#"
            let seen;
            let s = 'aaab'.replace(/((a)+b)|x/, (whole, outer, a, index, input) => {
                seen = [whole, outer, a, index, input];
                return 'z'
            });
            s === 'z' && seen.join(',') === 'aaab,aaab,a,0,aaab'
        "#,
    );
    assert_true(
        r#"
            let r = /(a+b)|x/dg,
                m = [...'aaab x'.matchAll(r)];
            m.length === 2 && m[0][1] === 'aaab' && m[0].indices[1][1] === 4 && m[1][1] === undefined && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let calls = 0;
            Object.defineProperty(Array.prototype, '1', {
                set() {
                    calls++
                },
                configurable: true
            });
            let m = /(a+b)|([x])/d.exec('aaab');
            calls === 0 && m[1] === 'aaab' && m[2] === undefined && Object.hasOwn(m, '2') && Object.hasOwn(
                m.indices, '2') && m.indices[2] === undefined
        "#,
    );
}

#[test]
fn pattern_variants_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:(a|bc)|x){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:(a+b)*|x)|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:a(a+b+)|x){2}){2})|){2,3}/.test('aab')",
        r"/(?:(?:(?:(?:(a+[b]+)|x){2}){2})|){2,3}/.test('ab')",
        "/(a+b)|x/u.test('ab')",
        r"/(?:(?:(?:(?<n>a)|x(?:\k<n>)+){2})|){2,3}/.test('a')",
        r"/(?:(?:(?:((a)(?:\1)+)|x){2})|){2,3}/.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
