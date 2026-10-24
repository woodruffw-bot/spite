//! Complete and empty point captures around one variable-width string class.
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
fn single_string_class_layouts_keep_complete_and_empty_capture_ranges() {
    let huge = 10u128.pow(30);
    for (class, input, word) in [
        (r"[\q{|a|ab}]", "ab!", Some("ab")),
        (r"[\q{|a|ab}]", "!", Some("")),
        (r"[\q{ab}\q{}]", "ab!", Some("ab")),
        (r"[x\q{|ab}]", "x!", Some("x")),
        (r"[a-z\q{|ab}]", "ab!", Some("ab")),
        (r"[[\q{|ab}][x\q{a}]]", "ab!", Some("ab")),
        (r"[\q{|a|ab}--\q{a}]", "a!", Some("")),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", "x!", Some("x")),
        (r"[\q{|a|ab}--[a-z]]", "a!", Some("")),
        (r"[\q{|😀|😀a}]", "😀a!", Some("😀a")),
        (r"[\q{|\u{10428}|\u{10428}a}]", "𐐨a!", Some("𐐨a")),
        (r"[\q{|s|sa|k|ka}]", "sa!", Some("sa")),
        (r"[\q{}]", "ab!", Some("")),
        (r"[\q{|ab}--\q{ab}]", "ab!", Some("")),
        (r"[\q{a|ab|abc}]", "abc!", Some("abc")),
        (r"[\q{ab}\q{a|abc}]", "abc!", Some("abc")),
        (r"[x\q{a|ab}]", "x!", Some("x")),
        (r"[a-z\q{ab|abc}]", "abc!", Some("abc")),
        (r"[[\q{a|ab}][x\q{abc}]]", "abc!", Some("abc")),
        (r"[\q{a|ab|abc}--\q{ab}]", "abc!", Some("abc")),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", "ab!", Some("ab")),
        (r"[\q{a|b|ab}--[b-z]]", "ab!", Some("ab")),
        (r"[\q{😀|😀a|a}]", "😀a!", Some("😀a")),
        (r"[\q{\u{10428}|\u{10428}a}]", "𐐨a!", Some("𐐨a")),
        (r"[\q{s|sa|k|ka}]", "sa!", Some("sa")),
        (r"[\q{ab}--\q{ab}]", "ab!", None),
        (r"[\q{|a|ab}--\q{}]", "ab!", Some("ab")),
        (r"[\q{a|ab}&&\q{ab}]", "ab!", Some("ab")),
        (r"[\q{a|ab}]", "!", None),
    ] {
        for layout in 0..6 {
            let (pattern, characters, before, after, never) = match layout {
                0 => (class.to_owned(), "", "", "", ""),
                1 => (format!("(?<part>{class})"), "part", "", "", ""),
                2 => (
                    format!("(?<before>)(?<part>{class})(?<after>)"),
                    "part",
                    "before",
                    "after",
                    "",
                ),
                3 => (
                    format!(
                        "(?<whole>(?<before>(?<first>)|(?<later>))(?<part>{class})(?<optional>)?(?<after>))"
                    ),
                    "whole,part",
                    "before,first",
                    "after",
                    "later,optional",
                ),
                4 => (
                    format!(
                        "(?<required>){{{huge}}}(?<part>(?<deep>{class}))(?<optional>){{0,{huge}}}?(?<after>(?<first>)|(?<later>))"
                    ),
                    "part,deep",
                    "required",
                    "after,first",
                    "optional,later",
                ),
                _ => (
                    format!(
                        "(?<whole>(?:(?<optional>)*|(?<later>))(?<part>{class})(?<after>(?<a>)(?<b>)))"
                    ),
                    "whole,part",
                    "",
                    "after,a,b",
                    "optional,later",
                ),
            };
            let pattern = JsString::from(pattern.as_str());
            let matched = word.is_some();
            let whole = JsString::from(word.unwrap_or(""));
            let end = 1 + whole.len();
            let text = JsString::from(format!("!{input}").as_str());
            let named = layout != 0;
            check(&format!(
                r#"
            var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec({text:?}),ok=true;
            if({matched}){{
                if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
                        if(characters.indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
                        if(before.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if(after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
                    }}ok=ok&&m.length===keys.length+1;
                }}else ok=ok&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined;
                ok=ok&&m[0]==={whole:?}&&m.index===1&&m.indices[0][0]===1&&m.indices[0][1]==={end}&&r.lastIndex==={end};
            }}else ok=m===null&&r.lastIndex===0;
            ok&&r.source==={pattern:?}
            "#
            ));
        }
    }
}
#[test]
fn single_string_class_search_sticky_empty_members_and_original_names_preserve_state() {
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}])(?<after>)/dvg;var a=r.exec('!abXa!'),b=r.exec('!abXa!'),n=r.exec('!abXa!');a[0]==='ab'&&a.index===1&&a.groups.before===''&&a.groups.after===''&&a.indices.groups.before[0]===1&&a.indices.groups.after[0]===3&&b.groups.part==='a'&&b.index===4&&n===null&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}])(?<after>)/dvgy;r.lastIndex=0;var n=r.exec('!ab!');r.lastIndex=1;var m=r.exec('!ab!');n===null&&m.groups.part==='ab'&&m.index===1&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{|a|ab}])(?<after>)/dvg;var m=r.exec('!ab!');m[0]===''&&m.groups.part===''&&m.groups.before===''&&m.groups.after===''&&m.indices.groups.part[0]===0&&m.indices.groups.part[1]===0&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{😀|😀a}])(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m.groups.part==='😀a'&&m.index===0&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===3&&r.lastIndex===3",
    );
    check(
        r"var m=/(?<before>)(?<\u{1D49C}>[\q{s|sa|k|ka}])(?<after>)/dvi.exec('!ſaKa');m.groups.𝒜==='ſa'&&m.index===1&&m.groups.before===''&&m.indices.groups.after[0]===3",
    );
    check(
        r"var m=/()([\q{\ud800|\ud800a}])()/dv.exec('!\ud800a!');m[2]==='\ud800a'&&m.index===1&&m.indices[3][0]===3",
    );
    check(
        r"var r=/()([\q{|ab}])()/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn single_string_class_consumers_keep_points_original_values_and_empty_progress() {
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}])(?<after>)/dvg;var a=Array.from('abXa'.matchAll(r));a.length===2&&a.map(m=>m[0]).join('|')==='ab|a'&&a.every(m=>m.groups.part===m[0]&&m.groups.before===''&&m.groups.after==='')&&a[1].indices.groups.after[0]===4&&r.lastIndex===0",
    );
    check(
        r"var a=Array.from('😀a'.matchAll(/()([\q{}])()/dvg));a.length===3&&a.map(m=>m.index).join(',')==='0,2,3'&&a.every(m=>m[1]===''&&m[2]===''&&m[3]==='')",
    );
    check(
        r"var calls=0;var out='abXa'.replace(/(?<before>)(?<part>[\q{a|ab}])(?<after>)/vg,function(all,b,p,a,offset,input,groups){calls++;return arguments.length===7&&all===p&&b===''&&a===''&&groups.part===p&&input==='abXa'?'#':'bad'});out==='#X#'&&calls===2",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}])(?<after>)/vg;r.lastIndex=9;var parts='abX'.split(r);'Xab'.search(r)===1&&r.lastIndex===9&&parts.length===5&&parts[0]===''&&parts[1]===''&&parts[2]==='ab'&&parts[3]===''&&parts[4]==='X'&&'abXa'.replace(r,'<$<before>:$<part>:$<after>>')==='<:ab:>X<:a:>'",
    );
    check(
        r"var a=Array.from('abXa'.matchAll(/(?<before>)(?<part>[\q{|a|ab}])(?<after>)/dvg));a.length===4&&a.map(m=>m[0]).join('|')==='ab||a|'&&a.every(m=>m.groups.part===m[0]&&m.groups.before===''&&m.groups.after==='')",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/()([\q{|a|ab}])()/dvg,clone=new RegExp(r),m=r.exec('!')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("m[1]===''&&m[2]===''&&m[3]===''&&clone.exec('ab!')[2]==='ab'&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_string_class_proofs_finish_finalizers_and_keep_other_features_separate() {
    check(
        r"var marker=0,m;try{m=/()([\q{a|ab}])()/dv.exec('!ab!')}finally{marker=3}marker===3&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m[3]===''&&m.indices[3][0]===3",
    );
    check(
        r"var marker=0,m;try{m=/()([\q{ab}&&\q{cd}])()/dv.exec('ab')}finally{marker=3}marker===3&&m===null",
    );
    check(
        r"var marker=0,m;try{m=/^()\b([\q{a|ab}])/dv.exec('ab!')}finally{marker=3}marker===3&&m.length===3&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m.indices[2][1]===2",
    );
    check(
        r"var marker=0,m;try{m=/()([\q{|ab}])\b()$/dv.exec('!ab')}finally{marker=3}marker===3&&m.length===4&&m.index===1&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m[3]===''&&m.indices[3][0]===3&&/()([\q{|ab}])\b()$/v.exec('!')===null",
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{a|ab}])+()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"\",\"ab\",\"\"],\"indices\":[[0,2],[0,0],[0,2],[2,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"ab\",\"\",\"ab\",\"\"],\"indices\":[[2,4],[2,2],[2,4],[4,4]],\"index\":2},4],[true,\"ab!\",{\"values\":[\"ab\",\"\",\"ab\",\"\"],\"indices\":[[0,2],[0,0],[0,2],[2,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\",\"\",\"ab\",\"\"],\"indices\":[[2,4],[2,2],[2,4],[4,4]],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{|ab}]){0}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"^()\b([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])",
        r"()([\q{|ab}&&\p{Any}&&\p{RGI_Emoji}])\b()$",
        r"()([\q{a|ab}])\1",
        r"()([[\q{a|ab}]&&\p{RGI_Emoji}])+()",
        r"()([[\q{|ab}]&&\p{RGI_Emoji}]){0}()",
        r"()([\q{a|ab}])x",
        r"()([\q{|ab}])([\q{cd}])",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn single_string_class_deep_captures_long_words_and_opt_in_aborts_remain_flat() {
    let pattern = JsString::from(
        format!("(){}[\\q{{|a|ab}}]{}()", "(".repeat(1000), ")".repeat(1000)).as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),a=r.exec('ab!'),e=r.exec('!');a.length===1003&&a[1]===''&&a[2]==='ab'&&a[1001]==='ab'&&a[1002]===''&&e[1]===''&&e[2]===''&&e[1001]===''&&e[1002]===''"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!("()([\\q{{{word}}}])()").as_str());
    let input = JsString::from(format!("!!{word}!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m.index===2&&m[1]===''&&m[2].length===20000&&m[3]===''&&m.indices[3][0]===20002"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'v')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm
        .eval(r"var r=/()([\q{a|ab}])()/vg;r.lastIndex=1")
        .unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(input)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("try{Array.from(input.matchAll(r))}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
