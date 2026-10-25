//! Conditional captures in quantified UnicodeSets empty branches (22.2.2.3).

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
fn empty_capture_effects_keep_required_optional_choice_and_empty_only_layouts_distinct() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let m=/(?<each>(?<part>{class})|(?<empty>(?<deep>)?)){{3}}/dv.exec('abX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===2 && m.indices.groups.deep===undefined"
        ));
        check(&format!(
            "let m=/(?:{class}|(?<empty>(?<deep>))?){{3}}/dv.exec('ab');m.length===3 && m[0]==='ab' && m.groups.empty===undefined && m.groups.deep===undefined && Object.keys(m.groups).join('|')==='empty|deep' && Object.hasOwn(m.groups,'empty') && Object.getPrototypeOf(m.groups)===null && m.indices[1]===undefined && m.indices[2]===undefined"
        ));
        check(&format!(
            "let m=/(?<each>(?<empty>(?<first>)|(?<later>)){{2}}|(?<part>{class})){{2}}/dv.exec('ab');m[0]==='' && m.groups.each==='' && m.groups.empty==='' && m.groups.first==='' && m.groups.later===undefined && m.groups.part===undefined && m.indices.groups.first[0]===0 && m.indices.groups.later===undefined"
        ));
    }
}

#[test]
fn conditional_empty_captures_keep_original_utf16_ranges_names_flags_and_sticky_state() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<part>{class})|(?<empty>(?<deep>)?)){{3}})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.whole==='😀😀' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===5 && m.indices.groups.empty[1]===5 && m.indices.groups.deep===undefined && r.lastIndex===5"
        ));
    }
    check(
        r"let source='(?<each>[\\q{s|k}]|(?<\\u{1D49C}>(?<first>)|(?<later>))){3}',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && m[0]==='ſK' && m.groups['𝒜']==='' && m.groups.first==='' && m.groups.later===undefined && m.indices.groups['𝒜']===m.indices[2] && m.indices[2][0]===2 && m.indices.groups.later===undefined",
    );
    check(
        r"let m=/(?<each>(?<part>[\q{\uD800a|\uD800}&&\p{Any}])|(?<empty>(?<deep>)?)){3}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===2",
    );
    check(
        r"let r=/(?:[\q{ab}&&a]|(?<empty>)?)+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn conditional_empty_capture_consumers_callbacks_clones_and_collection_keep_participation() {
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?:(?<empty>(?<first>)|(?<later>))+?(?<tail>))){3}/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a.every(m=>m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.first==='' && m.groups.later===undefined && m.groups.tail==='' && m.indices.groups.later===undefined) && 'abXba'.replace(r,'<$<empty>:$<later>>')==='<:><:>X<:><:>' && r.lastIndex===0",
    );
    check(
        r"let seen=[],out='abX'.replace(/(?<each>(?<part>[\q{a|b}])|(?:(?<empty>(?<first>)|(?<later>))+?(?<tail>))){3}/vg,(m,each,part,empty,first,later,tail,at,input,groups)=>{seen.push([each,part,empty,first,later,tail,at,groups.later]);return '#'});out==='##X#' && seen.length===3 && seen.every(row=>row[0]==='' && row[1]===undefined && row[2]==='' && row[3]==='' && row[4]===undefined && row[5]==='' && row[7]===undefined) && seen[0][6]===0 && seen[1][6]===2",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)?)+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && 'ab'.split(r).join('|')==='|b|b||'",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)?)+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && m.groups.empty===undefined && m.groups.part===undefined && c.lastIndex===0 && r.exec('ABB').groups.part==='B' && r.lastIndex===7",
    );
    check(
        r"let r=/(?:[\q{a}]|(?<empty>)?)+/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && a.every(m=>m.groups.empty===undefined && Object.hasOwn(m.groups,'empty') && m.indices.groups.empty===undefined) && r.lastIndex===0",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>(?<part>[\q{a|b}])|(?<empty>(?<deep>)?)){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_empty_capture_effects_finish_finalizers_and_keep_other_terms_deferred() {
    for source in [
        r"/(?:[\q{a}]|(?:|()))+/v.test('a')",
        r"/([\q{a|ab}&&a]|()?)+/v.test('a')",
        r"/([\q{a|b}]|()?){0}/v.test('a')",
        r"/([\q{a|b}]|()?){1,}/v.test('a')",
        r"/([\q{a|b}]|()?){1}/v.test('a')",
        r"/([\q{a|b}]|()?){2,3}/v.test('a')",
        r"/([\q{a|b}]|()?){2,}/v.test('a')",
        r"/([\q{a|b}]|()?){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|()))+/v.test('a')",
        r"/([\q{a|b}]|(?:|())){0}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){1}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2}/v.test('a')",
        r"/([\q{a}]|()?)+/v.test('a')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
    for inner in [
        r"([\q{a|b}])|(()|)",
        r"([\q{a|b}])|()*",
        r"([\q{a|b}])|()?",
        r"[\q{a|b}]|()?",
        r"[\q{a|b}]|(?:)()*",
        r"[\q{a|b}]|(?:|())",
    ] {
        for count in ["+?", "{0}", "{2,3}"] {
            check(&format!(
                "let marker=0,value;try{{value=/({inner}){count}/v.test('a')}}finally{{marker=3}}value && marker===3"
            ));
        }
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{ab}&&\\q{cd}]|()?)+?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for (inner, expected_zero) in [
        (
            r"[\q{a|b}]|\b()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|(?=())",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|\1()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|x()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])(\b)|()?",
            r#"[["","<undefined>","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])+|()?",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
    ] {
        for count in ["+?", "{0}", "{2,3}"] {
            let source = format!("({inner}){count}");
            if count == "{0}" {
                let pattern = JsString::from(source.as_str());
                let expected = JsString::from(expected_zero);
                check(&format!(
                    "var pattern={pattern:?},input='a😀b',r=new RegExp(pattern,'dvgy');r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==='dgvy'&&m.input===input&&Object.hasOwn(m,'groups')&&Object.hasOwn(m.indices,'groups')&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,r.lastIndex])==={expected:?}"
                ));
                continue;
            }
            assert!(
                crate::common::regexp_oracle::check(
                    &mut Realm::default(),
                    &format!("/{source}/v.test('a')")
                ),
                "{source}"
            );
        }
    }
    for pattern in [
        r"([^\q{ab}]|()?){0}",
        r"([\q{ab}&&a--a]|()?){2,3}",
        r"([\q{\d}&&a]|()?){1,}",
        r"([\q{a|b}]|()?){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
