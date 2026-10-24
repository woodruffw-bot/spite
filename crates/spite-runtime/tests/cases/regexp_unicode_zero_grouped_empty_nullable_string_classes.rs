//! Zero-minimum grouped-empty nullable string classes (22.2.2.3 RepeatMatcher).

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
fn zero_iterations_keep_inner_captures_undefined_and_enclosing_captures_defined() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        for branch in [format!("{class}|(?:(?:))"), format!("(?:(?:))|{class}")] {
            for q in [
                "{0}",
                "{00,00}?",
                "{0,3}",
                "{0,3}?",
                "{0,}",
                "{000,}?",
                "{0,999999999999999999999999999999999999}",
            ] {
                check(&format!(
                    "let r=/(?<whole>(?<each>(?<deep>{branch})){q})/dvgy;r.lastIndex=1;let m=r.exec('😀');m.index===0 && m[0]==='' && m[1]==='' && m[2]===undefined && m[3]===undefined && m.groups.whole==='' && Object.keys(m.groups).join('|')==='whole|each|deep' && m.groups.each===undefined && m.groups.deep===undefined && m.indices[1][0]===0 && m.indices[1][1]===0 && m.indices[2]===undefined && m.indices.groups.each===undefined && Object.hasOwn(m.indices.groups,'deep') && r.lastIndex===0"
                ));
            }
        }
    }
    check(
        r"let r=/(?<each>[\q{a|ab}&&a]|(?:(?:))){0}/dvgy;r.lastIndex=9;r.exec('😀')===null && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>(?:(?:))|[\q{a|b}]){0}/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn bounded_and_open_greedy_lazy_counts_reject_optional_empty_iterations_in_either_order() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        for branch in [format!("{class}|(?:(?:))"), format!("(?:(?:))|{class}")] {
            check(&format!(
                "let m=/(?<each>{branch}){{00,0002}}/dv.exec('abbb!');m[0]==='ab' && m[1]==='b' && m.groups.each==='b' && m.indices[1][0]===1 && m.indices[1][1]===2"
            ));
            check(&format!(
                "let m=/(?<each>{branch}){{0,3}}?/dv.exec('abbb!');m[0]==='' && m[1]===undefined && m.indices[1]===undefined"
            ));
            check(&format!(
                "let r=/(?<each>{branch}){{0,2}}/dvg,a=r.exec('abbb'),b=r.exec('abbb'),c=r.exec('abbb');a[0]==='ab' && b[0]==='bb' && b.groups.each==='b' && b.indices[1][0]===3 && c.index===4 && c[0]==='' && c.groups.each===undefined && r.lastIndex===4"
            ));
            check(&format!(
                "let m=/(?<whole>^\\b(?<each>{branch}){{0,3}}?\\b$)/dimv.exec('!\\r\\nABB\\u2028!');m.index===3 && m.groups.whole==='ABB' && m.groups.each==='B' && m.indices[2][0]===5 && m.indices[2][1]===6"
            ));
        }
    }
}

#[test]
fn bounded_nullable_captures_preserve_supplementary_folding_and_original_code_unit_ranges() {
    check(
        r"let r=/(?<whole>(?<each>((?:(?:))|[\q{😀|😀x}&&\p{Any}])){0,2})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀😀x'),n=r.exec('A😀😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.each==='😀' && m.indices[2][0]===3 && m.indices[3][1]===5 && n.index===5 && n[0]==='😀' && n.indices[2][0]===5 && n.indices[2][1]===7 && r.lastIndex===7",
    );
    for class in [r"[\q{\u{10428}}]", r"[\q{\u{10428}|ab}&&\p{Any}]"] {
        check(&format!(
            "let m=/(?<each>({class}|(?:(?:)))){{0,2}}/div.exec('𐐨𐐀');m[0]==='𐐨𐐀' && m[1]==='𐐀' && m[2]==='𐐀' && m.groups.each==='𐐀' && m.indices[1][0]===2 && m.indices[2][1]===4"
        ));
    }
    check(
        r"let m=/(?<each>([\q{\uD800a|\uD800}&&\p{Any}]|(?:(?:)))){0,}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.each.charCodeAt(0)===0xd800 && m.indices[1][0]===1 && m.indices[2][1]===2 && /(?<each>[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]|(?:(?:))){0,}/v.exec('𐀀').groups.each===undefined",
    );
    check(
        r"let a=/(?<each>[\q{ab|}&&a]|(?:(?:))){0,}/dv.exec('ab');a[0]==='' && a.groups.each===undefined && a.indices.groups.each===undefined",
    );
}

