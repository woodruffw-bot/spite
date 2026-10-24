//! Empty captures around consuming UnicodeSets repetitions (22.2.2.3).

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
fn consuming_point_capture_count_families_keep_last_character_and_undefined_slots() {
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
    ] {
        for (count, whole) in [
            ("*".to_owned(), Some("aaa")),
            ("*?".to_owned(), Some("")),
            ("+".to_owned(), Some("aaa")),
            ("+?".to_owned(), Some("a")),
            ("?".to_owned(), Some("a")),
            ("??".to_owned(), Some("")),
            ("{0}".to_owned(), Some("")),
            ("{00,003}?".to_owned(), Some("")),
            (format!("{{0,{huge}}}"), Some("aaa")),
            ("{000,}".to_owned(), Some("aaa")),
            ("{1}".to_owned(), Some("a")),
            ("{01,001}?".to_owned(), Some("a")),
            ("{1,}".to_owned(), Some("aaa")),
            ("{2}".to_owned(), Some("aa")),
            ("{02,002}?".to_owned(), Some("aa")),
            ("{2,4}".to_owned(), Some("aaa")),
            ("{02,004}?".to_owned(), Some("aa")),
            ("{3,}?".to_owned(), Some("aaa")),
            (format!("{{1,{huge}}}?"), Some("a")),
            (format!("{{{limit},{}}}", limit + 1), None),
            (format!("{{{limit},{}}}?", limit + 1), None),
            (format!("{{{huge},{}}}", huge + 1), None),
            (format!("{{{huge},{}}}?", huge + 1), None),
            (format!("{{{huge}}}?"), None),
            (format!("{{000{huge},0{huge}}}"), None),
            (format!("{{{huge},}}?"), None),
            (format!("{{{huge},{}}}?", huge * 100), None),
        ] {
            for layout in 0..6 {
                let part = format!("(?<part>{class})");
                let (body, before, after, never) = match layout {
                    0 => (format!("{part}(?<after>)"), "", "after", ""),
                    1 => (format!("(?<before>){part}"), "before", "", ""),
                    2 => (
                        format!("(?<before>(?<deep>)){part}(?<after>)"),
                        "before,deep",
                        "after",
                        "",
                    ),
                    3 => (
                        format!("(?<before>)?{part}(?<after>)+?"),
                        "",
                        "after",
                        "before",
                    ),
                    4 => (
                        format!("(?<before>(?<first>)|(?<later>)){part}(?<after>(?<deep>)?)"),
                        "before,first",
                        "after",
                        "later,deep",
                    ),
                    _ => (
                        format!(
                            "(?<before>(?<deep>)){{0,{huge}}}?{part}(?<after>(?<required>)){{{huge}}}"
                        ),
                        "",
                        "after,required",
                        "before,deep",
                    ),
                };
                let pattern = JsString::from(format!("(?<whole>(?<each>{body}){count})").as_str());
                let Some(whole) = whole else {
                    check(&format!(
                        "var r=new RegExp({pattern:?},'dvg');r.lastIndex=1;r.exec('aaa')===null&&r.lastIndex===0"
                    ));
                    continue;
                };
                let length = whole.len();
                let participating = length != 0;
                let each = if participating { "'a'" } else { "undefined" };
                let range = if participating {
                    format!(
                        "m.indices.groups.each[0]==={}&&m.indices.groups.part[1]==={length}",
                        length - 1
                    )
                } else {
                    "m.indices.groups.each===undefined&&m.indices.groups.part===undefined"
                        .to_owned()
                };
                check(&format!(
                    r#"
                    var r=new RegExp({pattern:?},'dvg'),m=r.exec('aaa'),keys=Object.keys(m.groups);
                    var before='{before}'.split(','),after='{after}'.split(','),never='{never}'.split(','),ok=true;
                    for(var name of keys){{
                        var slot=keys.indexOf(name)+1,expected=name==='whole'?{whole:?}:name==='each'||name==='part'?{each}:undefined;
                        var point=-1;
                        if({participating}&&before.indexOf(name)!==-1){{expected='';point={length}-1}}
                        if({participating}&&after.indexOf(name)!==-1){{expected='';point={length}}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&Object.hasOwn(m.indices.groups,name)&&m.indices.groups[name]===m.indices[slot];
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        if(point!==-1)ok=ok&&m.indices[slot][0]===point&&m.indices[slot][1]===point;
                    }}
                    ok&&m[0]==={whole:?}&&m.index===0&&m.length===keys.length+1&&{range}&&r.lastIndex==={length}&&r.source==={pattern:?}
                "#,
                    whole = JsString::from(whole)
                ));
            }
        }
    }
}

#[test]
fn consuming_point_captures_use_supplementary_boundaries_folding_gates_and_original_metadata() {
    check(
        r"var r=/(?<whole>(?<each>(?<before>)(?<part>[\q{😀}])(?<after>))+)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]==='😀😀'&&m.groups.each==='😀'&&m.groups.before===''&&m.indices.groups.before[0]===2&&m.indices.groups.after[0]===4&&m.indices.groups.part[0]===2&&r.lastIndex===4",
    );
    check(
        r"var r=/(?<each>(?<\u{1D49C}>)(?<part>[\q{s|k}])(?<after>))+/dvi;var m=r.exec('ſK!');m[0]==='ſK'&&m.groups.part==='K'&&m.groups.𝒜===''&&m.indices.groups.𝒜[0]===1&&m.indices.groups.after[0]===2&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?:(?<before>)(?<part>[^\q{ab}&&a])(?<after>)){1}/dv.exec('\ud800');m.groups.part==='\ud800'&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===1",
    );
    check(
        r"var r=/^\b(?:(?<before>)(?<part>[\q{a|b}])(?<after>)){2,3}?\b$/dvm;var m=r.exec('!\r\naaa\u2028!');m[0]==='aaa'&&m.index===3&&m.indices.groups.before[0]===5&&m.indices.groups.after[0]===6",
    );
    check(
        r"var r=/(?:()[\q{a}]())+/vg;Object.defineProperty(r,'lastIndex',{writable:false});var threw=false;try{r.exec('aaa')}catch(e){threw=e instanceof TypeError}threw&&r.lastIndex===0",
    );
}

