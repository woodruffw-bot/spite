//! Captures confined to a nullable UnicodeSets empty branch (22.2.2.3).

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
fn empty_branch_capture_families_preserve_branch_priority_progress_and_named_capture_values() {
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
    ] {
        for (count, wholes, captures) in [
            ("*".to_owned(), ["aaa", "aaa"], [Some("a"), Some("a")]),
            ("*?".to_owned(), ["", ""], [None, None]),
            ("+".to_owned(), ["aaa", "aaa"], [Some("a"), Some("a")]),
            ("+?".to_owned(), ["", "a"], [Some(""), Some("a")]),
            ("?".to_owned(), ["a", "a"], [Some("a"), Some("a")]),
            ("??".to_owned(), ["", ""], [None, None]),
            ("{0}".to_owned(), ["", ""], [None, None]),
            ("{00,003}?".to_owned(), ["", ""], [None, None]),
            (
                format!("{{0,{huge}}}"),
                ["aaa", "aaa"],
                [Some("a"), Some("a")],
            ),
            ("{000,}".to_owned(), ["aaa", "aaa"], [Some("a"), Some("a")]),
            ("{1}".to_owned(), ["", "a"], [Some(""), Some("a")]),
            ("{01,001}?".to_owned(), ["", "a"], [Some(""), Some("a")]),
            ("{1,}".to_owned(), ["aaa", "aaa"], [Some("a"), Some("a")]),
            ("{2}".to_owned(), ["", "aa"], [Some(""), Some("a")]),
            ("{02,002}?".to_owned(), ["", "aa"], [Some(""), Some("a")]),
            ("{2,4}".to_owned(), ["aa", "aaa"], [Some("a"), Some("a")]),
            ("{02,004}?".to_owned(), ["", "aa"], [Some(""), Some("a")]),
            ("{3,}?".to_owned(), ["", "aaa"], [Some(""), Some("a")]),
            (format!("{{1,{huge}}}?"), ["", "a"], [Some(""), Some("a")]),
            (
                format!("{{{limit},{}}}", limit + 1),
                ["a", "aaa"],
                [Some("a"), Some("")],
            ),
            (
                format!("{{{limit},{}}}?", limit + 1),
                ["", "aaa"],
                [Some(""), Some("")],
            ),
            (
                format!("{{{huge},{}}}", huge + 1),
                ["a", "aaa"],
                [Some("a"), Some("")],
            ),
            (
                format!("{{{huge},{}}}?", huge + 1),
                ["", "aaa"],
                [Some(""), Some("")],
            ),
            (format!("{{{huge}}}?"), ["", "aaa"], [Some(""), Some("")]),
            (
                format!("{{000{huge},0{huge}}}"),
                ["", "aaa"],
                [Some(""), Some("")],
            ),
            (format!("{{{huge},}}?"), ["", "aaa"], [Some(""), Some("")]),
            (
                format!("{{{huge},{}}}?", huge * 100),
                ["", "aaa"],
                [Some(""), Some("")],
            ),
        ] {
            for (empty_first, slot) in [(true, 0), (false, 1)] {
                let branch = if empty_first {
                    format!("(?<empty>)|(?<part>{class})")
                } else {
                    format!("(?<part>{class})|(?<empty>)")
                };
                let pattern = format!("(?<whole>(?<each>{branch}){count})");
                let source = JsString::from(pattern.as_str());
                let whole = JsString::from(wholes[slot]);
                let length = wholes[slot].len();
                let capture = if let Some(value) = captures[slot] {
                    let value = JsString::from(value);
                    let start = length - usize::from(!value.is_empty());
                    format!(
                        "m[2]==={value:?} && m.groups.each==={value:?} && m.indices[2][0]==={start} && m.indices.groups.each[1]==={length}"
                    )
                } else {
                    "m[2]===undefined && m.groups.each===undefined && m.indices[2]===undefined && m.indices.groups.each===undefined && Object.hasOwn(m.groups,'each')".to_owned()
                };
                let part = if captures[slot].is_some_and(|value| !value.is_empty()) {
                    let value = JsString::from(captures[slot].unwrap());
                    let start = length - 1;
                    format!(
                        "m[partSlot]==={value:?} && m.groups.part==={value:?} && m.indices.groups.part===m.indices[partSlot] && m.indices[partSlot][0]==={start} && m.indices[partSlot][1]==={length}"
                    )
                } else {
                    "m[partSlot]===undefined && m.groups.part===undefined && m.indices[partSlot]===undefined && m.indices.groups.part===undefined && Object.hasOwn(m.groups,'part')".to_owned()
                };
                let empty = if captures[slot] == Some("") {
                    format!(
                        "m[emptySlot]==='' && m.groups.empty==='' && m.indices.groups.empty===m.indices[emptySlot] && m.indices[emptySlot][0]==={length} && m.indices[emptySlot][1]==={length}"
                    )
                } else {
                    "m[emptySlot]===undefined && m.groups.empty===undefined && m.indices[emptySlot]===undefined && m.indices.groups.empty===undefined && Object.hasOwn(m.groups,'empty')".to_owned()
                };
                check(&format!(
                    "let source={source:?},r=new RegExp(source,'dvg'),m=r.exec('aaaX');let keys=Object.keys(m.groups),partSlot=keys.indexOf('part')+1,emptySlot=keys.indexOf('empty')+1;r.source===source && r.flags==='dgv' && m.length===5 && m.index===0 && m[0]==={whole:?} && m.groups.whole==={whole:?} && m.indices[1][0]===0 && m.indices[1][1]==={length} && r.lastIndex==={length} && {capture} && {part} && {empty}"
                ));
            }
        }
    }
}

