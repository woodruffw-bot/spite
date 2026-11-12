//! Complete noncapturing branch wrappers preserve one-unit union predicates.

use spite_runtime::{Error, Limits, Realm, Value};

use crate::common::assert_true;

#[test]
fn fixed_choices_preserve_source_ordered_capture_positions() {
    assert_true(
        r#"
            let a = /(x((?:[^z])|(?:b))y)(c|d)/d.exec('qxbyd');
            a.index === 1 && a[0] === 'xbyd' && a[1] === 'xby' && a[2] === 'b' && a[3] === 'd' && a
                .indices[1][0] === 1 && a.indices[1][1] === 4 && a.indices[2][0] === 2 && a.indices[3][
                0] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?:((?:[^z])|(?:b)))()((?:c|d))/d.exec('bd');
            a.length === 4 && a[1] === 'b' && a[2] === '' && a[3] === 'd' && a.indices[2][0] === 1 && a
                .indices[2][1] === 1 && a.indices[1] !== a.indices[3]
        "#,
    );
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b)))((c|d))/d.exec('ac');
            a[1] === 'a' && a[2] === 'a' && a[3] === 'c' && a[4] === 'c' && a.indices[1] !== a.indices[
                2] && a.indices[1][0] === 0 && a.indices[3][0] === 1
        "#,
    );
}

#[test]
fn repeated_fixed_choice_bodies_keep_last_iteration_and_undefined_slots() {
    assert_true(
        r#"
            let a = /(x((?:[^z])|(?:b)))+(y)/d.exec('xaxby');
            a[1] === 'xb' && a[2] === 'b' && a[3] === 'y' && a.indices[1][0] === 2 && a.indices[2][0] ===
                3 && a.indices[3][0] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b))(c|d)){2}(x)/d.exec('acbdx');
            a[1] === 'bd' && a[2] === 'b' && a[3] === 'd' && a[4] === 'x' && a.indices[1][0] === 2 && a
                .indices[2][0] === 2 && a.indices[3][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b))(c|d))*(x)/d.exec('x');
            a[1] === undefined && a[2] === undefined && a[3] === undefined && a[4] === 'x' && a.indices[
                1] === undefined && Object.hasOwn(a.indices, '3')
        "#,
    );
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b))())+/d.exec('ab');
            a[1] === 'b' && a[2] === 'b' && a[3] === '' && a.indices[1][0] === 1 && a.indices[3][0] === 2
        "#,
    );
}

#[test]
fn fixed_choices_surround_repetition_with_partial_enclosing_captures() {
    assert_true(
        r#"
            let a = /((x|y)(((?:[^z])|(?:b))(c|d))+)(e|f)/d.exec('y acbd'.replace(' ', '') + 'f');
            a[1] === 'yacbd' && a[2] === 'y' && a[3] === 'bd' && a[4] === 'b' && a[5] === 'd' && a[6] ===
                'f' && a.indices[1][0] === 0 && a.indices[1][1] === 5 && a.indices[3][0] === 3 && a
                .indices[6][0] === 5
        "#,
    );
    assert_true(
        r#"
            let a = /(x|y)(a+)(b|c)/d.exec('yaaac');
            a[1] === 'y' && a[2] === 'aaa' && a[3] === 'c' && a.indices[2][0] === 1 && a.indices[2][1] ===
                4
        "#,
    );
    assert_true(
        r#"
            let a = /((x|y)(((?:[^z])|(?:b))(c|d))*)(e|f)/d.exec('xf');
            a[1] === 'x' && a[2] === 'x' && a[3] === undefined && a[4] === undefined && a[5] ===
                undefined && a[6] === 'f' && a.indices[1][1] === 1
        "#,
    );
}

