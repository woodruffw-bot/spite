//! Common reversed line predicates retain whole-choice scope and capture values.
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
fn reversed_common_line_asserted_string_class_layouts_preserve_selected_branch_and_lexical_ranges()
{
    let huge = 10u128.pow(30);
    for (class, input, words, empty_matches) in [
        (
            r"[\q{a|ab|abc}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{a|ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{a|ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "😀a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "𐐨a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "sa",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{|ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{|ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{|ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{|ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|😀|😀a}]",
            "😀a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|😀|😀a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "𐐨a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "sa",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|\n|\na}]",
            "\na",
            [
                Some("\na"),
                None,
                None,
                None,
                Some("\n"),
                Some("\n"),
                None,
                None,
                None,
                None,
            ],
            [
                true, true, true, false, false, false, true, false, false, false,
            ],
        ),
        (
            r"[\q{a|\n|\na}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "\na",
            [
                Some("\na"),
                None,
                Some(""),
                None,
                Some("\n"),
                Some("\n"),
                Some(""),
                None,
                None,
                Some(""),
            ],
            [
                true, true, true, false, false, false, true, false, false, false,
            ],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\n\q{a|\na}]",
            "\na",
            [
                Some("\na"),
                None,
                None,
                None,
                Some("\n"),
                Some("\n"),
                None,
                None,
                None,
                None,
            ],
            [
                true, true, true, false, false, false, true, false, false, false,
            ],
        ),
        (
            r"[\n\q{a|\na}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "a\n",
            [
                None,
                Some("a\n"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "\na",
            [
                Some("\na"),
                None,
                None,
                None,
                Some("\n"),
                Some("\n"),
                None,
                None,
                None,
                None,
            ],
            [
                true, true, true, false, false, false, true, false, false, false,
            ],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
        ),
    ] {
        for (mode, (cp, cs, ep, es, outer_head, outer_tail)) in [
            (r"^", r"$", r"", r"", r"$", r""),
            (r"\b", r"\B", r"\B", r"", r"", r"^"),
            (r"", r"\B", r"^", r"", r"$", r"^"),
            (r"$", r"^", r"\b", r"", r"$\b", r"^"),
            (r"", r"", r"\b", r"", r"$\B", r"^"),
            (r"", r"", r"", r"", r"$", r"\b^"),
            (r"^", r"$", r"", r"", r"$", r"\B^"),
            (r"\b", r"\B", r"\B", r"", r"$\b", r"\B^"),
            (r"", r"\B", r"^", r"", r"$\B", r"\b^"),
            (r"$", r"^", r"\b", r"", r"$\B", r"\B^"),
        ]
        .into_iter()
        .enumerate()
        {
            for empty_first in [false, true] {
                for layout in 0..6 {
                    for frame in 0..3 {
                        let (character, empty, characters, before, after, empties, never) =
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

                        let c = format!("{cp}{character}{cs}");
                        let e = format!("{ep}{empty}{es}");
                        let source = if empty_first {
                            format!("{e}|{c}")
                        } else {
                            format!("{c}|{e}")
                        };
                        let enclosed = match frame {
                            0 => format!("(?<enclosing>{source})"),
                            1 => format!("(?:(?<enclosing>{source}))"),
                            _ => format!("(?<enclosing>(?:(?<inner>{source})))"),
                        };
                        let pattern =
                            JsString::from(format!("{outer_head}{enclosed}{outer_tail}").as_str());
                        let choice = if empty_first && empty_matches[mode] {
                            Some(false)
                        } else if words[mode].is_some() {
                            Some(true)
                        } else {
                            empty_matches[mode].then_some(false)
                        };
                        let matched = choice.is_some();
                        let class_selected = choice == Some(true);
                        let whole = JsString::from(if class_selected {
                            words[mode].unwrap()
                        } else {
                            ""
                        });
                        let end = whole.len();
                        let text = JsString::from(input);
                        let named = true;
                        check(&format!(
                            r#"
var r=new RegExp({pattern:?},'dvgym'),m=r.exec({text:?}),ok=true;
if({matched}){{
 if({named}){{var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(',');
  for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
   if({class_selected}&&characters.indexOf(name)!==-1){{expected={whole:?};from=0;to={end}}}
   if({class_selected}&&before.indexOf(name)!==-1){{expected='';from=0;to=0}}
   if({class_selected}&&after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
   if(!{class_selected}&&empty.indexOf(name)!==-1){{expected='';from=0;to=0}}
   if(['enclosing','inner'].indexOf(name)!==-1){{expected={whole:?};from=0;to={end}}}
   if(never.indexOf(name)!==-1)expected=undefined;
   ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
   if(expected===undefined)ok=ok&&m.indices[slot]===undefined;else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
  }}ok=ok&&m.length===keys.length+1;
 }}else ok=ok&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined;
 ok=ok&&m[0]==={whole:?}&&m.index===0&&m.indices[0][1]==={end}&&r.lastIndex==={end};
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

#[test]
fn reversed_common_line_asserted_string_class_consumers_keep_actual_endpoint_scope() {
    check(
        r"var m=/$((?<empty>)|(?<part>[\q{\n|\na}]))^/dvm.exec('x\nab');m.index===1&&m[0]==='\n'&&m[1]==='\n'&&m.groups.empty===undefined&&m.groups.part==='\n'&&m.indices.groups.part===m.indices[3]&&m.indices[3][1]===2",
    );
    check(
        r"var m=/$(?<enclosing>(?<part>[\q{|\n|\na}])|(?<empty>))^/dvm.exec('\n');m[0]==='\n'&&m.groups.enclosing==='\n'&&m.groups.part==='\n'&&m.groups.empty===undefined&&m.indices.groups.part===m.indices[2]",
    );
    check(
        r"var m=/$(?<enclosing>(?<empty>)|(?<part>[\q{|\n|\na}]))^/dvm.exec('\n');m[0]===''&&m.groups.enclosing===''&&m.groups.empty===''&&m.groups.part===undefined&&m.indices.groups.empty===m.indices[2]",
    );
    check(
        r"var m=/$\B(?<enclosing>(?<empty>)|(?<part>[\q{\n|\na}]))\b^/dvm.exec('\na');m[0]==='\n'&&m.groups.enclosing==='\n'&&m.groups.empty===undefined&&m.groups.part==='\n'&&m.indices.groups.part[1]===1",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{😀|😀\n}])|(?<empty>))^/dvgym;r.lastIndex=1;var m=r.exec('😀\nx');m.index===0&&m[0]==='😀\n'&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.enclosing[1]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<enclosing>(?<\u{1D49C}>[\q{s|s\n}])|(?<empty>))^/dvim,m=r.exec('ſ\n');m[0]==='ſ\n'&&m.groups.enclosing===m[0]&&m.groups.𝒜===m[0]&&m.groups.empty===undefined&&m.indices.groups.𝒜===m.indices[2]&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?<enclosing>(?<part>[\q{a|a\n}])|(?<empty>))^/dvm.exec('a\nx');m.index===0&&m[0]==='a\n'&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.part[1]===2",
    );
    check(
        r"var r=/$(?<enclosing>(?<part>[\q{\n|\na}])|(?<empty>))^/dvgm,a=Array.from('x\nab\n\n'.matchAll(r));a.length===4&&a.map(m=>m.index).join(',')==='1,4,5,6'&&a[0][0]==='\n'&&a[3][0]===''&&a[3].groups.part===undefined&&a[3].groups.empty===''&&a[3].indices.groups.empty===a[3].indices[3]&&a.every(m=>m.groups.enclosing===m[0])&&r.lastIndex===0",
    );
    check(
        r"var calls=0,out='x\nab\n\n'.replace(/$(?<enclosing>(?<part>[\q{\n|\na}])|(?<empty>))^/vgm,function(all,outer,p,e,offset,input,groups){calls++;return arguments.length===7&&outer===all&&groups.enclosing===all&&groups.part===p&&groups.empty===e&&input==='x\nab\n\n'?(p===undefined?'<E>':'#'):'bad'});out==='x#ab##<E>'&&calls===4",
    );
    check(
        r"var r=/$(?<enclosing>(?<part>[\q{\n|\na}])|(?<empty>))^/vgm;r.lastIndex=9;var a='x\nab'.split(r);'x\nab'.search(r)===1&&r.lastIndex===9&&a.length===5&&a[0]==='x'&&a[1]==='\n'&&a[2]==='\n'&&a[3]===undefined&&a[4]==='ab'",
    );
    check(
        r"var r=/$(([\q{\n|\na}])|())^/vgm;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('\n')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"var marker=0,m;try{m=/$\B(?<enclosing>\b(?<part>[\q{\n|\na}])|(?<empty>))\b^/dvm.exec('x\nab')}finally{marker=3}marker===3&&m===null",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/$(?<enclosing>(?<part>[\q{\n|\na}])|(?<empty>))^/dvgm,clone=new RegExp(r),m=r.exec('x\nab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"var n=clone.exec('');m.groups.enclosing==='\n'&&m.groups.part==='\n'&&m.groups.empty===undefined&&m.indices.groups.part===m.indices[2]&&n.groups.enclosing===''&&n.groups.part===undefined&&n.groups.empty===''&&n.indices.groups.empty===n.indices[3]&&clone.source===r.source&&r.lastIndex===2"),Ok(Value::Boolean(true)));
    for pattern in [
        r"$$((([\q{a|ab}])|()))^",
        r"$((([\q{a|ab}])|()))^^",
        r"$^((([\q{a|ab}])|()))^",
        r"$((([\q{a|ab}])|()))()^",
        r"$((([\q{a|ab}])|()))?^",
        r"$((([\q{a|ab}])|()|()))^",
        r"$((([\q{a|ab}])|([\q{}])))^",
        r"$((([\q{a|ab}])|(\b)?))^",
        r"$((([\q{a|ab}])|\1))^",
        r"$\b((\B([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))^",
        r"$((([\q{a|ab}])+|(^)))^",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'vm').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
#[test]
fn reversed_common_line_asserted_string_class_deep_prefixes_long_members_and_host_aborts_remain_flat()
 {
    let pattern = JsString::from(
        format!(
            "${}()|([\\q{{|\\n|\\na}}]){}^",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dvm').exec('x\\nab');m.length===1003&&m[0]==='\\n'&&m[1]==='\\n'&&m[1000]==='\\n'&&m[1001]===undefined&&m[1002]==='\\n'&&m.indices[1002][0]===1&&m.indices[1002][1]===2"
    ));
    let word = format!("{}\n", "x".repeat(20_000));
    let pattern =
        JsString::from(format!(r"(?<enclosing>(?<part>[\q{{{word}}}])|(?<empty>))^").as_str());
    let input = JsString::from(word.as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dvm').exec({input:?});m[0].length===20001&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.part===m.indices[2]"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'vm')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm.eval(r"var r=/$([\q{}]|)^/vgm;r.lastIndex=1").unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(input)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval(r"var r=/$(|[\q{}])^/dvgym,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", r"\n".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.index===0&&m[0]===''&&m.length===2&&m[1]===''&&m.indices[1][0]===0&&m.indices[1][1]===0&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}
