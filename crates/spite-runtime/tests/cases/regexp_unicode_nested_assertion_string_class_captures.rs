//! Canonical assertion layers interleaved with complete single-class wrappers.
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
fn nested_assertion_string_class_layouts_keep_layer_and_prefix_capture_ranges() {
    let huge = 2u128;
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
        for word_mode in 0..3 {
            for (head, tail) in [("^", "$")] {
                for (outer_head, outer_tail) in [("^", ""), ("", "$"), ("^", "$")] {
                    for frame in 0..3 {
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
                            let inner_tail = if word_mode == 2 { r"\B$" } else { tail };
                            let body = format!("{head}{pattern}{inner_tail}");
                            let pattern = JsString::from(
                                match frame {
                                    0 => format!("(?<enclosing>{body})"),
                                    1 => format!("(?:(?<enclosing>{body}))"),
                                    _ => format!("(?<enclosing>(?:(?<middle>(?<inner>{body}))))"),
                                }
                                .as_str(),
                            );
                            let word_tail = match word_mode {
                                0 => format!(r"\b{outer_tail}"),
                                1 => r"\B$".to_owned(),
                                _ => format!(r"\b{outer_tail}"),
                            };
                            let pattern = JsString::from(
                                format!(
                                    "{outer_head}(?<layer>^{}$){word_tail}",
                                    String::from_utf16(pattern.code_units()).unwrap(),
                                )
                                .as_str(),
                            );
                            let matched = match word_mode {
                                0 => word.is_some_and(|word| !word.is_empty()),
                                1 => word == Some(""),
                                _ => false,
                            };
                            let whole = JsString::from(word.unwrap_or(""));
                            let end = 1 + whole.len();
                            let text =
                                JsString::from(format!("\n{}", word.unwrap_or("!")).as_str());
                            let named = true;
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
                        if(['layer','enclosing','middle','inner'].indexOf(name)!==-1){{expected={whole:?};from=1;to={end}}}
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
        }
    }
}
#[test]
fn nested_assertion_string_class_values_aliases_consumers_and_scope_preserve_original_slots() {
    check(
        r"var m=/\b(?<layer>^(?<enclosing>\b()(?<part>[\q{a|ab}])()\b)$)\b/dv.exec('ab');m[0]==='ab'&&m.groups.layer==='ab'&&m.groups.enclosing==='ab'&&m.groups.part==='ab'&&m[3]===''&&m[5]===''&&m.indices.groups.layer===m.indices[1]&&m.indices.groups.part===m.indices[4]&&m.indices[5][0]===2",
    );
    check(
        r"var r=/\b(^(\B()([\q{|a|ab}])())$)\b/dvg;r.lastIndex=1;var m=r.exec('ab!');m===null&&r.lastIndex===0",
    );
    check(
        r"var m=/\B(^(\B()([\q{}])()\B)$)\B/dv.exec('');m.length===6&&m[0]===''&&m[1]===''&&m[2]===''&&m[3]===''&&m[4]===''&&m[5]===''&&m.indices[2][0]===0",
    );
    check(
        r"var m=/(^(\b()([\q{a|ab}])()\B))\B/dv.exec('ab!');m[0]==='a'&&m[1]==='a'&&m[4]==='a'&&m.indices[5][0]===1",
    );
    check(
        r"var r=/\B(^(\B()([\q{😀|😀a}])()\b)$)\b/dvgy;r.lastIndex=1;var m=r.exec('😀a');m.index===0&&m[1]==='😀a'&&m[4]==='😀a'&&m.indices[5][0]===3&&r.lastIndex===3",
    );
    check(
        r"var m=/\b(?<\u{1D49C}>^(?<enclosing>\b()(?<part>[\q{s|sa|k|ka}])()\b)$)\b/dvi.exec('ſa');m.groups.𝒜==='ſa'&&m.groups.enclosing==='ſa'&&m.groups.part==='ſa'&&Object.keys(m.groups).join(',')==='𝒜,enclosing,part'&&m.indices.groups.𝒜[1]===2",
    );
    check(
        r"var r=/\b(^(\b(?<before>)(?<part>[\q{a|ab}])(?<after>)\b)$)\b/dvgm;var a=Array.from('ab\na\n!'.matchAll(r));a.length===2&&a.every(m=>m[1]===m[0]&&m[2]===m[0]&&m.groups.part===m[0]&&m.groups.before===''&&m.groups.after==='')&&a[1].indices[2][1]===4&&r.lastIndex===0",
    );
    check(
        r"var calls=0;var out='ab\na\n!'.replace(/\b(^(\b(?<before>)(?<part>[\q{a|ab}])(?<after>)\b)$)\b/vgm,function(all,one,two,b,p,a,offset,input,groups){calls++;return arguments.length===9&&all===one&&one===two&&all===p&&b===''&&a===''&&groups.part===p&&input==='ab\na\n!'?'#':'bad'});out==='#\n#\n!'&&calls===2",
    );
    check(
        r"var r=/\b(^(\B()([\q{|a|ab}])())$)\b/vg;r.lastIndex=9;'ab'.search(r)===-1&&r.lastIndex===9&&Array.from('ab'.matchAll(r)).length===0&&'ab'.replace(r,'bad')==='ab'&&'ab'.split(r).length===1",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/\b(?<layer>^(?<enclosing>\b()(?<part>[\q{|a|ab}])()\b)$)\b/dvg,clone=new RegExp(r),m=r.exec('ab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.layer==='ab'&&m.groups.enclosing==='ab'&&m.groups.part==='ab'&&clone.exec('')===null&&r.lastIndex===2"),Ok(Value::Boolean(true)));
    check(
        r"var marker=0,m;try{m=/\b(^(\B()([\q{|a|ab}])())$)\b/dv.exec('ab')}finally{marker=3}marker===3&&m===null",
    );
    // The Unicode candidate regression uses specified boundaries rather than
    // V8's nonsticky empty-\\B search behavior inside a surrogate pair.
    let pattern = JsString::from(r"(?<layer>\B(?<enclosing>()(?<part>[\q{|a|ab}])()\B))\B");
    check(&format!(
        "var r=new RegExp({pattern:?},'dvg');r.lastIndex=2;var m=r.exec('😀a😀!');m.index===5&&m.groups.layer===''&&m.groups.enclosing===''&&m.groups.part===''&&m.indices.groups.part[0]===5&&r.lastIndex===5"
    ));
    check(
        r"var marker=0,m;try{m=/(^()([\q{a|ab}])()$)()/dv.exec('ab')}finally{marker=3}marker===3&&m.length===6&&m[0]==='ab'&&m[1]==='ab'&&m[2]===''&&m[3]==='ab'&&m[4]===''&&m[5]===''&&m.indices[5][0]===2",
    );
    check(
        r"var marker=0,m;try{m=/^\b\B((()([\q{a|ab}])()))$/dv.exec('ab')}finally{marker=3}marker===3&&m===null",
    );
    check(
        r"var marker=0,m,r=/\b(^(\B()[\q{a|ab}]|())$)\b/dvgm;r.lastIndex=1;try{m=r.exec('ab\n\n')}finally{marker=3}marker===3&&m===null&&r.lastIndex===0",
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b(^(\\B()([\\q{a|ab}])+())$)\\b",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"\b(^(\B()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])())$)\b",
        r"\b(^(\B()([\q{a|ab}])\1)$)\b",
        r"\b(^(\B()([[\q{a|ab}]&&\p{RGI_Emoji}])+())$)\b",
        r"\b(^(\B()[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|())$)\b",
        r"(^()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()$)()",
        r"^\b\B((()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()))$",
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
}
