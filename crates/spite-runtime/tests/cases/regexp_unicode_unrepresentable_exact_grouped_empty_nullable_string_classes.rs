//! Overflowing exact grouped-empty nullable string-class counts (22.2.2.3 RepeatMatcher).

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
fn overflowing_exact_counts_keep_mandatory_source_priority_independent_of_greediness() {
    let huge = "9".repeat(40);
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        for q in [
            format!("{{{huge}}}"),
            format!("{{{huge}}}?"),
            format!("{{{huge},{huge}}}"),
            format!("{{000{huge},0{huge}}}?"),
        ] {
            check(&format!(
                "let a=/(?<whole>(?<each>(?<deep>{class}|(?:(?:)))){q})/dv.exec('abX'),b=/(?<each>(?:(?:))|{class}){q}/dv.exec('abX');a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===2 && a.indices[3][1]===2 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && b.indices[1][1]===0"
            ));
        }
        check(&format!(
            "let a=/(?<each>(?:(?:))|{class}){{{huge}}}/dv.exec('abX'),b=/(?<each>(?:(?:))|{class}){{{huge},}}/dv.exec('abX');a[0]==='' && a.groups.each==='' && b[0]==='ab' && b.groups.each==='b' && b.indices[1][0]===1"
        ));
    }
}

#[test]
fn numeric_endpoint_equality_preserves_source_and_executes_unequal_captured_empty_choice_ranges() {
    let huge = "9".repeat(40);
    check(&format!(
        "let r=/(?<each>[\\q{{a|b|ab}}&&\\p{{Any}}]|(?:(?:))){{000{huge},0{huge}}}?/dvg,a=r.exec('abX'),b=r.exec('abX');r.source.includes('000{huge},0{huge}') && r.source.endsWith('?') && a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && b.index===2 && b[0]==='' && b.groups.each==='' && r.lastIndex===2"
    ));
    for (lower, upper, whole) in [
        (huge.clone(), format!("{huge}0"), "ab"),
        ("8".repeat(40), format!("{}9", "8".repeat(39)), "a"),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(realm.eval(&format!(r"marker=1;try{{let r=/((?:|())|[\q{{a|b}}]){{{lower},{upper}}}/dv,m=r.exec('ab');m[0]==='{whole}' && m[1]==='{whole}'.slice(-1) && m[2]===undefined && m.indices[2]===undefined}}catch{{marker=2}}finally{{marker=3}}")),Ok(Value::Boolean(true)));
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
    check(&format!(
        "let r=/(?<each>(?:(?:))|[\\q{{ab}}&&a]){{{huge}}}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
    ));
}

#[test]
fn exact_overflowing_capture_ranges_preserve_unicode_boundaries_names_and_outside_gates() {
    let huge = "9".repeat(40);
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|(?:(?:)))){{{huge}}}?)/dvgy;r.lastIndex=2;let a=r.exec('A😀😀x'),b=r.exec('A😀😀x'),c=/(?<each>(?:(?:))|{class}){{000{huge},0{huge}}}$/dv.exec('😀😀');a.index===1 && a[0]==='😀😀' && a.groups.whole==='😀😀' && a.groups.each==='' && a.indices[2][0]===5 && a.indices[3][1]===5 && b.index===5 && b[0]==='' && b.groups.each==='' && r.lastIndex===5 && c.index===0 && c[0]==='😀😀' && c.groups.each==='😀' && c.indices[1][0]===2 && c.indices[1][1]===4"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{{huge}}}?/div.exec('𐐀𐐨'),b=/(?<each>(?:(?:))|{class}){{{huge}}}$/div.exec('𐐀𐐨');a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && a.indices[1][1]===4 && b[0]==='𐐀𐐨' && b.groups.each==='𐐨' && b.indices[1][0]===2 && b.indices[1][1]===4"
        ));
    }
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>{class}|(?:(?:))){{{huge}}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!'),b=/(?<whole>^\\b(?<each>(?:(?:))|{class}){{{huge}}}\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='AB' && a.groups.each==='' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='B' && b.indices[2][0]===4 && b.indices[2][1]===5"
        ));
    }
    check(&format!(
        "let m=/(?<whole>(?<\\u{{1D49C}}>[\\q{{a|ab}}&&a]|(?:(?:))){{{huge}}})/dv.exec('aaX');m.groups.whole==='aa' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===2 && m.indices.groups['𝒜'][1]===2"
    ));
    check(&format!(
        "let m=/(?<each>[\\q{{\\uD800a|\\uD800}}&&\\p{{Any}}]|(?:(?:))){{{huge}}}/dv.exec('\\uD800\\uD800a');m[0].length===2 && m[0].charCodeAt(0)===0xd800 && m.groups.each==='' && m.indices[1][0]===2 && m.indices[1][1]===2"
    ));
}