#[test]
fn greedy_lazy_bounds_and_assertions_keep_candidate_order() {
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b))c)+(ac)/d.exec('acbcac'),
                b = /(((?:[^z])|(?:b))c)+?(ac)/d.exec('acbcac');
            a[0] === 'acbcac' && b[0] === 'acbcac' && a[1] === 'bc' && a[2] === 'b' && a.indices[2][0] ===
                2 && b.indices[2][0] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(((?:[^z])|(?:b))c){1,2}(ac)/d.exec('acbcacac');
            a.index === 0 && a[1] === 'bc' && a[2] === 'b' && a[3] === 'ac'
        "#,
    );
    assert_true(
        r#"
            let a = /^(((?:[^z])|(?:b))($)(\n))+(c|d)$/dm.exec('x\na\nb\nc');
            a.index === 0 && a[1] === 'b\n' && a[2] === 'b' && a[3] === '' && a[4] === '\n' && a[5] ===
                'c' && a.indices[2][0] === 4 && /^(((?:[^z])|(?:b))($)(\n))+(c|d)$/.exec('a\nc') === null
        "#,
    );
    assert_true(
        r#"
            let a = /\b(x((?:[^z])|(?:b)))\b/d.exec(' xb ');
            a.index === 1 && a[1] === 'xb' && a[2] === 'b' && a.indices[2][0] === 2 &&
                /\b(x((?:[^z])|(?:b)))\b/.exec('xbz') === null
        "#,
    );
}

#[test]
fn alternatives_and_state_keep_branch_capture_offsets_and_original_source() {
    assert_true(
        r#"
            let a = /(x((?:[^cd])|(?:b)))|(x(c|d))/d.exec('xd'),
                b = /(x((?:[^cd])|(?:b)))|(x(c|d))/d.exec('xa');
            a[1] === undefined && a[2] === undefined && a[3] === 'xd' && a[4] === 'd' && a.indices[4][
                0] === 1 && b[1] === 'xa' && b[2] === 'a' && b[3] === undefined && b[4] === undefined
        "#,
    );
    assert_true(
        r#"
            let r = /x((?:[^z])|(?:b))y/dg,
                copy = new RegExp(r),
                a = r.exec('xay xby'),
                b = r.exec('xay xby');
            a.indices[1][0] === 1 && b.index === 4 && b.indices[1][0] === 5 && r.lastIndex === 7 && copy
                .source === 'x((?:[^z])|(?:b))y' && copy.lastIndex === 0 && r.exec('xay xby') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(x((?:[^z])|(?:b)))+/dy;
            r.lastIndex = 1;
            let a = r.exec('qxaxb');
            a.index === 1 && a[1] === 'xb' && a[2] === 'b' && a.indices[2][0] === 4 && r.lastIndex === 5 &&
                r.exec('qxaxb') === null && r.lastIndex === 0
        "#,
    );
}

