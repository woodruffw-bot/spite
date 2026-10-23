//! Finite nullable string-class ranges with two overflowing bounds (22.2.2.3).

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
fn overflowing_finite_ranges_preserve_exact_optional_budgets_and_mandatory_source_priority() {
    let limit = 10u128.pow(30);
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>|{class}){{{limit},{}}}/dv.exec('aaaa'),b=/(?<each>|{class}){{{limit},{}}}/dv.exec('aaaa'),c=/(?<each>|{class}){{{limit},{}}}?/dv.exec('aaaa'),d=/(?<each>{class}|){{{limit},{}}}?/dv.exec('aaaa');a[0]==='a' && a.groups.each==='a' && a.indices[1][0]===0 && b[0]==='aaa' && b.groups.each==='a' && b.indices[1][0]===2 && c[0]==='' && c.groups.each==='' && d[0]==='aaaa' && d.groups.each==='' && d.indices[1][0]===4",
            limit + 1,
            limit + 3,
            limit + 1,
            limit + 1
        ));
        check(&format!(
            "let a=/(?<each>{class}|){{{limit},{}}}?/dv.exec('abX'),b=/(?<each>|{class}){{{limit},{}}}/dv.exec('abX'),c=/(?<each>|{class}){{{limit},{}}}/dv.exec('abX');a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ab' && b.groups.each==='b' && c[0]==='ab' && c.groups.each==='b'",
            limit + 1,
            limit * 100,
            limit + usize::MAX as u128
        ));
    }
}

#[test]
fn overflowing_finite_phases_keep_earliest_starts_and_outside_line_word_gates() {
    let limit = 10u128.pow(30);
    check(&format!(
        r"let r=/(?<whole>(?<each>|[\q{{a|ab}}&&a]){{{limit},{}}}$)/dvgy,m=r.exec('aa');m.index===0 && m[0]==='aa' && m.groups.whole==='aa' && m.groups.each==='a' && m.indices[2][0]===1 && r.lastIndex===2",
        limit + 1
    ));
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>|{class}){{{limit},{}}}\\b$)/dimv.exec('!\\r\\nABB\\u2028!'),b=/(?<whole>^\\b(?<each>{class}|){{{limit},{}}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='ABB' && a.groups.each==='B' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='' && b.indices[2][0]===5",
            limit + 1,
            limit + 1
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|){{{limit},{}}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>|{class}){{{limit},{}}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1",
            limit + 1,
            limit + 1
        ));
    }
}

#[test]
fn unequal_overflowed_bounds_keep_original_supplementary_capture_ranges_and_source_metadata() {
    let limit = 10u128.pow(30);
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|)){{{limit},{}}})/dvgy,s=/(?<each>(|{class})){{{limit},{}}}/dvgy;r.lastIndex=2;s.lastIndex=2;let a=r.exec('A😀😀x'),b=s.exec('A😀😀x'),c=/(?<each>|{class}){{{limit},{}}}$/dv.exec('😀😀');a.index===1 && a[0]==='😀😀' && a.groups.each==='' && a.indices[2][0]===5 && a.indices[3][1]===5 && b.index===1 && b[0]==='😀' && b.groups.each==='😀' && b.indices[1][0]===1 && b.indices[2][1]===3 && r.lastIndex===5 && s.lastIndex===3 && c.index===0 && c[0]==='😀😀' && c.indices[1][0]===2 && c.indices[1][1]===4",
            limit + 1,
            limit + 1,
            limit + 1
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|){{{limit},{}}}?/div.exec('𐐀𐐨'),b=/(?<each>|{class}){{{limit},{}}}/div.exec('𐐀𐐨');a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && b[0]==='𐐀' && b.groups.each==='𐐀' && b.indices[1][0]===0 && b.indices[1][1]===2",
            limit + 1,
            limit + 1
        ));
    }
    check(&format!(
        r"let source='(?<whole>(?<\\u{{1D49C}}>|[\\q{{a|b}}]){{00{limit},000{}}}?)',r=new RegExp(source,'dv'),m=r.exec('aaa');r.source===source && r.flags==='dv' && m[0]==='' && m.groups.whole==='' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===0",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>|[\q{{a|b}}]){{{limit},{}}}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1",
        limit + 1
    ));
}