#[test]
fn consumers_replacements_clones_and_collection_retain_overflowing_exact_capture_values() {
    let huge = "9".repeat(40);
    check(&format!(
        "let r=/(?<each>[\\q{{a|b|ab}}&&\\p{{Any}}]|(?:(?:))){{{huge}}}/dvg,s=/(?<each>(?:(?:))|[\\q{{a|b|ab}}&&\\p{{Any}}]){{{huge}}}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && a[2].indices[1][0]===5 && b.length===6 && b.every(m=>m[0]==='' && m.groups.each==='') && b.map(m=>m.index).join(',')==='0,1,2,3,4,5' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<:><:>X<:><:>' && 'abXba'.replace(s,'<$1:$<each>>')==='<:>a<:>b<:>X<:>b<:>a<:>'"
    ));
    check(&format!(
        "let seen=[],result='abX'.replace(/(?<each>[\\q{{a|b|ab}}&&\\p{{Any}}]|(?:(?:))){{{huge}}}/vg,(m,each,at,input,groups)=>{{seen.push([m,each,at,groups.each]);return '#'}});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][3]==='' && seen[1][2]===2 && seen[2][2]===3 && 'abXba'.split(/(?<each>(?:(?:))|[\\q{{a|b}}]){{{huge}}}/v).join('|')==='a||b||X||b||a'"
    ));
    check(&format!(
        "let r=/(?<each>(?:(?:))|[\\q{{a|b}}]){{{huge}}}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='|||||' && r.lastIndex===0"
    ));
    check(&format!(
        "let r=/(?<each>[\\q{{a|b|ab}}&&\\p{{Any}}]|(?:(?:))){{{huge}}}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB'),n=r.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && n[0]==='ABB' && n.groups.each==='' && n.index===0 && r.lastIndex===7"
    ));
    let mut realm = Realm::default();
    realm
        .eval(&format!(
            r"var r=/(?<each>([\q{{a|b}}]|(?:(?:)))){{{huge}}}/dvg;r.lastIndex=1"
        ))
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabb!');m[0]==='abb' && m.groups.each==='' && m.indices[1][0]===4 && m.indices[2][1]===4 && r.lastIndex===4"), Ok(Value::Boolean(true)));
}

#[test]
fn overflowing_exact_grouped_proofs_finish_finalizers_and_prove_unknown_leaves() {
    for source in [
        r"/([\q{a|b}]|(?:)){999999999999999999999999999999999999999}/v.test('a')",
        r"/(?<each>(?:(?:))|[\q{ab}&&a]){999999999999999999999999999999999999999}?/v.exec('X').groups.each===''",
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
    let huge = "9".repeat(40);
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="((?:)|[\\q{ab}&&\\q{cd}]){9999999999999999999999999999999999999999}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for inner in [
        r"(?:)|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
        r"([\q{a|b}])|(?:X|)?",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(?:\b|())",
        r"[\q{a|b}]|(\b)?",
        r"[\q{a|ab}&&a]||(?:)",
        r"\b[\q{a|ab}&&a]|(?:)",
    ] {
        let source = format!("/({inner}){{{huge}}}/v.test('a')");
        assert!(
            matches!(
                Realm::default().eval(&source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    for source in [
        r"/([\q{a|b}]|(?:\b|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|b}]|(?:\b|())){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
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
        format!(r"([^\q{{ab}}]|(?:)){{{huge}}}"),
        format!(r"([\q{{ab}}&&a--a]|(?:)){{{huge}}}"),
        format!(r"([\q{{\d}}&&a]|(?:)){{{huge}}}?"),
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn grouped_overflowing_exact_wrappers_and_endpoint_padding_stay_flat_and_host_aborts_are_atomic() {
    let huge = "9".repeat(40);
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]|(?:(?:)){}{{{huge}}}?",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aa');m.length===1001 && m[0]==='aa' && m[1000]==='' && m.indices[1000][0]===2 && m.indices[1000][1]===2"
    ));
    let pattern = format!(r"((?:(?:))|[\q{{{}}}&&a]){{{huge}}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0 && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|(?:(?:))){{{}}}", "9".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='aaa' && m[1]==='' && m.indices[1][0]===3 && m.indices[1][1]===3"
    ));
    let pattern = format!(
        r"((?:(?:))|[\q{{a|ab}}&&a]){{000{huge},{}{huge}}}",
        "0".repeat(20_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0"
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
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    let overflow = format!("{}0", usize::MAX);
    realm
        .eval(&format!(
            r"var r=/([\q{{a}}]|(?:(?:))){{{overflow}}}$/vg,marker=0"
        ))
        .unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(
        realm.eval(r"var probe=/x/vy;probe.lastIndex=1;probe.exec(text)[0]==='x'"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("r.lastIndex===1 && marker===0"),
        Ok(Value::Boolean(true))
    );
}
