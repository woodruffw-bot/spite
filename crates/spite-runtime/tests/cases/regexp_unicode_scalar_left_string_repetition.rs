//! Quantified scalar-left string operations consume character atoms (22.2.2.3).

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
fn projected_repetition_keeps_greedy_lazy_and_counted_character_priority() {
    check(
        r"/[\p{Any}&&\q{a|ab|}]+/v.exec('aaab')[0]==='aaa' && /[\p{Any}&&\q{a|ab|}]+?/v.exec('aaab')[0]==='a' && /[\p{Any}&&\q{a|ab|}]{1,2}/v.exec('aaab')[0]==='aa' && /[\p{Any}&&\q{a|ab|}]{1,2}?/v.exec('aaab')[0]==='a'",
    );
    check(
        r"/[\p{Any}--\q{a|ab|}]+/v.exec('ab😀c')[0]==='b😀c' && /[\p{Any}--\q{a|ab|}]+?/v.exec('ab😀c')[0]==='b' && /^[\p{Any}--\q{}]{3}$/v.test('ab😀') && !/^[\p{Any}--\q{}]{4}$/v.test('ab😀')",
    );
    check(
        r"/^[[0-9]--\q{0|2|4|9\uFE0F\u20E3}]+$/v.test('1356789') && !/^[[0-9]--\q{0|2|4|9\uFE0F\u20E3}]+$/v.test('9\uFE0F\u20E3') && /^[\p{ASCII_Hex_Digit}&&\q{0|2|4|9\uFE0F\u20E3}]+$/v.test('024') && !/^[_&&\q{0|2|4|9\uFE0F\u20E3}]+$/v.test('_')",
    );
}

#[test]
fn q_empty_members_cannot_create_empty_iterations_but_zero_counts_can_match_empty() {
    check(
        r"!/[\p{Any}&&\q{ab|}]+/v.test('ab') && !/[\p{Any}&&\q{ab|}]+/v.test('') && /[\p{Any}&&\q{ab|}]*/v.exec('ab')[0]==='' && /[\p{Any}&&\q{ab|}]?/v.exec('ab')[0]==='' && /[\p{Any}--\q{}]+/v.exec('ab')[0]==='ab' && !/[\p{Any}--\q{}]+/v.test('')",
    );
    check(
        r"let r=/(?<whole>([\p{Any}&&\q{ab|}]{0}))/dvgy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    let huge = "9".repeat(100);
    check(&format!(
        r"!/[\p{{Any}}--\q{{ab}}]{{{huge}}}/v.test('ab') && /^[\p{{Any}}--\q{{ab}}]{{1,{huge}}}$/v.test('ab😀') && /[\p{{Any}}&&\q{{ab|}}]{{0,{huge}}}/v.exec('ab')[0]===''"
    ));
}

#[test]
fn whole_repetition_assertions_folding_names_and_surrogate_ranges_use_original_input() {
    check(
        r"let r=/(?<whole>(^\b[[a-z]&&[a-z\q{ab}]]+?\b$))/dimv,m=r.exec('!\r\nſKA\u2028!');m.index===3 && m[0]==='ſKA' && m[1]==='ſKA' && m[2]==='ſKA' && m.groups.whole==='ſKA' && m.indices[2][1]===6 && m.indices.groups.whole===m.indices[1]",
    );
    check(
        r"let r=/(?<whole>([\p{Any}&&\q{😀|😀x|}]+))/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.whole==='😀😀' && m.indices[2][0]===1 && m.indices[2][1]===5 && r.lastIndex===5 && r.exec('A😀😀x')===null && r.lastIndex===0",
    );
    check(
        r"let r=/[\p{Any}&&\q{\uD800a|\uD800|\u{D800}\u{DC00}}]+/v,m=r.exec('\uD800\uD800a');m[0].length===2 && m[0].charCodeAt(0)===0xd800 && m[0].charCodeAt(1)===0xd800 && !r.test('𐀀') && /[\p{ASCII}--\q{k|sK|}]+/iv.exec('KſA')[0]==='ſA'",
    );
    check(
        r"let r=/[\p{Any}&&\q{ab|}]*/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn repeated_projection_string_consumers_clones_and_collection_keep_metadata_and_state() {
    check(
        r"let r=/(?<whole>([\p{Any}&&\q{ab|}]*))/dvg,a=[...'ab😀'.matchAll(r)];a.length===4 && a[0].index===0 && a[1].index===1 && a[2].index===2 && a[3].index===4 && a[3][0]==='' && a[3].groups.whole==='' && a[3].indices[2][1]===4 && 'ab😀'.replace(r,'#')==='#a#b#😀#' && 'ab😀'.split(/[\p{Any}&&\q{ab|}]*/v).join('|')==='a|b|😀'",
    );
    check(
        r"let r=/[\p{Any}--\q{a|ab}]+/vg;r.lastIndex=8;'aab😀c!'.search(r)===2 && r.lastIndex===8 && 'ab😀ca!'.match(r).join('|')==='b😀c|!' && r.lastIndex===0 && 'ab😀ca!'.replace(r,'#')==='a#a#'",
    );
    check(
        r"let r=/[[a-z]&&[a-z\q{ab}]]+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('ſK')===null && c.lastIndex===0 && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\p{Any}&&\q{a|ab}]+))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xaaab').groups.whole==='aaa' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_controls_finish_finalizers_and_zero_counts_still_require_complete_class_proof() {
    for (source, expected) in [
        (r"/([\p{Any}--\q{ab}])+/v.test('ab')", true),
        (r"/[^\p{Any}--\q{ab}]+/v.test('ab')", false),
        (r"/[[a\q{ab}]&&[a-z]]+/v.test('ab')", true),
        (r"/[\p{Any}--\q{ab}]+/v.test('ab')", true),
        (r"/[\p{Any}&&\q{ab}]+/v.test('ab')", false),
        (r"/[\p{Any}&&\q{ab}]{0}/v.test('')", true),
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
    assert_eq!(Realm::default().eval(r####"var pattern="[[]&&\\q{ab}&&\\p{RGI_Emoji}]{0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\"],[[1,1]],1,1,null,null]""####),Ok(Value::Boolean(true)));
    for source in [
        r"/[\p{Any}--\q{ab}]+[a]/v.test('aba')",
        r"/\b^[\p{Any}--\q{ab}]+/v.test('ab')",
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
        r"[^\q{ab}--\p{Any}]+",
        r"[\p{Any}&&\q{ab}--a]{0}",
        r"[\p{Any}&&\q{\d}]{0}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn deep_repeated_classes_use_defaults_and_opt_in_constructor_and_search_aborts_are_atomic() {
    let pattern = format!(
        r"[\p{{Any}}&&{}\q{{a|ab}}{}]+",
        "[".repeat(3_000),
        "]".repeat(3_000)
    );
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('aaa');m[0]==='aaa' && r.lastIndex===3"
    ));
    let pattern = format!(r"[\p{{Any}}--\q{{{}}}]{{2}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('😀a');m[0]==='😀a' && r.lastIndex===3"
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
    realm.eval(r"r=/[a&&[a-z\q{ab}]]+/vg").unwrap();
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
