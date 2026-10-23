//! One-minimum open braced nullable string classes (22.2.2.3 RepeatMatcher).

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
fn one_open_counts_preserve_branch_priority_greediness_and_outside_word_line_gates() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|){{01,}}/dv.exec('abbb!'),b=/(?<each>|{class}){{0001,}}/dv.exec('abbb!');a[0]==='abbb' && b[0]==='abbb' && a.groups.each==='b' && b.groups.each==='b' && a.indices[1][0]===3 && b.indices[1][1]===4"
        ));
        check(&format!(
            "let a=/(?<each>{class}|){{1,}}?/dv.exec('abbb!'),b=/(?<each>|{class}){{0001,}}?/dv.exec('abbb!');a[0]==='a' && a.groups.each==='a' && a.indices[1][1]===1 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && b.indices[1][1]===0"
        ));
        check(&format!(
            "let m=/(?<whole>^\\b(?<each>|{class}){{1,}}?\\b$)/dimv.exec('!\\r\\nABB\\u2028!');m.index===3 && m.groups.whole==='ABB' && m.groups.each==='B' && m.indices[2][0]===5 && m.indices[2][1]===6"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let r=/^\\b(?<each>{class}|){{001,}}?\\b$/div,m=r.exec('ſK');m[0]==='ſK' && m.groups.each==='K' && m.indices[1][0]===1 && m.indices[1][1]===2 && new RegExp(r,'dv').exec('ſK')===null"
        ));
    }
}

#[test]
fn mandatory_empty_iterations_keep_defined_captures_beside_zero_minimum_undefined_values() {
    for class in [r"[\q{a|b}]", r"[\q{ab}&&a]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<deep>|{class})){{001,}}?)/dvgy;r.lastIndex=1;let a=r.exec('😀'),b=/(?<whole>(?<each>(?<deep>|{class})){{0,}})/dv.exec('😀');a.index===0 && a[0]==='' && a[1]==='' && a[2]==='' && a[3]==='' && a.groups.each==='' && a.groups.deep==='' && Object.keys(a.groups).join('|')==='whole|each|deep' && a.indices[2][0]===0 && a.indices[3][1]===0 && r.lastIndex===0 && b[0]==='' && b[1]==='' && b[2]===undefined && b[3]===undefined && Object.hasOwn(b.groups,'each') && b.groups.each===undefined && b.indices.groups.deep===undefined"
        ));
    }
    check(
        r"let m=/(?<\u{1D49C}>|[\q{ab}&&a]){0001,}?/dv.exec('😀');Object.keys(m.groups).join('|')==='𝒜' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===0 && m.indices.groups['𝒜'][1]===0",
    );
    check(
        r"let a=[...'😀'.matchAll(/(?<each>|[\q{ab}&&a]){1,}?/dvg)];a.length===2 && a[0].index===0 && a[1].index===2 && a.every(m=>m.groups.each==='' && m.indices[1][0]===m.index && m.indices[1][1]===m.index)",
    );
    check(
        r"let r=/(?<each>|[\q{a|b}]){1,}?/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn one_open_final_iteration_ranges_keep_supplementary_folding_and_lone_surrogate_identity() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|)){{1,}})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x'),n=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.each==='😀' && m.indices[2][0]===3 && m.indices[3][1]===5 && n.index===5 && n[0]==='' && n.groups.each==='' && n.indices[2][0]===5 && n.indices[2][1]===5 && r.lastIndex===5"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let m=/(?<each>({class}|)){{1,}}/div.exec('𐐨𐐀');m[0]==='𐐨𐐀' && m[1]==='𐐀' && m[2]==='𐐀' && m.groups.each==='𐐀' && m.indices[1][0]===2 && m.indices[2][1]===4"
        ));
    }
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|)){1,}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.each.charCodeAt(0)===0xd800 && m.indices[1][0]===1 && m.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|){1,}/v.exec('𐀀').groups.each===''",
    );
}

#[test]
fn consumers_replacements_clones_and_collection_preserve_required_empty_capture_values() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){1,}/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a[0][0]==='ab' && a[0].groups.each==='b' && a[1].index===2 && a[1].groups.each==='' && a[1].indices[1][0]===2 && a[2][0]==='ba' && a[2].groups.each==='a' && a[3].index===5 && a[3].groups.each==='' && r.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<b:b><:>X<a:a><:>' && 'abXba'.split(/(?<each>[\q{a|b|ab}&&\p{Any}]|){1,}/v).join('|')==='|b|X|a|'",
    );
    check(
        r"let a=[],b=[];'😀'.replace(/(?<each>[\q{ab}&&a]|){1,}/vg,(m,each,at,input,groups)=>{a.push([each,at,groups.each]);return '#'});'😀'.replace(/(?<each>[\q{ab}&&a]|){0,}/vg,(m,each,at,input,groups)=>{b.push([each,at,groups.each]);return '#'});a.length===2 && b.length===2 && a[0][0]==='' && a[1][1]===2 && a[1][2]==='' && b[0][0]===undefined && b[1][1]===2 && b[1][2]===undefined",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){1,}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){1,}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|)){1,}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('zabb!').groups.each==='b' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn complete_one_open_proofs_finish_finalizers_and_never_hide_unknown_or_string_leaves() {
    for source in [
        r"/([\q{a|b}]|){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,3}/v.test('aaa')",
        r"/([\q{a|b}]|){2,4}/v.test('a')",
        r"/([\q{a|b}]|){2}/v.test('a')",
        r"/([\q{a|b}]|){2,2}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,}/v.test('aaa')",
        r"/([\q{a}]|){1,}/v.test('a')",
        r"/([\q{a|ab}&&a]|){1,}/v.test('aaa')",
        r"/([\q{a|b}]|){1}/v.test('a')",
        r"/([\q{a|b}]|){1,1}/v.test('a')",
        r"/(?<each>|[\q{ab}&&a]){1,}?/v.exec('X').groups.each===''",
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
    for source in [
        r"/(|[\q{ab}&&\q{cd}]){1,}/v.test('')",
        r"/(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){1,}?/v.test('')",
        r"/([\q{a|b}]|){2,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,99999999999999999999999999999999999999993}/v.test('aaa')",
        r"/([\q{a|b}]|){2,99999999999999999999999999999999999999994}/v.test('a')",
        r"/([\q{a|ab}&&a]|){1,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|ab}&&a])|){1,}/v.test('a')",
        r"/([\q{a|b}]|()){1,}/v.test('a')",
        r"/([\q{a|ab}&&a]||){1,}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|){1,}/v.test('a')",
        r"/\b^([\q{a|ab}&&a]|){1,}/v.test('a')",
        r"/([\q{a|ab}&&a]|){1,}[b]/v.test('ab')",
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
        r"([^\q{ab}]|){1,}",
        r"([\q{ab}&&a--a]|){1,}",
        r"([\q{\d}&&a]|){1,}?",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn one_open_wrappers_and_padded_counts_stay_flat_and_opt_in_aborts_preserve_state() {
    let pattern = format!(
        r"{}|[\q{{ab}}&&a]{}{{1,}}?",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('😀');m.length===1001 && m[0]==='' && m[1000]==='' && m.indices[1000][0]===0 && m.indices[1000][1]===0"
    ));
    let pattern = format!(r"(|[\q{{{}}}&&a]){{1,}}?", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0 && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|){{{}1,}}", "0".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');m[0]==='aaa' && m[1]==='a' && m.indices[1][0]===2 && m.indices[1][1]===3"
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
        .eval(r"var r=/([\q{a|ab}&&a]|){1,}/vg,marker=0")
        .unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "a".repeat(40_000)))
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
