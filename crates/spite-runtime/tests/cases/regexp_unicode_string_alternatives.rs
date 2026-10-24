//! Standalone variable-length ClassStringDisjunction matching (22.2.2.7).

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
fn earliest_position_uses_longest_member_regardless_of_source_order() {
    check(
        r"let a=/[\q{a|ab|abc}]/dv.exec('xabc!'),b=/[\q{abc|ab|a}]/v.exec('ab!');a.index===1 && a[0]==='abc' && a.indices[0][0]===1 && a.indices[0][1]===4 && b[0]==='ab'",
    );
    check(
        r"let a=/[\q{|a|ab|abc|ab|}]/v.exec('abc!'),b=/[\q{ab|a|}]/v.exec('xab');a[0]==='abc' && b.index===0 && b[0]==='' && !/[\q{ab|cd}]/v.test('ac') && /[\q{ab|cd}]/v.exec('acdab')[0]==='cd'",
    );
    check(r"/[\q{a\|b|a}]/v.exec('xa|b')[0]==='a|b' && /[\q{\]\}|a}]/v.exec(']}')[0]===']}'");
}

#[test]
fn outside_assertions_can_fall_back_to_shorter_and_empty_members() {
    check(
        r"let a=/[\q{ab|a!|a}]\b/dv.exec('a!'),b=/[\q{a!|}]\b/dv.exec('a!');a[0]==='a' && a.indices[0][1]===1 && b[0]==='' && b.index===0 && b.indices[0][1]===0 && /[\q{a!|a}]\B/v.exec('a!')[0]==='a!'",
    );
    check(
        r"let a=/[\q{ab|a}]$/v.exec('aa'),b=/^[\q{|ab}]$/mv.exec('x\r\nab\u2028z');a.index===1 && a[0]==='a' && b.index===2 && b[0]==='' && !/^[\q{ab|a}]$/v.test('ab\n') && /^[\q{ab|a}]$/mv.test('ab\n')",
    );
    check(
        r"let a=/^\b[\q{ab|a}]\b$/mv.exec('!\r\nab\u2028!');a.index===3 && a[0]==='ab' && /\B[\q{a!|a}]\B/v.exec('xa!')[0]==='a!' && !/\b[\q{ab|a}]\b/v.test('xabx')",
    );
}

#[test]
fn original_unicode_characters_surrogates_and_simple_folds_keep_utf16_ranges() {
    check(
        r"let r=/[\q{😀|😀x|🚀}]/dvy;r.lastIndex=2;let m=r.exec('A😀x!');m.index===1 && m[0]==='😀x' && m.indices[0][0]===1 && m.indices[0][1]===4 && r.lastIndex===4",
    );
    // CompileClassSetString concatenates character sequences; braced surrogate
    // escapes stay separate characters. CompileAtom uses CharacterSetMatcher
    // for each, so the first cannot match a decoded supplementary input point.
    // V8 13.6 joins these q members, unlike its ordinary concatenation control.
    // https://262.ecma-international.org/17.0/#sec-compileclasssetstring
    // https://262.ecma-international.org/17.0/#sec-compileatom
    check(r"!/[\q{\u{D800}\u{DC00}|ab}]/v.test('𐀀') && /[\q{\uD800\uDC00x|a}]/v.test('𐀀x')");
    check(
        r"let m=/[\q{\uD800\\|a}]/dv.exec('A\uD800\\B');m.index===1 && m[0].length===2 && m[0].charCodeAt(0)===0xd800 && m[0].charCodeAt(1)===92 && /[\q{\uD83D\uDE00x|a}]/v.exec('😀x')[0]==='😀x'",
    );
    check(
        r"let m=/[\q{sK|ßa|\u{10400}a|x}]/div.exec('!ſK!');m.index===1 && m[0]==='ſK' && m.indices[0][1]===3 && /[\q{ßa|x}]/iv.test('ẞA') && !/[\q{ßa|x}]/iv.test('SSa') && /[\q{\u{10400}a|x}]/iv.exec('𐐨A')[0]==='𐐨A' && !/[\q{sK|x}]/v.test('ſK')",
    );
    // Canonicalize applies to singleton members even beside longer strings.
    // V8 13.6 omits that closure; spelling the singleton as an ordinary class
    // operand supplies an independent engine control.
    check(
        r"/[\q{a|ab}]/iv.exec('A')[0]==='A' && /[\q{sK|k}]/iv.exec('K')[0]==='K' && /[\q{|a|ab}]\b/iv.exec('A!')[0]==='A'",
    );
    check(
        r"/[\q{\ba|x}]/v.test('\ba') && /[\q{\0a|x}]/v.test('\0a') && new RegExp('[\\q{a\n|b}]','v').test('a\n')",
    );
}

#[test]
fn empty_matches_rewind_surrogates_and_exec_does_not_advance_last_index() {
    check(
        r"let r=/[\q{}]/dvgy;r.lastIndex=1;let a=r.exec('😀'),b=r.exec('😀');a.index===0 && a[0]==='' && a.indices[0][0]===0 && a.indices[0][1]===0 && b.index===0 && r.lastIndex===0",
    );
    check(
        r"let r=/[\q{|ab}]/dvgy;r.lastIndex=2;let a=r.exec('ab'),b=r.exec('ab');a.index===2 && a[0]==='' && b.index===2 && r.lastIndex===2 && a.indices[0][1]===2",
    );
    check(
        r"let r=/[\q{}]/vy;r.lastIndex=3;let m=r.exec('😀');m===null && r.lastIndex===0 && /[\q{ab|}]/v.exec('')[0]===''",
    );
}

