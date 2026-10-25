//! Captures confined to a nullable UnicodeSets empty branch (22.2.2.3).

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
fn nested_sibling_and_empty_only_captures_keep_selected_branch_participation() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>(?<part>{class})|(?:(?<empty>(?<deep>))(?<sibling>))){{3}}/dv.exec('abX'),b=/(?<each>(?<part>{class})|(?:(?<empty>(?<deep>))(?<sibling>))){{2}}/dv.exec('abX');a.length===6 && a[0]==='ab' && a.groups.each==='' && a.groups.part===undefined && a.indices[2]===undefined && ['empty','deep','sibling'].every(k=>a.groups[k]==='' && a.indices.groups[k][0]===2 && a.indices.groups[k][1]===2) && b.groups.part==='b' && ['empty','deep','sibling'].every(k=>b.groups[k]===undefined && b.indices.groups[k]===undefined)"
        ));
        check(&format!(
            "let a=/(?:{class}|(?<empty>)){{3}}/dv.exec('ab'),b=/(?:(?<empty>)|{class}){{2,3}}/dv.exec('ab');a.length===2 && a[0]==='ab' && a.groups.empty==='' && a.indices[1][0]===2 && b[0]==='a' && b.groups.empty===undefined && b.indices[1]===undefined"
        ));
    }
    check(
        r"let a=/(?<each>(?<empty>)|(?<part>[\q{a|b}])){2}/dv.exec('ab');Object.keys(a.groups).join('|')==='each|empty|part' && a[0]==='' && a.groups.each==='' && a.groups.empty==='' && a.groups.part===undefined && a.indices[2][0]===0",
    );
}

#[test]
fn empty_capture_utf16_ranges_names_source_and_sticky_state_remain_original() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<part>{class})|(?<empty>)){{3}})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.whole==='😀😀' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===5 && m.indices.groups.empty[1]===5 && r.lastIndex===5"
        ));
    }
    check(
        r"let source='(?<each>[\\q{s|k}]|(?<\\u{1D49C}>)){3}',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && m[0]==='ſK' && m.groups['𝒜']==='' && m.indices.groups['𝒜']===m.indices[2] && m.indices[2][0]===2",
    );
    check(
        r"let m=/(?<each>(?<part>[\q{\uD800a|\uD800}&&\p{Any}])|(?<empty>)){3}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===2",
    );
    check(
        r"let r=/(?:[\q{ab}&&a]|(?<empty>))+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn empty_capture_string_consumers_callbacks_clones_and_collection_preserve_slots() {
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>))+/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a[0].groups.part==='b' && a[0].groups.empty===undefined && a[1].groups.part===undefined && a[1].groups.empty==='' && a[1].indices.groups.empty[0]===2 && 'abXba'.replace(r,'<$2:$<empty>>')==='<b:><:>X<a:><:>' && r.lastIndex===0",
    );
    check(
        r"let seen=[],out='abX'.replace(/(?<each>(?<part>[\q{a|b}])|(?<empty>)){3}/vg,(m,each,part,empty,at,input,groups)=>{seen.push([each,part,empty,at,groups.empty]);return '#'});out==='##X#' && seen.length===3 && seen.every(row=>row[0]==='' && row[1]===undefined && row[2]==='' && row[4]==='') && seen[0][3]===0 && seen[1][3]===2",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>))+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && 'ab'.split(r).join('|')==='|b|b||'",
    );
    check(
        r"let r=/(?<each>[\q{a|b}]|(?<empty>))+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && m.groups.empty==='' && c.lastIndex===0 && r.exec('ABB').groups.empty===undefined && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_empty_capture_proofs_finish_finalizers_and_keep_effects_deferred() {
    for source in [
        r"/([\q{a|ab}&&a]|())+/v.test('a')",
        r"/([\q{a|b}]|()){0}/v.test('a')",
        r"/([\q{a|b}]|()){1,}/v.test('a')",
        r"/([\q{a|b}]|()){1}/v.test('a')",
        r"/([\q{a|b}]|()){2,3}/v.test('a')",
        r"/([\q{a|b}]|()){2,}/v.test('a')",
        r"/([\q{a|b}]|()){2}/v.test('a')",
        r"/([\q{a}]|())+/v.test('a')",
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
    for inner in [r"([\q{a|b}])|()", r"[\q{a|b}]|()"] {
        for count in ["+?", "{0}", "{2,3}"] {
            check(&format!(
                "let marker=0,value;try{{value=/({inner}){count}/v.test('a')}}finally{{marker=3}}value && marker===3"
            ));
        }
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(|([\\q{ab}&&\\q{cd}])())+?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for (inner, expected_zero) in [
        (
            r"([\q{a|b}])|(\b)*",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])|(\b()|)",
            r#"[["","<undefined>","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])(\b)|()",
            r#"[["","<undefined>","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])|\b()",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])+|()",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|()",
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
        r"(([^\q{ab}])|()){0}",
        r"(([\q{ab}&&a--a])|()){2,3}",
        r"(([\q{\d}&&a])|()){1,}",
        r"(([\q{a|b}])|()){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
