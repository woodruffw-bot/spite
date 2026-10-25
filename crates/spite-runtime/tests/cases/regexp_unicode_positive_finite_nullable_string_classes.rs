//! Represented positive finite nullable string-class ranges (22.2.2.3).

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
fn finite_ranges_keep_mandatory_priority_exact_optional_budgets_and_lazy_stopping() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|){{2,3}}/dv.exec('abbb'),b=/(?<each>|{class}){{02,003}}/dv.exec('abbb'),c=/(?<each>{class}|){{2,3}}?/dv.exec('abbb'),d=/(?<each>|{class}){{2,3}}?/dv.exec('abbb');a[0]==='abb' && a.groups.each==='b' && a.indices[1][0]===2 && b[0]==='a' && b.groups.each==='a' && b.indices[1][0]===0 && c[0]==='ab' && c.groups.each==='b' && c.indices[1][0]===1 && d[0]==='' && d.groups.each==='' && d.indices[1][0]===0"
        ));
        check(&format!(
            "let a=/(?<each>{class}|){{1,2}}/dv.exec('abbb'),b=/(?<each>|{class}){{1,2}}/dv.exec('abbb'),c=/(?<each>{class}|){{1,2}}?/dv.exec('abbb');a[0]==='ab' && a.groups.each==='b' && b[0]==='a' && b.groups.each==='a' && c[0]==='a' && c.groups.each==='a'"
        ));
        check(&format!(
            "let a=/(?<each>|{class}){{2,4}}/dv.exec('abbb'),b=/(?<each>|{class}){{3,5}}/dv.exec('abbb');a[0]==='ab' && a.groups.each==='b' && b[0]==='ab' && b.groups.each==='b' && a.indices[1][0]===1 && b.indices[1][0]===1"
        ));
        check(&format!(
            "let a=/(?<whole>(?<each>(?<deep>{class}|)){{3,4}})/dv.exec('abX'),b=/(?<each>|{class}){{3,4}}/dv.exec('abX');a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===2 && a.indices[3][1]===2 && b[0]==='a' && b.groups.each==='a'"
        ));
    }
}

#[test]
fn phase_comparison_preserves_earliest_candidates_and_outside_word_line_gates() {
    check(
        r"let r=/(?<whole>(?<each>|[\q{a|ab}&&a]){2,3}$)/dvgy,a=r.exec('aa');a.index===0 && a[0]==='aa' && a.groups.whole==='aa' && a.groups.each==='a' && a.indices[2][0]===1 && a.indices[2][1]===2 && r.lastIndex===2",
    );
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>|{class}){{2,3}}\\b$)/dimv.exec('!\\r\\nABB\\u2028!'),b=/(?<whole>^\\b(?<each>{class}|){{3,4}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='ABB' && a.groups.each==='B' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='' && b.indices[2][0]===5 && b.indices[2][1]===5"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|){{3,4}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>|{class}){{2,3}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1 && b.indices[1][1]===2"
        ));
    }
}

#[test]
fn finite_consumers_replacements_clones_and_collection_preserve_selected_phase_values() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,4}/dvg,s=/(?<each>|[\q{a|b|ab}&&\p{Any}]){2,3}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && a[2].indices[1][0]===5 && b.length===6 && b.map(m=>m[0]).join('|')==='a|b||b|a|' && b[0].groups.each==='a' && b[1].groups.each==='b' && b[2].groups.each==='' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1>')==='<><>X<><>' && 'abXba'.replace(s,'<$1>')==='<a><b><>X<b><a><>' && 'ab'.split(/(?<each>|[\q{a|b}]){2,3}/v).join('|')==='|a||b|'",
    );
    check(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,4}/vg,(m,each,at,input,groups)=>{seen.push([m,each,at,groups.each]);return '#'});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][3]==='' && seen[1][2]===2 && seen[2][2]===3",
    );
    check(
        r"let r=/(?<each>|[\q{a|b}]){2,3}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'ab'.match(r).join('|')==='a|b|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){2,3}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|)){3,4}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.indices[1][0]===3 && m.indices[2][1]===3 && r.lastIndex===3"), Ok(Value::Boolean(true)));
}

#[test]
fn complete_positive_finite_proofs_finish_finalizers_and_keep_overflowing_ranges_deferred() {
    for source in [
        r"/([\q{a}]|){2,3}/v.test('')",
        r"/([\q{a}]|){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,3}/v.test('')",
        r"/([\q{a|ab}&&a]|){2,3}/v.test('aaa')",
        r"/([\q{a|b}]|){1,2}/v.test('a')",
        r"/([\q{a|b}]|){2,3}/v.test('a')",
        r"/([\q{a|b}]|){2,4}/v.test('a')",
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
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(|[\\q{ab}&&\\q{cd}]){2,3}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b^([\\q{a|ab}&&a]|){2,3}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaa\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3],[false,\"ababa\",{\"values\":[\"a\",\"\"],\"indices\":[[0,1],[1,1]],\"index\":0},1],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaa\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3],[true,\"ababa\",{\"values\":[\"a\",\"\"],\"indices\":[[0,1],[1,1]],\"index\":0},1],[true,\"\\nab\\n\",{\"values\":[\"a\",\"\"],\"indices\":[[1,2],[2,2]],\"index\":1},2],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/(|[[\q{ab}&&\q{cd}]&&\p{RGI_Emoji}]){2,3}/v.test('')",
        r"/(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2,3}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:X|)?){2,3}/v.test('a')",
        r"/([\q{a|b}]|(\b)?){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]||){2,3}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|){2,3}/v.test('a')",
        r"/\b^([[\q{a|ab}&&a]&&\p{RGI_Emoji}]|){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,3}[b]/v.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    for pattern in [
        r"([^\q{ab}]|){2,3}",
        r"([\q{ab}&&a--a]|){2,3}",
        r"([\q{\d}&&a]|){2,3}?",
        r"([\q{a|b}]|){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
