//! Nested noncapturing empty alternatives beside UnicodeSets string classes (22.2.2.3).

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
fn grouped_empty_branch_order_greediness_and_progress_keep_the_selected_iteration() {
    check(
        r"let a=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:)))+/dv.exec('abbb!'),b=/(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}])+/dv.exec('abbb!');a[0]==='abbb' && b[0]==='abbb' && a.groups.each==='b' && b.groups.each==='b' && a.indices[1][0]===3 && b.indices[1][0]===3",
    );
    check(
        r"let a=/(?<each>[\q{a|ab}&&a]|(?:(?:)))+?/dv.exec('aaa'),b=/(?<each>(?:(?:))|[\q{a|ab}&&a])+?/dv.exec('aaa');a[0]==='a' && a.groups.each==='a' && a.indices[1][1]===1 && b[0]==='' && b.groups.each==='' && b.indices[1][0]===0 && b.indices[1][1]===0",
    );
    check(
        r"let a=/(?<each>(?:(?:))|[\q{a|ab}&&a])?/dv.exec('aaa'),b=/(?<each>(?:(?:))|[\q{a|ab}&&a])??/dv.exec('aaa');a[0]==='a' && a.groups.each==='a' && b[0]==='' && Object.hasOwn(b.groups,'each') && b.groups.each===undefined && b.indices[1]===undefined",
    );
    check(
        r"let m=/(?<whole>(?<each>(?:(?:))|[\q{a|b|ab}&&\p{Any}])+?\b$)/dimv.exec('!\r\nABB\u2028!');m.index===3 && m.groups.whole==='ABB' && m.groups.each==='B' && m.indices[2][0]===5 && m.indices[2][1]===6",
    );
}

#[test]
fn grouped_required_empty_iterations_define_empty_captures_while_zero_iterations_leave_undefined() {
    check(
        r"let a=/(?<whole>(?<each>(?<deep>[\q{ab|}&&a]|(?:(?:))))+)/dv.exec('😀'),b=/(?<whole>(?<each>(?<deep>[\q{ab|}&&a]|(?:(?:))))*)/dv.exec('😀');a[0]==='' && a[1]==='' && a[2]==='' && a[3]==='' && a.groups.each==='' && a.groups.deep==='' && a.indices[2][0]===0 && a.indices[3][1]===0 && b[0]==='' && b[1]==='' && b[2]===undefined && b[3]===undefined && Object.hasOwn(b.groups,'each') && b.groups.each===undefined && b.indices[2]===undefined && b.indices.groups.deep===undefined",
    );
    check(
        r"let r=/(?<each>((?:(?:))|[\q{ab}&&a]))+?/dvgy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m[2]==='' && m.groups.each==='' && m.indices[1][0]===0 && m.indices[2][1]===0 && r.lastIndex===0",
    );
    check(
        r"/(?<each>[\q{ab|}&&a])+/v.exec('ab')===null && /(?<each>[\q{ab|}&&a]|(?:(?:)))+/v.exec('ab').groups.each==='' && /(?<each>[\q{ab|}&&a]|(?:(?:)))*/v.exec('ab').groups.each===undefined",
    );
}

