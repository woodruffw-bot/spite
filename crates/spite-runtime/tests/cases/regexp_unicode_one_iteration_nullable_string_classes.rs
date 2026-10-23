//! Exactly-one braced nullable string classes (22.2.2.3 RepeatMatcher).

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
fn exactly_one_mandatory_iteration_keeps_source_branch_priority_independent_of_greediness() {
    for class in [r"[\q{a|b}]", r"[\q{a|ab}&&a]"] {
        for q in ["{1}", "{1}?", "{1,1}", "{001,01}?"] {
            check(&format!(
                "let a=/(?<each>{class}|){q}/dv.exec('abbb!'),b=/(?<each>|{class}){q}/dv.exec('abbb!');a[0]==='a' && a.groups.each==='a' && a.indices[1][0]===0 && a.indices[1][1]===1 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && b.indices[1][1]===0"
            ));
        }
        check(&format!(
            "let a=/(?<each>|{class}){{1}}/v.exec('aaa'),b=/(?<each>|{class}){{1,}}/v.exec('aaa');a[0]==='' && a.groups.each==='' && b[0]==='aaa' && b.groups.each==='a'"
        ));
    }
}

#[test]
fn outside_word_line_gates_backtrack_mandatory_choices_without_expanding_the_count() {
    for class in [r"[\q{a|b}]", r"[\q{a|ab}&&a]"] {
        check(&format!(
            "let r=/(?<whole>^\\b(?<each>|{class}){{01,001}}?\\b$)/dimv,m=r.exec('!\\r\\nA\\u2028!');m.index===3 && m.groups.whole==='A' && m.groups.each==='A' && m.indices[2][0]===3 && m.indices[2][1]===4 && r.exec('!\\r\\nAA\\u2028!')===null"
        ));
    }
    check(
        r"let r=/\B(?<each>[\q{a|ab}&&a]|){1}\b/dvy;r.lastIndex=1;let a=r.exec('aaa!');r.lastIndex=2;let b=r.exec('aaa!');a===null && b.index===2 && b[0]==='a' && b.groups.each==='a' && b.indices[1][0]===2 && r.lastIndex===3",
    );
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let r=/^\\b(?<each>|{class}){{0001,01}}?\\b$/div,m=r.exec('ſ');m[0]==='ſ' && m.groups.each==='ſ' && m.indices[1][0]===0 && m.indices[1][1]===1 && r.exec('ſK')===null && new RegExp(r,'dv').exec('ſ')===null"
        ));
    }
}

