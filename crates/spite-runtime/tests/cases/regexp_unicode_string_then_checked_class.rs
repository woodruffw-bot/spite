//! A singleton UnicodeSets string class before an ordinary checked class (22.2.2).

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
fn pairs_support_nested_algebra_empty_sets_and_original_character_boundaries() {
    check(
        r"let m=/[\q{a}][[b]]/dv.exec('aab!');m.index===1 && m[0]==='ab' && m.indices[0][0]===1 && m.indices[0][1]===3 && !/[\q{a}][[b]]/v.test('ax') && !/[\q{a}][[b]]/v.test('a')",
    );
    check(
        r"/[\q{a|c}--c][b&&b]/v.test('ab') && !/[\q{a|c}--c][b&&b]/v.test('cb') && !/[\q{a}--a][[b]]/v.test('ab') && !/[\q{a}][b--b]/v.test('ab') && /[^\q{a}][[b]]/v.test('!b') && !/[^\q{a}][[b]]/v.test('ab')",
    );
    check(
        r"let r=/[\q{😀}][[🚀]]/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1 && m[0]==='😀🚀' && m.indices[0][0]===1 && m.indices[0][1]===5 && r.lastIndex===5 && !/[\q{\uD800}][[\uDC00]]/v.test('😀')",
    );
    check(
        r"let m=/[\q{\uD800}][[\\]]/v.exec('A\uD800\\B');m.index===1 && m[0].length===2 && m[0].charCodeAt(0)===0xd800 && m[0].charCodeAt(1)===92",
    );
}

#[test]
fn outside_words_lines_and_case_folding_use_original_context() {
    check(
        r"let r=/[\q{a}][a&&a]\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null && b.index===1 && b[0]==='aa' && b.indices[0][1]===3 && r.lastIndex===3 && r.exec('aaa!')===null && r.lastIndex===0",
    );
    check(
        r"/[\q{s}][[K]]/iv.exec('!ſK!')[0]==='ſK' && !/[\q{s}][[K]]/v.test('ſK') && !/[\q{K}--k][[s]]/iv.test('Kſ')",
    );
    check(
        r"let m=/\B[^\q{K|s}][[\p{Any}]]\B/iv.exec('KſA$');m.index===2 && m[0]==='A$' && !/\B[^\q{K|s}][[\p{Any}]]\B/v.test('KſA$')",
    );
    check(
        r"let m=/^[\q{\]|\[|\\}][[\$]]/mv.exec('z\r\n]$tail\u2028');m.index===3 && m[0]===']$' && /^[\q{\]|\[|\\}][[\$]]$/v.test('[$') && /^[\q{\]|\[|\\}][[\$]]$/v.test('\\$')",
    );
    check(
        r"let m=/^[\q{a}][[b]]$/mv.exec('x\r\nab\u2028z');m.index===3 && m[0]==='ab' && !/^[\q{a}][[b]]$/v.test('x\r\nab\u2028z') && !/^[\q{a}][[b]]$/v.test('ab\n')",
    );
}

#[test]
fn whole_body_captures_keep_names_indices_and_intrinsic_array_attributes() {
    check(
        r"let r=/(?<whole>([\q{a}][[b]]))/dvg,m=r.exec('aab!');m.length===3 && m[0]==='ab' && m[1]==='ab' && m[2]==='ab' && m.index===1 && m.input==='aab!' && Object.getPrototypeOf(m)===Array.prototype && Object.getPrototypeOf(m.groups)===null && m.groups.whole==='ab' && m.indices.length===3 && m.indices[0][0]===1 && m.indices[0][1]===3 && m.indices[1][0]===1 && m.indices[2][1]===3 && m.indices.groups.whole===m.indices[1] && m.indices[0]!==m.indices[1] && m.indices[1]!==m.indices[2] && r.lastIndex===3 && r.exec('aab!')===null && r.lastIndex===0",
    );
    check(
        r"let r=/(?<whole>([\q{a}][[b]]))/dv,m=r.exec('ab');let d=Object.getOwnPropertyDescriptor(m,'1'),e=Object.getOwnPropertyDescriptor(m.indices,'1');d.writable && d.enumerable && d.configurable && e.writable && e.enumerable && e.configurable && r.source==='(?<whole>([\\q{a}][[b]]))' && r.flags==='dv'",
    );
}

