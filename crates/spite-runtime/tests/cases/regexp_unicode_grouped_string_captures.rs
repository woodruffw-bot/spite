//! Grouped string alternatives retain their owner and unconditional siblings.
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
fn grouped_string_layouts_keep_whole_branch_and_sibling_capture_ranges() {
    let huge = 2u128;
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
    ] {
        for empty_first in [false, true] {
            for layout in 0..6 {
                let (character, empty, characters, before, after, empties, never) = match layout {
                    0 => (class.to_owned(), String::new(), "", "", "", "", ""),
                    1 => (
                        format!("(?<part>{class})"),
                        "(?<empty>)".to_owned(),
                        "part",
                        "",
                        "",
                        "empty",
                        "",
                    ),
                    2 => (
                        format!("(?<before>)(?<part>{class})(?<after>)"),
                        "(?<empty>)".to_owned(),
                        "part",
                        "before",
                        "after",
                        "empty",
                        "",
                    ),
                    3 => (
                        format!("(?<part>{class})"),
                        "(?<empty>(?<first>)|(?<later>))(?<optional>)?".to_owned(),
                        "part",
                        "",
                        "",
                        "empty,first",
                        "later,optional",
                    ),
                    4 => (
                        class.to_owned(),
                        format!("(?<empty>(?<deep>)?)(?<required>){{{huge}}}"),
                        "",
                        "",
                        "",
                        "empty,required",
                        "deep",
                    ),
                    _ => (
                        format!("(?:|)(?<part>(?<deep>{class}))(?<after>)"),
                        format!("(?:(?:)|(?:)){{{huge}}}"),
                        "part,deep",
                        "",
                        "after",
                        "",
                        "",
                    ),
                };
                let pattern = JsString::from(
                    if empty_first {
                        format!("(?<outsideBefore>)(?<whole>{empty}|{character})(?<outsideAfter>)")
                    } else {
                        format!("(?<outsideBefore>)(?<whole>{character}|{empty})(?<outsideAfter>)")
                    }
                    .as_str(),
                );
                let class_selected = !empty_first && word.is_some();
                let whole = JsString::from(if class_selected { word.unwrap() } else { "" });
                let end = 1 + whole.len();
                let text = JsString::from(format!("!{input}").as_str());
                let named = true;
                check(&format!(
                    r#"
                var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec({text:?}),ok=true;
                if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
                        if({class_selected}&&characters.indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
                        if({class_selected}&&before.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if({class_selected}&&after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
                        if(!{class_selected}&&empty.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        if(name==='whole'){{expected={whole:?};from=1;to={end}}}
                        if(name==='outsideBefore'){{expected='';from=1;to=1}}
                        if(name==='outsideAfter'){{expected='';from={end};to={end}}}
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
                    }}ok=ok&&m.length===keys.length+1;
                }}else ok=ok&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined;
                ok&&m[0]==={whole:?}&&m.index===1&&m.indices[0][1]==={end}&&r.lastIndex==={end}&&r.source==={pattern:?}
            "#
                ));
            }
        }
    }
}
#[test]
fn grouped_string_values_keep_owner_identity_original_utf16_and_names() {
    check(
        r"var m=/(?<outer>(?<part>[\q{a|ab}])|(?<empty>))/dv.exec('!');m[0]===''&&m.groups.outer===''&&m.groups.part===undefined&&m.groups.empty===''&&m.indices.groups.outer[0]===0&&m.indices.groups.outer[1]===0",
    );
    check(
        r"var m=/(?<outer>(?<part>[\q{|a|ab}])|(?<empty>))/dv.exec('!');m[0]===''&&m.groups.outer===''&&m.groups.part===''&&m.groups.empty===undefined&&m.indices.groups.part[0]===0&&m.indices.groups.part[1]===0",
    );
    check(
        r"var m=/(?<outer>(?<empty>)|(?<part>[\q{|a|ab}]))/dv.exec('ab!');m[0]===''&&m.groups.outer===''&&m.groups.empty===''&&m.groups.part===undefined",
    );
    check(
        r"var r=/(?<before>)(?<outer>(?<part>[\q{|😀|😀a}])|(?<empty>))(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m[0]==='😀a'&&m.index===0&&m.groups.outer==='😀a'&&m.groups.part==='😀a'&&m.groups.before===''&&m.groups.after===''&&m.indices.groups.outer[1]===3&&m.indices.groups.after[0]===3&&r.lastIndex===3",
    );
    check(
        r"var m=/(?<outer>(?<\u{1D49C}>[\q{|s|sa|k|ka}])|(?<empty>))/dvi.exec('ſaKa');m.groups.outer==='ſa'&&m.groups.𝒜==='ſa'&&m.groups.empty===undefined&&m.indices.groups.𝒜[1]===2",
    );
    check(
        r"var r=/(?<outer>[\q{|ab}]|)/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn grouped_string_consumers_keep_enclosing_captures_and_empty_progress() {
    check(
        r"var r=/(?<before>)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<after>)/dvg;var a=Array.from('abXa'.matchAll(r));a.length===4&&a.map(m=>m[0]).join('|')==='ab||a|'&&a.every(m=>m.groups.whole===m[0]&&m.groups.part===m[0]&&m.groups.before===''&&m.groups.after===''&&m.groups.empty===undefined)&&a[1].indices.groups.whole[0]===2&&a[1].indices.groups.whole[1]===2&&r.lastIndex===0",
    );
    check(
        r"var a=Array.from('😀a'.matchAll(/(?<whole>(?<part>[\q{ab}])|(?<empty>))/dvg));a.length===3&&a.map(m=>m.index).join(',')==='0,2,3'&&a.every(m=>m.groups.whole===''&&m.groups.part===undefined&&m.groups.empty==='')",
    );
    check(
        r"var calls=0;var out='abXa'.replace(/(?<before>)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<after>)/vg,function(all,b,w,p,e,a,offset,input,groups){calls++;return arguments.length===9&&all===w&&w===p&&b===''&&a===''&&e===undefined&&groups.whole===all&&input==='abXa'?'#':'bad'});out==='##X##'&&calls===4",
    );
    check(
        r"var r=/(?<before>)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<after>)/vg;r.lastIndex=9;var parts='abX'.split(r);'Xab'.search(r)===0&&r.lastIndex===9&&parts.length===7&&parts[0]===''&&parts[1]===''&&parts[2]==='ab'&&parts[3]==='ab'&&parts[4]===undefined&&parts[5]===''&&parts[6]==='X'&&'abXa'.replace(r,'<$<whole>:$<part>:$<empty>>')==='<ab:ab:><::>X<a:a:><::>'",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))/dvg;var clone=new RegExp(r);var m=r.exec('!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.whole===''&&m.groups.part===''&&m.groups.empty===undefined&&clone.exec('ab!').groups.whole==='ab'"),Ok(Value::Boolean(true)));
}

#[test]
fn grouped_string_proofs_finish_finalizers_and_keep_counts_assertions_and_references_separate() {
    check(
        r"var marker=0,m;try{m=/(?:([\q{a|ab}])|())/dv.exec('ab!')}finally{marker=3}marker===3&&m[0]==='ab'&&m[1]==='ab'&&m[2]===undefined&&m.indices[1][1]===2",
    );
    check(
        r"var marker=0,m;try{m=/(?:([\q{|a|ab}])|())/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/^([\q{a|ab}]|())/dv.exec('ab!')}finally{marker=3}var e=/^([\q{a|ab}]|())/dv.exec('!ab');marker===3&&m.length===3&&m[0]==='ab'&&m[1]==='ab'&&m[2]===undefined&&e[0]===''&&e[1]===''&&e[2]===''&&e.index===0",
    );
    check(
        r"var marker=0,m;try{m=/([\q{|ab}]|())$/dv.exec('ab!')}finally{marker=3}var a=/([\q{|ab}]|())$/dv.exec('ab');marker===3&&m.length===3&&m.index===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===3&&a[0]==='ab'&&a[1]==='ab'&&a[2]===undefined",
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{a|ab}]|())+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[false,\"\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[true,\"\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{|ab}]|()){0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="([[\\q{|ab}]&&\\p{RGI_Emoji}]|()){0}",input="a😀b",r=new RegExp(pattern,"dgvy");r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==="dgvy"&&r.hasIndices&&r.unicodeSets&&m.input===input&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"<undefined>\",\"<undefined>\"],[[1,1],\"<undefined>\",\"<undefined>\"],1,1]""####),Ok(Value::Boolean(true)));
    for pattern in [
        r"^([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|())",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|())$",
        r"([\q{|ab}]|())\1",
        r"([[\q{a|ab}]&&\p{RGI_Emoji}]|())+",
        r"([\q{a|ab}]x|())",
        r"([\q{|ab}]|[\q{cd}])",
        r"([\q{ab}]|()|())",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|())",
    ] {
        let mut realm = Realm::default();
        assert!(
            crate::common::regexp_oracle::check(
                &mut realm,
                &format!(
                    "var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}};marker",
                    JsString::from(pattern)
                )
            ),
            "{pattern}"
        );
    }
}
