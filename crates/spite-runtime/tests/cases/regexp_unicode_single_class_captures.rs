//! Empty point captures around one consuming UnicodeSets class (22.2.2).
use spite_core::JsString;
use spite_runtime::{Realm, Value};
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn single_point_capture_layouts_keep_names_ranges_and_participation_without_outer_wrappers() {
    let huge = 2u128;
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
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
            for (open, close) in [("", ""), ("(?:", ")"), ("(?<whole>", ")")] {
                let pattern = JsString::from(format!("{open}{body}{close}").as_str());
                check(&format!(
                    r#"
                    var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec('aaa'),keys=Object.keys(m.groups),ok=true;
                    var before='{before}'.split(','),after='{after}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{
                        var slot=keys.indexOf(name)+1,expected=name==='part'||name==='whole'?'a':undefined,point=-1;
                        if(before.indexOf(name)!==-1){{expected='';point=1}}
                        if(after.indexOf(name)!==-1){{expected='';point=2}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&Object.hasOwn(m.indices.groups,name)&&m.indices.groups[name]===m.indices[slot];
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        if(point!==-1)ok=ok&&m.indices[slot][0]===point&&m.indices[slot][1]===point;
                    }}
                    ok&&m[0]==='a'&&m.index===1&&m.length===keys.length+1&&m.indices.groups.part[0]===1&&m.indices.groups.part[1]===2&&r.lastIndex===2&&r.source==={pattern:?}
                "#
                ));
            }
        }
    }
    check(
        r"var m=/()[\q{a}]()/dv.exec('a');m.length===3&&m[1]===''&&m[2]===''&&m.indices[1][0]===0&&m.indices[2][0]===1",
    );
    check(r"var r=/()[\q{ab}&&a]()/dvg;r.lastIndex=1;r.exec('aaa')===null&&r.lastIndex===0");
}

#[test]
fn single_point_captures_keep_original_utf16_folding_gates_and_source_metadata() {
    check(
        r"var r=/(?<before>)(?<part>[\q{😀}])(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀😀!'),n=r.exec('😀😀!');m[0]==='😀'&&m.index===0&&m.groups.before===''&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===2&&n.index===2&&n.indices.groups.before[0]===2&&n.indices.groups.after[0]===4&&r.lastIndex===4",
    );
    check(
        r"var r=/(?<\u{1D49C}>)(?<part>[\q{s|k}])(?<after>)/dvgi;var m=r.exec('ſK'),n=r.exec('ſK');m.groups.part==='ſ'&&m.groups.𝒜===''&&m.indices.groups.𝒜[0]===0&&m.indices.groups.after[0]===1&&n.groups.part==='K'&&n.indices.groups.𝒜[0]===1&&n.indices.groups.after[0]===2&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?<before>)(?<part>[^\q{ab}&&a])(?<after>)/dv.exec('\ud800');m.groups.part==='\ud800'&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===1",
    );
    check(
        r"var r=/^\b(?<before>)(?<part>[\q{a|b}])(?<after>)\b$/dvm;var m=r.exec('!\r\na\u2028!');m[0]==='a'&&m.index===3&&m.indices.groups.before[0]===3&&m.indices.groups.after[0]===4&&r.exec('aaa')===null",
    );
    check(
        r"var r=/()[\q{a}]()/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn single_point_capture_consumers_callbacks_clones_and_collection_keep_aliases() {
    check(
        r"var r=/(?<before>)(?<part>[\q{a}])(?<after>)/dvg;var all=Array.from('aa!a'.matchAll(r));all.length===3&&all[1].index===1&&all[1].groups.before===''&&all[1].indices.groups.after[0]===2&&r.lastIndex===0",
    );
    check(
        r"var calls=0;var out='aa!a'.replace(/(?<before>)(?<part>[\q{a}])(?<after>)/vg,function(whole,before,part,after,offset,input,groups){calls++;return arguments.length===7&&whole==='a'&&before===''&&part==='a'&&after===''&&groups.before===''&&input==='aa!a'?String(offset):'bad'});out==='01!3'&&calls===3",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{a}])(?<after>)/vg;r.lastIndex=9;var parts='!a!'.split(r);'!aa'.search(r)===1&&r.lastIndex===9&&parts.length===5&&parts[0]==='!'&&parts[1]===''&&parts[2]==='a'&&parts[3]===''&&parts[4]==='!'&&'aa!a'.match(r).join('|')==='a|a|a'&&'aa!a'.replace(r,'<$<before>$<part>$<after>>')==='<a><a>!<a>'",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<part>[\q{a}])(?<after>)/dvg;var clone=new RegExp(r);var m=r.exec('aaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.before===''&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===1&&clone.exec('aa').groups.part==='a'&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn single_point_capture_proofs_finish_supported_finalizers_and_keep_other_terms_host_failures() {
    check(
        r"var marker=0,value;try{var m=/()[\q{a}]()/dv.exec('a');value=m[1]===''&&m.indices[2][0]===1}finally{marker=3}value&&marker===3",
    );
    check(
        r"var marker=0,m;try{m=/()[\q{ab}&&\q{cd}]/dv.exec('ab')}finally{marker=3}marker===3&&m===null",
    );
    for pattern in [
        r"()\b[\q{a}]",
        r"()[\q{a}]\1",
        r"()[\q{a}]x",
        r"()[\q{a}][\q{b}]",
        r"()[\q{a}]?",
        r"(\b)(?:[\q{a}]|)",
        r"()[\q{ab}&&\q{cd}&&\p{RGI_Emoji}]",
        r"()[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
    ] {
        let mut realm = Realm::default();
        assert!(
            crate::common::regexp_oracle::check(
                &mut realm,
                &format!(
                    "var marker=0;try{{new RegExp({:?},'v').test('a')}}catch{{marker=1}}finally{{marker=2}};marker",
                    JsString::from(pattern)
                )
            ),
            "{pattern}"
        );
    }
    check(
        "var yes=0;for(var p of ['()(?<x>[a])(?<x>)','()[\\q{ab}&&a--a]']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}