#[test]
fn enclosing_captures_project_complete_empty_and_consuming_matches() {
    check(
        r"let r=/(?<whole>([\q{|a|ab}]))/dvg,m=r.exec('ab!');m.length===3 && m[0]==='ab' && m[1]==='ab' && m[2]==='ab' && m.index===0 && m.input==='ab!' && m.groups.whole==='ab' && Object.getPrototypeOf(m.groups)===null && m.indices.groups.whole===m.indices[1] && m.indices[0]!==m.indices[1] && m.indices[1]!==m.indices[2] && m.indices[2][1]===2 && r.lastIndex===2",
    );
    check(
        r"let r=/(?<whole>([\q{a!|}]\b))/dv,m=r.exec('a!');m[0]==='' && m[1]==='' && m[2]==='' && m.groups.whole==='' && m.indices[0][0]===0 && m.indices[2][1]===0 && Object.getPrototypeOf(m)===Array.prototype && Object.getOwnPropertyDescriptor(m,'1').enumerable && Object.getOwnPropertyDescriptor(m.indices,'1').writable",
    );
}

#[test]
fn global_sticky_and_last_index_coercion_keep_specified_side_effects() {
    check(
        r"let r=/[\q{ab|a}]/vg,a=r.exec('ab-a'),b=r.exec('ab-a');a.index===0 && a[0]==='ab' && b.index===3 && b[0]==='a' && r.lastIndex===4 && r.exec('ab-a')===null && r.lastIndex===0",
    );
    check(
        r"let trace='',r=/[\q{ab|a}]/vy;r.lastIndex={valueOf(){trace+='i';return 1;}};let m=r.exec({toString(){trace+='s';return 'xab';}});trace==='si' && m.index===1 && m[0]==='ab' && r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[\q{ab|}]/v;r.lastIndex={valueOf(){seen++;return 7;}};r.exec('x').index===0 && seen===1 && typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/[\q{}]/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('');}catch(e){caught=e instanceof TypeError;}caught && r.lastIndex===0",
    );
}

#[test]
fn string_consumers_advance_empty_matches_by_original_unicode_character() {
    check(
        r"let r=/(?<whole>([\q{}]))/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && a[1].groups.whole==='' && a[1].indices[2][1]===2 && r.lastIndex===0 && '😀'.match(/[\q{}]/vg).length===2 && '😀'.replace(/[\q{}]/vg,'-')==='-😀-' && '😀'.split(/[\q{}]/v).join('|')==='😀'",
    );
    check(
        r"let r=/[\q{|a|ab}]/vg,a=[...'ab!'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===3 && 'ab!'.replace(r,'#')==='##!#' && 'ab!'.split(/[\q{|a|ab}]/v).join('|')==='|!'",
    );
    check(
        r"let r=/(?<whole>([\q{ab|a}]))/vg;r.lastIndex=8;let seen='';let out='xab-a!'.replace(r,(m,a,b,i,s,g)=>{seen+=i+':'+m+':'+a+':'+b+':'+g.whole+';';return '#';});out==='x#-#!' && seen==='1:ab:ab:ab:ab;4:a:a:a:a;' && r.lastIndex===0 && 'xab-a!'.search(r)===1",
    );
    check(
        r"let r=/[\q{sK|x}]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{sK|x}]' && c.flags==='dvy' && c.lastIndex===0 && c.exec('ſK')===null && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
}

#[test]
fn former_unsupported_patterns_complete_finalizers_but_unproved_patterns_do_not() {
    for (source, expected) in [
        (r"/[\q{ab}]$\b/v.test('ab')", true),
        (r"/\b^[\q{ab}]/v.test('ab')", true),
        (r"/[\q{}]/v.test('')", true),
        (r"/[\q{a|ab}]/v.test('ab')", true),
        (r"/[\q{\u{D800}\u{DC00}}]/v.test('𐀀')", false),
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\"],\"indices\":[[0,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\"],\"indices\":[[2,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}]{0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\"],\"indices\":[[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="[[\\q{ab}]&&\\p{RGI_Emoji}]{0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\"],[[1,1]],1,1,null,null]""####),Ok(Value::Boolean(true)));
    for source in [
        r"/[[\q{ab}]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[\q{}]a/v.test('a')",
        r"/a[\q{ab}]/v.test('aab')",
        r"/([\q{ab}])x/v.test('abx')",
        r"/\b^[\q{ab}&&\p{RGI_Emoji}]/v.test('ab')",
        r"/[\q{ab}&&\p{RGI_Emoji}]$\b/v.test('ab')",
        r"/[\p{RGI_Emoji}]/v.test('😀')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)));
    }
    for pattern in [
        r"[^\q{ab}]",
        r"[^\q{}]",
        r"[\q{\d|ab}]",
        r"[\q{\u{110000}x}]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn collection_and_opt_in_aborts_preserve_compiled_state_and_last_index() {
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>([\q{|ab|a}]))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"kept.exec('zab!').groups.whole==='ab' && kept.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "ab|".repeat(4000) + "ab";
    let source = format!(r"try{{r=/[\q{{{choices}}}]/vg;}}catch{{marker=1;}}finally{{marker=2;}}");
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0 && r===7"), Ok(Value::Boolean(true)));
    realm.eval(r"r=/[\q{ab|ac}]/vg").unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "a".repeat(20_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0 && r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn default_resource_policy_accepts_long_members_without_recursive_matcher_storage() {
    let word = "a".repeat(20_000);
    let source = format!(
        r"let r=/[\q{{{word}|a}}]/vy,m=r.exec('{word}');m[0].length===20000 && r.lastIndex===20000"
    );
    check(&source);
}
