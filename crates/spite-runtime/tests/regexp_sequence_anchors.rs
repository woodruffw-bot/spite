//! CompileAssertion boundaries around fixed ordinary character-set sequences.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn input_anchors_use_complete_input_and_preserve_original_slots() {
    check(
        "let r=/^([a])a$/di;r.lastIndex=99;let m=r.exec('AA');m[0]==='AA'&&m[1]==='A'&&m.index===0&&m.indices[1][0]===0&&m.indices[1][1]===1&&r.lastIndex===99&&r.exec('xAA')===null&&r.exec('AAx')===null",
    );
    check(
        r"/^[a]a$/.exec('aa\n')===null&&/^[a]a$/.exec('aa\r')===null&&/^[a]a$/.exec('aa\u2028')===null&&/^[a]a$/.exec('aa\u2029')===null",
    );
    check("let r=/^[a]a$/g;r.lastIndex=1;r.exec('aa')===null&&r.lastIndex===0");
}

#[test]
fn multiline_boundaries_include_all_line_terminators_and_crlf_positions() {
    check(
        r"['\n','\r','\u2028','\u2029'].every(t=>{let m=/^([ab])a$/dm.exec('x'+t+'ba'+t+'y');return m.index===2&&m[0]==='ba'&&m[1]==='b'&&m.indices[1][0]===2&&m.indices[1][1]===3})",
    );
    check(
        r"let r=/^[a]a$/gdm,s='x\r\naa\r\naa';let a=r.exec(s),b=r.exec(s);a.index===3&&b.index===7&&r.lastIndex===9&&r.exec(s)===null&&r.lastIndex===0",
    );
}

#[test]
fn sticky_anchors_keep_original_input_boundaries_and_failure_resets() {
    check(
        r"let r=/^[a]a/my;r.lastIndex=1;let a=r.exec('xaa');r.lastIndex=2;let b=r.exec('x\naa');a===null&&b[0]==='aa'&&b.index===2&&r.lastIndex===4",
    );
    check(r"let r=/[a]a$/my;r.lastIndex=1;let m=r.exec('xaa\r');m.index===1&&r.lastIndex===3");
}

#[test]
fn dotall_empty_captures_surrogates_and_escape_parity_remain_orthogonal() {
    check(
        r"/^([a]).([b])$/s.exec('a\nb')[2]==='b'&&/^([a]).([b])$/.exec('a\nb')===null&&/^.[b]$/m.exec('\nab').index===1",
    );
    check(
        "let m=/^([a])()$/d.exec('a');m[1]==='a'&&m[2]===''&&m.indices[2][0]===1&&m.indices[2][1]===1",
    );
    check(
        r"let m=/^([\uD800])([\uDC00])$/d.exec('\uD800\uDC00');m[0].length===2&&m[1].length===1&&m[2].length===1&&m.indices[2][0]===1&&m.indices[2][1]===2",
    );
    check(r"/^[a]\$$/.test('a$')&&/^[a]\\$/.test('a\\')&&/^[.$]$/.test('$')&&/^[.$]$/.test('.')");
}

#[test]
fn generic_string_consumers_share_anchored_fixed_sequence_execution() {
    check(
        r"'xaa\naa'.match(/^[a]a$/gm).join(',')==='aa'&&'x\naa\ny'.search(/^[a]a$/m)===2&&'aa\nab'.replace(/^([a])([ab])$/gm,'$2$1')==='aa\nba'",
    );
    check(
        r"let it='aa\nba'.matchAll(/^([ab])([a])$/dgmi),a=it.next().value,b=it.next().value;a.index===0&&b.index===3&&b[1]==='b'&&b.indices[2][0]===4&&it.next().done",
    );
}

#[test]
fn construction_and_search_aborts_remain_uncatchable_host_errors() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/^\d$/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[a]aaaaab$/g,flag=0,s='x'.repeat(8000)")
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
fn internal_assertions_choices_quantifiers_and_unicode_remain_explicit_gaps() {
    for source in [
        r"/(?:(?:(?:(?:a+(^[a])+){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:((?:(?:[a]+^b+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:^[a]+[b]+|b$){2}){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:((?:(?:^[a]*[b]+$){2}){2}){2})|){2,3}/.test('a')",
        "/^[a]$/u.test('a')",
        "/^[a]$/v.test('a')",
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
