//! Conditional captures in quantified UnicodeSets empty branches (22.2.2.3).

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
fn empty_capture_effect_families_preserve_branch_priority_progress_and_named_capture_values() {
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
                for (empty_pattern, defined_names) in [
                    ("(?<empty>(?<deep>))?", ""),
                    ("(?:(?<empty>(?<deep>))?(?<tail>))", "tail"),
                    ("(?:(?<empty>(?<deep>))+?(?<tail>))", "empty|deep|tail"),
                    (
                        "(?:(?<empty>(?<first>)|(?<later>))(?<tail>))",
                        "empty|first|tail",
                    ),
                    ("(?:(?<empty>(?<first>)|(?<later>))*?(?<tail>))", "tail"),
                    (
                        "(?:(?<empty>(?<first>)|(?<later>)){2}(?<tail>))",
                        "empty|first|tail",
                    ),
                ] {
                    let branch = if empty_first {
                        format!("{empty_pattern}|(?<part>{class})")
                    } else {
                        format!("(?<part>{class})|{empty_pattern}")
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
                    let selected = captures[slot] == Some("");
                    let definitions = JsString::from(defined_names);
                    let empty = format!(
                        "keys.filter(name=>name!=='whole' && name!=='each' && name!=='part').every(name=>({selected} && {definitions:?}.split('|').includes(name)) ? (m.groups[name]==='' && m[keys.indexOf(name)+1]==='' && m.indices.groups[name]===m.indices[keys.indexOf(name)+1] && m.indices.groups[name][0]==={length} && m.indices.groups[name][1]==={length}) : (m.groups[name]===undefined && m[keys.indexOf(name)+1]===undefined && m.indices.groups[name]===undefined && m.indices[keys.indexOf(name)+1]===undefined && Object.hasOwn(m.groups,name)))"
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
fn empty_capture_effects_keep_required_optional_choice_and_empty_only_layouts_distinct() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let m=/(?<each>(?<part>{class})|(?<empty>(?<deep>)?)){{3}}/dv.exec('abX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===2 && m.indices.groups.deep===undefined"
        ));
        check(&format!(
            "let m=/(?:{class}|(?<empty>(?<deep>))?){{3}}/dv.exec('ab');m.length===3 && m[0]==='ab' && m.groups.empty===undefined && m.groups.deep===undefined && Object.keys(m.groups).join('|')==='empty|deep' && Object.hasOwn(m.groups,'empty') && Object.getPrototypeOf(m.groups)===null && m.indices[1]===undefined && m.indices[2]===undefined"
        ));
        check(&format!(
            "let m=/(?<each>(?<empty>(?<first>)|(?<later>)){{2}}|(?<part>{class})){{2}}/dv.exec('ab');m[0]==='' && m.groups.each==='' && m.groups.empty==='' && m.groups.first==='' && m.groups.later===undefined && m.groups.part===undefined && m.indices.groups.first[0]===0 && m.indices.groups.later===undefined"
        ));
    }
}

#[test]
fn conditional_empty_captures_keep_original_utf16_ranges_names_flags_and_sticky_state() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>(?<part>{class})|(?<empty>(?<deep>)?)){{3}})/dvgy;r.lastIndex=2;let m=r.exec('A😀😀x');m.index===1 && m[0]==='😀😀' && m.groups.whole==='😀😀' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===5 && m.indices.groups.empty[1]===5 && m.indices.groups.deep===undefined && r.lastIndex===5"
        ));
    }
    check(
        r"let source='(?<each>[\\q{s|k}]|(?<\\u{1D49C}>(?<first>)|(?<later>))){3}',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && m[0]==='ſK' && m.groups['𝒜']==='' && m.groups.first==='' && m.groups.later===undefined && m.indices.groups['𝒜']===m.indices[2] && m.indices[2][0]===2 && m.indices.groups.later===undefined",
    );
    check(
        r"let m=/(?<each>(?<part>[\q{\uD800a|\uD800}&&\p{Any}])|(?<empty>(?<deep>)?)){3}/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===2",
    );
    check(
        r"let r=/(?:[\q{ab}&&a]|(?<empty>)?)+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn conditional_empty_capture_consumers_callbacks_clones_and_collection_keep_participation() {
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?:(?<empty>(?<first>)|(?<later>))+?(?<tail>))){3}/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a.every(m=>m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.first==='' && m.groups.later===undefined && m.groups.tail==='' && m.indices.groups.later===undefined) && 'abXba'.replace(r,'<$<empty>:$<later>>')==='<:><:>X<:><:>' && r.lastIndex===0",
    );
    check(
        r"let seen=[],out='abX'.replace(/(?<each>(?<part>[\q{a|b}])|(?:(?<empty>(?<first>)|(?<later>))+?(?<tail>))){3}/vg,(m,each,part,empty,first,later,tail,at,input,groups)=>{seen.push([each,part,empty,first,later,tail,at,groups.later]);return '#'});out==='##X#' && seen.length===3 && seen.every(row=>row[0]==='' && row[1]===undefined && row[2]==='' && row[3]==='' && row[4]===undefined && row[5]==='' && row[7]===undefined) && seen[0][6]===0 && seen[1][6]===2",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)?)+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && 'ab'.split(r).join('|')==='|b|b||'",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?<empty>)?)+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && m[0]==='' && m.groups.each==='' && m.groups.empty===undefined && m.groups.part===undefined && c.lastIndex===0 && r.exec('ABB').groups.part==='B' && r.lastIndex===7",
    );
    check(
        r"let r=/(?:[\q{a}]|(?<empty>)?)+/dvg,a=[...'😀'.matchAll(r)];a.length===2 && a[0].index===0 && a[1].index===2 && a.every(m=>m.groups.empty===undefined && Object.hasOwn(m.groups,'empty') && m.indices.groups.empty===undefined) && r.lastIndex===0",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>(?<part>[\q{a|b}])|(?<empty>(?<deep>)?)){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.groups.empty==='' && m.groups.deep===undefined && m.indices.groups.empty[0]===3 && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn complete_empty_capture_effects_finish_finalizers_and_keep_other_terms_deferred() {
    for source in [
        r"/(?:[\q{a}]|(?:|()))+/v.test('a')",
        r"/([\q{a|ab}&&a]|()?)+/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:|())){99999999999999999999999999999999999999991,99999999999999999999999999999999999999992}/v.test('a')",
        r"/([\q{a|ab}&&a]|(?:|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a|b}]|()?){0}/v.test('a')",
        r"/([\q{a|b}]|()?){1,}/v.test('a')",
        r"/([\q{a|b}]|()?){1}/v.test('a')",
        r"/([\q{a|b}]|()?){2,3}/v.test('a')",
        r"/([\q{a|b}]|()?){2,}/v.test('a')",
        r"/([\q{a|b}]|()?){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|()))+/v.test('a')",
        r"/([\q{a|b}]|(?:|())){0}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){1,}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){1}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,3}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2,}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){2}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999998,999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999999,9999999999999999999999999999999999999990}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
        r"/([\q{a|b}]|(?:|())){99999999999999999999999999999999999999992,99999999999999999999999999999999999999993}/v.test('a')",
        r"/([\q{a}]|()?)+/v.test('a')",
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
    for inner in [
        r"([\q{a|b}])|(()|)",
        r"([\q{a|b}])|()*",
        r"([\q{a|b}])|()?",
        r"[\q{a|b}]|()?",
        r"[\q{a|b}]|(?:)()*",
        r"[\q{a|b}]|(?:|())",
    ] {
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
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{ab}&&\\q{cd}]|()?)+?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for (inner, expected_zero) in [
        (
            r"[\q{a|b}]|\b()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|(?=())",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|\1()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{a|b}]|x()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])(\b)|()?",
            r#"[["","<undefined>","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])+|()?",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|()?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
    ] {
        for count in ["+?", "{0}", "{2,3}", "{999999999999999999999999999999}"] {
            let source = format!("({inner}){count}");
            if count == "{0}" {
                let pattern = JsString::from(source.as_str());
                let expected = JsString::from(expected_zero);
                check(&format!(
                    "var pattern={pattern:?},input='a😀b',r=new RegExp(pattern,'dvgy');r.lastIndex=2;var m=r.exec(input);r.source===pattern&&r.flags==='dgvy'&&m.input===input&&Object.hasOwn(m,'groups')&&Object.hasOwn(m.indices,'groups')&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify([Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,r.lastIndex])==={expected:?}"
                ));
                continue;
            }
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
        r"([^\q{ab}]|()?){0}",
        r"([\q{ab}&&a--a]|()?){2,3}",
        r"([\q{\d}&&a]|()?){1,}",
        r"([\q{a|b}]|()?){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn conditional_empty_capture_layouts_huge_counts_and_host_aborts_stay_flat_and_atomic() {
    let pattern = format!(
        r"(([\q{{a|b}}])|{}{}?){{3}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let m=/{pattern}/dv.exec('ab');m.length===1003 && m[0]==='ab' && m[1]==='' && m[2]===undefined && m[3]===undefined && m[1002]===undefined && m.indices[1][0]===2 && m.indices[1002]===undefined"
    ));
    let pattern = format!(r"(([\q{{{}}}&&a])|()?)+", "x".repeat(20_000));
    check(&format!(
        "let m=/{pattern}/dv.exec('😀');m[0]==='' && m[1]==='' && m[2]===undefined && m[3]===undefined && m.indices[3]===undefined"
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
        let pattern = format!(r"(([\q{{a|b}}])|{inner}){{3}}");
        let captures = if defined {
            "m[3]==='' && m[4]==='' && m.indices[3][0]===2 && m.indices[4][1]===2"
        } else {
            "m[3]===undefined && m[4]===undefined && m.indices[3]===undefined && m.indices[4]===undefined"
        };
        check(&format!(
            "let r=/{pattern}/dv,m=r.exec('ab');r.source.length>20000 && m.length===5 && m[0]==='ab' && m[1]==='' && m[2]===undefined && {captures}"
        ));
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm.eval(r"var r=/([\q{a}]|()?)+$/vg").unwrap();
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