#[test]
fn consuming_point_captures_survive_string_consumers_callbacks_clones_and_collection() {
    check(
        r"var r=/(?:(?<before>)(?<part>[\q{a}])(?<after>))+?/dvg;var all=Array.from('aa!a'.matchAll(r));all.length===3&&all[1].index===1&&all[1].groups.before===''&&all[1].indices.groups.after[0]===2&&r.lastIndex===0",
    );
    check(
        r"var calls=0;var out='aaa!a'.replace(/(?<each>(?<before>)(?<part>[\q{a}])(?<after>))+/vg,function(whole,each,before,part,after,offset,input,groups){calls++;return arguments.length===8&&each==='a'&&before===''&&part==='a'&&after===''&&groups.before===''&&input==='aaa!a'?String(offset):'bad'});out==='0!4'&&calls===2",
    );
    check(
        r"'aa!a'.replace(/(?:(?<before>)(?<part>[\q{a}])(?<after>))+/vg,'[$<before>$<part>$<after>]')==='[a]![a]'&&'aaa'.match(/(?:()[\q{a}]())+/vg)[0]==='aaa'",
    );
    check(
        r"var r=/(?:()[\q{a}]())+/vg;r.lastIndex=9;var index='!aaa'.search(r);var parts='!aa!'.split(r);index===1&&r.lastIndex===9&&parts.length===4&&parts[0]==='!'&&parts[1]===''&&parts[2]===''&&parts[3]==='!'",
    );
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r"var r=/(?:(?<before>)(?<part>[\q{a}])(?<after>))+/dvg;var clone=new RegExp(r);var m=r.exec('aaa');true"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.before===''&&m.indices.groups.before[0]===2&&m.indices.groups.after[0]===3&&clone.exec('aa').groups.part==='a'&&r.lastIndex===3"),Ok(Value::Boolean(true)));
}

#[test]
fn consuming_point_capture_host_failures_skip_handlers_and_supported_programs_finish_finalizers() {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(?:()[\\q{ab}&&\\q{cd}]){0}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"ab!\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"(?:()\b[\q{a}])+",
        r"(?:()[\q{a}]\1)+",
        r"(?:()[\q{a}]x)+",
        r"(?:()[\q{a}][\q{b}])+",
        r"(?:()[[\q{ab}&&\q{cd}]&&\p{RGI_Emoji}]){0}",
        r"(?:()[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}",
        r"(?:()([\q{a}])?)+",
    ] {
        let mut realm = Realm::default();
        assert!(
            matches!(
                realm.eval(&format!(
                    "var finished=0;try{{new RegExp({:?},'v').test('a')}}finally{{finished++}}",
                    JsString::from(pattern)
                )),
                Err(Error::Unsupported { .. })
            ),
            "{pattern}"
        );
        assert_eq!(realm.eval("finished"), Ok(Value::Number(0.0)));
    }
    check("var finished=0;try{/(?:()[\\q{a}]())+$/v.test('aaa')}finally{finished++}finished===1");
    check(
        "var yes=0;for(var p of ['(?:()[a]){3,2}','(?<x>)(?<x>[a])+']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}

#[test]
fn consuming_point_capture_large_layouts_and_counts_preserve_state_under_opt_in_aborts() {
    let groups = format!("{}{}", "(".repeat(1000), ")".repeat(1000));
    let pattern = JsString::from(format!("(?:{groups}[\\q{{a}}]())+").as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('aaa');m.length===1002&&m[1]===''&&m[1000]===''&&m.indices[1][0]===2&&m.indices[1001][0]===3"
    ));
    let digits = "9".repeat(20_000);
    for (count, defined) in [
        (format!("{{0,{digits}}}?"), false),
        (format!("{{{digits}}}"), true),
    ] {
        let pattern = JsString::from(format!("(?:(){count}[\\q{{a}}]())+").as_str());
        check(&format!(
            "var m=new RegExp({pattern:?},'dv').exec('aaa');m[0]==='aaa'&&m[1]==={}&&m[2]===''&&m.indices[2][0]===3",
            if defined { "''" } else { "undefined" }
        ));
    }
    let pattern = JsString::from(format!("(?:()[\\q{{{}}}&&a]())+", "x".repeat(20_000)).as_str());
    check(&format!("new RegExp({pattern:?},'v').exec('aaa')===null"));
    let mut limited = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    assert!(matches!(
        limited.eval(&format!(
            "var marker=0;try{{new RegExp({pattern:?},'v')}}finally{{marker=1}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(limited.eval("marker"), Ok(Value::Number(0.0)));
    let mut limited = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    limited.eval("var marker=0").unwrap();
    limited.eval(r"var r=/(?:()[\q{a}]())+$/vg").unwrap();
    limited
        .eval(&format!(
            "var input='{}!';r.lastIndex=3",
            "a".repeat(40_000)
        ))
        .unwrap();
    assert!(matches!(
        limited.eval("try{r.exec(input)}finally{marker=1}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        limited.eval("marker===0&&r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
}
