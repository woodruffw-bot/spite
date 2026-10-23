//! Complete ordinary wrappers around asserted single string-class captures.
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
fn enclosed_asserted_single_string_class_layouts_keep_lexical_complete_ranges() {
    let huge = 10u128.pow(30);
    for (class, _input, word) in [
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
        for (head, tail) in [("^", ""), ("", "$"), ("^", "$")] {
            for frame in 0..3 {
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
                    let body = format!("{head}{pattern}{tail}");
                    let pattern = JsString::from(
                        match frame {
                            0 => format!("(?<enclosing>{body})"),
                            1 => format!("(?:(?<enclosing>{body}))"),
                            _ => format!("(?<enclosing>(?:(?<middle>(?<inner>{body}))))"),
                        }
                        .as_str(),
                    );
                    let matched = word.is_some();
                    let whole = JsString::from(word.unwrap_or(""));
                    let end = 1 + whole.len();
                    let text = JsString::from(format!("\n{}", word.unwrap_or("!")).as_str());
                    let named = true;
                    check(&format!(
                        r#"
            var r=new RegExp({pattern:?},'dvgym');r.lastIndex=1;var m=r.exec({text:?}),ok=true;
            if({matched}){{
                if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
                        if(characters.indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
                        if(before.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if(after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        if(['enclosing','middle','inner'].indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
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
    }
}
#[test]
fn enclosed_asserted_single_string_class_values_consumers_and_scope_keep_prefix_slots() {
    check(
        r"var m=/(?<enclosing>(?:(?<before>)(?<part>[\q{a|ab}])(?<after>)\B))/dv.exec('ab!');m[0]==='a'&&m[1]==='a'&&m.groups.enclosing==='a'&&m.groups.part==='a'&&m.groups.before===''&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.after[0]===1",
    );
    check(
        r"var m=/(?:(?<enclosing>()(?<part>[\q{}])()$))/dv.exec('!ab');m.index===3&&m[1]===''&&m[2]===''&&m.groups.part===''&&m[4]===''&&m.indices.groups.enclosing[0]===3",
    );
    check(
        r"var m=/((?:^[\q{|ab}]$))/dv.exec('');m.length===2&&m[0]===''&&m[1]===''&&m.groups===undefined&&m.indices.groups===undefined",
    );
    check(
        r"var r=/(?<enclosing>(?:^()(?<part>[\q{😀|😀a}])()$))/dvgy;r.lastIndex=1;var m=r.exec('😀a');m[1]==='😀a'&&m.groups.enclosing==='😀a'&&m.index===0&&m.indices.groups.part[1]===3&&m.indices[4][0]===3&&r.lastIndex===3",
    );
    check(
        r"var m=/(?<\u{1D49C}>(?:\b(?<part>[\q{s|sa|k|ka}])\b))/dvi.exec('!ſa!');m.groups.𝒜==='ſa'&&m.groups.part==='ſa'&&Object.keys(m.groups).join(',')==='𝒜,part'&&m.indices.groups.𝒜[0]===1",
    );
    check(
        r"var r=/((?:^(?<before>)(?<part>[\q{a|ab}])(?<after>)$))/dvgm;var a=Array.from('ab\na\n!'.matchAll(r));a.length===2&&a.every(m=>m[1]===m[0]&&m.groups.part===m[0]&&m.groups.before===''&&m.groups.after==='')&&a[1].indices[1][1]===4&&r.lastIndex===0",
    );
    check(
        r"var calls=0;var out='ab\na\n!'.replace(/((?:^(?<before>)(?<part>[\q{a|ab}])(?<after>)$))/vgm,function(all,outer,b,p,a,offset,input,groups){calls++;return arguments.length===8&&all===outer&&all===p&&b===''&&a===''&&groups.part===p&&input==='ab\na\n!'?'#':'bad'});out==='#\n#\n!'&&calls===2",
    );
    check(
        r"var r=/(()([\q{a|ab}])()\B)/vg;r.lastIndex=9;var parts='ab!'.split(r);'!ab!'.search(r)===1&&r.lastIndex===9&&parts.length===6&&parts[0]===''&&parts[1]==='a'&&parts[2]===''&&parts[3]==='a'&&parts[4]===''&&parts[5]==='b!'&&'ab!'.replace(r,'<$1:$2:$3:$4>')==='<a::a:>b!'",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<enclosing>(()(?<part>[\q{|a|ab}])()$))/dvg,clone=new RegExp(r),m=r.exec('!ab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.enclosing==='ab'&&m.groups.part==='ab'&&m.index===1&&m.indices.groups.enclosing[1]===3&&clone.exec('!')[1]===''&&r.lastIndex===3"),Ok(Value::Boolean(true)));
    check(
        r"var marker=0,m;try{m=/((^()([\q{|a|ab}])()$))/dv.exec('')}finally{marker=3}marker===3&&m[1]===''&&m[2]===''&&m[3]===''&&m[4]===''&&m[5]===''&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/((^()([\q{a|ab}])()$)())/dv.exec('ab')}finally{marker=3}marker===3&&m.length===7&&m[0]==='ab'&&m[1]==='ab'&&m[2]==='ab'&&m[3]===''&&m[4]==='ab'&&m[5]===''&&m[6]===''&&m.indices[6][0]===2",
    );
    check(
        r"var marker=0,m;try{m=/(^()\b([\q{a|ab}])$)/dv.exec('ab')}finally{marker=3}marker===3&&m.length===4&&m[0]==='ab'&&m[1]==='ab'&&m[2]===''&&m[3]==='ab'&&m.indices[3][1]===2",
    );
    for pattern in [
        r"(^()([\q{a|ab}])|())",
        r"(()([\q{a|ab}])|()$)",
        r"(^()([\q{a|ab}])()$)+",
        r"((^()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()$)())",
        r"(^()\b([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])$)",
        r"(^()([\q{a|ab}])\1$)",
        r"(^()([\q{a|ab}])+()$)",
        r"(^()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()$)",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn enclosed_asserted_single_string_class_deep_prefixes_words_and_host_aborts_stay_flat() {
    let pattern = JsString::from(
        format!(
            "{}^()([\\q{{|a|ab}}])()${}",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),a=r.exec('ab'),e=r.exec('');a.length===1004&&a[1]==='ab'&&a[1000]==='ab'&&a[1001]===''&&a[1002]==='ab'&&a[1003]===''&&e[1]===''&&e[1000]===''&&e[1001]===''&&e[1002]===''&&e[1003]===''"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!("(?<enclosing>()(?<part>[\\q{{{word}}}])()$)").as_str());
    let input = JsString::from(format!("!!{word}").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m.index===2&&m.groups.enclosing.length===20000&&m.groups.part===m[0]&&m[2]===''&&m[4]===''&&m.indices.groups.enclosing[1]===20002"
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
        .eval(r"var r=/(()([\q{|a|ab}])()$)/vg;r.lastIndex=1")
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
}
