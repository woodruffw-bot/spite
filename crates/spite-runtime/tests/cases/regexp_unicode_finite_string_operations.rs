//! Finite character/string operands in set operations (22.2.2.9).

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn nested_unions_and_literal_operands_use_complete_member_equality_and_longest_priority() {
    check(
        r"let a=/[[a\q{ab|abc}]&&[a\q{ab}]]/dv.exec('abc!'),b=/[[a\q{ab|abc}]--[a\q{abc}]]/v.exec('abc!');a[0]==='ab' && a.indices[0][1]===2 && b[0]==='ab' && !/[[a\q{ab}]&&[a\q{cd}]]/v.test('cd')",
    );
    check(
        r"!/[a&&\q{ab}]/v.test('ab') && !/[\q{ab}&&a]/v.test('ab') && /[\q{ab}--a]/v.exec('ab')[0]==='ab' && /[[\q{a|ab|abc}]&&[\q{a|ab}]&&[\q{ab}]]/v.exec('abc')[0]==='ab' && /[[a\q{ab|abc}]--a--[\q{ab}]]/v.exec('abc')[0]==='abc'",
    );
    check(
        r"/[[[\q{a\]b}]\]]&&[\q{a\]b}]]/v.exec('xa]b')[0]==='a]b' && /[[\q{ab}\b\0]--[\q{ab}]]/v.exec('\0')[0]==='\0' && /[[\q{ab}\b\0]--[\q{ab}]]/v.exec('\b')[0]==='\b'",
    );
}

#[test]
fn canonical_members_outside_assertions_and_capture_indices_share_original_input_ranges() {
    check(
        r"let r=/(?<whole>(^\b[[K\q{sK|}]&&[k\q{ſK|}]]\b$))/dimv,a=r.exec('!\r\nſK\u2028!'),b=r.exec('!\r\nK\u2028!');a.index===3 && a[0]==='ſK' && a[1]==='ſK' && a[2]==='ſK' && a.groups.whole==='ſK' && a.indices[2][1]===5 && a.indices.groups.whole===a.indices[1] && b[0]==='K'",
    );
    check(
        r"let a=/[[\q{a!}a]&&[\q{a!}a]]\b/dv.exec('a!');a[0]==='a' && a.indices[0][1]===1 && /[[K\q{sK|}]--[\q{ſK}]]/iv.exec('ſK')[0]==='' && /[[K\q{sK|}]--[\q{ſK}]]/iv.exec('K')[0]==='K'",
    );
    check(
        r"/[[\q{ßa}]&&[\q{ẞA}]]/iv.exec('ẞA')[0]==='ẞA' && !/[[\q{ßa}]&&[\q{SSa}]]/iv.test('SSa') && !/[[\q{é}]&&[\q{é}]]/iv.test('é') && /[[\q{\u{10400}a}]&&[\q{\u{10428}A}]]/iv.exec('𐐨A')[0]==='𐐨A'",
    );
}

#[test]
fn empty_classes_and_empty_members_preserve_last_index_and_surrogate_boundaries() {
    check(
        r"let r=/(?<whole>([[\q{😀x|}🚀]--🚀]))/dvgy;r.lastIndex=2;let a=r.exec('A😀x'),b=r.exec('A😀x');a.index===1 && a[0]==='😀x' && a.indices[2][1]===4 && b.index===4 && b.groups.whole==='' && r.lastIndex===4",
    );
    check(
        r"let r=/(?<whole>([\q{|ab}--[\q{ab}]]))/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    check(
        r"let r=/[[]&&[\q{|ab}]]/vg;r.lastIndex=1;let m=r.exec('ab');m===null && r.lastIndex===0 && !r.test('') && /[\q{}--[]]/v.exec('')[0]==='' && !/[\q{}&&[]]/v.test('')",
    );
    check(
        r"let r=/[[]&&[\q{|ab}]]/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ab')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
    check(
        r"!/[\q{\u{D800}\u{DC00}|ab}&&[\q{\uD800\uDC00|ab}]]/v.test('𐀀') && /[[\q{\uD800\uDC00x}]--[]]/v.exec('𐀀x')[0]==='𐀀x' && /[[\q{\uD800a}]--[]]/v.exec('\uD800a')[0].charCodeAt(0)===0xd800",
    );
}

#[test]
fn consumers_clones_and_collection_keep_filtered_words_and_capture_metadata() {
    check(
        r"let r=/(?<whole>([[a\q{|ab}]--a]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===4 && a[2].indices[2][1]===4 && 'ab😀'.replace(r,'#')==='##😀#' && 'ab😀'.split(/[[a\q{|ab}]--a]/v).join('|')==='|😀'",
    );
    check(
        r"let r=/[[a\q{ab|abc}]--a]/vg;r.lastIndex=8;'xabc!'.search(r)===1 && r.lastIndex===8 && 'ababc'.match(r).join('|')==='ab|abc' && r.lastIndex===0 && 'ababc'.replace(r,'#')==='##'",
    );
    check(
        r"let r=/[[K\q{sK}]&&[k\q{ſK}]]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('ſK')===null && c.lastIndex===0 && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([[a\q{ab|abc}]--a]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabc').groups.whole==='abc' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_unsupported_controls_finish_finalizers_and_remaining_grammar_is_proved_completely() {
    for (source, expected) in [
        (r"/[[\q{ab}]&&[\q{ab}]]/v.test('ab')", true),
        (r"/[a&&\q{ab}]/v.test('ab')", false),
        (r"/[a--a--\q{ab}]/v.test('a')", false),
        (r"/[\q{}&&a]/v.test('a')", false),
        (r"/[[\q{a|b}]--[\q{a|bc}]]/v.test('b')", true),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
            )),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
    for source in [
        r"/[[\q{ab}]&&[a-z]]/v.test('ab')",
        r"/[[]&&[\q{ab}]&&[\p{Any}]]/v.test('')",
        r"/[[a--a]&&[\q{ab}]]/v.test('a')",
        r"/[[\q{ab}]--[^a]]/v.test('ab')",
        r"/[[\q{ab}]--[\q{ab}]]+/v.test('ab')",
        r"/[[\q{ab}]--[\q{ab}]][a]/v.test('aba')",
        r"/\b^[[\q{ab}]&&[\q{ab}]]/v.test('ab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    for pattern in [
        r"[^[\q{ab}]&&[\q{ab}]]",
        r"[[\q{ab}]&&[\q{ab}]--a]",
        r"[[\q{ab}]&&[\q{\d}]]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn deep_operands_use_defaults_and_opt_in_aborts_preserve_assignment_and_last_index() {
    let pattern = format!(r"[{}\q{{ab}}{}--[]]", "[".repeat(3_000), "]".repeat(3_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('ab');m[0]==='ab' && r.lastIndex===2"
    ));
    let pattern = format!(r"[[\q{{ab}}]--[\q{{{}}}]]", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('ab');m[0]==='ab' && r.lastIndex===2"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=/{pattern}/vg;}}catch{{marker=1;}}finally{{marker=2;}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7 && marker===0"), Ok(Value::Boolean(true)));
    realm.eval(r"r=/[[a\q{ab}]--a]/vg").unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "x".repeat(20_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("r.lastIndex===1 && marker===0"),
        Ok(Value::Boolean(true))
    );
}