#[test]
fn string_consumers_replacements_clones_and_collection_keep_undefined_empty_match_captures() {
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){0,3}/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a[0][0]==='ab' && a[0].groups.each==='b' && a[1].index===2 && a[1].groups.each===undefined && a[1].indices[1]===undefined && a[2][0]==='ba' && a[2].groups.each==='a' && a[3].index===5 && a[3].groups.each===undefined && r.lastIndex===0 && 'abXba'.replace(r,'<$1:$<each>>')==='<b:b><:>X<a:a><:>' && 'abXba'.split(/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){0,3}/v).join('|')==='|b|X|a|'",
    );
    check(
        r"let a=[];'😀'.replace(/(?<each>[\q{a|b}]|(?:(?:))){0}/vg,(m,each,at,input,groups)=>{a.push([each,at,groups.each]);return '#'});let b='😀a'.split(/(?<each>[\q{a|ab}&&a]|(?:(?:))){0}/v);a.length===2 && a[0][0]===undefined && a[1][1]===2 && a[1][2]===undefined && b.length===3 && b[0]==='😀' && b[1]===undefined && b[2]==='a'",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){0,}/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0",
    );
    check(
        r"let r=/(?<each>[\q{a|b|ab}&&\p{Any}]|(?:(?:))){0,3}/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');m[0]==='' && m.groups.each===undefined && c.lastIndex===0 && r.exec('ABB').groups.each==='B' && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>([\q{a|b}]|(?:(?:)))){0,3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"r.exec('xabbb').groups.each==='b' && r.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn zero_grouped_empty_proofs_finish_finalizers_and_prove_all_unreachable_leaves() {
    for source in [
        r"/([\q{a|b}]|(?:)){2}/v.test('a')",
        r"/([\q{a|b}]|(?:)){1}/v.test('a')",
        r"/([\q{a|b}]|(?:)){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:)){0}/v.test('a')",
        r"/([\q{a}]|(?:(?:))){0,3}/v.test('aaa')",
        r"/(?:((?:)|[\q{ab}&&a])){000,}/v.exec('X')[1]===undefined",
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="((?:)|[\\q{ab}&&\\q{cd}]){0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r####"var pattern="(\\b[\\q{a|ab}&&a]|(?:)){0}",input="a",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\"],[[0,0],\"<undefined>\"],0,0]""####,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="((?:)|[[\\q{ab}&&\\q{cd}]&&\\p{RGI_Emoji}]){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="((?:)|[\\q{ab}&&\\p{Any}&&\\p{RGI_Emoji}]){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="(([\\q{a|ab}&&a])|(?:X|)?){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="([\\q{a|b}]|(?:\\b|())){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="([\\q{a|b}]|(\\b)?){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="([\\q{a|ab}&&a]||(?:)){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&(m.groups===undefined||Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null)&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex,m.groups===undefined?null:Object.keys(m.groups).map(k=>[k,m.groups[k]===undefined?"<undefined>":m.groups[k]]),m.indices.groups===undefined?null:Object.keys(m.indices.groups).map(k=>[k,m.indices.groups[k]===undefined?"<undefined>":m.indices.groups[k]])])==="[[\"\",\"<undefined>\"],[[1,1],\"<undefined>\"],1,1,null,null]""####),Ok(Value::Boolean(true)));
    for source in [
        r"/([\q{a|ab}&&a]|(?:)){0}[b]/v.test('ab')",
        r"/([\q{a|b}]|(?:\b|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:\b|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
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
        r"([^\q{ab}]|(?:)){0}",
        r"([\q{ab}&&a--a]|(?:)){0,3}",
        r"([\q{\d}&&a]|(?:)){0}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
    check(
        r"let m=/(?<\u{1D49C}>[\q{a|ab}&&a]|(?:)){0}/dv.exec('aaa');Object.hasOwn(m.groups,'𝒜') && m.groups['𝒜']===undefined && m.indices.groups['𝒜']===undefined",
    );
    for class in [r"[\q{s|k}]", r"[\q{s|k|sk}&&\p{Any}]"] {
        check(&format!(
            "let m=/^\\b(?<each>{class}|(?:(?:))){{0,3}}?\\b$/div.exec('ſK');m[0]==='ſK' && m.groups.each==='K' && m.indices[1][0]===1 && m.indices[1][1]===2"
        ));
    }
}

#[test]
fn grouped_zero_and_huge_upper_counts_keep_deep_captures_flat_and_opt_in_aborts_atomic() {
    let pattern = format!(
        r"{}[\q{{a|ab}}&&a]|(?:(?:)){}{{0}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');m.length===1001 && m[0]==='' && m[1000]===undefined && m.indices[1000]===undefined"
    ));
    let pattern = format!(r"((?:(?:))|[\q{{{}}}&&a]){{0,}}", "x".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dvy,m=r.exec('😀');m[0]==='' && m[1]===undefined && m.indices[1]===undefined && r.lastIndex===0"
    ));
    let pattern = format!(r"([\q{{a|ab}}&&a]|(?:(?:))){{0,{}}}", "9".repeat(20_000));
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
        .eval(r"var r=/([\q{a|ab}&&a]|(?:(?:))){0,}/vg,marker=0")
        .unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "a".repeat(40_000)))
        .unwrap();
    assert_eq!(
        realm.eval(r"var probe=/a/vy;probe.lastIndex=1;probe.exec(text)[0]==='a'"),
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