#[test]
fn mandatory_empty_and_single_character_captures_keep_original_utf16_ranges_and_names() {
    check(
        r"let r=/(?<whole>(?<each>(?<deep>|[\q{ab}&&a])){01,001}?)/dvgy;r.lastIndex=1;let a=r.exec('😀'),b=/(?<whole>(?<each>(?<deep>|[\q{ab}&&a])){0})/dv.exec('😀');a.index===0 && a[0]==='' && a[1]==='' && a[2]==='' && a[3]==='' && a.groups.each==='' && a.groups.deep==='' && Object.keys(a.groups).join('|')==='whole|each|deep' && a.indices[2][0]===0 && a.indices[3][1]===0 && r.lastIndex===0 && b[2]===undefined && b[3]===undefined && Object.hasOwn(b.groups,'each') && b.indices.groups.deep===undefined",
    );
    check(
        r"let m=/(?<\u{1D49C}>|[\q{ab}&&a]){1}/dv.exec('😀');Object.keys(m.groups).join('|')==='𝒜' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===0 && m.indices.groups['𝒜'][1]===0",
    );
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|)){{1}})/dvgy;r.lastIndex=2;let a=r.exec('A😀😀x'),b=r.exec('A😀😀x'),c=r.exec('A😀😀x');a.index===1 && a[0]==='😀' && a.groups.each==='😀' && a.indices[2][0]===1 && a.indices[3][1]===3 && b.index===3 && b[0]==='😀' && b.indices[2][0]===3 && b.indices[2][1]===5 && c.index===5 && c[0]==='' && c.groups.each==='' && c.indices[2][0]===5 && r.lastIndex===5"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<each>({class}|)){{1,1}}?/divgy;r.lastIndex=1;let a=r.exec('𐐀𐐨'),b=r.exec('𐐀𐐨');a.index===0 && a[0]==='𐐀' && a.groups.each==='𐐀' && a.indices[1][0]===0 && a.indices[2][1]===2 && b.index===2 && b.groups.each==='𐐨' && b.indices[1][0]===2 && b.indices[2][1]===4 && r.lastIndex===4"
        ));
    }
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|)){1}/dv.exec('\uD800\uD800a');m[0].length===1 && m.groups.each.charCodeAt(0)===0xd800 && m.indices[1][0]===0 && m.indices[2][1]===1 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|){1}/v.exec('𐀀').groups.each===''",
    );
    check(
        r"let r=/(?<each>|[\q{a|b}]){1}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn consumers_replacements_clones_and_collection_retain_exactly_one_branch_results() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=[...'ab'.matchAll(/(?<each>{class}|){{1}}/dvg)],b=[...'ab'.matchAll(/(?<each>|{class}){{1}}/dvg)];a.length===3 && a.map(m=>m[0]).join('|')==='a|b|' && a[0].groups.each==='a' && a[1].groups.each==='b' && a[2].groups.each==='' && b.length===3 && b.every(m=>m[0]==='' && m.groups.each==='') && b.map(m=>m.index).join(',')==='0,1,2' && 'ab'.replace(/(?<each>{class}|){{1}}/vg,'<$1:$<each>>')==='<a:a><b:b><:>' && 'ab'.replace(/(?<each>|{class}){{1}}/vg,'<$1:$<each>>')==='<:>a<:>b<:>' && 'ab'.split(/(?<each>{class}|){{1}}/v).join('|')==='|a||b|' && 'ab'.split(/(?<each>|{class}){{1}}/v).join('|')==='a||b'"
        ));
    }
    check(
        r"let a='😀a'.split(/(?<each>|[\q{a|b}]){1}/v),b='😀a'.split(/(?<each>|[\q{a|b}]){0}/v);a.length===3 && a[0]==='😀' && a[1]==='' && a[2]==='a' && b.length===3 && b[1]===undefined",
    );
    check(
        r"let seen=[];'😀'.replace(/(?<each>|[\q{ab}&&a]){1}/vg,(m,each,at,input,groups)=>{seen.push([each,at,groups.each]);return '#'});seen.length===2 && seen[0][0]==='' && seen[1][1]===2 && seen[1][2]===''",
    );
    check(
        r"let r=/(?<each>|[\q{a|b|ab}&&\p{Any}]){1}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'ab'.match(r).join('|')==='||' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|){1}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='A' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|)){1}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"r.exec('zabb!').groups.each==='a' && r.lastIndex===2 && r.exec('zabb!').groups.each==='b' && r.lastIndex===3"), Ok(Value::Boolean(true)));
}

#[test]
fn complete_exactly_one_proofs_finish_finalizers_and_do_not_hide_deferred_leaves() {
    for source in [
        r"/([\q{a|b}]|){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,3}/v.test('aaa')",
        r"/([\q{a|b}]|){1,2}/v.test('a')",
        r"/([\q{a|b}]|){2}/v.test('a')",
        r"/([\q{a|ab}&&a]|){2,}/v.test('aaa')",
        r"/([\q{a}]|){1}/v.test('')",
        r"/([\q{a|ab}&&a]|){1}/v.test('')",
        r"/([\q{a|b}]|){1}/v.test('a')",
        r"/([\q{a|b}]|){1,1}/v.test('a')",
        r"/(?<each>|[\q{ab}&&a]){1}/v.exec('X').groups.each===''",
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
        r"/(|[\q{ab}&&\q{cd}]){1}/v.test('')",
        r"/(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){1}/v.test('')",
        r"/(([\q{a|b}])|(?:)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('aaa')",
        r"/(([\q{a|b}])|(?:)?){888888888888888888888888888888,99999999999999999999999999999999999999992}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:)?){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:)?){1}/v.test('a')",
        r"/([\q{a|b}]|()?){1}/v.test('a')",
        r"/([\q{a|ab}&&a]||){1}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|){1}/v.test('a')",
        r"/\b^([\q{a|ab}&&a]|){1}/v.test('a')",
        r"/([\q{a|ab}&&a]|){1}[b]/v.test('ab')",
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
        r"([^\q{ab}]|){1}",
        r"([\q{ab}&&a--a]|){1,1}",
        r"([\q{\d}&&a]|){1}?",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn exact_one_wrappers_and_padded_counts_stay_flat_and_opt_in_aborts_keep_state_atomic() {
    let pattern = format!(
        r"{}|[\q{{a|ab}}&&a]{}{{1}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');m.length===1001 && m[0]==='' && m[1000]==='' && m.indices[1000][0]===0 && m.indices[1000][1]===0"
    ));
    let pattern = format!(r"(|[\q{{{}}}&&a]){{1}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0 && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|){{{}1}}", "0".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='a' && m[1]==='a' && m.indices[1][0]===0 && m.indices[1][1]===1"
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
        .eval(r"var r=/([\q{a|ab}&&a]|){1}$/vg,marker=0")
        .unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "x".repeat(40_000)))
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