#[test]
fn nested_sibling_and_empty_only_captures_keep_selected_branch_participation() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<each>(?<part>{class})|(?:(?<empty>(?<deep>))(?<sibling>))){{3}}/dv.exec('abX'),b=/(?<each>(?<part>{class})|(?:(?<empty>(?<deep>))(?<sibling>))){{2}}/dv.exec('abX');a.length===6 && a[0]==='ab' && a.groups.each==='' && a.groups.part===undefined && a.indices[2]===undefined && ['empty','deep','sibling'].every(k=>a.groups[k]==='' && a.indices.groups[k][0]===2 && a.indices.groups[k][1]===2) && b.groups.part==='b' && ['empty','deep','sibling'].every(k=>b.groups[k]===undefined && b.indices.groups[k]===undefined)"
        ));
        check(&format!(
            "let a=/(?:{class}|(?<empty>)){{3}}/dv.exec('ab'),b=/(?:(?<empty>)|{class}){{2,3}}/dv.exec('ab');a.length===2 && a[0]==='ab' && a.groups.empty==='' && a.indices[1][0]===2 && b[0]==='a' && b.groups.empty===undefined && b.indices[1]===undefined"
        ));
    }
    check(
        r"let a=/(?<each>(?<empty>)|(?<part>[\q{a|b}])){2}/dv.exec('ab');Object.keys(a.groups).join('|')==='each|empty|part' && a[0]==='' && a.groups.each==='' && a.groups.empty==='' && a.groups.part===undefined && a.indices[2][0]===0",
    );
}

