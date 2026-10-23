//! Root string-class captures whose class result excludes the empty string.
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
fn nonempty_word_root_layouts_keep_longest_words_and_conditional_capture_ranges() {
    let huge = 10u128.pow(30);
    for (class, input, word) in [
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
                        format!("{empty}|{character}")
                    } else {
                        format!("{character}|{empty}")
                    }
                    .as_str(),
                );
                let consuming = !empty_first && word.is_some();
                let whole = JsString::from(if consuming { word.unwrap() } else { "" });
                let end = 1 + whole.len();
                let text = JsString::from(format!("!{input}").as_str());
                let named = layout != 0;
                check(&format!(
                    r#"
                var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec({text:?}),ok=true;
                if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected={consuming}&&characters.indexOf(name)!==-1?{whole:?}:undefined,point=-1;
                        if({consuming}&&before.indexOf(name)!==-1){{expected='';point=1}}
                        if({consuming}&&after.indexOf(name)!==-1){{expected='';point={end}}}
                        if(!{consuming}&&empty.indexOf(name)!==-1){{expected='';point=1}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        else ok=ok&&m.indices[slot][0]===(expected===''?point:1)&&m.indices[slot][1]===(expected===''?point:{end});
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
fn nonempty_word_root_values_keep_original_utf16_folding_lone_surrogates_and_names() {
    check(
        r"var r=/(?<before>)(?<part>[\q{😀|😀a}])(?<after>)|(?<empty>)/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m[0]==='😀a'&&m.index===0&&m.groups.part==='😀a'&&m.groups.before===''&&m.groups.after===''&&m.groups.empty===undefined&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<empty>)|(?<part>[\q{😀|😀a}])/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m[0]===''&&m.index===0&&m.groups.empty===''&&m.groups.part===undefined&&m.indices.groups.empty[0]===0&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<\u{1D49C}>[\q{s|sa|k|ka}])|(?<empty>)/dvgi;var m=r.exec('ſaKa'),n=r.exec('ſaKa');m.groups.𝒜==='ſa'&&n.groups.𝒜==='Ka'&&n.indices.groups.𝒜[0]===2&&n.indices.groups.𝒜[1]===4&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?<part>[\q{\ud800|\ud800a}])|(?<empty>)/dv.exec('\ud800a!');m.groups.part==='\ud800a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===2",
    );
    check(
        r"var r=/(?<part>[\q{a|ab}])|(?<empty>)/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('ab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn nonempty_word_root_consumers_callbacks_clones_and_collection_keep_variable_widths() {
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}])(?<after>)|(?<empty>)/dvg;var all=Array.from('abXa'.matchAll(r));all.length===4&&all.map(m=>m[0]).join('|')==='ab||a|'&&all[0].groups.part==='ab'&&all[0].indices.groups.after[0]===2&&all[1].groups.before===undefined&&all[1].groups.empty===''&&all[2].groups.part==='a'&&all[2].indices.groups.after[0]===4&&r.lastIndex===0",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/(?<empty>)|(?<part>[\q{😀|😀a}])/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m[0]===''&&m.groups.empty===''&&m.groups.part===undefined)",
    );
    check(
        r"var calls=0;var out='abXa'.replace(/(?<before>)(?<part>[\q{a|ab}])(?<after>)|(?<empty>)/vg,function(whole,before,part,after,empty,offset,input,groups){calls++;return arguments.length===8&&groups.part===part&&input==='abXa'&&(whole!==''?before===''&&part===whole&&after===''&&empty===undefined:before===undefined&&part===undefined&&after===undefined&&empty==='')?'#':'bad'});out==='##X##'&&calls===4",
    );
    check(
        r"var r=/(?<part>[\q{a|ab}])|(?<empty>)/vg;r.lastIndex=9;var parts='abX'.split(r);'Xab'.search(r)===0&&r.lastIndex===9&&parts.length===4&&parts[0]===''&&parts[1]==='ab'&&parts[2]===undefined&&parts[3]==='X'&&'abXa'.match(r).join('|')==='ab||a|'&&'abXa'.replace(r,'<$<part>:$<empty>>')==='<ab:><:>X<a:><:>'",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<part>[\q{a|ab|abc}])|(?<empty>)/dvg;var clone=new RegExp(r);var m=r.exec('abc!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.part==='abc'&&m.groups.empty===undefined&&m.indices.groups.part[1]===3&&clone.exec('ab!').groups.part==='ab'&&r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn nonempty_word_root_proofs_finish_finalizers_and_keep_other_features_separate() {
    check(
        r"var marker=0,m;try{m=/([\q{ab}&&\q{cd}])|()/dv.exec('ab')}finally{marker=3}marker===3&&m[0]===''&&m[1]===undefined&&m[2]===''&&m.indices[2][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/([\q{a|ab}])|()/dv.exec('ab!')}finally{marker=3}marker===3&&m[0]==='ab'&&m[1]==='ab'&&m[2]===undefined&&m.indices[1][1]===2",
    );
    check(
        r"var marker=0,m;try{m=/([\q{a|ab}])|()/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===undefined&&m[2]===''&&m.indices[2][0]===0",
    );
    for pattern in [
        r"([\q{|a|ab}])\b|()",
        r"()|\b([\q{|ab}])",
        r"^([\q{ab}])|()",
        r"([\q{ab}])|()$",
        r"([\q{ab}])|()\1",
        r"([\q{ab}])x|()",
        r"([\q{ab}])?|()",
        r"([\q{ab}])|([\q{cd}])",
        r"([\q{ab}])|()|()",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    check(
        "var yes=0;for(var p of ['(?<x>[a])|(?<x>)(?<x>)','[\\q{ab}&&a--a]|']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}

#[test]
fn nonempty_word_root_deep_captures_long_members_and_opt_in_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            "{}[\\q{{a|ab|abc}}]{}|()",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('abc!');m.length===1002&&m[1]==='abc'&&m[1000]==='abc'&&m[1001]===undefined&&m.indices[1000][1]===3"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!("([\\q{{{word}}}])|()").as_str());
    let input = JsString::from(format!("{word}!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===20000&&m[1]===m[0]&&m[2]===undefined&&m.indices[1][1]===20000"
    ));
    let mut limited = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    limited.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        limited.eval(&format!(
            "try{{r=new RegExp({pattern:?},'v')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(limited.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
    let mut limited = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    limited.eval("var marker=0").unwrap();
    limited
        .eval(r"var r=/([\q{a|ab}])|()/vg;r.lastIndex=1")
        .unwrap();
    limited
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        limited.eval("try{Array.from(input.matchAll(r))}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        limited.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
