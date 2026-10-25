//! Intersections and subtractions of q-only string operands (22.2.2.9).

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
fn string_operations_compare_complete_members_and_preserve_longest_priority() {
    check(
        r"let a=/[\q{a|ab|abc|}&&\q{ab|}]/dv.exec('ab!'),b=/[\q{a|ab|abc|}--\q{ab|}]/v.exec('abc!');a[0]==='ab' && a.indices[0][1]===2 && b[0]==='abc' && /[\q{a|ab|abc|}&&\q{ab|}]/v.exec('xab')[0]===''",
    );
    check(
        r"!/[\q{a|ab}&&\q{abc}]/v.test('abc') && !/[\q{ab}--\q{cd}]/v.test('cd') && /[\q{a|ab}--\q{a}]/v.exec('ab')[0]==='ab' && !/[\q{a|ab}--\q{a}]/v.test('a') && /[\q{a\|b|ab}&&\q{a\|b}]/v.exec('xa|b')[0]==='a|b'",
    );
    check(
        r"/[\q{a|ab|abc}&&\q{a|ab}&&\q{ab}]/v.exec('abc')[0]==='ab' && /[\q{a|ab|abc|abcd}--\q{a}--\q{abc}]/v.exec('abcd')[0]==='abcd' && /[\q{a|ab|abc|abcd}--\q{a}--\q{abc}]/v.exec('abc!')[0]==='ab'",
    );
}

#[test]
fn canonical_word_equality_precedes_operations_without_expansion_or_normalization() {
    check(
        r"let a=/[\q{K|sK|}&&\q{k|ſK|}]/iv.exec('ſK'),b=/[\q{K|sK|}&&\q{k|ſK|}]/iv.exec('K');a[0]==='ſK' && b[0]==='K' && /[\q{sK|K|}--\q{ſK}]/iv.exec('ſK')[0]==='' && /[\q{sK|K|}--\q{ſK}]/iv.exec('K')[0]==='K'",
    );
    check(
        r"/[\q{ßa|sa}&&\q{ẞA}]/iv.exec('ẞA')[0]==='ẞA' && !/[\q{ßa}&&\q{SSa}]/iv.test('SSa') && !/[\q{é}&&\q{é}]/iv.test('é') && !/[\q{é}&&\q{é}]/iv.test('é') && /[\q{\u{10400}a|x}&&\q{\u{10428}A|X}]/iv.exec('𐐨A')[0]==='𐐨A'",
    );
    check(
        r"let r=/(?<whole>(^[\q{K|sK|}&&\q{k|ſK|}]\b$))/dimv,m=r.exec('!\r\nſK\u2028!');m.index===3 && m[0]==='ſK' && m[1]==='ſK' && m[2]==='ſK' && m.groups.whole==='ſK' && m.indices[0][0]===3 && m.indices[2][1]===5 && m.indices.groups.whole===m.indices[1]",
    );
    check(r"let a=/[\q{a!|a}&&\q{a!|a}]\b/dv.exec('a!');a[0]==='a' && a.indices[0][1]===1");
}

#[test]
fn surviving_empty_members_and_empty_sets_keep_distinct_state_and_surrogate_ranges() {
    check(
        r"let r=/(?<whole>([\q{😀x|🚀|}&&\q{😀x|}]))/dvgy;r.lastIndex=2;let a=r.exec('A😀x'),b=r.exec('A😀x');a.index===1 && a[0]==='😀x' && a.groups.whole==='😀x' && a.indices[2][1]===4 && b.index===4 && b[0]==='' && b.groups.whole==='' && r.lastIndex===4",
    );
    check(
        r"let r=/(?<whole>([\q{|ab}--\q{ab}]))/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m.groups.whole==='' && m.indices[2][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    check(
        r"let r=/[\q{ab}&&\q{cd}]/vg;r.lastIndex=1;let m=r.exec('abcd');m===null && r.lastIndex===0 && !r.test('') && /[\q{|ab}--\q{ab}]/v.exec('')[0]===''",
    );
    check(
        r"let r=/[\q{ab}&&\q{cd}]/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('ab');}catch(e){caught=e instanceof TypeError;}caught && r.lastIndex===1",
    );
    check(
        r"!/[\q{\u{D800}\u{DC00}|ab}&&\q{\uD800\uDC00|ab}]/v.test('𐀀') && /[\q{\u{D800}\u{DC00}|ab}&&\q{\uD800\uDC00|ab}]/v.exec('ab')[0]==='ab'",
    );
}

#[test]
fn consumers_clones_and_collection_use_the_filtered_member_plan() {
    check(
        r"let r=/(?<whole>([\q{|ab}&&\q{|ab}]))/dvg,a=[...'ab😀'.matchAll(r)];a.length===3 && a[0][0]==='ab' && a[1].index===2 && a[1][0]==='' && a[2].index===4 && a[2].indices[2][1]===4 && 'ab😀'.replace(r,'#')==='##😀#' && 'ab😀'.split(/[\q{|ab}&&\q{|ab}]/v).join('|')==='|😀'",
    );
    check(
        r"let r=/[\q{a|ab|abc}--\q{a}]/vg;r.lastIndex=8;'xabc!'.search(r)===1 && r.lastIndex===8 && 'ababc'.match(r).join('|')==='ab|abc' && r.lastIndex===0 && 'ababc'.replace(r,'#')==='##'",
    );
    check(
        r"let r=/[\q{K|sK}&&\q{k|ſK}]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.exec('ſK')===null && c.lastIndex===0 && r.exec('ſK')[0]==='ſK' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>([\q{a|ab|abc}--\q{a}]))/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabc').groups.whole==='abc' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn former_controls_complete_finalizers_and_non_q_algebra_and_early_errors_stay_distinct() {
    for (source, expected) in [
        (r"/\b^[\q{ab}&&\q{ab}]/v.test('ab')", true),
        (r"/[\q{ab}&&\p{Any}]/v.test('ab')", false),
        (r"/[[\q{ab}]&&[\q{ab}]]/v.test('ab')", true),
        (r"/[\q{ab}&&\q{ab}]/v.test('ab')", true),
        (r"/[\q{ab}&&\q{cd}]/v.test('ab')", false),
        (r"/[\q{ab}--\q{cd}]/v.test('ab')", true),
        (r"/[\q{a|ab}&&\q{a}]/v.test('a')", true),
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
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}--\\q{ab}]+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",null,0],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",null,0],[true,\"ab!\",null,0],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/[\q{ab}&&\q{cd}&&\p{RGI_Emoji}]/v.test('')",
        r"/[[\q{ab}--\q{ab}]&&\p{RGI_Emoji}]+/v.test('ab')",
        r"/[\q{ab}--\q{ab}][a]/v.test('aba')",
        r"/\b^[\q{ab}&&\q{ab}&&\p{RGI_Emoji}]/v.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    for pattern in [
        r"[^\q{ab}--\q{a}]",
        r"[\q{ab}&&\q{ab}--\q{ab}]",
        r"[\q{ab}&&\q{\d}]",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
