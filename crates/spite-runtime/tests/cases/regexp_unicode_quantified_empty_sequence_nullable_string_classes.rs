//! Nullable string-class counts beside quantified noncapturing empty sequences (22.2.2.3).

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
fn quantified_sequence_finite_phases_keep_earliest_starts_and_outside_line_word_gates() {
    let limit = 8u128;
    check(&format!(
        r"let r=/(?<whole>(?<each>(?:)*?(?:)*?|[\q{{a|ab}}&&a]){{{limit},{}}}$)/dvgy,m=r.exec('aa');m.index===0 && m[0]==='aa' && m.groups.whole==='aa' && m.groups.each==='a' && m.indices[2][0]===1 && r.lastIndex===2",
        limit + 1
    ));
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>(?:)*?(?:)*?|{class}){{{limit},{}}}\\b$)/dimv.exec('!\\r\\nABB\\u2028!'),b=/(?<whole>^\\b(?<each>{class}|(?:)*?(?:)*?){{{limit},{}}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='ABB' && a.groups.each==='B' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='' && b.indices[2][0]===5",
            limit + 1,
            limit + 1
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|(?:)*?(?:)*?){{{limit},{}}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>(?:)*?(?:)*?|{class}){{{limit},{}}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1",
            limit + 1,
            limit + 1
        ));
    }
}

#[test]
fn quantified_sequence_supplementary_bounds_keep_original_supplementary_capture_ranges_and_source_metadata()
 {
    let limit = 8u128;
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|(?:)*?(?:)*?)){{{limit},{}}})/dvgy,s=/(?<each>((?:)*?(?:)*?|{class})){{{limit},{}}}/dvgy;r.lastIndex=2;s.lastIndex=2;let a=r.exec('A😀😀x'),b=s.exec('A😀😀x'),c=/(?<each>(?:)*?(?:)*?|{class}){{{limit},{}}}$/dv.exec('😀😀');a.index===1 && a[0]==='😀😀' && a.groups.each==='' && a.indices[2][0]===5 && a.indices[3][1]===5 && b.index===1 && b[0]==='😀' && b.groups.each==='😀' && b.indices[1][0]===1 && b.indices[2][1]===3 && r.lastIndex===5 && s.lastIndex===3 && c.index===0 && c[0]==='😀😀' && c.indices[1][0]===2 && c.indices[1][1]===4",
            limit + 1,
            limit + 1,
            limit + 1
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|(?:)*?(?:)*?){{{limit},{}}}?/div.exec('𐐀𐐨'),b=/(?<each>(?:)*?(?:)*?|{class}){{{limit},{}}}/div.exec('𐐀𐐨');a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && b[0]==='𐐀' && b.groups.each==='𐐀' && b.indices[1][0]===0 && b.indices[1][1]===2",
            limit + 1,
            limit + 1
        ));
    }
    check(&format!(
        r"let source='(?<whole>(?<\\u{{1D49C}}>(?:)*?(?:)*?|[\\q{{a|b}}]){{00{limit},000{}}}?)',r=new RegExp(source,'dv'),m=r.exec('aaa');r.source===source && r.flags==='dv' && m[0]==='' && m.groups.whole==='' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===0",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>(?:)*?(?:)*?|[\q{{a|b}}]){{{limit},{}}}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1",
        limit + 1
    ));
}

#[test]
fn quantified_sequence_finite_consumers_clones_and_collection_keep_selected_iteration_values() {
    let limit = 8u128;
    check(&format!(
        r"let r=/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|(?:)*?(?:)*?){{{limit},{}}}/dvg,s=/(?<each>(?:)*?(?:)*?|[\q{{a|b}}]){{{limit},{}}}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && b.length===6 && b.map(m=>m[0]).join('|')==='a|b||b|a|' && b[0].groups.each==='a' && b[2].groups.each==='' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1>')==='<><>X<><>' && 'abXba'.replace(s,'<$1>')==='<a><b><>X<b><a><>' && 'ab'.split(s).join('|')==='|a||b|'",
        limit + 1,
        limit + 1
    ));
    check(&format!(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|(?:)*?(?:)*?){{{limit},{}}}/vg,(m,each,at,input,groups)=>{{seen.push([m,each,at,groups.each]);return '#'}});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][3]==='' && seen[1][2]===2 && seen[2][2]===3",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>(?:)*?(?:)*?|[\q{{a|b}}]){{{limit},{}}}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'ab'.match(r).join('|')==='a|b|' && r.lastIndex===0",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|(?:)*?(?:)*?){{{limit},{}}}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='' && r.lastIndex===7",
        limit + 1
    ));
    let mut realm = Realm::default();
    realm
        .eval(&format!(
            r"var r=/(?<each>([\q{{a|b}}]|(?:)*?(?:)*?)){{{limit},{}}}/dvg;r.lastIndex=1",
            limit + 1
        ))
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.indices[1][0]===3 && m.indices[2][1]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn quantified_sequence_complete_proofs_finish_finalizers_and_keep_choices_deferred() {
    for source in [
        r"/(?:[\q{a}]|(?:)(?:)*)+/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*)+/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){0}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){1}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){2,}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:)*){2}/v.test('a')",
        r"/([\q{a|b}]|(?:)(?:){0})+/v.test('a')",
        r"/([\q{a|b}]|(?:)*)+/v.test('a')",
        r"/([\q{a|b}]|(?:)*){0}/v.test('a')",
        r"/([\q{a|b}]|(?:)*){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:)*){1}/v.test('a')",
        r"/([\q{a|b}]|(?:)*){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:)*){2,}/v.test('a')",
        r"/([\q{a|b}]|(?:)*){2}/v.test('a')",
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
    for (lower, upper, whole, each, at) in [
        ("9".repeat(3), format!("{}0", "9".repeat(3)), "ab", "b", 1),
        ("8".repeat(3), format!("{}9", "8".repeat(2)), "a", "a", 0),
    ] {
        check(&format!(
            r"let marker=0,value;try{{value=/((?:)(?:)*|[\q{{a|b}}]){{{lower},{upper}}}/dv.exec('ab')}}finally{{marker=3}}marker===3 && value[0]==='{whole}' && value[1]==='{each}' && value.indices[1][0]==={at}"
        ));
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="((?:)*|[\\q{ab}&&\\q{cd}])+?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for (inner, expected_zero) in [
        (
            r"(?:)*|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
            r#"[["","<undefined>"],[[1,1],"<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])|(?:X|)*",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|(?:\b|())",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|(\b)?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|(?:\b)()*",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|ab}&&a]||(?:)*",
            r#"[["","<undefined>"],[[1,1],"<undefined>"],1,1]"#,
        ),
        (
            r"\b[\q{a|ab}&&a]|(?:)*",
            r#"[["","<undefined>"],[[1,1],"<undefined>"],1,1]"#,
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
        r"([^\q{ab}]|(?:)*){0}",
        r"([\q{ab}&&a--a]|(?:)*){2,3}",
        r"([\q{\d}&&a]|(?:)*){1,}",
        r"([\q{a|b}]|(?:)*){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
