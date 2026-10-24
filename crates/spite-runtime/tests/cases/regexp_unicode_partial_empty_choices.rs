//! Character captures beside effect-free noncapturing UnicodeSets empty choices (22.2.2.3).

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
fn partial_choice_capture_families_preserve_branch_priority_progress_and_named_capture_values() {
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
                    format!(
                        "(?:(?:(?:(?:|)*?)(?:|)+?|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)*?)(?:(?:(?:|)*?)(?:|)+?|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+?|(?<part>{class})"
                    )
                } else {
                    format!(
                        "(?<part>{class})|(?:(?:(?:(?:|)*?)(?:|)+?|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)*?)(?:(?:(?:|)*?)(?:|)+?|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+?"
                    )
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
                        "m[3]==={value:?} && m.groups.part==={value:?} && m.indices.groups.part===m.indices[3] && m.indices[3][0]==={start} && m.indices[3][1]==={length}"
                    )
                } else {
                    "m[3]===undefined && m.groups.part===undefined && m.indices[3]===undefined && m.indices.groups.part===undefined && Object.hasOwn(m.groups,'part')".to_owned()
                };
                check(&format!(
                    "let source={source:?},r=new RegExp(source,'dvg'),m=r.exec('aaaX');r.source===source && r.flags==='dgv' && m.length===4 && m.index===0 && m[0]==={whole:?} && m.groups.whole==={whole:?} && m.indices[1][0]===0 && m.indices[1][1]==={length} && r.lastIndex==={length} && {capture} && {part}"
                ));
            }
        }
    }
}

#[test]
fn required_empty_iterations_clear_character_groups_after_earlier_characters() {
    for class in [r"[\q{a|b}]", r"[\q{a|b|ab}&&\p{Any}]"] {
        check(&format!(
            "let a=/(?<whole>(?<each>(?<x>(?<y>{class}))|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){{3}})/dv.exec('abX'),b=/(?<each>(?<part>{class})|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){{2}}/dv.exec('abX'),c=/(?<each>(?:(?:|)*?)(?:|)+?|(?<part>{class})){{1}}/dv.exec('ab');a.length===5 && a[0]==='ab' && a.groups.whole==='ab' && a.groups.each==='' && a.groups.x===undefined && a.groups.y===undefined && a.indices[2][0]===2 && a.indices[3]===undefined && a.indices.groups.y===undefined && b.groups.each==='b' && b.groups.part==='b' && b.indices.groups.part[0]===1 && c.groups.each==='' && c.groups.part===undefined && c.indices[2]===undefined"
        ));
    }
    check(
        r"let a=/(?:(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){3}/dv.exec('ab'),b=/(?:(?:(?:|)*?)(?:|)+?|(?<part>[\q{a|b}]))+?/dv.exec('ab'),c=/(?<each>(?<part>[\q{ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)*/dv.exec('ab');a[0]==='ab' && a.groups.part===undefined && a.indices[1]===undefined && b[0]==='' && b.groups.part===undefined && c[0]==='' && c.groups.each===undefined && c.groups.part===undefined",
    );
}

#[test]
fn partial_choice_capture_ranges_names_and_sticky_state_keep_original_utf16_values() {
    for class in [r"[\q{😀}]", r"[\q{😀|😀x}&&\p{Any}]"] {
        check(&format!(
            "let r=/(?<whole>(?<each>((?<part>{class}))|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+)/dvgy;r.lastIndex=2;let a=r.exec('A😀😀x'),s=/(?<whole>(?<each>((?<part>{class}))|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){{3}})/dvgy;s.lastIndex=2;let b=s.exec('A😀😀x');a.index===1 && a[0]==='😀😀' && a.groups.each==='😀' && a.groups.part==='😀' && a.indices[3][0]===3 && a.indices.groups.part[1]===5 && r.lastIndex===5 && b.index===1 && b[0]==='😀😀' && b.groups.each==='' && b.groups.part===undefined && b[3]===undefined && b.indices[3]===undefined && b.indices.groups.part===undefined && b.indices.groups.each[0]===5 && s.lastIndex===5"
        ));
    }
    check(
        r"let source='(?<each>(?<\\u{1D49C}>[\\q{s|k}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+',r=new RegExp(source,'div'),m=r.exec('ſK');r.source===source && r.flags==='div' && m[0]==='ſK' && m.groups.each==='K' && m.groups['𝒜']==='K' && m.indices.groups['𝒜']===m.indices[2] && m.indices[2][0]===1 && m.indices[2][1]===2",
    );
    check(
        r"let m=/(?<each>((?<part>[\q{\uD800a|\uD800}&&\p{Any}]))|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/dv.exec('\uD800\uD800a');m[0].length===2 && m.groups.part.charCodeAt(0)===0xd800 && m.indices.groups.part[0]===1 && m.indices[3][1]===2",
    );
    check(
        r"let r=/(?:(?<part>[\q{ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/dvy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught && r.lastIndex===1",
    );
}

