//! Empty point captures around one nullable UnicodeSets group (22.2.2.3).
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
fn single_nullable_point_layouts_keep_branch_participation_without_outer_counts_or_wrappers() {
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
    ] {
        for empty_first in [false, true] {
            for layout in 0..6 {
                let part = format!("(?<part>{class})");
                let (
                    prefix,
                    character,
                    empty,
                    suffix,
                    always_before,
                    always_after,
                    char_before,
                    char_after,
                    empty_names,
                    never,
                ) = match layout {
                    0 => (
                        "",
                        part,
                        "(?<empty>)",
                        "(?<after>)",
                        "",
                        "after",
                        "",
                        "",
                        "empty",
                        "",
                    ),
                    1 => (
                        "(?<before>)",
                        part,
                        "(?<empty>)",
                        "",
                        "before",
                        "",
                        "",
                        "",
                        "empty",
                        "",
                    ),
                    2 => (
                        "",
                        format!("(?<left>){part}"),
                        "(?<empty>)",
                        "",
                        "",
                        "",
                        "left",
                        "",
                        "empty",
                        "",
                    ),
                    3 => (
                        "",
                        format!("{part}(?<right>)"),
                        "(?<empty>)",
                        "",
                        "",
                        "",
                        "",
                        "right",
                        "empty",
                        "",
                    ),
                    4 => (
                        "(?<before>)",
                        format!("(?<left>){part}(?<right>)"),
                        "(?<empty>(?<first>)|(?<later>))",
                        "(?<after>)",
                        "before",
                        "after",
                        "left",
                        "right",
                        "empty,first",
                        "later",
                    ),
                    _ => (
                        "(?<before>)?",
                        format!("(?<left>(?<leftDeep>)?){part}(?<right>)"),
                        "(?<empty>(?<emptyDeep>)?)",
                        "(?<after>)+?",
                        "",
                        "after",
                        "left",
                        "right",
                        "empty",
                        "before,leftDeep,emptyDeep",
                    ),
                };
                let branch = if empty_first {
                    format!("{empty}|{character}")
                } else {
                    format!("{character}|{empty}")
                };
                for (open, close) in [("", ""), ("(?:", ")"), ("(?<whole>", ")")] {
                    let pattern = JsString::from(
                        format!("{open}{prefix}(?:{branch}){suffix}{close}").as_str(),
                    );
                    let whole = if empty_first { "" } else { "a" };
                    let end = if empty_first { 1 } else { 2 };
                    check(&format!(
                        r#"
                    var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec('aaa'),keys=Object.keys(m.groups),ok=true;
                    var before='{always_before}'.split(','),after='{always_after}'.split(','),left='{char_before}'.split(','),right='{char_after}'.split(','),empty='{empty_names}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{
                        var slot=keys.indexOf(name)+1,expected=name==='whole'?{whole:?}:name==='part'&&!{empty_first}?'a':undefined,point=-1;
                        if(before.indexOf(name)!==-1){{expected='';point=1}}
                        if(after.indexOf(name)!==-1){{expected='';point={end}}}
                        if(!{empty_first}&&left.indexOf(name)!==-1){{expected='';point=1}}
                        if(!{empty_first}&&right.indexOf(name)!==-1){{expected='';point={end}}}
                        if({empty_first}&&empty.indexOf(name)!==-1){{expected='';point=1}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&Object.hasOwn(m.indices.groups,name)&&m.indices.groups[name]===m.indices[slot];
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        if(point!==-1)ok=ok&&m.indices[slot][0]===point&&m.indices[slot][1]===point;
                    }}
                    ok&&m[0]==={whole:?}&&m.index===1&&m.length===keys.length+1&&r.lastIndex==={end}&&r.source==={pattern:?}
                "#
                    ));
                }
            }
        }
    }
    check(
        r"var m=/((([\q{a}])|()))()/dv.exec('a');m[0]==='a'&&m[1]==='a'&&m[2]==='a'&&m[3]==='a'&&m[4]===undefined&&m[5]===''&&m.indices[5][0]===1",
    );
}

