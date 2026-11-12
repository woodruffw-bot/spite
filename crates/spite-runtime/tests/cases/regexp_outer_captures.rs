//! Enclosing ordinary captures span the complete selected body match (22.2.2.3).

use spite_runtime::Realm;

use crate::common::assert_true;

#[test]
fn whole_match_captures_precede_inner_repetition_and_suffix_captures() {
    assert_true(
        r#"
            let a = /(a+b)/d.exec('xaaab'),
                b = /(((a)+(b)))/d.exec('aaab');
            a[1] === 'aaab' && a.indices[1][0] === 1 && a.indices[1][1] === 5 && b.length === 5 && b[1] ===
                'aaab' && b[2] === 'aaab' && b[3] === 'a' && b[4] === 'b' && b.indices[3][0] === 2 && b
                .indices[4][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /((?:a+?aa))/.exec('aaaa'),
                b = /(?:(a+b))/.exec('aaab');
            a[1] === 'aaa' && b[1] === 'aaab' && /((a)+b)/.exec('aaab')[2] === 'a'
        "#,
    );
    assert_true(
        r#"
            let m = /(^((a)+)(b)$)/md.exec('x\naab\ny');
            m.index === 2 && m[1] === 'aab' && m[2] === 'aa' && m[3] === 'a' && m[4] === 'b' && m.indices[
                1][0] === 2 && m.indices[4][0] === 4
        "#,
    );
}

#[test]
fn choices_preserve_source_order_and_own_undefined_inner_slots() {
    assert_true(
        r#"
            let m = /((x)|(a)+(b)|(y))/d.exec('aaab');
            m.length === 6 && m[1] === 'aaab' && m[2] === undefined && m[3] === 'a' && m[4] === 'b' && m[
                5] === undefined && Object.hasOwn(m, '2') && Object.hasOwn(m.indices, '5') && m.indices[4][
                    1
                ] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(a+|b)/.exec('baa'),
                b = /((a+?)|(aa))/.exec('aaa');
            a[1] === 'b' && b[1] === 'a' && b[2] === 'a' && b[3] === undefined
        "#,
    );
    assert_true(
        "let m=/((a)|(b))/.exec('b');m[1]==='b'&&m[2]===undefined&&m[3]==='b'&&m.groups===undefined",
    );
}

#[test]
fn zero_repetitions_empty_groups_and_final_iterations_remain_distinct() {
    assert_true(
        r#"
            let a = /((a)*())/d.exec(''),
                b = /((a*)())/d.exec('');
            a[1] === '' && a[2] === undefined && a[3] === '' && a.indices[1][1] === 0 && a.indices[2] ===
                undefined && b[1] === '' && b[2] === '' && b[3] === ''
        "#,
    );
    assert_true(
        r#"
            let a = /((a)*)/d.exec('aaa'),
                b = /(a)*/d.exec('aaa');
            a[1] === 'aaa' && a[2] === 'a' && a.indices[2][0] === 2 && b[1] === 'a' && b.indices[1][0] ===
                2 && /((a)*)/.exec('')[2] === undefined
        "#,
    );
    assert_true(
        r#"
            let a = /((?:))/.exec('x'),
                b = /((a)*?())/.exec('aaa');
            a[1] === '' && b[1] === '' && b[2] === undefined && b[3] === ''
        "#,
    );
}

#[test]
fn global_sticky_state_and_strict_lastindex_writes_preserve_complete_ranges() {
    assert_true(
        r#"
            let r = /(a+b)/g,
                a = r.exec('aab xaab'),
                b = r.exec('aab xaab');
            a[1] === 'aab' && b.index === 5 && b[1] === 'aab' && r.lastIndex === 8 && r.exec(
                'aab xaab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /((a)+(b))/y;
            r.lastIndex = 1;
            let m = r.exec('xaab');
            m.index === 1 && m[1] === 'aab' && m[2] === 'a' && m[3] === 'b' && r.lastIndex === 4 && r.exec(
                'xaab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a+b)/g;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let failed = false;
            try {
                r.exec('aab')
            } catch (e) {
                failed = e instanceof TypeError
            }
            failed
        "#,
    );
}

#[test]
fn original_source_flags_pinned_case_and_utf16_captures_are_preserved() {
    assert_true(
        r#"
            let r = new RegExp('(?:(a+b))', 'gi'),
                c = new RegExp(r);
            r.source === '(?:(a+b))' && r.toString() === '/(?:(a+b))/gi' && c.source === r.source && c !==
                r && RegExp(r) === r && c.exec('AAAB')[1] === 'AAAB'
        "#,
    );
    assert_true(
        r#"
            let m = /((.)+\uD800)/sd.exec('a\n\uD800');
            m[1] === 'a\n\uD800' && m[2] === '\n' && m.indices[1][1] === 3 && m.indices[2][0] === 1
        "#,
    );
    assert_true(
        "let m=/((µ)+(x))/i.exec('ΜΜX');m[1]==='ΜΜX'&&m[2]==='Μ'&&m[3]==='X'&&/(ſ+x)/i.exec('ssx')===null",
    );
    assert_true(r"let m=/([()|]+)/.exec('x(|)');m[1]==='(|)'&&/(\(\))/.exec('x()')[1]==='()'");
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_share_capture_layout() {
    assert_true(
        "'aaab'.replace(/((a)+(b))/,'$1-$2-$3')==='aaab-a-b'&&'aabxaab'.split(/((a)+(b))/).join(',')===',aab,a,b,x,aab,a,b,'",
    );
    assert_true(
        r#"
            let seen;
            let s = 'aaab'.replace(/((a)+(b))/, (whole, outer, a, b, index, input) => {
                seen = [whole, outer, a, b, index, input];
                return 'x'
            });
            s === 'x' && seen.join(',') === 'aaab,aaab,a,b,0,aaab'
        "#,
    );
    assert_true(
        r#"
            let r = /((a)+(b))/dg,
                m = [...'aaab aab'.matchAll(r)];
            m.length === 2 && m[1][1] === 'aab' && m[1][3] === 'b' && m[1].indices[3][0] === 7 && r
                .lastIndex === 0 && 'a'.match(/(a*)/g).length === 2
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
            let m = /((x)|(a)+(b))/d.exec('aaab'),
                p = Object.getOwnPropertyDescriptor(m, '1');
            calls === 0 && m[1] === 'aaab' && m[2] === undefined && m.indices[2] === undefined && Object
                .hasOwn(m, '2') && p.writable && p.enumerable && p.configurable
        "#,
    );
}

#[test]
fn pattern_variants_match_reference_results() {
    for source in [
        r"/(?:(?:(a+b)*)|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:a(a|bc)){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((a|bc)+))|){2,3}/.test('ab')",
        "/(a+b)/u.test('ab')",
        r"/(?:(?:(?:((?<x>a))(?:\k<x>)+){2})|){2,3}/.test('a')",
        r"/(?:(?:(?:(\1a)+){2})|){2,3}/.test('a')",
        r"/(?:(?:(?:((a)\1)+){2})|){2,3}/.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
