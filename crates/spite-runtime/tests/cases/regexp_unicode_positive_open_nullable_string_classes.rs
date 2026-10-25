//! Positive open-count nullable string classes (22.2.2.3 RepeatMatcher).

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
fn positive_minimums_keep_mandatory_branch_priority_and_final_empty_captures() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>(?<each>(?<deep>{class}|)){{003,}})/dv.exec('abX'),b=/(?<each>|{class}){{3,}}/dv.exec('abX');a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===2 && a.indices[3][1]===2 && b[0]==='ab' && b.groups.each==='b' && b.indices[1][0]===1 && b.indices[1][1]===2"
        ));
        check(&format!(
            "let a=/(?<each>{class}|){{3,}}?/dv.exec('abX'),b=/(?<each>|{class}){{03,}}?/dv.exec('abX');a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0"
        ));
        check(&format!(
            "let a=/(?<each>{class}|){{2,}}?/dv.exec('abbb'),b=/(?<each>{class}|){{3,}}?/dv.exec('abbb');a[0]==='ab' && a.groups.each==='b' && a.indices[1][0]===1 && b[0]==='abb' && b.groups.each==='b' && b.indices[1][0]===2"
        ));
        check(&format!(
            "let m=/(?<whole>^\\b(?<each>|{class}){{3,}}?\\b$)/dimv.exec('!\\r\\nABB\\u2028!');m.index===3 && m.groups.whole==='ABB' && m.groups.each==='B' && m.indices[2][0]===5 && m.indices[2][1]===6"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|){{03,}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>|{class}){{3,}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && a.indices[1][1]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1 && b.indices[1][1]===2"
        ));
    }
}

#[test]
fn unrepresentable_minimums_preserve_empty_choices_without_expanding_repetitions() {
    let huge = "9".repeat(3);
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|){{{huge},}}/dv.exec('abX'),b=/(?<each>|{class}){{{huge},}}/dv.exec('abX'),c=/(?<each>{class}|){{{huge},}}?/dv.exec('abX'),d=/(?<each>|{class}){{{huge},}}?/dv.exec('abX');a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ab' && b.groups.each==='b' && b.indices[1][0]===1 && c[0]==='ab' && c.groups.each==='' && c.indices[1][0]===2 && d[0]==='' && d.groups.each==='' && d.indices[1][0]===0"
        ));
    }
    check(&format!(
        "let r=/(?<each>[\\q{{a|b|ab}}&&\\p{{Any}}]|){{{huge},}}/dvg,a=r.exec('abX'),b=r.exec('abX');r.source.includes('{huge}') && a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && b.index===2 && b[0]==='' && b.groups.each==='' && r.lastIndex===2"
    ));
    check(&format!(
        "let r=/(?<each>|[\\q{{ab}}&&a]){{{huge},}}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
    ));
}

#[test]
fn mandatory_character_counts_use_decoded_characters_and_original_unicode_ranges() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|)){{2,}})/dvgy,s=/(?<each>(|{class})){{2,}}/dvgy;r.lastIndex=2;s.lastIndex=2;let a=r.exec('A😀x'),b=s.exec('A😀x'),c=r.exec('A😀x');a.index===1 && a[0]==='😀' && a.groups.whole==='😀' && a.groups.each==='' && a.indices[2][0]===3 && a.indices[3][1]===3 && b.index===1 && b[0]==='😀' && b.groups.each==='😀' && b.indices[1][0]===1 && b.indices[2][1]===3 && c.index===3 && c[0]==='' && c.groups.each==='' && r.lastIndex===3 && s.lastIndex===3"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<each>({class}|)){{3,}}/divgy,s=/(?<each>(|{class})){{3,}}/divgy;r.lastIndex=1;s.lastIndex=1;let a=r.exec('𐐀𐐨x'),b=s.exec('𐐀𐐨x');a.index===0 && a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && a.indices[2][1]===4 && b[0]==='𐐀𐐨' && b.groups.each==='𐐨' && b.indices[1][0]===2 && b.indices[2][1]===4 && r.lastIndex===4 && s.lastIndex===4"
        ));
    }
    check(
        r"let a=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|)){3,}/dv.exec('\uD800\uD800a'),b=/(?<each>(|[\q{\uD800a|\uD800}&&\p{Any}])){3,}/dv.exec('\uD800\uD800a');a[0].length===2 && a[0].charCodeAt(0)===0xd800 && a.groups.each==='' && a.indices[1][0]===2 && a.indices[2][1]===2 && b.groups.each.charCodeAt(0)===0xd800 && b.indices[1][0]===1 && b.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|){2,}/v.exec('𐀀').groups.each===''",
    );
    check(
        r"let m=/(?<whole>(?<\u{1D49C}>[\q{a|ab}&&a]|){3,})/dv.exec('aaX');m.groups.whole==='aa' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===2 && m.indices.groups['𝒜'][1]===2",
    );
}

