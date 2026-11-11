//! CompileAssertion boundaries around fixed ordinary character-set sequences.

use spite_runtime::Realm;

use crate::common::assert_true;

#[test]
fn input_anchors_use_complete_input_and_preserve_original_slots() {
    assert_true(
        r#"
            let r = /^([a])a$/di;
            r.lastIndex = 99;
            let m = r.exec('AA');
            m[0] === 'AA' && m[1] === 'A' && m.index === 0 && m.indices[1][0] === 0 && m.indices[1][1] ===
                1 && r.lastIndex === 99 && r.exec('xAA') === null && r.exec('AAx') === null
        "#,
    );
    assert_true(
        r#"
            /^[a]a$/.exec('aa\n') === null && /^[a]a$/.exec('aa\r') === null && /^[a]a$/.exec(
                'aa\u2028') === null && /^[a]a$/.exec('aa\u2029') === null
        "#,
    );
    assert_true("let r=/^[a]a$/g;r.lastIndex=1;r.exec('aa')===null&&r.lastIndex===0");
}

#[test]
fn multiline_boundaries_include_all_line_terminators_and_crlf_positions() {
    assert_true(
        r"['\n','\r','\u2028','\u2029'].every(t=>{let m=/^([ab])a$/dm.exec('x'+t+'ba'+t+'y');return m.index===2&&m[0]==='ba'&&m[1]==='b'&&m.indices[1][0]===2&&m.indices[1][1]===3})",
    );
    assert_true(
        r"let r=/^[a]a$/gdm,s='x\r\naa\r\naa';let a=r.exec(s),b=r.exec(s);a.index===3&&b.index===7&&r.lastIndex===9&&r.exec(s)===null&&r.lastIndex===0",
    );
}

#[test]
fn sticky_anchors_keep_original_input_boundaries_and_failure_resets() {
    assert_true(
        r#"
            let r = /^[a]a/my;
            r.lastIndex = 1;
            let a = r.exec('xaa');
            r.lastIndex = 2;
            let b = r.exec('x\naa');
            a === null && b[0] === 'aa' && b.index === 2 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r"let r=/[a]a$/my;r.lastIndex=1;let m=r.exec('xaa\r');m.index===1&&r.lastIndex===3",
    );
}

#[test]
fn dotall_empty_captures_surrogates_and_escape_parity_remain_orthogonal() {
    assert_true(
        r"/^([a]).([b])$/s.exec('a\nb')[2]==='b'&&/^([a]).([b])$/.exec('a\nb')===null&&/^.[b]$/m.exec('\nab').index===1",
    );
    assert_true(
        "let m=/^([a])()$/d.exec('a');m[1]==='a'&&m[2]===''&&m.indices[2][0]===1&&m.indices[2][1]===1",
    );
    assert_true(
        r#"
            let m = /^([\uD800])([\uDC00])$/d.exec('\uD800\uDC00');
            m[0].length === 2 && m[1].length === 1 && m[2].length === 1 && m.indices[2][0] === 1 && m
                .indices[2][1] === 2
        "#,
    );
    assert_true(
        r"/^[a]\$$/.test('a$')&&/^[a]\\$/.test('a\\')&&/^[.$]$/.test('$')&&/^[.$]$/.test('.')",
    );
}

#[test]
fn generic_string_consumers_share_anchored_fixed_sequence_execution() {
    assert_true(
        r"'xaa\naa'.match(/^[a]a$/gm).join(',')==='aa'&&'x\naa\ny'.search(/^[a]a$/m)===2&&'aa\nab'.replace(/^([a])([ab])$/gm,'$2$1')==='aa\nba'",
    );
    assert_true(
        r#"
            let it = 'aa\nba'.matchAll(/^([ab])([a])$/dgmi),
                a = it.next().value,
                b = it.next().value;
            a.index === 0 && b.index === 3 && b[1] === 'b' && b.indices[2][0] === 4 && it.next().done
        "#,
    );
}

#[test]
fn internal_assertions_choices_quantifiers_and_unicode_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:a+(^[a])+){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:((?:(?:[a]+^b+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:^[a]+[b]+|b$){2}){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:((?:(?:^[a]*[b]+$){2}){2}){2})|){2,3}/.test('a')",
        "/^([a])+b$/u.test('a')",
        "/^([a])+b$/v.test('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
