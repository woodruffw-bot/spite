//! Tagged class/empty selection for root string classes containing empty members.
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
fn empty_member_root_layouts_distinguish_defined_zero_width_class_captures() {
    let huge = 2u128;
    for (class, input, word) in [
        (r"[\q{|a|ab}]", "ab!", "ab"),
        (r"[\q{|a|ab}]", "!", ""),
        (r"[\q{ab}\q{}]", "ab!", "ab"),
        (r"[x\q{|ab}]", "x!", "x"),
        (r"[a-z\q{|ab}]", "ab!", "ab"),
        (r"[[\q{|ab}][x\q{a}]]", "ab!", "ab"),
        (r"[\q{|a|ab}--\q{a}]", "a!", ""),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", "x!", "x"),
        (r"[\q{|a|ab}--[a-z]]", "a!", ""),
        (r"[\q{|😀|😀a}]", "😀a!", "😀a"),
        (r"[\q{|\u{10428}|\u{10428}a}]", "𐐨a!", "𐐨a"),
        (r"[\q{|s|sa|k|ka}]", "sa!", "sa"),
        (r"[\q{}]", "ab!", ""),
        (r"[\q{|ab}--\q{ab}]", "ab!", ""),
    ] {
        for empty_first in [false, true] {
            for layout in 0..6 {
                let (character, empty, characters, before, after, empties, never) = match layout {
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
                let class_selected = !empty_first;
                let whole = JsString::from(if class_selected { word } else { "" });
                let end = 1 + whole.len();
                let text = JsString::from(format!("!{input}").as_str());
                let named = layout != 0;
                check(&format!(
                    r#"
                var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec({text:?}),ok=true;
                if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(',');
                    for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
                        if({class_selected}&&characters.indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
                        if({class_selected}&&before.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if({class_selected}&&after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
                        if(!{class_selected}&&empty.indexOf(name)!==-1){{expected='';from=1;to=1}}
                        if(never.indexOf(name)!==-1)expected=undefined;
                        ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
                        if(expected===undefined)ok=ok&&m.indices[slot]===undefined;
                        else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
                    }}ok=ok&&m.length===keys.length+1;
                }}else ok=ok&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined;
                ok&&m[0]==={whole:?}&&m.index===1&&m.indices[0][1]==={end}&&r.lastIndex==={end}&&r.source==={pattern:?}
            "#
                ));
            }
        }
    }
}

#[test]
fn empty_member_root_selection_keeps_class_identity_for_empty_and_longest_words() {
    check(
        r"var m=/(?<before>)(?<part>[\q{|a|ab}])(?<after>)|(?<empty>)/dv.exec('!');m[0]===''&&m.groups.part===''&&m.groups.before===''&&m.groups.after===''&&m.groups.empty===undefined&&m.indices.groups.part[0]===0&&m.indices.groups.part[1]===0&&m.indices.groups.after[0]===0",
    );
    check(
        r"var m=/(?<empty>)|(?<before>)(?<part>[\q{|a|ab}])(?<after>)/dv.exec('!');m[0]===''&&m.groups.empty===''&&m.groups.part===undefined&&m.groups.before===undefined&&m.groups.after===undefined&&m.indices.groups.empty[0]===0",
    );
    check(
        r"var r=/(?<part>[\q{|😀|😀a}])|(?<empty>)/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m[0]==='😀a'&&m.index===0&&m.groups.part==='😀a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<\u{1D49C}>[\q{|s|sa|k|ka}])|(?<empty>)/dvgi;var m=r.exec('ſaKa'),n=r.exec('ſaKa'),e=r.exec('ſaKa');m.groups.𝒜==='ſa'&&n.groups.𝒜==='Ka'&&e.groups.𝒜===''&&e.groups.empty===undefined&&e.indices.groups.𝒜[0]===4&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?<part>[\q{|\ud800|\ud800a}])|(?<empty>)/dv.exec('\ud800a!');m.groups.part==='\ud800a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===2",
    );
    check(
        r"var r=/(?<part>[\q{}])|(?<empty>)/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn empty_member_root_consumers_preserve_defined_empty_captures_and_unicode_progress() {
    check(
        r"var r=/(?<before>)(?<part>[\q{|a|ab}])(?<after>)|(?<empty>)/dvg;var all=Array.from('abXa'.matchAll(r));all.length===4&&all.map(m=>m[0]).join('|')==='ab||a|'&&all.map(m=>m.groups.part).join('|')==='ab||a|'&&all.every(m=>m.groups.before===''&&m.groups.after===''&&m.groups.empty===undefined)&&all[1].indices.groups.part[0]===2&&all[1].indices.groups.part[1]===2&&r.lastIndex===0",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/(?<part>[\q{}])|(?<empty>)/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m.groups.part===''&&m.groups.empty===undefined)",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/(?<empty>)|(?<part>[\q{|😀|😀a}])/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m.groups.empty===''&&m.groups.part===undefined)",
    );
    check(
        r"var calls=0;var out='abXa'.replace(/(?<before>)(?<part>[\q{|a|ab}])(?<after>)|(?<empty>)/vg,function(whole,before,part,after,empty,offset,input,groups){calls++;return arguments.length===8&&whole===part&&before===''&&after===''&&empty===undefined&&groups.part===part&&input==='abXa'?'#':'bad'});out==='##X##'&&calls===4",
    );
    check(
        r"var r=/(?<part>[\q{|a|ab}])|(?<empty>)/vg;r.lastIndex=9;var parts='abX'.split(r);'Xab'.search(r)===0&&r.lastIndex===9&&parts.length===4&&parts[0]===''&&parts[1]==='ab'&&parts[2]===undefined&&parts[3]==='X'&&'abXa'.match(r).join('|')==='ab||a|'&&'abXa'.replace(r,'<$<part>:$<empty>>')==='<ab:><:>X<a:><:>'",
    );
    let mut realm = Realm::default();
    realm
        .eval(
            r"var r=/(?<part>[\q{|a|ab}])|(?<empty>)/dvg;var clone=new RegExp(r);var m=r.exec('!')",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.part===''&&m.groups.empty===undefined&&m.indices.groups.part[1]===0&&clone.exec('ab!').groups.part==='ab'&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn empty_member_root_proofs_finish_finalizers_and_keep_other_features_separate() {
    check(
        r"var marker=0,m;try{m=/([\q{|a|ab}])|()/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/()|([\q{|ab}])/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
    check(
        r"var m=/(?<part>[\q{}--\q{}])|(?<empty>)/dv.exec('!');m.groups.part===undefined&&m.groups.empty===''&&m.indices.groups.empty[0]===0",
    );
    check(
        r"var marker=0,m;try{m=/^([\q{|ab}])|()/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/([\q{|ab}])|()$/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m[2]===undefined&&m.indices[1][0]===0",
    );
    for pattern in [
        r"^([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()$",
        r"([\q{|ab}])|()\1",
        r"([\q{|ab}])x|()",
        r"([\q{|ab}])?|()",
        r"([\q{|ab}])|([\q{|cd}])",
        r"([\q{|ab}])|()|()",
        r"([\q{|ab}&&\p{Any}&&\p{RGI_Emoji}])|()",
    ] {
        let mut realm = Realm::default();
        assert!(
            crate::common::regexp_oracle::check(
                &mut realm,
                &format!(
                    "var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}};marker",
                    JsString::from(pattern)
                )
            ),
            "{pattern}"
        );
    }
    check(
        "var yes=0;for(var p of ['(?<x>[a])|(?<x>)(?<x>)','[\\q{|ab}&&a--a]|']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}