#[test]
fn consumers_replacements_clones_and_collection_keep_final_required_empty_values() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,}/dvg,s=/(?<each>|[\q{a|b|ab}&&\p{Any}]){3,}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && a[2].indices[1][0]===5 && b.length===4 && b[0].groups.each==='b' && b[2].groups.each==='a' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<:><:>X<:><:>' && 'abXba'.replace(s,'<$1:$<each>>')==='<b:b><:>X<a:a><:>' && 'abXba'.split(/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,}/v).join('|')==='||X||'",
    );
    check(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,}/vg,(m,each,at,input,groups)=>{seen.push([m,each,at,groups.each]);return '#'});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][2]===0 && seen[0][3]==='' && seen[1][0]==='' && seen[1][2]===2 && seen[2][2]===3",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){3,}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|)){3,}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zab!');m[0]==='ab' && m.groups.each==='' && m.indices[1][0]===3 && m.indices[2][1]===3 && r.lastIndex===3"), Ok(Value::Boolean(true)));
}

#[test]
fn complete_positive_open_proofs_finish_finalizers_and_never_hide_deferred_leaves() {
    for source in [
        r"/([\q{a|b}]|){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,3}/v.test('aaa')",
        r"/([\q{a|b}]|){2,4}/v.test('a')",
        r"/([\q{a|b}]|){2}/v.test('a')",
        r"/([\q{a|b}]|){2,2}/v.test('a')",
        r"/([\q{a}]|){2,}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,}/v.test('aaa')",
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(|[\\q{ab}&&\\q{cd}]){2,}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b^([\\q{a|ab}&&a]|){2,}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"a\"],\"indices\":[[0,6],[5,6]],\"index\":0},6],[false,\"ababa\",{\"values\":[\"a\",\"\"],\"indices\":[[0,1],[1,1]],\"index\":0},1],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"a\"],\"indices\":[[0,6],[5,6]],\"index\":0},6],[true,\"ababa\",{\"values\":[\"a\",\"\"],\"indices\":[[0,1],[1,1]],\"index\":0},1],[true,\"\\nab\\n\",{\"values\":[\"a\",\"\"],\"indices\":[[1,2],[2,2]],\"index\":1},2],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/(|[[\q{ab}&&\q{cd}]&&\p{RGI_Emoji}]){2,}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:X|)?){2,}/v.test('a')",
        r"/([\q{a|b}]|(\b)?){2,}/v.test('a')",
        r"/([\q{a|ab}&&a]||){2,}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|){2,}/v.test('a')",
        r"/\b^([[\q{a|ab}&&a]&&\p{RGI_Emoji}]|){2,}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,}[b]/v.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    for pattern in [
        r"([^\q{ab}]|){2,}",
        r"([\q{ab}&&a--a]|){3,}",
        r"([\q{\d}&&a]|){2,}?",
        r"([\q{a|b}]|){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
