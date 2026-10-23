//! Character-only right operands in string-set operations (22.2.2.9).

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
fn scalar_membership_keeps_complete_characters_and_never_prefixes_or_empty_words() {
    check(
        r"let a=/[\q{|a|ab|abc|5|😀}&&\p{Any}]/dv.exec('abc'),b=/[\q{|a|ab|abc|5|😀}--\p{Any}]/v.exec('abc');a[0]==='a' && a.indices[0][1]===1 && b[0]==='abc' && !/[\q{ab}&&\p{Any}]/v.test('ab') && /[\q{ab}--\p{Any}]/v.exec('ab')[0]==='ab' && !/[\q{a|ab}--\p{Any}]/v.test('a')",
    );
    check(
        r"let a=/[[a\q{ab|abc}]--[a-z]--[\q{abc}]]/v.exec('abc'),b=/[\q{a|b|ab}&&[a-z]&&\q{b|ab}]/v.exec('ab');a[0]==='ab' && b[0]==='b' && b.index===1 && /[\q{a|ab|}--[a-z]--[]]/v.exec('!')[0]==='' && !/[\q{a|ab|}&&\p{Any}&&[^a]]/v.test('ab')",
    );
    check(
        r"/[\q{a|b|ab|5}&&[[a-z]--[aeiou]]]/v.exec('ab5')[0]==='b' && /[\q{a|ab|5}--\d]/v.exec('5ab')[0]==='ab' && /[\q{a|ab|5}&&\D]/v.exec('5ab')[0]==='a' && !/[\q{ab}&&\w]/v.test('ab')",
    );
}

#[test]
fn property_folding_complements_and_space_sets_apply_before_word_equality() {
    // CompileToCharSet and MaybeSimpleCaseFolding close the scalar set first.
    check(
        r"let r=/[[Kſ\q{sK|}]&&\p{ASCII}]/iv;r.exec('K')[0]==='K' && r.exec('ſK')[0]==='ſ' && !/[[Kſ\q{sK|}]&&\P{ASCII}]/iv.test('ſK') && /[[Kſ\q{sK|}]--\P{ASCII}]/iv.exec('ſK')[0]==='ſK' && /[\q{K|ab}&&[a-z]]/iv.exec('K')[0]==='K'",
    );
    check(
        r"/[[μ\q{ab}]&&\p{Script=Greek}]/iv.exec('µ')[0]==='µ' && !/[[μ\q{ab}]&&\p{Script=Greek}]/v.test('µ') && /[[μ\q{ab}]&&\p{Script=Greek}]/v.exec('μ')[0]==='μ' && !/[\q{ab}&&\P{Any}]/v.test('ab') && /[\q{|ab}--\P{Any}]/v.exec('ab')[0]==='ab'",
    );
    check(
        r"/[\q{\uFEFF|\u0085|ab}&&\s]/v.exec('\uFEFF')[0]==='\uFEFF' && !/[\q{\uFEFF|\u0085|ab}&&\s]/v.test('\u0085') && /[\q{\uFEFF|\u0085|ab}&&\p{White_Space}]/v.exec('\u0085')[0]==='\u0085' && !/[\q{\uFEFF|\u0085|ab}&&\p{White_Space}]/v.test('\uFEFF')",
    );
}

#[test]
fn captures_assertions_and_surrogates_use_original_character_boundaries() {
    check(
        r"let r=/(?<whole>(^\b[\q{a|ab|abc}&&[a-z]]\b$))/dmv,m=r.exec('!\r\na\u2028!');m.index===3 && m[0]==='a' && m[1]==='a' && m[2]==='a' && m.groups.whole==='a' && m.indices[0][0]===3 && m.indices[2][1]===4 && m.indices.groups.whole===m.indices[1]",
    );
    check(
        r"let r=/(?<whole>([\q{😀|😀x}&&[\u{0}-\u{10FFFF}]]))/dvgy;r.lastIndex=2;let m=r.exec('A😀x');m.index===1 && m[0]==='😀' && m.groups.whole==='😀' && m.indices[2][0]===1 && m.indices[2][1]===3 && r.lastIndex===3",
    );
    check(
        r"let r=/[\q{\uD800a|\uD800|\u{D800}\u{DC00}}&&\p{Any}]/v;r.exec('\uD800a')[0].length===1 && r.exec('\uD800a')[0].charCodeAt(0)===0xd800 && !r.test('𐀀') && /[\q{\u{D800}\u{DC00}|ab}--\p{Any}]/v.exec('ab')[0]==='ab' && !/[\q{\u{D800}\u{DC00}|ab}--\p{Any}]/v.test('𐀀')",
    );
}