#[test]
fn empty_capture_utf16_ranges_names_source_and_sticky_state_remain_original() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<part>{class})|(?<empty>)){{3}})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.whole==='😀😀' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===5 && m.indices.groups.empty[1]===5 && r.lastIndex===5"
        ));
    }
    check(
        r"let source='(?<each>[\\q{s|k}]|(?<\\u{1D49C}>)){3}',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && m[0]==='ſK' && m.groups['𝒜']==='' && m.indices.groups['𝒜']===m.indices[2] && m.indices[2][0]===2",
    );
    check(
        r"let m=/(?<each>(?<part>[\q{\uD800a|\uD800}&&\p{Any}])|(?<empty>)){3}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===2",
    );
    check(
        r"let r=/(?:[\q{ab}&&a]|(?<empty>))+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn empty_capture_string_consumers_callbacks_clones_and_collection_preserve_slots() {
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>))+/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a[0].groups.part==='b' && a[0].groups.empty===undefined && a[1].groups.part===undefined && a[1].groups.empty==='' && a[1].indices.groups.empty[0]===2 && 'abXba'.replace(r,'<$2:$<empty>>')==='<b:><:>X<a:><:>' && r.lastIndex===0",
    );
    check(
        r"let seen=[],out='abX'.replace(/(?<each>(?<part>[\q{a|b}])|(?<empty>)){3}/vg,(m,each,part,empty,at,input,groups)=>{seen.push([each,part,empty,at,groups.empty]);return '#'});out==='##X#' && seen.length===3 && seen.every(row=>row[0]==='' && row[1]===undefined && row[2]==='' && row[4]==='') && seen[0][3]===0 && seen[1][3]===2",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>))+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && 'ab'.split(r).join('|')==='|b|b||'",
    );
    check(
        r"let r=/(?<each>[\q{a|b}]|(?<empty>))+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && m.groups.empty==='' && c.lastIndex===0 && r.exec('ABB').groups.empty===undefined && r.lastIndex===7",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.indices.groups.empty[0]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_empty_capture_proofs_finish_finalizers_and_keep_effects_deferred() {
    for source in [
        r"/([\q{a|ab}&&a]|())+/v.test('a')",
        r"/([\q{a|b}]|()){0}/v.test('a')",
        r"/([\q{a|b}]|()){1,}/v.test('a')",
        r"/([\q{a|b}]|()){1}/v.test('a')",
        r"/([\q{a|b}]|()){2,3}/v.test('a')",
        r"/([\q{a|b}]|()){2,}/v.test('a')",
        r"/([\q{a|b}]|()){2}/v.test('a')",
        r"/([\q{a}]|())+/v.test('a')",
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
    for inner in [r"([\q{a|b}])|()", r"[\q{a|b}]|()"] {
        for count in [
            "+?",
            "{0}",
            "{2,3}",
            "{999999999999999999999999999999,1000000000000000000000000000000}",
        ] {
            check(&format!(
                "let marker=0,value;try{{value=/({inner}){count}/v.test('a')}}finally{{marker=3}}value && marker===3"
            ));
        }
    }
    for inner in [
        r"([\q{a|b}])|(\b)*",
        r"([\q{a|b}])|(\b()|)",
        r"([\q{a|b}])()|()",
        r"([\q{a|b}])|\b()",
        r"([\q{a|b}])+|()",
        r"|([\q{ab}&&\q{cd}])()",
        r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|()",
    ] {
        for count in ["+?", "{0}", "{2,3}", "{999999999999999999999999999999}"] {
            let source = format!("({inner}){count}");
            assert!(
                matches!(
                    Realm::default().eval(&format!("/{source}/v.test('a')")),
                    Err(Error::Unsupported { .. })
                ),
                "{source}"
            );
        }
    }
    for pattern in [
        r"(([^\q{ab}])|()){0}",
        r"(([\q{ab}&&a--a])|()){2,3}",
        r"(([\q{\d}&&a])|()){1,}",
        r"(([\q{a|b}])|()){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn empty_capture_layouts_huge_counts_and_opt_in_aborts_stay_flat_and_atomic() {
    let pattern = format!(
        r"(([\q{{a|b}}])|{}){{999999999999999999999999999999,1000000000000000000000000000000}}",
        "()".repeat(1_000)
    );
    check(&format!(
        "let m=/{pattern}/dv.exec('aaa');m.length===1003 && m[0]==='aaa' && m[1]==='' && m[2]===undefined && m[3]==='' && m[1002]==='' && m.indices[1002][0]===3"
    ));
    let pattern = format!(r"(([\q{{{}}}&&a])|())+", "x".repeat(20_000));
    check(&format!(
        "let m=/{pattern}/dv.exec('😀');m[0]==='' && m[1]==='' && m[2]===undefined && m[3]==='' && m.indices[3][0]===0"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=/{pattern}/vg}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7 && marker===0"), Ok(Value::Boolean(true)));
    let pattern = format!(r"(([\q{{a|b}}])|()){{{}2,3}}", "0".repeat(20_000));
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='aaa' && m[1]==='a' && m[2]==='a' && m[3]===undefined && m.indices[2][0]===2"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm.eval(r"var r=/(([\q{a}])|())+$/vg").unwrap();
    realm
        .eval(&format!("var text='{}';r.lastIndex=1", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("r.lastIndex===1 && marker===0"),
        Ok(Value::Boolean(true))
    );
}
