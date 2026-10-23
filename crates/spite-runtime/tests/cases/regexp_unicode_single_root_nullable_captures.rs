//! Ungrouped class-versus-empty alternatives (22.2.2).
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
fn root_nullable_capture_layouts_keep_zero_branch_point_and_undefined_slots() {
    let huge = 10u128.pow(30);
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
    ] {
        for empty_first in [false, true] {
            for layout in 0..6 {
                let (character, empty, characters, points_before, points_after, empties, never) =
                    match layout {
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
                let whole = if empty_first { "" } else { "a" };
                let end = if empty_first { 1 } else { 2 };
                let named = layout != 0;
                check(&format!(
                    r#"
                var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec('aaa'),ok=true;
                if({named}){{
                    var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{points_before}'.split(','),after='{points_after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected=!{empty_first}&&characters.indexOf(name)!==-1?'a':undefined,point=-1;
                        if(!{empty_first}&&before.indexOf(name)!==-1){{expected='';point=1}}
                        if(!{empty_first}&&after.indexOf(name)!==-1){{expected='';point={end}}}
                        if({empty_first}&&empty.indexOf(name)!==-1){{expected='';point=1}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        else ok=ok&&m.indices[slot][0]===(expected===''?point:1)&&m.indices[slot][1]===(expected===''?point:{end});
                    }}ok=ok&&m.length===keys.length+1;
                }}else ok=ok&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined;
                ok&&m[0]==={whole:?}&&m.index===1&&m.indices[0][0]===1&&m.indices[0][1]==={end}&&r.lastIndex==={end}&&r.source==={pattern:?}
            "#
                ));
            }
        }
    }
    check(
        r"var m=/([\q{a}])|()/dv.exec('a');m.length===3&&m[1]==='a'&&m[2]===undefined&&m.indices[1][1]===1",
    );
    check(
        r"var m=/()|([\q{a}])/dv.exec('a');m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
}

#[test]
fn root_nullable_ranges_keep_utf16_folding_names_and_original_source() {
    check(
        r"var r=/(?<before>)(?<part>[\q{😀}])(?<after>)|(?<empty>)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]==='😀'&&m.index===0&&m.groups.before===''&&m.groups.part==='😀'&&m.groups.after===''&&m.groups.empty===undefined&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===2&&r.lastIndex===2",
    );
    check(
        r"var r=/(?<empty>)|(?<before>)(?<part>[\q{😀}])(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]===''&&m.index===0&&m.groups.empty===''&&m.groups.before===undefined&&m.groups.part===undefined&&m.groups.after===undefined&&m.indices.groups.empty[0]===0&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<\u{1D49C}>[\q{s|k}])|(?<empty>)/dvgi;var m=r.exec('ſK'),n=r.exec('ſK');m.groups.𝒜==='ſ'&&n.groups.𝒜==='K'&&n.groups.empty===undefined&&n.indices.groups.𝒜[0]===1&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/[^\q{ab}&&a]|/dv.exec('\ud800');m[0]==='\ud800'&&m.length===1&&m.indices[0][1]===1&&m.groups===undefined",
    );
    check(
        r"var r=/(?<empty>)|(?<part>[\q{a}])/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn root_nullable_consumers_callbacks_clones_and_collection_keep_branch_priority() {
    check(
        r"var r=/(?<part>[\q{a}])|(?<empty>(?<first>)|(?<later>))/dvg;var all=Array.from('aa!'.matchAll(r));all.length===4&&all.map(m=>m[0]).join('|')==='a|a||'&&all[0].groups.part==='a'&&all[0].groups.empty===undefined&&all[2].groups.part===undefined&&all[2].groups.empty===''&&all[2].groups.first===''&&all.every(m=>m.groups.later===undefined)&&r.lastIndex===0",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/(?<empty>)|(?<part>[\q{😀}])/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m[0]===''&&m.groups.empty===''&&m.groups.part===undefined)",
    );
    check(
        r"var calls=0;var out='aa!'.replace(/(?<part>[\q{a}])|(?<empty>(?<first>)|(?<later>))/vg,function(whole,part,empty,first,later,offset,input,groups){calls++;return arguments.length===8&&groups.later===undefined&&input==='aa!'&&(whole==='a'?part==='a'&&empty===undefined&&first===undefined:part===undefined&&empty===''&&first==='')?'#':'bad'});out==='###!#'&&calls===4",
    );
    check(
        r"var r=/(?<part>[\q{a}])|(?<empty>)/vg;r.lastIndex=9;var parts='ab'.split(r);'baa'.search(r)===0&&r.lastIndex===9&&parts.length===4&&parts[0]===''&&parts[1]==='a'&&parts[2]===undefined&&parts[3]==='b'&&'aa!'.match(r).join('|')==='a|a||'&&'aa!'.replace(r,'<$<part>:$<empty>>')==='<a:><a:><:>!<:>'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<part>[\q{a}])|(?<empty>)/dvg;var clone=new RegExp(r);var m=r.exec('aaa')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.part==='a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===1&&clone.exec('aa').groups.part==='a'&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn root_nullable_proofs_finish_finalizers_and_keep_branch_assertions_in_scope() {
    check(
        r"var marker=0,m;try{m=/([\q{a}])|()/dv.exec('a')}finally{marker=3}marker===3&&m[0]==='a'&&m[1]==='a'&&m[2]===undefined&&m.indices[1][1]===1",
    );
    check(
        r"var marker=0,m;try{m=/([\q{a}])|()/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===undefined&&m[2]===''&&m.indices[2][0]===0",
    );
    for pattern in [
        r"^([\q{a}])|()",
        r"([\q{a}])|()$",
        r"\b([\q{a}])|()",
        r"()|^([\q{a}])",
        r"([\q{a}])$|()",
        r"([\q{a}])|()\B",
        r"([\q{a}])x|()",
        r"([\q{a}])|()\1",
        r"([\q{a}])|([\q{b}])",
        r"([\q{a}])|()|()",
        r"([\q{ab}&&\p{RGI_Emoji}])|()",
        r"()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('a')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    check(
        "var yes=0;for(var p of ['(?<x>[a])|(?<x>)(?<x>)','[\\q{ab}&&a--a]|']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}

#[test]
fn root_nullable_deep_capture_tables_counts_and_opt_in_aborts_remain_flat() {
    let groups = format!("{}{}", "(".repeat(1000), ")".repeat(1000));
    let pattern = JsString::from(format!("[\\q{{a}}]|{groups}").as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),a=r.exec('a'),e=r.exec('!');a.length===1001&&a[1]===undefined&&a[1000]===undefined&&e[0]===''&&e[1]===''&&e[1000]===''&&e.indices[1000][0]===0"
    ));
    let digits = "9".repeat(20_000);
    for (count, defined) in [
        (format!("{{0,{digits}}}?"), false),
        (format!("{{{digits}}}"), true),
    ] {
        let pattern = JsString::from(format!("(){count}|([\\q{{a}}])").as_str());
        check(&format!(
            "var m=new RegExp({pattern:?},'dv').exec('aaa');m[0]===''&&m[1]==={}&&m[2]===undefined",
            if defined { "''" } else { "undefined" }
        ));
    }
    let pattern = JsString::from(format!("([\\q{{{}}}&&a])|()", "x".repeat(20_000)).as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('😀');m[0]===''&&m[1]===undefined&&m[2]===''&&m.indices[2][0]===0"
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
    limited.eval(r"var r=/([\q{a}])|()/vg").unwrap();
    limited
        .eval(&format!("var input='{}';r.lastIndex=1", "x".repeat(40_000)))
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
