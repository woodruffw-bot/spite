//! Scalar-left string operations remain character sets (22.2.2.9).

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
fn scalar_left_nested_algebra_projects_complete_members_and_never_matches_empty_words() {
    check(
        r"let a=/[\p{Any}&&\q{a|ab|}]/dv.exec('ab'),b=/[\p{Any}--\q{a|ab|}]/v.exec('ab');a[0]==='a' && a.indices[0][1]===1 && b[0]==='b' && b.index===1 && !/[\p{Any}&&\q{a|ab|}]/v.test('') && !/[\p{Any}--\q{a|ab|}]/v.test('') && /[\p{Any}--\q{}]/v.exec('😀')[0]==='😀'",
    );
    check(
        r"/[\p{ASCII}&&[a-z\q{ab}]]/v.exec('ab')[0]==='a' && /[a&&[a-z\q{ab}]]/v.exec('ab')[0]==='a' && /[\d--[\q{5|55}]]/v.exec('05')[0]==='0' && !/[\d--[\q{5|55}]]/v.test('5') && !/[[]--[\q{|ab}]]/v.test('')",
    );
    check(
        r"/[\p{Any}--[\q{}--\q{ab}]]/v.exec('😀')[0]==='😀' && !/[\p{Any}--[^\q{ab}&&\p{Any}]]/v.test('ab') && !/[\p{Any}&&[\q{ab}--\q{ab}]]/v.test('ab') && /[[^]--[[\q{a|ab}]&&[\q{b|ab}]]]/v.exec('😀')[0]==='😀'",
    );
}

#[test]
fn canonical_projection_property_complements_and_normalization_keep_distinct_members() {
    check(
        r"let a=/[\p{ASCII}--[\q{k|sK|}]]/iv.exec('Kſ');a.index===1 && a[0]==='ſ' && /[[a-z]&&[\q{K|sK}--\q{sk}]]/iv.exec('K')[0]==='K' && !/[[a-z]&&[\q{K|sK}--\q{sk}]]/v.test('K')",
    );
    check(
        r"/[\p{Any}&&[\q{ßa|ß}]]/iv.exec('ẞA')[0]==='ẞ' && !/[\p{Any}&&[\q{ßa|ß}]]/iv.test('SSa') && /[\p{Any}&&[\q{é|é}]]/v.exec('é')[0]==='é' && !/[\p{Any}&&[\q{é|é}]]/v.test('é')",
    );
    check(
        r"!/[\s--\q{\uFEFF|ab}]/v.test('\uFEFF') && /[\p{White_Space}--\q{}]/v.exec('\u0085')[0]==='\u0085' && !/[\p{White_Space}--\q{}]/v.test('\uFEFF') && /[\p{Any}&&[\q{\uFEFF|\u0085|ab}--\s]]/v.exec('\u0085')[0]==='\u0085'",
    );
}

#[test]
fn scalar_results_keep_original_capture_ranges_pair_starts_and_strict_last_index_writes() {
    check(
        r"let r=/(?<whole>(^\b[[a-z]&&[\q{a|ab}]]\b$))/dimv,m=r.exec('!\r\nA\u2028!');m.index===3 && m[0]==='A' && m[1]==='A' && m[2]==='A' && m.groups.whole==='A' && m.indices[2][1]===4 && m.indices.groups.whole===m.indices[1]",
    );
    check(
        r"let r=/(?<whole>([\p{Any}&&\q{😀|😀x|}]))/dvgy;r.lastIndex=2;let m=r.exec('A😀x');m.index===1 && m[0]==='😀' && m.groups.whole==='😀' && m.indices[2][0]===1 && m.indices[2][1]===3 && r.lastIndex===3 && r.exec('A😀x')===null && r.lastIndex===0",
    );
    check(
        r"let r=/[\p{Any}&&\q{\uD800a|\uD800|\u{D800}\u{DC00}}]/v,m=r.exec('\uD800a');m[0].length===1 && m[0].charCodeAt(0)===0xd800 && !r.test('𐀀') && !/[\p{Any}&&\q{\u{D800}\u{DC00}}]/v.test('𐀀')",
    );
    for pattern in [r"[\p{Any}&&\q{ab}]", r"[\p{Any}--\q{ab}]"] {
        check(&format!(
            "let r=/{pattern}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
        ));
    }
}

#[test]
fn consumers_clones_and_collection_use_consuming_character_results() {
    check(
        r"let r=/(?<whole>([\p{Any}--\q{a|ab|}]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===2 && a[0][0]==='b' && a[0].index===1 && a[1][0]==='😀' && a[1].indices[2][1]===4 && r.lastIndex===0 && 'ab😀'.replace(r,'#')==='a##' && 'ab😀'.split(/[\p{Any}--\q{a|ab|}]/v).join('|')==='a||'",
    );
    check(
        r"let r=/[\p{Any}--\q{a|ab|}]/vg;r.lastIndex=8;'ab😀'.search(r)===1 && r.lastIndex===8 && 'ab😀'.match(r).join('|')==='b|😀' && r.lastIndex===0",
    );
    check(
        r"let r=/[\p{ASCII}--[\q{k|sK|}]]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('K')===null && c.lastIndex===0 && r.exec('ſK')[0]==='ſ' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\p{Any}--\q{a|ab}]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xab').groups.whole==='b' && r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_unsupported_controls_finish_finalizers_and_complete_later_proof_and_early_errors() {
    for (source, expected) in [
        (r"/[[a--a]&&[\q{ab}]]/v.test('a')", false),
        (r"/[[a-z]&&\q{ab}]/v.test('ab')", false),
        (r"/[\p{Any}--\q{ab}]/v.test('ab')", true),
        (r"/[\p{Any}&&\q{ab}]/v.test('ab')", false),
        (r"/[\p{Any}&&[a-z\q{ab}]]/v.test('ab')", true),
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
        r"/[[]&&[\q{ab}]&&\p{RGI_Emoji}]/v.test('')",
        r"/[[a\q{ab}]&&[a-z\q{ab}]]/v.test('ab')",
        r"/[^\p{Any}&&\q{ab}]/v.test('ab')",
        r"/[\p{Any}--\q{ab}]+/v.test('ab')",
        r"/[\p{Any}--\q{ab}][a]/v.test('aba')",
        r"/\b^[\p{Any}--\q{ab}]/v.test('ab')",
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
        r"[^\q{ab}--\p{Any}]",
        r"[\p{Any}&&\q{ab}--a]",
        r"[\p{Any}&&\q{\d}]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn deep_and_long_projected_operands_use_defaults_and_opt_in_aborts_preserve_state() {
    let pattern = format!(
        r"[\p{{Any}}&&{}\q{{a|ab}}{}]",
        "[".repeat(3_000),
        "]".repeat(3_000)
    );
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('ab');m[0]==='a' && r.lastIndex===1"
    ));
    let pattern = format!(r"[\p{{Any}}--[\q{{{}}}]]", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('😀');m[0]==='😀' && r.lastIndex===2"
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
    // Keep construction small so this opted-in quota reaches scalar-left search.
    realm.eval(r"r=/[a&&[a-z\q{ab}]]/vg").unwrap();
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
