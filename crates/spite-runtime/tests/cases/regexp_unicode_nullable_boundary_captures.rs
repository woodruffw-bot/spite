//! Empty point captures beside nullable UnicodeSets alternatives (22.2.2.3).

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
fn nullable_boundary_captures_keep_last_iteration_points_and_branch_participation() {
    check(
        r"let a=/(?:(?:[\q{a}]|)())+/dv.exec('aa'),b=/(?:()(?:[\q{a}]|))+/dv.exec('aa');a[0]==='aa' && a[1]==='' && a.indices[1][0]===2 && a.indices[1][1]===2 && b[0]==='aa' && b[1]==='' && b.indices[1][0]===1 && b.indices[1][1]===1",
    );
    check(
        r"let a=/(?:()(?:[\q{a}]|)()){3}/dv.exec('aa'),b=/(?:()(?:|[\q{a}])()){2,3}/dv.exec('aa');a[0]==='aa' && a[1]==='' && a[2]==='' && a.indices[1][0]===2 && a.indices[2][0]===2 && b[0]==='a' && b[1]==='' && b[2]==='' && b.indices[1][0]===0 && b.indices[2][0]===1",
    );
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?:(?<left>){class}(?<right>)|(?<empty>)){{2}}/dv.exec('ab'),b=/(?:(?<left>){class}(?<right>)|(?<empty>)){{3}}/dv.exec('ab');a[0]==='ab' && a.groups.left==='' && a.groups.right==='' && a.groups.empty===undefined && a.indices.groups.left[0]===1 && a.indices.groups.right[0]===2 && b[0]==='ab' && b.groups.left===undefined && b.groups.right===undefined && b.groups.empty==='' && b.indices.groups.empty[0]===2"
        ));
    }
    check(
        r"let m=/(?:(?<before>)(?:[\q{a}]|)(?<after>)){0}/dv.exec('a');m[0]==='' && m.groups.before===undefined && m.groups.after===undefined && Object.hasOwn(m.groups,'before') && m.indices.groups.before===undefined",
    );
}