#[test]
fn escapes_flags_surrogates_and_adjacent_empty_groups_keep_exact_units() {
    assert_true(
        r#"
            let a = /\0()(\x61|\u0062)1/d.exec('\0b1');
            a[1] === '' && a[2] === 'b' && a.indices[1][0] === 1 && a.indices[2][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(µ|Μ)(ſ|S)(σ|ς)/di.exec('ΜSΣ');
            a[1] === 'Μ' && a[2] === 'S' && a[3] === 'Σ' && /(ſ|S)/i.exec('s')[0] === 's' && /(ſ|S)/i.exec(
                'ſ')[0] === 'ſ'
        "#,
    );
    assert_true(
        r#"
            let a = /.((?:[^z])|(?:b))./ds.exec('\nab\n');
            a[0] === '\nab' && a[1] === 'a' && a.indices[1][0] === 1 && /.((?:[^z])|(?:b))./.exec(
                '\nab\n') === null
        "#,
    );
    assert_true(
        r#"
            let a = /(\uD83D|a)(\uDCA9|b)/d.exec('💩');
            a[1].charCodeAt(0) === 55357 && a[2].charCodeAt(0) === 56489 && a.indices[1][1] === 1 && a
                .indices[2][0] === 1
        "#,
    );
}

#[test]
fn consumers_callbacks_and_intrinsic_arrays_observe_fixed_choice_captures() {
    assert_true(
        r#"
            'qxbyz'.replace(/(x((?:[^z])|(?:b))y)/, '<$1,$2>') === 'q<xby,b>z' && 'qxbyz'.split(
                /(x((?:[^z])|(?:b))y)/).join(',') === 'q,xby,b,z' && 'qxby'.search(
                /x((?:[^z])|(?:b))y/) === 1
        "#,
    );
    assert_true(
        r#"
            let r = /(x((?:[^z])|(?:b))y)/dg,
                a = [...'xay xby'.matchAll(r)];
            a.length === 2 && a[1][1] === 'xby' && a[1][2] === 'b' && a[1].indices[2][0] === 5 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen;
            let s = 'qxby'.replace(/(x((?:[^z])|(?:b))y)/, (whole, outer, inner, index, input) => {
                seen = [whole, outer, inner, index, input];
                return 'z'
            });
            s === 'qz' && seen[1] === 'xby' && seen[2] === 'b' && seen[3] === 1 && seen[4] === 'qxby'
        "#,
    );
    assert_true(
        r#"
            let calls = 0;
            Object.defineProperty(Array.prototype, '2', {
                set() {
                    calls++
                },
                configurable: true
            });
            let a = /(((?:[^z])|(?:b)))()/d.exec('b');
            calls === 0 && a[1] === 'b' && a[2] === 'b' && a[3] === '' && a.indices[1] !== a.indices[2] &&
                Object.hasOwn(a, '2') && Object.hasOwn(a.indices, '2')
        "#,
    );
}

#[test]
fn each_branch_inverts_after_canonicalization_before_union() {
    assert_true(
        r#"
            /((?:[^a])|(?:b))/i.exec('A') === null && /((?:[^a])|(?:b))/.exec('A')[1] === 'A' &&
                /((?:[^a])|(?:a))/i.exec('A')[1] === 'A' && /((?:[^a])|(?:[^b]))/i.exec('Ab')[1] === 'A'
        "#,
    );
    assert_true(
        r#"
            let a = /((?:[^µ])|(?:[Μ]))/di.exec('µ');
            a[1] === 'µ' && a.indices[1][0] === 0 && /((?:[^µ])|(?:[Μ]))/.exec('µ') === null &&
                /((?:[^ſ])|(?:S))/i.exec('ſ') === null && /((?:[^ſ])|(?:S))/i.exec('s')[1] === 's'
        "#,
    );
    assert_true(
        r#"
            /([^\w]|\w)/i.exec('ſ')[1] === 'ſ' && /([^\D]|[1])/.exec('2')[1] === '2' && /([^\D]|[1])/.exec(
                'x') === null && /([^\s]|\s)/.exec('\n')[1] === '\n'
        "#,
    );
    assert_true(
        r#"
            let a = /([]|[^])+/d.exec('\n\uD800');
            a[0].length === 2 && a[1].charCodeAt(0) === 55296 && a.indices[1][0] === 1 && /([]|[^])+/.exec(
                '') === null
        "#,
    );
    assert_true(
        r#"
            let a = /((?:[^a])|(?:b))|a/d.exec('a'),
                b = /((?:[^a])|(?:b))|a/d.exec('b');
            a[1] === undefined && a.indices[1] === undefined && b[1] === 'b' && b.indices[1][0] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /((x|[^z])((a|[^z])(c|d))+)(e|f)/d.exec('yacbdf');
            a[1] === 'yacbd' && a[2] === 'y' && a[3] === 'bd' && a[4] === 'b' && a[5] === 'd' && a[6] ===
                'f' && a.indices[3][0] === 3 && a.indices[6][0] === 5
        "#,
    );
}

#[test]
fn opted_in_abort_and_wider_or_conditional_choices_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(x((?:[^z])|(?:b)))+y/y,s='xb'.repeat(20000)+'y',flag=0")
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
        r"/(?:(?:(?:(?:x(a|bc)y){2}){2})|){2,3}/.test('xay')",
        r"/(?:(?:(x(a|bc))+)|){2,3}/.test('xa')",
        r"/(?:(?:(?:(?:x((a)|(b))y){2}){2})|){2,3}/.test('xay')",
        r"/(?:(?:(?:(?:x(a|)y){2}){2})|){2,3}/.test('xay')",
        r"/(?:(?:(?:(?:x(a|[^b]c)y){2}){2})|){2,3}/.test('xay')",
        r"/(?:(?:(((?:[^z])|(?:b))+(c|d)+))|){2,3}/.test('ac')",
        r"/(?:(?:(?:x(?<n>a|b)y(?:\k<n>)+){2})|){2,3}/.test('xay')",
        r"/(?:(?:(?:x((?:[^z])|(?:b))y(?:\1)+){2})|){2,3}/.test('xaya')",
        r"/x((?:[^z])|(?:b))y/u.test('xay')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
