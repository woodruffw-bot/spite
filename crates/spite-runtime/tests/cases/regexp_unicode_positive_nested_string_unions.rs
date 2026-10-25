//! Positive nested class unions with variable-length q members (22.2.2.9).

use spite_core::JsString;
use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn positive_children_and_siblings_share_priority_and_complete_captures() {
    check(
        r"let m=/(?<whole>([[\q{a}][\q{ab|abc}]]))/dvg.exec('xabc!');m.length===3 && m[0]==='abc' && m[1]==='abc' && m[2]==='abc' && m.index===1 && m.groups.whole==='abc' && m.indices[0][0]===1 && m.indices[2][1]===4 && m.indices.groups.whole===m.indices[1]",
    );
    check(
        r"/[a[[\q{ab}]]b]/v.exec('ab')[0]==='ab' && /[a[[\q{ab}]]b]/v.exec('a!')[0]==='a' && /[[\q{ab}][]]/v.exec('ab')[0]==='ab' && !/[[\q{ab}][]]/v.test('ac') && /[[\q{}][]]/v.exec('!')[0]===''",
    );
    check(
        r"let a=/[[\q{a!}][\p{Any}]]\b/dv.exec('a!'),b=/[[\q{a!}][\p{Any}]]\B/v.exec('a!');a[0]==='a' && a.indices[0][1]===1 && b[0]==='a!'",
    );
}

#[test]
fn escaped_brackets_surrogates_and_empty_children_preserve_original_ranges() {
    check(
        r"let m=/[[\q{a\]b}][\[]]/dv.exec('xa]b!');m.index===1 && m[0]==='a]b' && m.indices[0][1]===4 && /[[\q{a\]b}][\[]]/v.exec('[')[0]==='['",
    );
    check(
        r"let r=/[[\q{😀x}][\q{🚀}]]/dvy;r.lastIndex=2;let m=r.exec('A😀x!');m.index===1 && m[0]==='😀x' && m.indices[0][0]===1 && m.indices[0][1]===4 && r.lastIndex===4",
    );
    check(
        r"let r=/(?<whole>([[\q{}][\P{Any}]]))/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m[2]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    check(
        r"let r=/[[\q{}][\p{Any}]]/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='😀' && m.indices[0][1]===2 && r.lastIndex===2",
    );
}

#[test]
fn nested_scalar_folding_properties_and_line_word_gates_keep_their_context() {
    check(
        r"let m=/^\b[[a-z][\q{ab}]]\b$/mv.exec('!\r\nab\u2028!');m.index===3 && m[0]==='ab' && !/^\b[[a-z][\q{ab}]]\b$/v.test('!\r\nab\u2028!') && /[[a-z][\q{sK}]]/iv.exec('ſK')[0]==='ſK'",
    );
    check(
        r"/[[\p{ASCII}][\q{ab}]]/iv.exec('K')[0]==='K' && !/[[\P{ASCII}][\q{ab}]]/iv.test('K') && /[[\p{Script=Greek}][\q{ab}]]/iv.exec('µ')[0]==='µ' && !/[[\p{Script=Greek}][\q{ab}]]/v.test('µ')",
    );
    check(
        r"/[[\q{ab}][\s]]/v.test('\uFEFF') && !/[[\q{ab}][\p{White_Space}]]/v.test('\uFEFF') && !/[[\q{ab}][\s]]/v.test('\u0085') && /[[\q{ab}][\p{White_Space}]]/v.test('\u0085')",
    );
}

#[test]
fn consumers_clones_and_collection_use_the_positive_nested_plan() {
    check(
        r"let r=/(?<whole>([[\q{}][\P{Any}]]))/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && a[1].groups.whole==='' && a[1].indices[2][1]===2 && '😀'.replace(r,'#')==='#😀#' && '😀'.split(/[[\q{}][\P{Any}]]/v).join('|')==='😀'",
    );
    check(
        r"let r=/[[\q{ab}][\d]]/vg;r.lastIndex=8;'xab5!'.search(r)===1 && r.lastIndex===8 && 'ab5'.match(r).join('|')==='ab|5' && r.lastIndex===0 && 'ab5'.replace(r,'#')==='##' && 'ab5'.split(/[[\q{ab}][\d]]/v).join('|')==='||'",
    );
    check(
        r"let r=/[[\p{ASCII}][\q{ab}]]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('K')===null && r.exec('K')[0]==='K' && r.lastIndex===7 && c.lastIndex===0",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([[\q{ab}][\p{Script=Greek}]]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xab!').groups.whole==='ab' && r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_controls_complete_finalizers_and_other_algebra_remains_pending() {
    for (source, expected) in [
        (r"/\b^[[\q{ab}]]/v.test('ab')", true),
        (r"/[[\q{ab}]--[a-z]]/v.test('ab')", true),
        (r"/[[\q{ab}]&&[\q{ab}]]/v.test('ab')", true),
        (r"/[[\q{ab}]]/v.test('ab')", true),
        (r"/[[\q{ab}\q{cd}]]/v.test('ab')", true),
        (r"/[\q{ab}[a]]/v.test('ab')", true),
        (r"/[[a-z]\q{ab}]/v.test('ab')", true),
        (r"/[[\q{ab}][]]/v.test('ac')", false),
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[[\\q{ab}]]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(?:[[\\q{ab}]])+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/[[^a]\q{ab}]/v.test('ab')",
        r"/[[\q{}][\p{RGI_Emoji}]]/v.test('')",
        r"/[[[\q{ab}]]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[[\q{ab}]][a]/v.test('aba')",
        r"/(?:[[[\q{ab}]]&&\p{RGI_Emoji}])+/v.test('ab')",
        r"/\b^[[\q{ab}]&&\p{RGI_Emoji}]/v.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    let source = JsString::from(r"marker=1;/[[^\q{ab}]]/v.test('ab');");
    check(&format!(
        "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
    ));
}