#[test]
fn grouped_nullable_capture_ranges_keep_original_supplementary_and_lone_surrogate_boundaries() {
    check(
        r"let r=/(?<whole>(?<each>([\q{😀|😀x}&&\p{Any}]|(?:(?:))))+)/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x'),n=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.each==='😀' && m.indices[2][0]===3 && m.indices[3][1]===5 && n.index===5 && n[0]==='' && n.groups.each==='' && n.indices[2][0]===5 && n.indices[2][1]===5 && r.lastIndex===5",
    );
    check(
        r"let m=/(?<each>([\q{\u{10428}|ab}&&\p{Any}]|(?:(?:))))+/div.exec('𐐨𐐀');m[0]==='𐐨𐐀' && m[1]==='𐐀' && m[2]==='𐐀' && m.groups.each==='𐐀' && m.indices[1][0]===2 && m.indices[2][1]===4",
    );
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|(?:(?:))))+/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.each.charCodeAt(0)===0xd800 && m.indices[1][0]===1 && m.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|(?:(?:)))+/v.exec('𐀀').groups.each===''",
    );
    check(
        r"let r=/(?<each>(?:(?:))|[\q{ab}&&a])+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn grouped_empty_consumers_replacements_clones_and_collection_keep_empty_and_undefined_capture_values()
 {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:)))+/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a[0][0]==='ab' && a[0].groups.each==='b' && a[1].index===2 && a[1].groups.each==='' && a[2][0]==='ba' && a[2].groups.each==='a' && a[3].index===5 && a[3].groups.each==='' && r.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<b:b><:>X<a:a><:>' && 'abXba'.split(/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:)))+/v).join('|')==='|b|X|a|'",
    );
    check(
        r"let a=[],b=[];'😀'.replace(/(?<each>[\q{ab}&&a]|(?:(?:)))+/vg,(m,each,at,input,groups)=>{a.push([each,at,groups.each]);return '#'});'😀'.replace(/(?<each>[\q{ab}&&a]|(?:(?:)))*/vg,(m,each,at,input,groups)=>{b.push([each,at,groups.each]);return '#'});a.length===2 && b.length===2 && a[0][0]==='' && a[1][1]===2 && a[1][2]==='' && b[0][0]===undefined && b[1][1]===2 && b[1][2]===undefined",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:)))+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:)))+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');m[0]==='' && m.groups.each==='' && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b|ab}&&\p{Any}]|(?:(?:))))+/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabbb').groups.each==='b' && r.lastIndex===5"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn grouped_empty_complete_proofs_finish_finalizers_and_keep_other_shapes_deferred() {
    for source in [
        r"/([\q{a|b}]|(?:)){2}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){2}/v.test('a')",
        r"/([\q{a|b}]|(?:)){1}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){1}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:)){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:)){0}/v.test('a')",
        r"/(?:[\q{a}]|(?:))+/v.test('a')",
        r"/([\q{a}]|(?:))+/v.test('a')",
        r"/(?:(?:)|[\q{a}])+/v.test('a')",
        r"/((?:(?:))|[\q{ab}&&a])+?/v.test('')",
        r"/(?<each>[\q{ab}&&a]|(?:))+/v.exec('X').groups.each===''",
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
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="((?:)|[\\q{ab}&&\\q{cd}])+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\"],\"indices\":[[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for source in [
        r"/((?:)|[[\q{ab}&&\q{cd}]&&\p{RGI_Emoji}])+/v.test('')",
        r"/((?:)|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}])+?/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:X|)?)+/v.test('a')",
        r"/([\q{a|b}]|(?:\b|()))+/v.test('a')",
        r"/([\q{a|b}]|(?:\b|()))+/v.test('a')",
        r"/([\q{a|b}]|(?:\b|()))+/v.test('a')",
        r"/([\q{a|ab}&&a]|(\b)?)+/v.test('a')",
        r"/([\q{a|ab}&&a]||(?:))+/v.test('a')",
        r"/(\b[\q{a|ab}&&a]|(?:))+/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:))+[b]/v.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    for pattern in [
        r"([^\q{ab}]|(?:))+",
        r"([\q{ab}&&a--a]|(?:))*",
        r"([\q{\d}&&a]|(?:))?",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let a=/^\\b(?<each>{class}|(?:(?:)))+?\\b$/div.exec('ſK'),b=/(?<whole>(?<\\u{{1D49C}}>(?:)|{class}))+?/div.exec('ſK');a[0]==='ſK' && a.groups.each==='K' && a.indices[1][0]===1 && b[0]==='' && b.groups['𝒜']==='' && b.indices.groups['𝒜'][0]===0"
        ));
    }
}
