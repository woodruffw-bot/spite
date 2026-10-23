//! Empty point captures beside nullable UnicodeSets alternatives (22.2.2.3).

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
fn boundary_capture_families_preserve_branch_priority_progress_and_named_capture_values() {
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
                for layout in 0..6 {
                    let (prefix, character, empty_pattern, suffix) = match layout {
                        0 => ("", format!("(?<part>{class})"), "(?<empty>)", "(?<after>)"),
                        1 => ("(?<before>)", format!("(?<part>{class})"), "(?<empty>)", ""),
                        2 => ("", format!("(?<left>)(?<part>{class})"), "(?<empty>)", ""),
                        3 => ("", format!("(?<part>{class})(?<right>)"), "(?<empty>)", ""),
                        4 => (
                            "(?<before>)",
                            format!("(?<left>)(?<part>{class})(?<right>)"),
                            "(?<empty>(?<first>)|(?<later>))",
                            "(?<after>)",
                        ),
                        _ => (
                            "(?<before>)?",
                            format!("(?<left>(?<leftDeep>)?)(?<part>{class})(?<right>)"),
                            "(?<empty>(?<emptyDeep>)?)",
                            "(?<after>)+?",
                        ),
                    };
                    let branch = if empty_first {
                        format!("{empty_pattern}|{character}")
                    } else {
                        format!("{character}|{empty_pattern}")
                    };
                    let pattern = format!("(?<whole>(?<each>{prefix}(?:{branch}){suffix}){count})");
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
                    let names = if captures[slot].is_some_and(|value| !value.is_empty()) {
                        match layout {
                            0 => "after",
                            1 => "before",
                            2 => "left",
                            3 => "right",
                            4 => "before|left|right|after",
                            _ => "left|right|after",
                        }
                    } else if captures[slot] == Some("") {
                        match layout {
                            0 => "empty|after",
                            1 => "before|empty",
                            2 | 3 => "empty",
                            4 => "before|empty|first|after",
                            _ => "empty|after",
                        }
                    } else {
                        ""
                    };
                    let definitions = JsString::from(names);
                    let start =
                        length - usize::from(captures[slot].is_some_and(|value| !value.is_empty()));
                    let empty = format!(
                        "keys.filter(name=>name!=='whole' && name!=='each' && name!=='part').every(name=>{definitions:?}.split('|').includes(name) ? (m.groups[name]==='' && m[keys.indexOf(name)+1]==='' && m.indices.groups[name]===m.indices[keys.indexOf(name)+1] && m.indices.groups[name][0]===((name==='before'||name==='left')?{start}:{length}) && m.indices.groups[name][1]===m.indices.groups[name][0]) : (m.groups[name]===undefined && m[keys.indexOf(name)+1]===undefined && m.indices.groups[name]===undefined && m.indices[keys.indexOf(name)+1]===undefined && Object.hasOwn(m.groups,name)))"
                    );
                    check(&format!(
                        "let source={source:?},r=new RegExp(source,'dvg'),m=r.exec('aaaX');let keys=Object.keys(m.groups),partSlot=keys.indexOf('part')+1,emptySlot=keys.indexOf('empty')+1;r.source===source && r.flags==='dgv' && m.length===keys.length+1 && m.index===0 && m[0]==={whole:?} && m.groups.whole==={whole:?} && m.indices[1][0]===0 && m.indices[1][1]==={length} && r.lastIndex==={length} && {capture} && {part} && {empty}"
                    ));
                }
            }
        }
    }
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
    for pattern in [
        r"(?:(?:[\q{a}]|)\b())+",
        r"(?:(?:[\q{a}]|)\1())+",
        r"(?:(?:[\q{a}]|)|())+",
        r"(?:(?:[\q{ab}&&\q{cd}]|)()){0}",
        r"(?:()(?:|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("/{pattern}/v.test('a')")),
                Err(Error::Unsupported { .. })
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

#[test]
fn nullable_boundary_capture_layouts_counts_and_host_aborts_stay_flat_and_atomic() {
    let prefix = format!("{}{}", "(".repeat(1_000), ")".repeat(1_000));
    let pattern = format!(r"(?:{prefix}(?:[\q{{a|b}}]|)){{3}}");
    check(&format!(
        "let m=/{pattern}/dv.exec('ab');m.length===1001 && m[0]==='ab' && m[1]==='' && m[1000]==='' && m.indices[1][0]===2 && m.indices[1000][1]===2"
    ));
    let pattern = format!(r"(?:()(?:[\q{{{}}}&&a]|)())+", "x".repeat(20_000));
    check(&format!(
        "let m=/{pattern}/dv.exec('😀');m[0]==='' && m[1]==='' && m[2]==='' && m.indices[1][0]===0 && m.indices[2][1]===0"
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
    let huge = "9".repeat(20_000);
    for (inner, defined) in [
        (format!("(()){{0,{huge}}}"), false),
        (format!("(()){{{huge}}}"), true),
    ] {
        let pattern = format!(r"(?:{inner}(?:[\q{{a|b}}]|)()){{2,3}}");
        let captures = if defined {
            "m[1]==='' && m[2]==='' && m.indices[1][0]===2 && m.indices[2][0]===2"
        } else {
            "m[1]===undefined && m[2]===undefined && m.indices[1]===undefined && m.indices[2]===undefined"
        };
        check(&format!(
            "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m.length===4 && m[0]==='aaa' && m[3]==='' && m.indices[3][0]===3 && {captures}"
        ));
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm.eval(r"var r=/(?:(?:[\q{a}]|)())+$/vg").unwrap();
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
