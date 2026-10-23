//! Canonical outside assertions around single string-class captures.
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
fn asserted_single_string_class_layouts_keep_complete_and_point_ranges() {
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
                let pattern = JsString::from(format!("{head}{pattern}{tail}").as_str());
                let matched = word.is_some();
                let whole = JsString::from(word.unwrap_or(""));
                let end = 1 + whole.len();
                let text = JsString::from(format!("\n{}", word.unwrap_or("!")).as_str());
                let named = layout != 0;
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
#[test]
fn asserted_single_string_class_retries_shorter_words_and_rejected_empty_starts() {
    check(
        r"var m=/()([\q{a|ab}])()\B/dv.exec('ab!');m[0]==='a'&&m[1]===''&&m[2]==='a'&&m[3]===''&&m.indices[2][1]===1&&m.indices[3][0]===1",
    );
    check(
        r"var m=/()([\q{|ab}])()$/dv.exec('!ab');m[0]==='ab'&&m.index===1&&m[1]===''&&m[2]==='ab'&&m[3]===''&&m.indices[1][0]===1&&m.indices[3][0]===3",
    );
    check(
        r"var m=/()([\q{}])()$/dv.exec('!ab');m[0]===''&&m.index===3&&m[1]===''&&m[2]===''&&m[3]===''&&m.indices[2][0]===3",
    );
    check(
        r"var r=/^()([\q{|ab}])()$/dvg;r.lastIndex=1;var n=r.exec('ab');var m=r.exec('ab');n===null&&m[0]==='ab'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"var m=/^()([\q{|ab}])()$/dvm.exec('!\r\nab\r\n');m[0]===''&&m.index===2&&m[1]===''&&m[2]===''&&m.indices[3][0]===2",
    );
    check(
        r"var r=/^()([\q{😀|😀a}])()$/dvgy;r.lastIndex=1;var m=r.exec('😀a');m[0]==='😀a'&&m.index===0&&m.indices[1][0]===0&&m.indices[3][0]===3&&r.lastIndex===3",
    );
    check(
        r"var m=/\b(?<before>)(?<\u{1D49C}>[\q{s|sa|k|ka}])(?<after>)\b/dvi.exec('!ſa!');m.groups.𝒜==='ſa'&&m.index===1&&m.groups.before===''&&m.indices.groups.after[0]===3",
    );
    check(
        r"var r=/()([\q{}])()$/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    // Spec regression: failed Unicode search candidates never enter a pair.
    // V8's nonsticky \B search chooses the interior code unit here; control
    // transcripts instead iterate native sticky matches at code-point starts.
    let pattern = JsString::from(r"()(?<part>[\q{|a|ab}])()\B");
    for (flags, expected) in [
        (
            "dvg",
            "m!==null&&m.index===5&&m[0]===''&&m[1]===''&&m.groups.part===''&&m[3]===''&&m.indices.groups.part[0]===5&&m.indices[3][0]===5&&r.lastIndex===5",
        ),
        ("dvgy", "m===null&&r.lastIndex===0"),
    ] {
        check(&format!(
            "var r=new RegExp({pattern:?},'{flags}'),ok=true;for(var start of [2,3,4]){{r.lastIndex=start;var m=r.exec('😀a😀!');ok=ok&&({expected})}}ok"
        ));
    }
    check(&format!(
        "var r=new RegExp({pattern:?},'dvg');var all=Array.from('😀a😀!'.matchAll(r));all.map(m=>m.index).join(',')==='0,5,6'&&all.every(m=>m[1]===''&&m.groups.part===''&&m[3]===''&&m.indices.groups.part[0]===m.index)&&r.lastIndex===0"
    ));
}

#[test]
fn asserted_single_string_class_consumers_keep_points_names_and_unicode_progress() {
    check(
        r"var r=/^(?<before>)(?<part>[\q{a|ab}])(?<after>)$/dvgm;var a=Array.from('ab\na\n!'.matchAll(r));a.length===2&&a.map(m=>m[0]).join('|')==='ab|a'&&a.every(m=>m.groups.part===m[0]&&m.groups.before===''&&m.groups.after==='')&&a[1].indices.groups.after[0]===4&&r.lastIndex===0",
    );
    check(
        r"var a=Array.from('😀a'.matchAll(/()([\q{}])()$/dvg));a.length===1&&a[0].index===3&&a[0][1]===''&&a[0][2]===''&&a[0][3]===''",
    );
    check(
        r"var calls=0;var out='ab\na\n!'.replace(/^(?<before>)(?<part>[\q{a|ab}])(?<after>)$/vgm,function(all,b,p,a,offset,input,groups){calls++;return arguments.length===7&&all===p&&b===''&&a===''&&groups.part===p&&input==='ab\na\n!'?'#':'bad'});out==='#\n#\n!'&&calls===2",
    );
    check(
        r"var r=/()([\q{a|ab}])()\B/vg;r.lastIndex=9;var parts='ab!'.split(r);'!ab!'.search(r)===1&&r.lastIndex===9&&parts.length===5&&parts[0]===''&&parts[1]===''&&parts[2]==='a'&&parts[3]===''&&parts[4]==='b!'&&'ab!'.replace(r,'<$1:$2:$3>')==='<:a:>b!'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/()(?<part>[\q{|a|ab}])()$/dvg,clone=new RegExp(r),m=r.exec('!ab')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.part==='ab'&&m.index===1&&m.indices[1][0]===1&&m.indices[3][0]===3&&clone.exec('!')[2]===''&&r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn asserted_single_string_class_proofs_finish_finalizers_and_keep_inner_scope_closed() {
    check(
        r"var marker=0,m;try{m=/^()([\q{a|ab}])/dv.exec('ab')}finally{marker=3}marker===3&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m.indices[2][1]===2",
    );
    check(
        r"var marker=0,m;try{m=/()([\q{|ab}])$/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m.index===1&&m[1]===''&&m[2]===''&&m.indices[2][0]===1",
    );
    check(
        r"var marker=0,m;try{m=/^()\b([\q{a|ab}])$/dv.exec('ab')}finally{marker=3}marker===3&&m.length===3&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m.indices[2][1]===2&&/^()\b([\q{a|ab}])$/v.exec('ab!')===null",
    );
    check(
        r"var marker=0,m;try{m=/()([\q{|ab}])\b()$/dv.exec('!ab')}finally{marker=3}marker===3&&m.length===4&&m.index===1&&m[0]==='ab'&&m[1]===''&&m[2]==='ab'&&m[3]===''&&m.indices[3][0]===3&&/()([\q{|ab}])\b()$/v.exec('!')===null",
    );
    for pattern in [
        r"^()\b([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])$",
        r"()([\q{|ab}&&\p{Any}&&\p{RGI_Emoji}])\b()$",
        r"^()([\q{a|ab}])\1$",
        r"^()([\q{a|ab}])+()$",
        r"^()([\q{|ab}]){0}()$",
        r"^()([\q{a|ab}])x$",
        r"^()([\q{|ab}])([\q{cd}])$",
        r"^()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()$",
        r"^()([\q{a|ab}])|()",
        r"()([\q{a|ab}])|()$",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn asserted_single_string_class_deep_captures_long_words_and_opt_in_search_aborts_stay_flat() {
    let pattern = JsString::from(
        format!(
            "^(){}[\\q{{|a|ab}}]{}()$",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),a=r.exec('ab'),e=r.exec('');a.length===1003&&a[1]===''&&a[2]==='ab'&&a[1001]==='ab'&&a[1002]===''&&e[1]===''&&e[2]===''&&e[1001]===''&&e[1002]===''"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!("^()([\\q{{{word}}}])()$").as_str());
    let input = JsString::from(word.as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m.index===0&&m[1]===''&&m[2].length===20000&&m[3]===''&&m.indices[3][0]===20000"
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
        .eval(r"var r=/()([\q{|a|ab}])()$/vg;r.lastIndex=1")
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