#[test]
fn partial_choice_capture_consumers_callbacks_clones_and_collection_keep_undefined_slots() {
    check(
        r"let r=/(?<each>(?<part>[\q{a|b|ab}&&\p{Any}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a.map(m=>m[0]).join('|')==='ab||ba|' && a[0].groups.each==='b' && a[0].groups.part==='b' && a[1].groups.each==='' && a[1].groups.part===undefined && a[1].indices[2]===undefined && a[2].groups.part==='a' && r.lastIndex===0 && 'abXba'.replace(r,'<$2:$<part>>')==='<b:b><:>X<a:a><:>'",
    );
    check(
        r"let seen=[],result='abX'.replace(/(?<each>(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/vg,(m,each,part,at,input,groups)=>{seen.push([each,part,at,groups.part]);return '#'});result==='##X#' && seen.length===3 && seen[0][0]==='b' && seen[0][1]==='b' && seen[1][0]==='' && seen[1][1]===undefined && seen[1][2]===2 && seen[2][3]===undefined",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){3}/dvg,a=[...'abXba'.matchAll(r)];a.length===4 && a.every(m=>m.groups.each==='' && m.groups.part===undefined) && a[0].indices[1][0]===2 && 'abXba'.replace(r,'<$2>')==='<><>X<><>'",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('ABB');c.source===r.source && c.flags==='dvy' && c.lastIndex===0 && m[0]==='' && m.groups.each==='' && m.groups.part===undefined && r.exec('ABB').groups.part==='B' && r.lastIndex===7",
    );
    check(
        r"let r=/(?<each>(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/vg;r.lastIndex=9;'Xab'.search(r)===0 && r.lastIndex===9 && 'abXba'.match(r).join('|')==='ab||ba|' && r.lastIndex===0 && 'ab'.split(r).join('|')==='|b|b|'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<each>(?<part>[\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){3}/dvg;r.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=r.exec('zabX');m[0]==='ab' && m.groups.each==='' && m.groups.part===undefined && m.indices[2]===undefined && r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn partial_choice_capture_complete_proofs_finish_finalizers_and_keep_empty_side_captures_deferred()
{
    for source in [
        r"/(([\q{a|ab}&&a])|(?:|)?(?:))+/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?)+/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){0}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){1,}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){1}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){2,3}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){2,}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){2}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('aaa')",
        r"/(([\q{a|ab}&&a])|(?:|)?){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999992}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999994}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){999999999999999999999999999999999999999,9999999999999999999999999999999999999990}/v.test('a')",
        r"/(([\q{a|b}])|(?:|)?){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a}])|(?:|)?)+/v.test('a')",
        r"/(([\q{a}])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/(([\q{a}])|(?:|)?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('aaa')",
        r"/(([\q{a}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){0}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){1,}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){1}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){2,3}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){2,}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){2}/v.test('a')",
        r"/(([\q{a|ab}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999992}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999994}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){999999999999999999999999999999999999999,9999999999999999999999999999999999999990}/v.test('a')",
        r"/(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){999999999999999999999999999999999999999,9999999999999999999999999999999999999999}/v.test('a')",
        r"/(([\q{a}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+/v.test('a')",
        r"/(([\q{a}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){888888888888888888888888888888,99999999999999999999999999999999999999993}/v.test('a')",
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
        r"([\q{a|ab}&&a])|(?:|)?",
        r"([\q{a|b}])|(?:(?:(?:))(?:)|(?:(?:))(?:))",
        r"([\q{a|b}])|(?:(?:(?:)*?)(?:)+?|(?:(?:)*?)(?:)+?)",
        r"([\q{a|b}])|(?:(?:(?:)*?)(?:)+?|(?:(?:)*?)(?:)+?)?",
        r"([\q{a|b}])|(?:|(?:)?)",
        r"([\q{a|b}])|(?:|)",
        r"([\q{a|b}])|(?:|)*",
        r"([\q{a|b}])|(?:|)?",
        r"([\q{a|b}])|(?:|)?(?:)",
        r"([\q{a|ab}&&a])|(?:(?:|)*?)(?:|)+?",
        r"([\q{a|b}])|(?:(?:|)*?)(?:|)+?",
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
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="((?:(?:|)*?)(?:|)+?|([\\q{ab}&&\\q{cd}]))+?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"<undefined>\"],\"indices\":[[0,0],[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for (inner, expected_zero) in [
        (
            r"([\q{a|b}])|(\b)?",
            r#"[["","<undefined>","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])|(?:X|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])|(?:X|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])+|(?:(?:|)*?)(?:|)+?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"\b([\q{a|b}])|(?:(?:|)*?)(?:|)+?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"([\q{a|b}])||(?:(?:|)*?)(?:|)+?",
            r#"[["","<undefined>","<undefined>"],[[1,1],"<undefined>","<undefined>"],1,1]"#,
        ),
        (
            r"(?:(?:|)*?)(?:|)+?|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])",
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
        r"(([^\q{ab}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){0}",
        r"(([\q{ab}&&a--a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){2,3}",
        r"(([\q{\d}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){1,}",
        r"(([\q{a|b}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){3,2}",
    ] {
        let source = JsString::from(format!("marker=1;/{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({source:?});}}catch(e){{caught=e instanceof SyntaxError;}}caught && marker===0"
        ));
    }
}

#[test]
fn partial_choice_capture_layouts_huge_counts_and_opt_in_aborts_remain_flat_and_atomic() {
    let pattern = format!(
        r"({}[\q{{a|b}}]{}|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){{999999999999999999999999999999,1000000000000000000000000000000}}",
        "(".repeat(1_000),
        ")".repeat(1_000)
    );
    check(&format!(
        "let m=/{pattern}/dv.exec('aaa');m.length===1002 && m[0]==='aaa' && m[1]==='' && m[2]===undefined && m[1001]===undefined && m.indices[1][0]===3 && m.indices[1001]===undefined"
    ));
    let pattern = format!(
        r"(([\q{{{}}}&&a])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?)+",
        "x".repeat(20_000)
    );
    check(&format!(
        "let m=/{pattern}/dv.exec('😀');m[0]==='' && m[1]==='' && m[2]===undefined && m.indices[1][0]===0 && m.indices[2]===undefined"
    ));

    let counted = format!(r"(([\q{{a|b}}])|(?:|){{0,{}}}?)+", "9".repeat(20_000));
    check(&format!(
        "let r=/{counted}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='aaa' && m[1]==='a' && m[2]==='a' && m.indices[2][0]===2"
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
    let pattern = format!(
        r"(([\q{{a|b}}])|(?:(?:(?:(?:|)*?)(?:|)+?|)*?)(?:(?:(?:|)*?)(?:|)+?|)+?){{{}2,3}}",
        "0".repeat(20_000)
    );
    check(&format!(
        "let r=/{pattern}/dv,m=r.exec('aaa');r.source.length>20000 && m[0]==='aaa' && m[1]==='a' && m[2]==='a' && m.indices[2][0]===2"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm.eval(r"var r=/(([\q{a}])|(?:|))+$/vg").unwrap();
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
