//! Static MayContainStrings false enables character projection (22.2.1).

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
fn static_character_results_match_nested_unions_complements_and_string_first_intersections() {
    check(
        r"let a=/[[\q{ab}&&a]b]/v.exec('abc'),b=/[[^\q{ab}&&a]&&\q{b|bc|}]/v.exec('abc');a[0]==='b' && a.index===1 && b[0]==='b' && b.index===1 && /[\q{a|ab}&&[[a-z]&&\q{a|ab}]]/v.exec('ab')[0]==='a'",
    );
    check(
        r"/[^\q{ab}&&a]/v.exec('😀')[0]==='😀' && !/[^\q{ab}&&a]/v.test('') && /[^\q{}&&\p{Any}]/v.exec('😀')[0]==='😀' && !/[\q{ab}&&a]/v.test('ab') && !/[[\q{ab}--\q{ab}]&&\p{Any}]/v.test('ab')",
    );
    check(
        r"/[\q{a|ab}&&\p{Any}]+/v.exec('aaab')[0]==='aaa' && /[\q{a|ab}&&\p{Any}]+?/v.exec('aaab')[0]==='a' && /^[^\q{ab}&&a]{3}$/v.test('ab😀') && /^[\q{0|2|4|9\uFE0F\u20E3}&&\d]+$/v.test('024') && !/^[\q{0|2|4|9\uFE0F\u20E3}&&\d]+$/v.test('9\uFE0F\u20E3')",
    );
}

#[test]
fn character_results_keep_canonical_property_folding_original_captures_and_surrogate_ranges() {
    check(
        r"let r=/(?<whole>(^\b[[\q{ab}&&a]b]+?\b$))/dimv,m=r.exec('!\r\nBBB\u2028!');m.index===3 && m[0]==='BBB' && m[1]==='BBB' && m[2]==='BBB' && m.groups.whole==='BBB' && m.indices[2][1]===6 && m.indices.groups.whole===m.indices[1]",
    );
    check(
        r"let r=/(?<whole>([^\q{ab}&&a]))/dvgy;r.lastIndex=2;let m=r.exec('A😀x');m.index===1 && m[0]==='😀' && m.groups.whole==='😀' && m.indices[2][0]===1 && m.indices[2][1]===3 && r.lastIndex===3 && /[^\q{ab}&&a]+/v.exec('A😀x')[0]==='A😀x'",
    );
    check(
        r"!/[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]+/v.test('𐀀ab') && /[[\q{\uD800a}&&\p{Any}]\uD800]+/v.exec('\uD800\uD800a')[0].length===2 && /[\q{μ|ab}&&\p{Script=Greek}]+/iv.exec('µμ')[0]==='µμ'",
    );
    check(
        r"let r=/[^\q{k|sK|}&&\p{ASCII}]/iv;r.exec('Kſ')[0]==='ſ' && r.exec('Kſ').index===1 && /[\q{é|é}&&\p{Any}]+/v.exec('é')[0]==='é' && !/[\q{é|é}&&\p{Any}]+/v.test('é')",
    );
}