#[test]
fn single_nullable_points_keep_utf16_folding_outside_gates_and_first_success() {
    check(
        r"var r=/(?<before>)(?:(?<left>)(?<part>[\q{😀}])(?<right>)|(?<empty>))(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]==='😀'&&m.index===0&&m.groups.before===''&&m.groups.left===''&&m.groups.right===''&&m.groups.empty===undefined&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===2&&r.lastIndex===2",
    );
    check(
        r"var r=/(?<before>)(?:(?<empty>)|(?<left>)(?<part>[\q{😀}])(?<right>))(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]===''&&m.index===0&&m.groups.before===''&&m.groups.empty===''&&m.groups.part===undefined&&m.groups.left===undefined&&m.groups.right===undefined&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===0&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<\u{1D49C}>)(?:(?<part>[\q{s|k}])|(?<empty>))(?<after>)/dvgi;var m=r.exec('ſK'),n=r.exec('ſK');m.groups.part==='ſ'&&m.groups.𝒜===''&&m.indices.groups.after[0]===1&&n.groups.part==='K'&&n.indices.groups.𝒜[0]===1&&n.indices.groups.after[0]===2&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?<before>)(?:(?<part>[^\q{ab}&&a])|(?<empty>))(?<after>)/dv.exec('\ud800');m.groups.part==='\ud800'&&m.groups.empty===undefined&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===1",
    );
    for branch in [
        r"(?<part>[\q{a|b}])|(?<empty>)",
        r"(?<empty>)|(?<part>[\q{a|b}])",
    ] {
        check(&format!(
            r"var r=/^\b(?<before>)(?:{branch})(?<after>)\b$/dvm;var m=r.exec('!\r\na\u2028!');m[0]==='a'&&m.index===3&&m.groups.part==='a'&&m.groups.empty===undefined&&m.indices.groups.before[0]===3&&m.indices.groups.after[0]===4"
        ));
    }
    check(
        r"var m=/\b()(?:(?<part>[\q{a}])|(?<empty>))()\b/dv.exec(' aa');m[0]===''&&m.index===1&&m.groups.part===undefined&&m.groups.empty===''&&m.indices[1][0]===1&&m.indices[4][0]===1",
    );
    check(
        r"var r=/()(?:|[\q{a}])()/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn single_nullable_point_consumers_callbacks_clones_and_collection_keep_empty_progress() {
    check(
        r"var r=/(?<before>)(?:(?<left>)(?<part>[\q{a}])(?<right>)|(?<empty>))(?<after>)/dvg;var all=Array.from('aa!'.matchAll(r));all.length===4&&all.map(m=>m[0]).join('|')==='a|a||'&&all.map(m=>m.indices.groups.before[0]).join(',')==='0,1,2,3'&&all.map(m=>m.indices.groups.after[0]).join(',')==='1,2,2,3'&&all[2].groups.empty===''&&all[2].groups.part===undefined&&r.lastIndex===0",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/()(?:(?<empty>)|(?<part>[\q{😀}]))()/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m[0]===''&&m.groups.empty===''&&m.groups.part===undefined)",
    );
    check(
        r"var calls=0;var out='aa!'.replace(/(?<before>)(?:(?<left>)(?<part>[\q{a}])(?<right>)|(?<empty>))(?<after>)/vg,function(whole,before,left,part,right,empty,after,offset,input,groups){calls++;return arguments.length===10&&before===''&&after===''&&groups.before===''&&input==='aa!'&&(whole==='a'?left===''&&part==='a'&&right===''&&empty===undefined:left===undefined&&part===undefined&&right===undefined&&empty==='')?'#':'bad'});out==='###!#'&&calls===4",
    );
    check(
        r"var r=/(?<before>)(?:[\q{a}]|)(?<after>)/vg;r.lastIndex=9;var parts='ab'.split(r);'baa'.search(r)===0&&r.lastIndex===9&&parts.length===4&&parts[0]===''&&parts[1]===''&&parts[2]===''&&parts[3]==='b'&&'aa!'.match(r).join('|')==='a|a||'&&'aa!'.replace(r,'<$<before>:$<after>>')==='<:><:><:>!<:>'",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?:(?<part>[\q{a}])|(?<empty>))(?<after>)/dvg;var clone=new RegExp(r);var m=r.exec('aaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.before===''&&m.groups.part==='a'&&m.groups.empty===undefined&&m.indices.groups.after[0]===1&&clone.exec('aa').groups.part==='a'&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn single_nullable_point_scope_guard_keeps_assertions_and_host_failures_separate() {
    check(
        r"var marker=0,m;try{m=/()(?:[\q{a}]|)/dv.exec('a')}finally{marker=3}marker===3&&m[0]==='a'&&m[1]===''&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/()(?:[\q{a}]|)/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,value;try{var m=/()(?:[\q{a}]|)()/dv.exec('a');value=m[1]===''&&m.indices[2][0]===1}finally{marker=3}value&&marker===3",
    );
    for pattern in [
        r"^()[\q{a}]|()",
        r"()[\q{a}]|()$",
        r"\b()[\q{a}]|()",
        r"()[\q{a}]|()\b",
        r"()(?:[\q{a}]|)\b()",
        r"()(?:[\q{a}]|)\1",
        r"()(?:[\q{a}]|)x",
        r"()(?:[\q{a}][\q{b}]|)",
        r"()(?:[\q{ab}&&\q{cd}]|)",
        r"()(?:[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|)",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('a')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    check(
        "var yes=0;for(var p of ['()(?:(?<x>[a])|(?<x>))(?<x>)','()(?:[\\q{ab}&&a--a]|)']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}

#[test]
fn single_nullable_point_large_captures_counts_and_host_aborts_remain_flat_and_atomic() {
    let groups = format!("{}{}", "(".repeat(1000), ")".repeat(1000));
    let pattern = JsString::from(format!("{groups}(?:[\\q{{a}}]|)()").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('aaa');m[0]==='a'&&m.length===1002&&m[1]===''&&m[1000]===''&&m.indices[1][0]===0&&m.indices[1001][0]===1"
    ));
    let digits = "9".repeat(20_000);
    for (count, defined) in [
        (format!("{{0,{digits}}}?"), false),
        (format!("{{{digits}}}"), true),
    ] {
        let pattern = JsString::from(format!("(){count}(?:[\\q{{a}}]|)()").as_str());
        check(&format!(
            "var m=new RegExp({pattern:?},'dv').exec('aaa');m[0]==='a'&&m[1]==={}&&m[2]===''&&m.indices[2][0]===1",
            if defined { "''" } else { "undefined" }
        ));
    }
    let pattern = JsString::from(format!("()(?:[\\q{{{}}}&&a]|)()", "x".repeat(20_000)).as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('😀');m[0]===''&&m[1]===''&&m[2]===''&&m.indices[2][0]===0"
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
    limited.eval(r"var r=/()(?:[\q{a}]|)()$/vg").unwrap();
    limited
        .eval(&format!("var input='{}';r.lastIndex=1", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        limited.eval("try{r.exec(input)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        limited.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