#[test]
fn overflowing_finite_consumers_clones_and_collection_keep_selected_iteration_values() {
    let limit = 10u128.pow(30);
    check(&format!(
        r"let r=/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|){{{limit},{}}}/dvg,s=/(?<each>|[\q{{a|b}}]){{{limit},{}}}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && b.length===6 && b.map(m=>m[0]).join('|')==='a|b||b|a|' && b[0].groups.each==='a' && b[2].groups.each==='' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1>')==='<><>X<><>' && 'abXba'.replace(s,'<$1>')==='<a><b><>X<b><a><>' && 'ab'.split(s).join('|')==='|a||b|'",
        limit + 1,
        limit + 1
    ));
    check(&format!(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|){{{limit},{}}}/vg,(m,each,at,input,groups)=>{{seen.push([m,each,at,groups.each]);return '#'}});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][3]==='' && seen[1][2]===2 && seen[2][2]===3",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>|[\q{{a|b}}]){{{limit},{}}}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'ab'.match(r).join('|')==='a|b|' && r.lastIndex===0",
        limit + 1
    ));
    check(&format!(
        r"let r=/(?<each>[\q{{a|b|ab}}&&\p{{Any}}]|){{{limit},{}}}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='' && r.lastIndex===7",
        limit + 1
    ));
    let mut realm = Realm::default();
    realm
        .eval(&format!(
            r"var r=/(?<each>([\q{{a|b}}]|)){{{limit},{}}}/dvg;r.lastIndex=1",
            limit + 1
        ))
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.indices[1][0]===3 && m.indices[2][1]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_unrepresentable_finite_proofs_finish_finalizers_without_hiding_unknown_leaves() {
    for source in [
        r"/([\q{a|ab}&&a]|){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/([\q{a|ab}&&a]|){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('aaa')",
        r"/([\q{a|b}]|){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|b}]|){888888888888888888888888888888,99999999999999999999999999999999999999994}/v.test('a')",
        r"/([\q{a|ab}&&a]|){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|){888888888888888888888888888888,99999999999999999999999999999999999999992}/v.test('a')",
        r"/([\q{a|b}]|){999999999999999999999999999999999999999,9999999999999999999999999999999999999990}/v.test('a')",
        r"/([\q{a|b}]|){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a}]|){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/([\q{a}]|){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|b}]|){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|ab}&&a]|){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('aaa')",
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
    let limit = 10u128.pow(30);
    for inner in [
        r"|[\q{ab}&&\q{cd}]",
        r"|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
        r"([\q{a|b}])|(?:)",
        r"[\q{a|b}]|()",
        r"[\q{a|ab}&&a]||",
        r"\b[\q{a|ab}&&a]|",
    ] {
        let source = format!("({inner}){{{limit},{}}}", limit + 1);
        assert!(
            matches!(
                Realm::default().eval(&format!("/{source}/v.test('a')")),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    for pattern in [
        format!(r"([^\q{{ab}}]|){{{limit},{}}}", limit + 1),
        format!(r"([\q{{ab}}&&a--a]|){{{limit},{}}}", limit + 1),
        format!(r"([\q{{\d}}&&a]|){{{limit},{}}}", limit + 1),
        format!(r"([\q{{a|b}}]|){{{},{limit}}}", limit + 1),
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn overflowing_finite_counts_stay_flat_under_defaults_and_opt_in_aborts_preserve_state() {
    let limit = 10u128.pow(30);
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]|{}{{{limit},{}}}",
        "(".repeat(1_000),
        ")".repeat(1_000),
        limit + 1
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aa');m.length===1001 && m[0]==='aa' && m[1000]==='' && m.indices[1000][0]===2"
    ));
    let pattern = format!(
        r"(|[\q{{{}}}&&a]){{{limit},{}}}",
        "x".repeat(20_000),
        "9".repeat(20_000)
    );
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && r.lastIndex===0"
    ));
    let pattern = format!(
        r"([\q{{a|ab}}&&a]|){{{}{limit},000{}}}",
        "0".repeat(20_000),
        limit + 1
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaaa');r.source.length>20000 && m[0]==='aaaa' && m[1]==='' && m.indices[1][0]===4"
    ));
    let borrow = format!(
        r"(|[\q{{a|ab}}&&a]){{{},1{}}}",
        "9".repeat(20_000),
        "0".repeat(20_000)
    );
    check(&format!(
        "let m=/{borrow}/dv.exec('aaa');m[0]==='a' && m[1]==='a' && m.indices[1][0]===0 && m.indices[1][1]===1"
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
    realm
        .eval(&format!(
            r"var r=/([\q{{a|ab}}&&a]|){{{limit},{}}}$/vg,marker=0",
            limit + 1
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