#[test]
fn nullable_boundary_points_preserve_utf16_names_gates_flags_and_sticky_state() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<before>)(?:(?<left>)(?<part>{class})(?<right>)|(?<empty>))(?<after>))+)/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.part==='😀' && m.groups.empty===undefined && m.indices.groups.before[0]===3 && m.indices.groups.left[0]===3 && m.indices.groups.right[0]===5 && m.indices.groups.after[0]===5 && r.lastIndex===5"
        ));
    }
    check(
        r"let source='(?:(?<\\u{1D49C}>)(?:[\\q{s|k}]|)(?<after>))+',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && r.flags==='div' && m[0]==='ſK' && m.groups['𝒜']==='' && m.groups.after==='' && m.indices.groups['𝒜']===m.indices[1] && m.indices[1][0]===1 && m.indices.groups.after[0]===2",
    );
    check(
        r"let m=/(?:(?<before>)(?:[\q{\uD800a|\uD800}&&\p{Any}]|)(?<after>))+/dv.exec('\uD800\uD800a');m[0].length===2 && m.indices.groups.before[0]===1 && m.indices.groups.after[0]===2",
    );
    check(
        r"let m=/(?<whole>^\b(?:(?<before>)(?:[\q{a|b}]|)(?<after>))+\b$)/dimv.exec('!\r\nAB\u2028!');m.index===3 && m.groups.whole==='AB' && m.groups.before==='' && m.groups.after==='' && m.indices.groups.before[0]===4 && m.indices.groups.after[0]===5",
    );
    check(
        r"let r=/(?:(?<before>)(?:[\q{ab}&&a]|)(?<after>))+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn nullable_boundary_point_consumers_callbacks_clones_and_collection_keep_empty_values() {
    check(
        r"let r=/(?<each>(?<before>)(?:(?<left>)(?<part>[\q{a|b}])(?<right>)|(?<empty>(?<first>)|(?<later>)))(?<after>))+/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a.map(m=>m.indices.groups.before[0]).join(',')==='1,2,4,5' && a.map(m=>m.indices.groups.after[0]).join(',')==='2,2,5,5' && a[0].groups.part==='b' && a[1].groups.left===undefined && a[1].groups.empty==='' && a[1].groups.first==='' && a.every(m=>m.groups.before==='' && m.groups.after==='' && m.groups.later===undefined) && r.lastIndex===0",
    );
    check(
        r"let seen=[],out='abX'.replace(/(?<each>(?<before>)(?:(?<left>)(?<part>[\q{a|b}])(?<right>)|(?<empty>(?<first>)|(?<later>)))(?<after>))+/vg,(m,each,before,left,part,right,empty,first,later,after,at,input,groups)=>{seen.push([before,left,part,right,empty,first,later,after,at,groups.before]);return '#'});out==='##X#' && seen.length===3 && seen[0][0]==='' && seen[0][1]==='' && seen[0][2]==='b' && seen[0][3]==='' && seen[0][4]===undefined && seen[1][0]==='' && seen[1][1]===undefined && seen[1][4]==='' && seen[1][5]==='' && seen[1][6]===undefined && seen[1][7]==='' && seen[1][8]===2 && seen[1][9]===''",
    );
    check(
        r"let r=/(?:(?<before>)(?:[\q{a|b}]|)(?<after>))+/vg;r.lastIndex=9;let parts='ab'.split(r);'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && parts.length===4 && parts.every(value=>value==='') && 'ab'.replace(r,'<$<before>:$<after>>')==='<:><:>'",
    );
    check(
        r"let r=/(?:(?<before>)(?:[\q{a|b}]|)(?<after>))+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB'),n=r.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.before==='' && m.groups.after==='' && m.indices.groups.before[0]===0 && c.lastIndex===0 && n[0]==='ABB' && n.indices===undefined && n.groups.before==='' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?:(?<before>)(?:[\q{a|b}]|)(?<after>)){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.before==='' && m.groups.after==='' && m.indices.groups.before[0]===3 && m.indices.groups.after[0]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_nullable_boundary_proofs_finish_finalizers_and_keep_other_effects_deferred() {
    for source in [
        r"let m=/(?:(?:[\q{a}]|)())+/dv.exec('aa');m[0]==='aa' && m[1]==='' && m.indices[1][0]===2",
        r"let m=/(?:()(?:|[\q{a}])){2,3}/dv.exec('aa');m[0]==='a' && m[1]==='' && m.indices[1][0]===0",
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
    for inner in [r"([\q{a|b}])()|()", r"([\q{a|b}])()|()?"] {
        for count in ["+?", "{0}", "{2,3}"] {
            check(&format!(
                "let marker=0,value;try{{value=/({inner}){count}/v.test('a')}}finally{{marker=3}}value && marker===3"
            ));
        }
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(?:(?:[\\q{ab}&&\\q{cd}]|)()){0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="(?:(?:[[\\q{ab}&&\\q{cd}]&&\\p{RGI_Emoji}]|)()){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="(?:()(?:|[\\q{ab}&&\\p{Any}&&\\p{RGI_Emoji}])){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1]""####),Ok(Value::Boolean(true)));
    for pattern in [
        r"(?:(?:[\q{a}]|)\b())+",
        r"(?:(?:[\q{a}]|)\1())+",
        r"(?:(?:[\q{a}]|)|())+",
    ] {
        assert!(
            crate::common::regexp_oracle::check(
                &mut Realm::default(),
                &format!("/{pattern}/v.test('a')")
            ),
            "{pattern}"
        );
    }
    for pattern in [
        r"(?:(?:[^\q{ab}]|)()){0}",
        r"(?:(?:[\q{ab}&&a--a]|)()){2,3}",
        r"(?:(?:[\q{\d}&&a]|)()){1,}",
        r"(?:(?:[\q{a|b}]|)()){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}
