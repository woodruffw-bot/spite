//! Ranges, character escapes and properties beside variable q strings.

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
fn ranges_and_character_escapes_share_longest_priority_and_assertion_fallback() {
    check(
        r"let a=/[a-z\q{ab|abc}]/dv.exec('abc!'),b=/[\q{ab}\d]/v.exec('ab'),c=/[\D\q{ab}]/v.exec('ab');a[0]==='abc' && a.indices[0][1]===3 && b[0]==='ab' && c[0]==='ab' && /[\q{ab}\d]/v.exec('!5ab')[0]==='5' && !/[\q{ab}\d]/v.test('ac')",
    );
    check(
        r"let a=/[\q{a!}\p{Any}]\b/dv.exec('a!'),b=/[\q{a!}\p{Any}]\B/v.exec('a!');a[0]==='a' && a.indices[0][1]===1 && b[0]==='a!' && /[\w\q{sK}]/iv.exec('ſK')[0]==='ſK' && /[\w\q{sK}]/iv.exec('K')[0]==='K'",
    );
    check(
        r"let m=/^\b[\q{ab}a-z]\b$/mv.exec('!\r\nab\u2028!');m.index===3 && m[0]==='ab' && !/^\b[\q{ab}a-z]\b$/v.test('!\r\nab\u2028!')",
    );
}

#[test]
fn property_folding_complements_and_whitespace_retain_their_distinct_sets() {
    check(
        r"let r=/[\P{ASCII}\q{ab}]/iv;r.exec('ab')[0]==='ab' && r.exec('😀')[0]==='😀' && r.exec('K')===null && r.exec('ſ')===null && r.exec('A')===null && /[\P{ASCII}\q{ab}]/v.exec('K')[0]==='K' && /[\q{ab}\p{ASCII}]/iv.exec('K')[0]==='K'",
    );
    check(
        r"/[\p{Script=Greek}\q{ab}]/iv.exec('µ')[0]==='µ' && !/[\p{Script=Greek}\q{ab}]/v.test('µ') && /[\p{Script=Greek}\q{ab}]/v.exec('μ')[0]==='μ' && /[\p{Script=Greek}\q{ab}]/iv.exec('AB')[0]==='AB'",
    );
    check(
        r"/[\q{ab}\s]/v.exec('\uFEFF')[0]==='\uFEFF' && !/[\q{ab}\p{White_Space}]/v.test('\uFEFF') && !/[\q{ab}\s]/v.test('\u0085') && /[\q{ab}\p{White_Space}]/v.exec('\u0085')[0]==='\u0085'",
    );
}

#[test]
fn full_ranges_surrogates_names_indices_and_last_index_keep_original_boundaries() {
    check(
        r"let r=/(?<whole>([\u{0}-\u{10FFFF}\q{ab}]))/dvy;r.lastIndex=1;let m=r.exec('😀!');m.index===0 && m[0]==='😀' && m[1]==='😀' && m[2]==='😀' && m.groups.whole==='😀' && m.indices[0][0]===0 && m.indices[2][1]===2 && m.indices.groups.whole===m.indices[1] && r.lastIndex===2",
    );
    check(
        r"let r=/[\q{}\p{Any}]/dvy;r.lastIndex=1;let a=r.exec('😀'),b=r.exec('😀');a[0]==='😀' && a.index===0 && a.indices[0][1]===2 && b.index===2 && b[0]==='' && r.lastIndex===2",
    );
    check(
        r"let r=/[\q{ab}\u{D7FF}-\u{E000}]/vy;r.lastIndex=1;let a=r.exec('😀');a===null && r.lastIndex===0 && r.exec('\uD800a')[0].charCodeAt(0)===0xd800 && /[\q{ab}\u{D7FF}-\u{E000}]/v.exec('ab')[0]==='ab'",
    );
    check(
        r"let r=/[\q{}\p{Any}]/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀');}catch(e){caught=e instanceof TypeError;}caught && r.lastIndex===0",
    );
}

#[test]
fn consumers_clones_and_collection_use_the_complete_interval_string_plan() {
    check(
        r"let r=/(?<whole>([\q{}\P{Any}]))/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && a[1].groups.whole==='' && a[1].indices[2][1]===2 && '😀'.replace(r,'#')==='#😀#' && '😀'.split(/[\q{}\P{Any}]/v).join('|')==='😀'",
    );
    check(
        r"let r=/[\q{ab}\d]/vg;r.lastIndex=8;'xab5!'.search(r)===1 && r.lastIndex===8 && 'ab5'.match(r).join('|')==='ab|5' && r.lastIndex===0 && 'ab5'.replace(r,'#')==='##' && 'ab5'.split(/[\q{ab}\d]/v).join('|')==='||'",
    );
    check(
        r"let r=/[\q{ab}\p{ASCII}]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('K')===null && r.exec('K')[0]==='K' && c.lastIndex===0 && r.lastIndex===7 && c.source==='[\\q{ab}\\p{ASCII}]'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\q{ab}\p{Script=Greek}]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xab!').groups.whole==='ab' && r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_controls_complete_finalizers_and_unproved_algebra_and_early_errors_stay_distinct() {
    for (source, expected) in [
        (r"/\b^[\q{ab}\d]/v.test('ab')", true),
        (r"/[\q{ab}--\d]/v.test('ab')", true),
        (r"/[\q{ab}&&\p{Any}]/v.test('ab')", false),
        (r"/[a-z\q{ab}]/v.test('ab')", true),
        (r"/[\q{ab}\d]/v.test('ab')", true),
        (r"/[\q{ab}\d]/v.test('ac')", false),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
            )),
            Ok(Value::Boolean(expected))
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}\\d]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/[\q{ab}\p{RGI_Emoji}]/v.test('ab')",
        r"/[[\q{ab}\d]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[\q{ab}\d]{0}/v.test('')",
        r"/[\q{ab}\d]x/v.test('abx')",
        r"/\b^[\q{ab}\d\p{RGI_Emoji}]/v.test('ab')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    for pattern in [r"[^\q{ab}\d]", r"[a-\q{ab}]", r"[\q{\d}\p{Any}]"] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn opt_in_construction_and_search_aborts_preserve_values_and_skip_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let source = format!(
        r"try{{r=/[\q{{}}{}]/vg;}}catch{{marker=1;}}finally{{marker=2;}}",
        r"\p{Any}".repeat(4000)
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("r===7 && marker===0"), Ok(Value::Boolean(true)));
    realm.eval(r"r=/[\q{ab}\d]/vg").unwrap();
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
