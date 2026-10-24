//! Capture wrappers around repeated character-result v classes (22.2.2.3).

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
fn complete_inner_wrappers_capture_the_final_iteration_beside_whole_repetition_captures() {
    check(
        r"let r=/(?<whole>(?<each>(?<deep>[\q{a|ab}&&a]))+)/dvg,m=r.exec('xaaab!');m.length===4 && m.index===1 && m[0]==='aaa' && m[1]==='aaa' && m[2]==='a' && m[3]==='a' && m.groups.whole==='aaa' && m.groups.each==='a' && m.groups.deep==='a' && m.indices[1][0]===1 && m.indices[1][1]===4 && m.indices[2][0]===3 && m.indices[3][1]===4 && m.indices.groups.each===m.indices[2] && m.indices.groups.deep===m.indices[3] && r.lastIndex===4 && r.exec('xaaab!')===null && r.lastIndex===0 && m.groups.each==='a'",
    );
    check(
        r"let a=/(?<whole>(?<each>[\q{a|b|ab}&&\p{Any}]){2,3})/dv.exec('abbb'),b=/(?<whole>(?<each>[\q{a|b|ab}&&\p{Any}]){2,3}?)/dv.exec('abbb');a.groups.whole==='abb' && a.groups.each==='b' && a.indices[2][0]===2 && b.groups.whole==='ab' && b.groups.each==='b' && b.indices[2][0]===1",
    );
    check(
        r"let m=/(?<\u{1D49C}>[\q{a|ab}&&a])+/dv.exec('aaa');m.groups['𝒜']==='a' && m.indices.groups['𝒜']===m.indices[1] && m.indices[1][0]===2 && m.indices[1][1]===3",
    );
    check(
        r"let m=/(?:(?:[\q{a|ab}&&a])){2}/dv.exec('aaa');m.length===1 && m[0]==='aa' && m.groups===undefined && m.indices.length===1",
    );
}

#[test]
fn zero_iterations_leave_inner_captures_undefined_and_keep_names_and_indices_present() {
    check(
        r"let m=/(?<whole>(?<each>([\q{ab}&&a]))*)/dv.exec('ab');m.length===4 && m[0]==='' && m[1]==='' && m[2]===undefined && m[3]===undefined && m.groups.whole==='' && Object.hasOwn(m.groups,'each') && m.groups.each===undefined && Object.getPrototypeOf(m.groups)===null && m.indices[1][0]===0 && m.indices[1][1]===0 && m.indices[2]===undefined && m.indices[3]===undefined && Object.hasOwn(m.indices.groups,'each') && m.indices.groups.each===undefined",
    );
    check(
        r"let r=/(?<each>([\q{a|ab}&&a])){0}/dvgy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]===undefined && m[2]===undefined && Object.hasOwn(m.groups,'each') && m.groups.each===undefined && m.indices[1]===undefined && m.indices[2]===undefined && r.lastIndex===0",
    );
    check(
        r"let m=/(?<each>[\q{a|ab}&&a])*?/dv.exec('aaa');m[0]==='' && m[1]===undefined && m.groups.each===undefined && m.indices[1]===undefined && /(?<each>[\q{ab}&&a])+/v.exec('ab')===null",
    );
    let huge = "9".repeat(100);
    check(&format!(
        r"!/([\q{{a|ab}}&&a]){{{huge}}}/v.test('aaa') && (()=>{{let m=/([\q{{ab}}&&a]){{0,{huge}}}/dv.exec('ab');return m[0]==='' && m[1]===undefined && m.indices[1]===undefined && m.groups===undefined;}})()"
    ));
}

#[test]
fn captured_iteration_ranges_keep_supplementary_case_folding_and_lone_surrogate_identity() {
    check(
        r"let r=/(?<whole>(?<each>([\q{😀|😀x}&&\p{Any}]))+)/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m[1]==='😀😀' && m[2]==='😀' && m[3]==='😀' && m.indices[0][0]===1 && m.indices[2][0]===3 && m.indices[3][1]===5 && m.indices.groups.each===m.indices[2] && r.lastIndex===5",
    );
    check(
        r"let m=/(?<each>([\q{\u{10428}|ab}&&\p{Any}])){2}/div.exec('𐐨𐐀');m[0]==='𐐨𐐀' && m[1]==='𐐀' && m[2]==='𐐀' && m.groups.each==='𐐀' && m.indices[1][0]===2 && m.indices[2][1]===4",
    );
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]))+/dv.exec('\uD800\uD800a');m[0].length===2 && m[1].length===1 && m.groups.each.charCodeAt(0)===0xd800 && m.indices[1][0]===1 && m.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}])+/v.exec('𐀀ab')===null",
    );
    check(
        r"let r=/(?<each>[\q{a|ab}&&a])*/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn consumers_clones_and_collection_use_last_iteration_capture_values() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}])+/dvg,a=[...'abbbXaba'.matchAll(r)];a.length===2 && a[0][0]==='abbb' && a[0].groups.each==='b' && a[0].indices[1][0]===3 && a[1][0]==='aba' && a[1].groups.each==='a' && a[1].indices[1][0]===7 && r.lastIndex===0 && 'abbbXaba'.replace(r,'[$1:$<each>]')==='[b:b]X[a:a]' && 'abbbXaba'.split(/(?<each>[\q{a|b|ab}&&\p{Any}])+/v).join('|')==='|b|X|a|'",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}])+/vg;r.lastIndex=9;'xabba'.search(r)===1 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab|ba' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}])+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('ABB')===null && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    check(
        r"let r=/(?<each>([\q{ab}&&a]))*/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && Object.hasOwn(a[0].groups,'each') && a[0].groups.each===undefined && a[0][1]===undefined && a[0][2]===undefined && '😀'.replace(r,'<$1:$<each>>')==='<:>😀<:>'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b|ab}&&\p{Any}]))+/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabbb').groups.each==='b' && r.lastIndex===5"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_wrapped_controls_finish_finalizers_and_other_atom_shapes_keep_complete_proof() {
    for (source, expected) in [
        (r"/([\q{ab}&&a])+/v.test('ab')", false),
        (r"/([\p{Any}--\q{ab}])+/v.test('ab')", true),
        (r"/(?<each>[\q{a|ab}&&a])+/v.test('aaa')", true),
        (r"/([\q{ab}&&a]){0}/v.test('')", true),
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{ab}--\\q{ab}])*",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/([\q{ab}&&\q{cd}]){0}/v.test('')",
        r"/([[\q{ab}--\q{ab}]&&\p{RGI_Emoji}])*/v.test('')",
        r"/(?<each>[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}/v.test('')",
        r"/(\b[\q{a|ab}&&a])+/v.test('a')",
        r"/([\q{a|ab}&&a]|b)+/v.test('a')",
        r"/([\q{a|ab}&&a])+[b]/v.test('ab')",
        r"/\b^([\q{a|ab}&&a])+/v.test('a')",
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
        r"(?<each>[^\q{ab}--a]){0}",
        r"(?<each>[\q{ab}&&a--a]){0}",
        r"(?<each>[\q{\d}&&a]){0}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn complete_deep_wrappers_use_defaults_and_opt_in_aborts_keep_capture_and_index_state_atomic() {
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]{}+",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');m.length===1001 && m[0]==='aaa' && m[1000]==='a' && m.indices[1000][0]===2 && m.indices[1000][1]===3"
    ));
    let pattern = format!(r"([\q{{{}}}&&\p{{Any}}]){{0}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]===undefined && m.indices[1]===undefined && r.lastIndex===0"
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
    realm.eval(r"r=/([\q{a|ab}&&a])+/vg").unwrap();
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