#[test]
fn scalar_empty_results_and_surviving_empty_members_keep_last_index_contracts() {
    check(
        r"let r=/[\q{ab|}&&\p{Any}]/vg;r.lastIndex=1;let m=r.exec('ab');m===null && r.lastIndex===0 && !r.test('') && !/[[]&&\p{Any}&&\q{ab}]/v.test('')",
    );
    check(
        r"let r=/(?<whole>([\q{}--\p{Any}]))/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    for pattern in [r"[\q{}--\p{Any}]", r"[\q{ab|}&&\p{Any}]"] {
        check(&format!(
            "let r=/{pattern}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
        ));
    }
}

#[test]
fn string_consumers_clones_and_collection_share_the_filtered_member_plan() {
    check(
        r"let r=/(?<whole>([\q{a|ab|}--[a-z]]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===4 && a[2].indices[2][1]===4 && 'ab😀'.replace(r,'#')==='##😀#' && 'ab😀'.split(/[\q{a|ab|}--[a-z]]/v).join('|')==='|😀'",
    );
    check(
        r"let r=/[\q{a|ab|abc}--[a-z]]/vg;r.lastIndex=8;'xabc!'.search(r)===1 && r.lastIndex===8 && 'ababc'.match(r).join('|')==='ab|abc' && r.lastIndex===0 && 'ababc'.replace(r,'#')==='##'",
    );
    check(
        r"let r=/[[K\q{sK}]--\P{ASCII}]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('K')===null && c.lastIndex===0 && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\q{a|ab|abc}--[a-z]]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabc').groups.whole==='abc' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_controls_complete_finalizers_while_remaining_domains_and_early_errors_stay_distinct() {
    for (source, expected) in [
        (r"/\b^[\q{ab}--\d]/v.test('ab')", true),
        (r"/[^\q{ab}&&\p{Any}]/v.test('ab')", true),
        (r"/[[a--a]&&[\q{ab}]]/v.test('a')", false),
        (r"/[[a-z]&&\q{ab}]/v.test('ab')", false),
        (r"/[\q{ab}&&\p{Any}]/v.test('ab')", false),
        (r"/[\q{ab}--\d]/v.test('ab')", true),
        (r"/[[\q{ab}]--[a-z]]/v.test('ab')", true),
        (r"/[[]&&[\q{ab}]&&[\p{Any}]]/v.test('')", false),
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
        r"/[\q{a|ab}&&[a-z\q{ab}]]/v.test('ab')",
        r"/[[]&&\p{Any}&&\p{RGI_Emoji}&&\q{ab}]/v.test('')",
        r"/[\q{ab}--\d]+/v.test('ab')",
        r"/[\q{ab}--\d][a]/v.test('aba')",
        r"/\b^[\q{ab}--\d--\p{RGI_Emoji}]/v.test('ab')",
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
        r"[\q{ab}&&[a-z]--a]",
        r"[\q{\d}&&\p{Any}]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn deep_scalar_operands_use_defaults_and_opt_in_aborts_preserve_prior_state() {
    let pattern = format!(
        r"[\q{{a|ab}}&&{}a-z{}]",
        "[".repeat(3_000),
        "]".repeat(3_000)
    );
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('ab');m[0]==='a' && r.lastIndex===1"
    ));
    let pattern = format!(
        r"[\q{{ab}}--[{}a-z{}]]",
        "[".repeat(6_000),
        "]".repeat(6_000)
    );
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
    realm.eval(r"r=/[\q{a|ab}--[a-z]]/vg").unwrap();
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
