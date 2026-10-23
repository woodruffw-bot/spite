//! Represented positive finite grouped-empty nullable string-class ranges (22.2.2.3).

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
fn finite_ranges_keep_mandatory_priority_exact_optional_budgets_and_lazy_stopping() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{2,3}}/dv.exec('abbb'),b=/(?<each>(?:(?:))|{class}){{02,003}}/dv.exec('abbb'),c=/(?<each>{class}|(?:(?:))){{2,3}}?/dv.exec('abbb'),d=/(?<each>(?:(?:))|{class}){{2,3}}?/dv.exec('abbb');a[0]==='abb' && a.groups.each==='b' && a.indices[1][0]===2 && b[0]==='a' && b.groups.each==='a' && b.indices[1][0]===0 && c[0]==='ab' && c.groups.each==='b' && c.indices[1][0]===1 && d[0]==='' && d.groups.each==='' && d.indices[1][0]===0"
        ));
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{1,2}}/dv.exec('abbb'),b=/(?<each>(?:(?:))|{class}){{1,2}}/dv.exec('abbb'),c=/(?<each>{class}|(?:(?:))){{1,2}}?/dv.exec('abbb');a[0]==='ab' && a.groups.each==='b' && b[0]==='a' && b.groups.each==='a' && c[0]==='a' && c.groups.each==='a'"
        ));
        check(&format!(
            "let a=/(?<each>(?:(?:))|{class}){{2,4}}/dv.exec('abbb'),b=/(?<each>(?:(?:))|{class}){{3,5}}/dv.exec('abbb');a[0]==='ab' && a.groups.each==='b' && b[0]==='ab' && b.groups.each==='b' && a.indices[1][0]===1 && b.indices[1][0]===1"
        ));
        check(&format!(
            "let a=/(?<whole>(?<each>(?<deep>{class}|(?:(?:)))){{3,4}})/dv.exec('abX'),b=/(?<each>(?:(?:))|{class}){{3,4}}/dv.exec('abX');a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===2 && a.indices[3][1]===2 && b[0]==='a' && b.groups.each==='a'"
        ));
    }
}

#[test]
fn phase_comparison_preserves_earliest_candidates_and_outside_word_line_gates() {
    check(
        r"let r=/(?<whole>(?<each>(?:(?:))|[\q{a|ab}&&a]){2,3}$)/dvgy,a=r.exec('aa');a.index===0 && a[0]==='aa' && a.groups.whole==='aa' && a.groups.each==='a' && a.indices[2][0]===1 && a.indices[2][1]===2 && r.lastIndex===2",
    );
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>(?:(?:))|{class}){{2,3}}\\b$)/dimv.exec('!\\r\\nABB\\u2028!'),b=/(?<whole>^\\b(?<each>{class}|(?:(?:))){{3,4}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='ABB' && a.groups.each==='B' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='' && b.indices[2][0]===5 && b.indices[2][1]===5"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|(?:(?:))){{3,4}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>(?:(?:))|{class}){{2,3}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1 && b.indices[1][1]===2"
        ));
    }
}

