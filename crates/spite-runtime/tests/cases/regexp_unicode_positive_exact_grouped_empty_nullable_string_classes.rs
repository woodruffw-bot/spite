//! Representable positive exact nullable string classes (22.2.2.3).

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
fn exact_counts_keep_mandatory_source_priority_and_final_empty_captures() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        for (q, whole, capture_start) in [
            ("{2}", "ab", 1),
            ("{2}?", "ab", 1),
            ("{02,0002}", "ab", 1),
            ("{3,3}?", "abb", 2),
        ] {
            check(&format!(
                "let a=/(?<each>{class}|(?:(?:))){q}/dv.exec('abbbX'),b=/(?<each>(?:(?:))|{class}){q}/dv.exec('abbbX');a[0]==='{whole}' && a.groups.each==='b' && a.indices[1][0]==={capture_start} && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && b.indices[1][1]===0"
            ));
        }
        check(&format!(
            "let a=/(?<whole>(?<each>(?<deep>{class}|(?:(?:)))){{3}})/dv.exec('abX'),b=/(?<each>(?:(?:))|{class}){{3}}/dv.exec('abX');a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===2 && a.indices[3][1]===2 && b[0]==='' && b.groups.each===''"
        ));
        check(&format!(
            "let a=/(?<whole>^\\b(?<each>{class}|(?:(?:))){{3}}\\b$)/dimv.exec('!\\r\\nAB\\u2028!'),b=/(?<whole>^\\b(?<each>(?:(?:))|{class}){{03,003}}?\\b$)/dimv.exec('!\\r\\nAB\\u2028!');a.index===3 && a.groups.whole==='AB' && a.groups.each==='' && a.indices[2][0]===5 && b.index===3 && b.groups.whole==='AB' && b.groups.each==='B' && b.indices[2][0]===4 && b.indices[2][1]===5"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|(?:(?:))){{03}}?\\b$/div.exec('ſK'),b=/^\\b(?<each>(?:(?:))|{class}){{3}}\\b$/div.exec('ſK');a[0]==='ſK' && a.groups.each==='' && a.indices[1][0]===2 && a.indices[1][1]===2 && b[0]==='ſK' && b.groups.each==='K' && b.indices[1][0]===1 && b.indices[1][1]===2"
        ));
    }
}

#[test]
fn largest_represented_exact_bounds_preserve_priority_source_and_required_empty_values() {
    let maximum = usize::MAX;
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        for q in [
            format!("{{{maximum}}}"),
            format!("{{000{maximum},0{maximum}}}?"),
        ] {
            check(&format!(
                "let r=/(?<each>{class}|(?:(?:))){q}/dv,a=r.exec('abX'),b=/(?<each>(?:(?:))|{class}){q}/dv.exec('abX'),c=/(?<each>(?:(?:))|{class}){q}$/dv.exec('ab');r.source.includes('{maximum}') && a[0]==='ab' && a.groups.each==='' && a.indices[1][0]===2 && a.indices[1][1]===2 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && c[0]==='ab' && c.groups.each==='b' && c.indices[1][0]===1 && c.indices[1][1]===2"
            ));
        }
    }
    check(&format!(
        "let r=/(?<each>(?:(?:))|[\\q{{ab}}&&a]){{{maximum}}}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('😀')}}catch(e){{caught=e instanceof TypeError}}caught && r.lastIndex===1"
    ));
}

#[test]
fn exact_decoded_counts_keep_supplementary_trimming_and_original_capture_ranges() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>({class}|(?:(?:)))){{2}}?)/dvgy;r.lastIndex=2;let a=r.exec('A😀😀😀x'),b=r.exec('A😀😀😀x');a.index===1 && a[0]==='😀😀' && a.groups.each==='😀' && a.indices[2][0]===3 && a.indices[3][1]===5 && b.index===5 && b[0]==='😀' && b.groups.each==='' && b.indices[2][0]===7 && b.indices[3][1]===7 && r.lastIndex===7"
        ));
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{3}}?$/dv.exec('😀😀😀😀'),b=/(?<each>(?:(?:))|{class}){{3}}$/dv.exec('😀😀😀😀');a.index===2 && a[0]==='😀😀😀' && a.groups.each==='😀' && a.indices[1][0]===6 && a.indices[1][1]===8 && b.index===2 && b[0]==='😀😀😀' && b.groups.each==='😀' && b.indices[1][0]===6"
        ));
        check(&format!(
            "let a=/(?<each>{class}|(?:(?:))){{3}}$/dv.exec('😀'),b=/(?<each>(?:(?:))|{class}){{3}}$/dv.exec('😀');a[0]==='😀' && a.groups.each==='' && a.indices[1][0]===2 && a.indices[1][1]===2 && b[0]==='😀' && b.groups.each==='😀' && b.indices[1][0]===0 && b.indices[1][1]===2"
        ));
    }
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>({class}|(?:(?:)))){{3,03}}/div.exec('𐐀𐐨'),b=/(?<each>((?:(?:))|{class})){{3}}?$/div.exec('𐐀𐐨');a[0]==='𐐀𐐨' && a.groups.each==='' && a.indices[1][0]===4 && a.indices[2][1]===4 && b[0]==='𐐀𐐨' && b.groups.each==='𐐨' && b.indices[1][0]===2 && b.indices[2][1]===4"
        ));
    }
    check(
        r"let m=/(?<whole>(?<\u{1D49C}>[\q{a|ab}&&a]|(?:(?:))){3})/dv.exec('aaX');m.groups.whole==='aa' && m.groups['𝒜']==='' && m.indices.groups['𝒜'][0]===2 && m.indices.groups['𝒜'][1]===2",
    );
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|(?:(?:)))){3}/dv.exec('\uD800\uD800a');m[0].length===2 && m[0].charCodeAt(0)===0xd800 && m.groups.each==='' && m.indices[1][0]===2 && m.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|(?:(?:))){3}/v.exec('𐀀').groups.each===''",
    );
}