#[test]
fn global_sticky_and_last_index_coercion_preserve_order_and_writes() {
    check(
        r"let r=/[\q{a}][[b]]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0 && b.index===2 && r.lastIndex===4 && r.exec('abab!')===null && r.lastIndex===0",
    );
    check(
        r"let trace='',r=/[\q{a}][[b]]/vy;r.lastIndex={valueOf(){trace+='i';return 1;}};let m=r.exec({toString(){trace+='s';return 'aab!';}});trace==='si' && m.index===1 && m[0]==='ab' && r.lastIndex===3",
    );
    check(
        r"let seen=0,r=/[\q{a}][[b]]/v;r.lastIndex={valueOf(){seen++;return 7;}};r.exec('aab!').index===1 && seen===1 && typeof r.lastIndex==='object'",
    );
    check(
        r"let a=/[\q{a}][[b]]/vy,b=/[\q{a}][[b]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!');}catch(e){x=e instanceof TypeError;}try{b.exec('aab!');}catch(e){y=e instanceof TypeError;}x && y && a.lastIndex===1 && b.lastIndex===0",
    );
}

#[test]
fn string_consumers_and_clones_use_the_same_pair_proof() {
    check(
        r"let r=/(?<whole>([\q{a}][[b]]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2 && a[0].index===1 && a[1].index===4 && a[1].groups.whole==='ab' && a[1].indices[2][1]===6 && 'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!' && 'aab-ab!'.split(/[\q{a}][[b]]/v).join('|')==='a|-|!'",
    );
    check(
        r"let r=/[\q{a}][[b]]/vg;r.lastIndex=4;'aab!'.search(r)===1 && r.lastIndex===4 && 'ab-ab'.match(r).join('|')==='ab|ab' && r.lastIndex===0",
    );
    check(
        r"let seen='',r=/(?<whole>([\q{a}][[b]]))/vg;let out='aab-ab!'.replace(r,(m,a,b,i,s,g)=>{seen+=i+':'+m+':'+a+':'+b+':'+g.whole+';';return '#';});out==='a#-#!' && seen==='1:ab:ab:ab:ab;4:ab:ab:ab:ab;' && r.lastIndex===0",
    );
    check(
        r"let r=/[\q{s}][[K]]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{s}][[K]]' && c.flags==='dvy' && c.lastIndex===0 && c.exec('ſK')===null && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
}

#[test]
fn earlier_unsupported_controls_now_complete_language_finalizers() {
    for (source, expected) in [
        (r"/[\q{a}][[b]]/v.test('ab')", true),
        (r"/[\q{a}][b&&b]/v.test('ab')", true),
        (r"/\b[\q{a}][[a--a]]/v.test('ab')", false),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
            )),
            Ok(Value::Boolean(expected))
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)));
    }
}

#[test]
fn unsupported_syntax_and_parser_early_errors_remain_distinct() {
    for source in [
        r"/([\q{a}])[[b]]/v.test('ab')",
        r"/[\q{a}]([[b]])/v.test('ab')",
        r"/(?:[\q{a}][[b]])+/v.test('ab')",
        r"/[\q{a}]+[[b]]/v.test('ab')",
        r"/[\q{a}][[b]]+/v.test('ab')",
        r"/[\q{a}][[b]][c]/v.test('abc')",
        r"/[\q{a}]\b[[b]]/v.test('ab')",
        r"/\b^[\q{a}][[b]]/v.test('ab')",
        r"/[\q{a}][[b]]$\b/v.test('ab')",
        r"/[\q{a|bc}][[b]]/v.test('ab')",
        r"/[\q{}][[b]]/v.test('ab')",
        r"/[\q{a}--a][[\p{RGI_Emoji}]]/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2;}}finally{{marker=3;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)));
    }
    for pattern in [
        r"[^\q{ab}][[b]]",
        r"[\q{\d}][[b]]",
        r"[\q{\u{110000}}][[b]]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn compiled_state_survives_collection_and_construction_aborts_keep_prior_values() {
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>([\q{a|c}--c][b&&b]))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"kept.exec('zab!').groups.whole==='ab' && kept.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r"try{{r=/[\q{{{choices}}}][[b]]/vg;}}catch{{marker=1;}}finally{{marker=2;}}");
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0 && r===7"), Ok(Value::Boolean(true)));
}