#[test]
fn finite_capture_ranges_keep_decoded_unicode_counts_and_largest_optional_budgets() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|(?:(?:)))){{2,3}})/dvgy,s=/(?<each>((?:(?:))|{class})){{2,3}}/dvgy;r.lastIndex=2;s.lastIndex=2;let a=r.exec('A😀x'),b=s.exec('A😀x'),c=/(?<each>(?:(?:))|{class}){{2,3}}$/dv.exec('😀😀');a.index===1 && a[0]==='😀' && a.groups.each==='' && a.indices[2][0]===3 && a.indices[3][1]===3 && b.index===1 && b[0]==='😀' && b.groups.each==='😀' && b.indices[1][0]===1 && b.indices[2][1]===3 && r.lastIndex===3 && s.lastIndex===3 && c.index===0 && c[0]==='😀😀' && c.indices[1][0]===2 && c.indices[1][1]===4"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{3,4}}?/div.exec('𐐀𐐨'),b=/(?<each>(?:(?:))|{class}){{3,4}}/div.exec('𐐀𐐨');a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && b[0]==='𐐀' && b.groups.each==='𐐀' && b.indices[1][0]===0 && b.indices[1][1]===2"
        ));
    }
    let lower = usize::MAX - 1;
    let upper = usize::MAX;
    check(&format!(
        r"let a=/(?<each>(?:(?:))|[\q{{a|b}}]){{{lower},{upper}}}/dv.exec('aaaa'),b=/(?<each>[\q{{a|b}}]|(?:(?:))){{{lower},{upper}}}/dv.exec('aaaa'),c=/(?<each>(?:(?:))|[\q{{a|b}}]){{{lower},{upper}}}$/dv.exec('aa');a[0]==='a' && a.groups.each==='a' && a.indices[1][0]===0 && b[0]==='aaaa' && b.groups.each==='' && b.indices[1][0]===4 && c.index===0 && c[0]==='aa' && c.groups.each==='a' && c.indices[1][0]===1"
    ));
    check(
        r"let m=/(?<whole>(?<\u{1D49C}>[\q{a|ab}&&a]|(?:(?:))){3,4})/dv.exec('aaX');m.groups.whole==='aa' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===2 && m.indices.groups['𝒜'][1]===2",
    );
    check(
        r"let r=/(?<each>(?:(?:))|[\q{a|b}]){2,3}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn finite_consumers_replacements_clones_and_collection_preserve_selected_phase_values() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){3,4}/dvg,s=/(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}]){2,3}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && a[2].indices[1][0]===5 && b.length===6 && b.map(m=>m[0]).join('|')==='a|b||b|a|' && b[0].groups.each==='a' && b[1].groups.each==='b' && b[2].groups.each==='' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1>')==='<><>X<><>' && 'abXba'.replace(s,'<$1>')==='<a><b><>X<b><a><>' && 'ab'.split(/(?<each>(?:(?:))|[\q{a|b}]){2,3}/v).join('|')==='|a||b|'",
    );
    check(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){3,4}/vg,(m,each,at,input,groups)=>{seen.push([m,each,at,groups.each]);return '#'});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][3]==='' && seen[1][2]===2 && seen[2][2]===3",
    );
    check(
        r"let r=/(?<each>(?:(?:))|[\q{a|b}]){2,3}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'ab'.match(r).join('|')==='a|b|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){2,3}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|(?:(?:)))){3,4}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.indices[1][0]===3 && m.indices[2][1]===3 && r.lastIndex===3"), Ok(Value::Boolean(true)));
}

#[test]
fn complete_grouped_positive_finite_proofs_finish_finalizers_and_keep_overflowing_bounds_deferred()
{
    for source in [
        r"/([\q{a|b}]|(?:)){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){1,2}/v.test('a')",
        r"/([\q{a|b}]|(?:)){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:(?:))){1,2}?/v.test('aaa')",
        r"/(?<each>(?:(?:))|[\q{ab}&&a]){03,004}/v.exec('X').groups.each===''",
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
        r"/((?:)|[\q{ab}&&\q{cd}]){2,3}/v.test('')",
        r"/((?:)|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){3,4}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:)){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,3}/v.test('a')",
        r"/([\q{a|b}]|()){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]||(?:)){2,3}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|(?:)){2,3}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){2,3}[b]/v.test('ab')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999998,999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
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
        r"([^\q{ab}]|(?:)){2,3}",
        r"([\q{ab}&&a--a]|(?:)){3,4}",
        r"([\q{\d}&&a]|(?:)){2,3}?",
        r"([\q{a|b}]|(?:)){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn grouped_finite_wrappers_padding_and_opt_in_aborts_preserve_flat_atomic_state() {
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]|(?:(?:)){}{{3,4}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aa');m.length===1001 && m[0]==='aa' && m[1000]==='' && m.indices[1000][0]===2 && m.indices[1000][1]===2"
    ));
    let pattern = format!(r"((?:(?:))|[\q{{{}}}&&a]){{2,3}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0 && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|(?:(?:))){{{}2,3}}", "0".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaaa');r.source.length>20000 && m[0]==='aaa' && m[1]==='a' && m.indices[1][0]===2 && m.indices[1][1]===3"
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
        .eval(r"var r=/([\q{a|ab}&&a]|(?:(?:))){2,3}$/vg,marker=0")
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