#[test]
fn consumers_replacements_clones_and_collection_preserve_exact_count_results() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){3}/dvg,s=/(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}]){3}/dvg,a=[...'abXba'.matchAll(r)],b=[...'abXba'.matchAll(s)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.every(m=>m.groups.each==='') && a[0].indices[1][0]===2 && a[2].indices[1][0]===5 && b.length===6 && b.every(m=>m[0]==='' && m.groups.each==='') && b.map(m=>m.index).join(',')==='0,1,2,3,4,5' && r.lastIndex===0 && s.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<:><:>X<:><:>' && 'abXba'.replace(s,'<$1:$<each>>')==='<:>a<:>b<:>X<:>b<:>a<:>' && 'abXba'.split(/(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}]){3}/v).join('|')==='a||b||X||b||a'",
    );
    check(
        r"let seen=[],result='abX'.replace(/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){3}/vg,(m,each,at,input,groups)=>{seen.push([m,each,at,groups.each]);return '#'});result==='##X#' && seen.length===3 && seen[0][0]==='ab' && seen[0][1]==='' && seen[0][2]===0 && seen[0][3]==='' && seen[1][0]==='' && seen[1][2]===2 && seen[2][2]===3",
    );
    check(
        r"let r=/(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}]){3}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='|||||' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){3}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|(?:(?:)))){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let a=r.exec('zabbb!'),b=r.exec('zabbb!');a[0]==='abb' && a.groups.each==='b' && a.indices[1][0]===3 && b[0]==='b' && b.groups.each==='' && b.indices[1][0]===5 && b.indices[2][1]===5 && r.lastIndex===5"), Ok(Value::Boolean(true)));
}

#[test]
fn positive_exact_grouped_proofs_finish_finalizers_and_keep_other_domains_deferred() {
    for source in [
        r"/([\q{a|b}]|(?:)){999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|(?:)){2}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:(?:))){03,003}?/v.test('aaa')",
        r"/(?<each>(?:(?:))|[\q{ab}&&a]){0002}?/v.exec('X').groups.each===''",
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
        r"/((?:)|[\q{ab}&&\q{cd}]){2}/v.test('')",
        r"/((?:)|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){3,3}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:X|)?){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2}/v.test('a')",
        r"/([\q{a|b}]|()?){2}/v.test('a')",
        r"/([\q{a|ab}&&a]||(?:)){2}/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|(?:)){2}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){2}[b]/v.test('ab')",
        r"/([\q{a|ab}&&a]|(?:|())){99999999999999999999999999999999999999991,99999999999999999999999999999999999999992}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999999,9999999999999999999999999999999999999990}/v.test('a')",
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
        r"([^\q{ab}]|(?:)){2}",
        r"([\q{ab}&&a--a]|(?:)){3,3}",
        r"([\q{\d}&&a]|(?:)){2}?",
        r"([\q{a|b}]|(?:)){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn grouped_exact_count_wrappers_padding_and_opt_in_aborts_preserve_flat_atomic_state() {
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]|(?:(?:)){}{{3}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aa');m.length===1001 && m[0]==='aa' && m[1000]==='' && m.indices[1000][0]===2 && m.indices[1000][1]===2"
    ));
    let pattern = format!(r"((?:(?:))|[\q{{{}}}&&a]){{3}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]==='' && m.indices[1][0]===0 && m.indices[1][1]===0 && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|(?:(?:))){{{}2}}", "0".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='aa' && m[1]==='a' && m.indices[1][0]===1 && m.indices[1][1]===2"
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
        .eval(r"var r=/([\q{a|ab}&&a]|(?:(?:))){3}$/vg,marker=0")
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