#[test]
fn empty_character_results_zero_iterations_huge_bounds_and_last_index_keep_their_contracts() {
    check(
        r"!/[\q{ab}&&a]+/v.test('ab') && !/[\q{ab}&&a]+/v.test('') && /[\q{ab}&&a]*/v.exec('ab')[0]==='' && /[\q{ab}&&a]?/v.exec('ab')[0]===''",
    );
    check(
        r"let r=/(?<whole>([\q{ab}&&a]{0}))/dvgy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    let huge = "9".repeat(100);
    check(&format!(
        r"!/[\q{{a|ab}}&&\p{{Any}}]{{{huge}}}/v.test('aaa') && /^[\q{{a|ab}}&&\p{{Any}}]{{1,{huge}}}$/v.test('aaa') && /[\q{{ab}}&&a]{{0,{huge}}}/v.exec('ab')[0]===''"
    ));
    for pattern in [r"[^\q{ab}&&a]", r"[\q{ab}&&a]*"] {
        check(&format!(
            "let r=/{pattern}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
        ));
    }
}

#[test]
fn consumers_clones_collection_and_direct_eval_retain_character_result_metadata() {
    check(
        r"let r=/(?<whole>([^\q{ab}&&a]+))/dvg,a=[...'ab😀'.matchAll(r)];a.length===1 && a[0][0]==='ab😀' && a[0].indices[2][1]===4 && a[0].groups.whole==='ab😀' && r.lastIndex===0 && 'ab😀'.replace(r,'#')==='#' && 'ab😀'.split(/[^\q{ab}&&a]+/v).join('|')==='|'",
    );
    check(
        r"let r=/(?<whole>([\q{ab}&&a]*))/dvg,a=[...'ab😀'.matchAll(r)];a.length===4 && a[0].index===0 && a[1].index===1 && a[2].index===2 && a[3].index===4 && a[3].groups.whole==='' && 'ab😀'.replace(r,'#')==='#a#b#😀#' && 'ab😀'.split(/[\q{ab}&&a]*/v).join('|')==='a|b|😀'",
    );
    check(
        r"let r=/[[\q{ab}&&a]b]+/ivg;r.lastIndex=8;'xBB!'.search(r)===1 && r.lastIndex===8 && 'BBxB'.match(r).join('|')==='BB|B' && r.lastIndex===0 && 'BBxB'.replace(r,'#')==='#x#'",
    );
    check(
        r"let r=/[[\q{ab}&&a]b]+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('BB')===null && c.lastIndex===0 && r.exec('BB')[0]==='BB' && r.lastIndex===7",
    );
    check(
        r#"let marker=0,result;try{result=eval('marker=1;/[^\\q{ab}&&a]+/v.test("ab")')}catch{marker=2}finally{marker=3}marker===3 && result===true"#,
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([[\q{ab}&&a]b]+))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xbb').groups.whole==='bb' && r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_unsupported_results_complete_finalizers_while_conservative_and_unknown_operands_stay_distinct()
 {
    for (source, expected) in [
        (r"/([\q{ab}&&a])+/v.test('ab')", false),
        (r"/[^\q{ab}&&a]/v.test('a')", true),
        (r"/[^\q{ab}&&a]{0}/v.test('')", true),
        (r"/[\q{a}&&[]&&\q{b|cd}]{0}/v.test('')", true),
        (r"/[\q{ab}&&a]+/v.test('ab')", false),
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
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}--\\q{ab}]*",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}&&\\q{cd}]{0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b^[\\q{ab}&&a]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}&&a]+$\\b",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="[[\\q{ab}&&\\q{cd}]&&\\p{RGI_Emoji}]{0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\"],[[1,1]],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="[\\q{ab}&&\\p{Any}&&\\p{RGI_Emoji}]{0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\"],[[1,1]],1,1,null,null]""####),Ok(Value::Boolean(true)));
    for source in [
        r"/[[\q{ab}--\q{ab}]&&\p{RGI_Emoji}]*/v.test('')",
        r"/[[\q{ab}&&a]\q{ab}]+/v.test('ab')",
        r"/\b^[[\q{ab}&&a]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[[\q{ab}&&a]&&\p{RGI_Emoji}]+$\b/v.test('ab')",
        r"/[\q{ab}&&a]+[b]/v.test('ab')",
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
        r"[^\q{ab}--\q{ab}]",
        r"[\q{ab}&&a--a]{0}",
        r"[\q{\d}&&a]{0}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn deep_and_long_character_results_use_defaults_and_opt_in_aborts_preserve_state() {
    let pattern = format!(r"{}\q{{ab}}&&a{}*", "[".repeat(3_000), "]".repeat(3_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('ab');m[0]==='' && r.lastIndex===0"
    ));
    let pattern = format!(r"[\q{{{}}}&&\p{{Any}}]{{0}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/vy,m=r.exec('😀');m[0]==='' && r.lastIndex===0"
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
    realm.eval(r"r=/[\q{a|ab}&&a]+/vg").unwrap();
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
