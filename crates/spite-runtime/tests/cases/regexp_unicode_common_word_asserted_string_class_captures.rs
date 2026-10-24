//! Common outside word predicates retain whole-choice scope and capture values.
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
fn common_word_asserted_string_class_layouts_preserve_selected_branch_and_lexical_ranges() {
    let huge = 10u128.pow(30);
    for (class, input, words, empty_matches) in [
        (
            r"[\q{a|ab|abc}]",
            "abc",
            [
                Some("abc"),
                None,
                None,
                None,
                Some("abc"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "abc",
            [
                Some("abc"),
                None,
                None,
                None,
                Some("abc"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[x\q{a|ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{a|ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "abc",
            [
                Some("abc"),
                None,
                None,
                None,
                Some("abc"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "abc",
            [
                Some("abc"),
                None,
                None,
                None,
                Some("abc"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "abc",
            [
                Some("abc"),
                None,
                None,
                None,
                Some("abc"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "😀a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "𐐨a",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "sa",
            [
                Some("sa"),
                None,
                None,
                None,
                Some("sa"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "abc",
            [None, None, None, None, None, None, None, None, None, None],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{ab}\q{}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[x\q{|ab}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[x\q{|ab}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{|ab}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[a-z\q{|ab}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|😀|😀a}]",
            "😀a",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|😀|😀a}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "𐐨a",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "sa",
            [
                Some("sa"),
                None,
                None,
                None,
                Some("sa"),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "abc",
            [
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|\n|\na}]",
            "\na",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{a|\n|\na}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "\na",
            [
                None,
                None,
                None,
                Some(""),
                None,
                Some(""),
                None,
                None,
                None,
                Some(""),
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\n\q{a|\na}]",
            "\na",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\n\q{a|\na}]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "a\n",
            [
                Some("a\n"),
                None,
                None,
                None,
                Some(""),
                None,
                Some("a\n"),
                None,
                None,
                None,
            ],
            [
                true, false, true, false, true, false, false, false, false, false,
            ],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "!",
            [
                None,
                None,
                None,
                None,
                None,
                Some(""),
                None,
                None,
                None,
                None,
            ],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "\na",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "!",
            [None, None, None, None, None, None, None, None, None, None],
            [
                false, true, false, false, false, true, false, false, false, false,
            ],
        ),
    ] {
        for (mode, (cp, cs, ep, es, outer_head, outer_tail)) in [
            (r"^", r"$", r"", r"", r"\b", r""),
            (r"\b", r"\B", r"\B", r"", r"\B", r""),
            (r"", r"\B", r"^", r"", r"", r"\b"),
            (r"$", r"^", r"\b", r"", r"", r"\B"),
            (r"", r"", r"\b", r"", r"\b", r"\b"),
            (r"", r"", r"", r"", r"\B", r"\B"),
            (r"^", r"$", r"", r"", r"\b", r"\B"),
            (r"\b", r"\B", r"\B", r"", r"\B", r"\b"),
            (r"", r"\B", r"^", r"", r"^\b", r"\b$"),
            (r"$", r"^", r"\b", r"", r"^\B", r"\B$"),
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
fn common_word_asserted_string_class_consumers_keep_distinct_endpoints_and_branch_aliases() {
    check(
        r"var m=/\b(?<enclosing>(?<empty>)|(?<part>[\q{a|ab}]))\B/dv.exec('ab!');m[0]==='a'&&m.groups.enclosing==='a'&&m.groups.empty===undefined&&m.groups.part==='a'&&m.indices.groups.part===m.indices[3]&&m.indices[3][1]===1",
    );
    check(
        r"var m=/\B(?<enclosing>(?<part>[\q{|a|ab}])|(?<empty>))\B/dv.exec('!');m[0]===''&&m.groups.enclosing===''&&m.groups.part===''&&m.groups.empty===undefined&&m.indices.groups.part===m.indices[2]",
    );
    check(
        r"var m=/\B(?<enclosing>(?<empty>)|(?<part>[\q{|a|ab}]))\B/dv.exec('!');m[0]===''&&m.groups.enclosing===''&&m.groups.empty===''&&m.groups.part===undefined&&m.indices.groups.empty===m.indices[2]",
    );
    check(
        r"var r=/\B(?<enclosing>(?<part>[\q{😀|😀a}])|(?<empty>))\b/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.enclosing[1]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/\b(?<enclosing>(?<\u{1D49C}>[\q{s|sa|k|ka}])|(?<empty>))\B/dvi,m=r.exec('ſa!');m[0]==='ſ'&&m.groups.enclosing==='ſ'&&m.groups.𝒜==='ſ'&&m.groups.empty===undefined&&m.indices.groups.𝒜===m.indices[2]&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var marker=0,m;try{m=/\b(?<enclosing>\B(?<part>[\q{a|ab}])|(?<empty>))/dv.exec('ab!')}finally{marker=3}marker===3&&m.index===0&&m[0]===''&&m.groups.enclosing===''&&m.groups.part===undefined&&m.groups.empty===''",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{a|ab}])|(?<empty>))\B/dvg,m=r.exec('ab!');m[0]==='a'&&m.groups.enclosing==='a'&&m.groups.part==='a'&&m.groups.empty===undefined&&r.lastIndex===1",
    );
    check(
        r"var r=/^\b(?<enclosing>(?<empty>)|(?<part>[\q{a|ab}]))\b$/dvgm;r.lastIndex=1;var m=r.exec('\nab');m.index===1&&m[0]==='ab'&&m.groups.enclosing==='ab'&&m.groups.empty===undefined&&m.groups.part==='ab'&&m.indices.groups.part[1]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/\B(?<enclosing>(?<part>[\q{|a|ab}])|(?<empty>))\B/dvg,a=Array.from('!!'.matchAll(r));a.length===3&&a.map(m=>m.index).join(',')==='0,1,2'&&a.every(m=>m[0]===''&&m.groups.enclosing===''&&m.groups.part===''&&m.groups.empty===undefined)&&r.lastIndex===0",
    );
    check(
        r"var calls=0,out='ab!'.replace(/\b(?<enclosing>(?<empty>)|(?<part>[\q{a|ab}]))\B/vg,function(all,outer,e,p,offset,input,groups){calls++;return arguments.length===7&&outer===all&&all==='a'&&e===undefined&&p==='a'&&groups.part===p&&groups.enclosing===all&&input==='ab!'?'#':'bad'});out==='#b!'&&calls===1",
    );
    check(
        r"var r=/\b(?<enclosing>(?<part>[\q{a|ab}])|(?<empty>))\b/vg;r.lastIndex=9;var a='ab!'.split(r);'!ab!'.search(r)===1&&r.lastIndex===9&&a.length===5&&a[0]===''&&a[1]==='ab'&&a[2]==='ab'&&a[3]===undefined&&a[4]==='!'",
    );
    check(
        r"var r=/\B(([\q{|a|ab}])|())\B/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/\b(?<enclosing>(?<part>[\q{a|ab}])|(?<empty>))/dvg,clone=new RegExp(r),m=r.exec('ab!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('z');m.groups.enclosing==='ab'&&m.groups.part==='ab'&&m.groups.empty===undefined&&m.indices.groups.part===m.indices[2]&&n.groups.enclosing===''&&n.groups.part===undefined&&n.groups.empty===''&&n.indices.groups.empty===n.indices[3]&&clone.source===r.source&&r.lastIndex===2"),Ok(Value::Boolean(true)));
    let pattern = JsString::from(r"\B(?<enclosing>(?<part>[\q{|a|ab}])|(?<empty>))\B");
    check(&format!(
        "var r=new RegExp({pattern:?},'dvg');r.lastIndex=2;var m=r.exec('😀a😀!');m.index===5&&m.groups.enclosing===''&&m.groups.part===''&&m.groups.empty===undefined&&m.indices.groups.enclosing[0]===5&&m.indices.groups.part===m.indices[2]&&r.lastIndex===5"
    ));
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b\\b((([\\q{a|ab}])|()))",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b((([\\q{a|ab}])|()))\\b\\b",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^\\b^((([\\q{a|ab}])|()))$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",null,0],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",null,0],[true,\"ab!\",null,0],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[2,4],[2,4],[2,4],[2,4],\"<undefined>\"],\"index\":2},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b((([\\q{a|ab}])|()))()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\",[2,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\",[2,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="\\b((([\\q{a|ab}])|()))?",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"a\",\"a\",\"a\",\"a\",\"<undefined>\"],\"indices\":[[0,1],[0,1],[0,1],[0,1],\"<undefined>\"],\"index\":0},1],[false,\"ababa\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[false,\"\\nab\\n\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[1,3],[1,3],[1,3],[1,3],\"<undefined>\"],\"index\":1},3],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"a\",\"a\",\"a\",\"a\",\"<undefined>\"],\"indices\":[[0,1],[0,1],[0,1],[0,1],\"<undefined>\"],\"index\":0},1],[true,\"ababa\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[0,2],[0,2],[0,2],[0,2],\"<undefined>\"],\"index\":0},2],[true,\"\\nab\\n\",{\"values\":[\"ab\",\"ab\",\"ab\",\"ab\",\"<undefined>\"],\"indices\":[[1,3],[1,3],[1,3],[1,3],\"<undefined>\"],\"index\":1},3],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"\b\b((([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))",
        r"\b((([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))\b\b",
        r"^\b^((([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))$",
        r"\b((([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))()",
        r"\b((([[\q{a|ab}]&&\p{RGI_Emoji}])|()))?",
        r"\b((([\q{a|ab}])|()|()))",
        r"\b((([\q{a|ab}])|([\q{}])))",
        r"\b((([\q{a|ab}])|(\b)?))",
        r"\b((([\q{a|ab}])|\1))",
        r"\b((\B([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()))",
        r"\b((([\q{a|ab}])+|(^)))",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
#[test]
fn common_word_asserted_string_class_deep_prefixes_long_members_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            "\\b{}()|([\\q{{|a|ab}}]){}\\B",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('ab!');m.length===1003&&m[0]==='a'&&m[1]==='a'&&m[1000]==='a'&&m[1001]===undefined&&m[1002]==='a'&&m.indices[1000][1]===1"
    ));
    let word = "x".repeat(20_000);
    let pattern =
        JsString::from(format!(r"\b(?<enclosing>(?<part>[\q{{{word}}}])|(?<empty>))\b").as_str());
    let input = JsString::from(word.as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===20000&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.part===m.indices[2]"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'v')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval("var marker=0").unwrap();
    realm
        .eval(r"var r=/([\q{}]|\b\B)\b/vg;r.lastIndex=1")
        .unwrap();
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
    realm.eval(r"var r=/\B(|[\q{ab}])/dvgy,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "!".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.index===0&&m[0]===''&&m.length===2&&m[1]===''&&m.indices[1][0]===0&&m.indices[1][1]===0&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}
