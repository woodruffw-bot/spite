//! Required surrounding assertion captures retain original lexical slots.
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
fn surrounding_asserted_string_class_layouts_keep_spine_points_and_conditional_branch_slots() {
    let huge = 10u128.pow(30);
    for (class, input, words, empty_matches) in [
        (
            r"[\q{a|ab|abc}]",
            "abc",
            [Some("ab"), None, None, None, None, None, Some("abc")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{a|ab|abc}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "abc",
            [Some("ab"), None, None, None, None, None, Some("abc")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[x\q{a|ab}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[x\q{a|ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "abc",
            [Some("ab"), None, None, None, None, None, Some("abc")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "abc",
            [Some("ab"), None, None, None, None, None, Some("abc")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "abc",
            [Some("a"), None, None, None, None, None, Some("abc")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "😀a",
            [None, None, None, None, None, None, Some("😀a")],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "𐐨a",
            [None, None, None, None, None, None, Some("𐐨a")],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "sa",
            [Some("s"), None, None, None, None, None, Some("sa")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|ab}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|a|ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{ab}\q{}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{ab}\q{}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[x\q{|ab}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[x\q{|ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[a-z\q{|ab}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[a-z\q{|ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "abc",
            [Some("ab"), None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|😀|😀a}]",
            "😀a",
            [None, None, None, None, None, None, Some("😀a")],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|😀|😀a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "𐐨a",
            [None, None, None, None, None, None, Some("𐐨a")],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "sa",
            [Some("s"), None, None, None, None, None, Some("sa")],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{}]",
            "abc",
            [None, None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "abc",
            [None, None, None, None, None, None, None],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{a|\n|\na}]",
            "\na",
            [None, None, None, Some("\n"), None, None, Some("\na")],
            [false, true, true, true, false, false, false],
        ),
        (
            r"[\q{a|\n|\na}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "\na",
            [None, Some(""), None, Some("\n"), None, None, Some("\na")],
            [false, true, true, true, false, false, false],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\n\q{a|\na}]",
            "\na",
            [None, None, None, Some("\n"), None, None, Some("\na")],
            [false, true, true, true, false, false, false],
        ),
        (
            r"[\n\q{a|\na}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "a\n",
            [
                Some("a\n"),
                None,
                Some("a\n"),
                None,
                None,
                None,
                Some("a\n"),
            ],
            [false, false, false, false, false, false, true],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "\na",
            [None, None, None, Some("\n"), None, None, Some("\na")],
            [false, true, true, true, false, false, false],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "!",
            [None, None, None, None, None, None, None],
            [false, true, false, false, false, false, false],
        ),
    ] {
        for (mode, (cp, cs, ep, es, head, tail)) in [
            (r"", r"", r"", r"", r"\b", r"\B"),
            (r"^", r"$", r"", r"", r"\B", r"\B"),
            (r"\b", r"\B", r"\B", r"", r"^", r"$"),
            (r"$", r"^", r"^", r"$", r"$", r"^"),
            (r"\b\B", r"", r"", r"", r"^\b", r"\b$"),
            (r"", r"", r"\b\B", r"", r"\b\B", r""),
            (r"^", r"$", r"\b", r"", r"", r""),
        ]
        .into_iter()
        .enumerate()
        {
            for empty_first in [false, true] {
                for layout in 0..6 {
                    for frame in 0..7 {
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
                        let body = if empty_first {
                            format!("{e}|{c}")
                        } else {
                            format!("{c}|{e}")
                        };
                        let (pattern, starts, wholes, ends, outside_never) = match frame {
                            0 => (
                                format!(
                                    "(?<scopeStart>{head})(?<scopeWhole>{body})(?<scopeEnd>{tail})"
                                ),
                                "scopeStart",
                                "scopeWhole",
                                "scopeEnd",
                                "",
                            ),
                            1 => (
                                format!(
                                    "(?<scopeOuter>(?<scopeStart>{head})(?<scopeWhole>{body})(?<scopeEnd>{tail}))"
                                ),
                                "scopeStart",
                                "scopeOuter,scopeWhole",
                                "scopeEnd",
                                "",
                            ),
                            2 => (
                                format!(
                                    "(?<scopeNeverBefore>){{0}}(?<scopeStart>{head})(?<scopeWhole>{body})(?<scopeEnd>{tail})(?<scopeNeverAfter>)?"
                                ),
                                "scopeStart",
                                "scopeWhole",
                                "scopeEnd",
                                "scopeNeverBefore,scopeNeverAfter",
                            ),
                            3 => (
                                format!(
                                    "(?<scopeStartOuter>(?<scopeStart>{head})(?<scopeStartEmpty>))(?<scopeWhole>{body})(?<scopeEndOuter>(?<scopeEnd>{tail})(?<scopeEndEmpty>))"
                                ),
                                "scopeStartOuter,scopeStart,scopeStartEmpty",
                                "scopeWhole",
                                "scopeEndOuter,scopeEnd,scopeEndEmpty",
                                "",
                            ),
                            4 => (
                                format!(
                                    "(?<scopeStart>{head})(?<scopeChoice>(?<scopeFirst>(?<scopeFirstInner>))|(?<scopeLater>(?<scopeLaterInner>)))(?<scopeWhole>{body})(?<scopeEnd>{tail})"
                                ),
                                "scopeStart,scopeChoice,scopeFirst,scopeFirstInner",
                                "scopeWhole",
                                "scopeEnd",
                                "scopeLater,scopeLaterInner",
                            ),
                            5 => (
                                format!(
                                    "(?<scopeStart>{head}){{2,3}}(?<scopeWhole>{body})(?<scopeEnd>{tail}){{{huge}}}"
                                ),
                                "scopeStart",
                                "scopeWhole",
                                "scopeEnd",
                                "",
                            ),
                            _ => (
                                format!("(?<scopeStart>{head})(?:{body})(?<scopeEnd>{tail})"),
                                "scopeStart",
                                "",
                                "scopeEnd",
                                "",
                            ),
                        };
                        let pattern = JsString::from(pattern.as_str());
                        let selected = if empty_first && empty_matches[mode] {
                            Some(false)
                        } else if words[mode].is_some() {
                            Some(true)
                        } else {
                            empty_matches[mode].then_some(false)
                        };
                        let matched = selected.is_some();
                        let class_selected = selected == Some(true);
                        let whole = JsString::from(if class_selected {
                            words[mode].unwrap()
                        } else {
                            ""
                        });
                        let end = whole.len();
                        let text = JsString::from(input);
                        check(&format!(
                            r#"
var r=new RegExp({pattern:?},'dvgym'),m=r.exec({text:?}),ok=true;
if({matched}){{
 var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),empty='{empties}'.split(','),never='{never}'.split(','),starts='{starts}'.split(','),wholes='{wholes}'.split(','),ends='{ends}'.split(','),outsideNever='{outside_never}'.split(',');
 for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
  if({class_selected}&&characters.indexOf(name)!==-1){{expected={whole:?};from=0;to={end}}}
  if({class_selected}&&before.indexOf(name)!==-1){{expected='';from=0;to=0}}
  if({class_selected}&&after.indexOf(name)!==-1){{expected='';from={end};to={end}}}
  if(!{class_selected}&&empty.indexOf(name)!==-1){{expected='';from=0;to=0}}
  if(starts.indexOf(name)!==-1){{expected='';from=0;to=0}}
  if(wholes.indexOf(name)!==-1){{expected={whole:?};from=0;to={end}}}
  if(ends.indexOf(name)!==-1){{expected='';from={end};to={end}}}
  if(never.indexOf(name)!==-1||outsideNever.indexOf(name)!==-1)expected=undefined;
  ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
  if(expected===undefined)ok=ok&&m.indices[slot]===undefined;else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
 }}ok=ok&&m.length===keys.length+1&&m[0]==={whole:?}&&m.index===0&&m.indices[0][1]==={end}&&r.lastIndex==={end};
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
fn surrounding_asserted_string_class_consumers_keep_point_whole_and_branch_aliases() {
    check(
        r"var m=/(?<start>\b)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\B)/dv.exec('ab!');m[0]==='a'&&m.groups.start===''&&m.groups.whole==='a'&&m.groups.part==='a'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start===m.indices[1]&&m.indices.groups.start[0]===0&&m.indices.groups.part===m.indices[3]&&m.indices.groups.end===m.indices[5]&&m.indices.groups.end[0]===1",
    );
    check(
        r"var m=/(?<start>\b)(?<whole>(?<empty>)|(?<part>[\q{a|ab}]))(?<end>\B)/dv.exec('ab!');m[0]==='a'&&m.groups.empty===undefined&&m.groups.part==='a'&&m.indices.groups.part===m.indices[4]&&Object.keys(m.groups).join(',')==='start,whole,empty,part,end'",
    );
    check(
        r"var m=/(?<start>\B)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<end>\B)/dv.exec('!');m[0]===''&&m.groups.start===''&&m.groups.whole===''&&m.groups.part===''&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.part===m.indices[3]",
    );
    check(
        r"var m=/(?<start>\B)(?<whole>(?<empty>)|(?<part>[\q{|a|ab}]))(?<end>\B)/dv.exec('!');m[0]===''&&m.groups.whole===''&&m.groups.empty===''&&m.groups.part===undefined&&m.indices.groups.empty===m.indices[3]&&m.groups.end===''",
    );
    check(
        r"var m=/(?<prefixNever>){0}(?<start>\b)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\B)(?<suffixNever>)?/dv.exec('ab!');m.length===8&&m.groups.prefixNever===undefined&&m.groups.suffixNever===undefined&&m.groups.whole==='a'&&m.groups.part==='a'&&m.groups.end===''&&Object.hasOwn(m.indices.groups,'prefixNever')&&m.indices.groups.prefixNever===undefined&&m.indices.groups.end===m.indices[6]",
    );
    check(
        r"var m=/(?<start>\b)(?<choice>(?<first>(?<firstInner>))|(?<later>(?<laterInner>)))(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\B)/dv.exec('ab!');m.groups.choice===''&&m.groups.first===''&&m.groups.firstInner===''&&m.groups.later===undefined&&m.groups.laterInner===undefined&&m.groups.whole==='a'&&m.groups.part==='a'&&m.indices.groups.first[0]===0&&m.indices.groups.end[0]===1",
    );
    check(
        r"var m=/(?<start>^){2,3}(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>$){2}/dv.exec('ab');m[0]==='ab'&&m.groups.start===''&&m.groups.whole==='ab'&&m.groups.part==='ab'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.end[0]===2",
    );
    check(
        r"var m=/(?<start>$)(?<whole>(?<part>[\q{\n|\na}])|(?<empty>))(?<end>^)/dvm.exec('x\nab');m.index===1&&m[0]==='\n'&&m.groups.start===''&&m.groups.whole==='\n'&&m.groups.part==='\n'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start[0]===1&&m.indices.groups.end[0]===2",
    );
    check(
        r"var r=/(?<start>\B)(?<whole>(?<part>[\q{😀|😀\n}])|(?<empty>))(?<end>^)/dvgym;r.lastIndex=1;var m=r.exec('😀\nx');m.index===0&&m[0]==='😀\n'&&m.groups.start===''&&m.groups.end===''&&m.indices.groups.start[0]===0&&m.indices.groups.end[0]===3&&m.indices.groups.part===m.indices[3]&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<start>\b)(?<whole>(?<\u{1D49C}>[\q{s|sa|k|ka}])|(?<empty>))(?<end>\B)/dvi,m=r.exec('ſa!');m[0]==='ſ'&&m.groups.start===''&&m.groups.whole==='ſ'&&m.groups.𝒜==='ſ'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.𝒜===m.indices[3]&&m.indices.groups.end[0]===1&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var r=/(?<start>\B)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<end>\B)/dvg,a=Array.from('!!'.matchAll(r));a.length===3&&a.map(m=>m.index).join(',')==='0,1,2'&&a.every(m=>m.groups.start===''&&m.groups.whole===''&&m.groups.part===''&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start[0]===m.index&&m.indices.groups.end[0]===m.index)&&r.lastIndex===0",
    );
    check(
        r"var calls=0,out='ab!'.replace(/(?<start>\b)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\B)/vg,function(all,b,outer,p,e,a,offset,input,groups){calls++;return arguments.length===9&&b===''&&outer===all&&p==='a'&&e===undefined&&a===''&&offset===0&&input==='ab!'&&groups.start===b&&groups.end===a&&groups.whole===all?'#':'bad'});out==='#b!'&&calls===1",
    );
    check(
        r"var r=/(?<start>\b)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\b)/vg;r.lastIndex=9;var a='ab!'.split(r);'!ab!'.search(r)===1&&r.lastIndex===9&&a.length===7&&a[0]===''&&a[1]===''&&a[2]==='ab'&&a[3]==='ab'&&a[4]===undefined&&a[5]===''&&a[6]==='!'",
    );
    check(
        r"var r=/(?<start>\B)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<end>\B)/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"var m=/(?<start>)(?<whole>(?:(?<part>^[\q{a|ab}]$)|(?<empty>\b)))(?<end>)/dv.exec('ab');m[0]==='ab'&&m.groups.start===''&&m.groups.whole==='ab'&&m.groups.part==='ab'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start[0]===0&&m.indices.groups.end[0]===2",
    );
    check(
        r"var marker=0,m;try{m=/(?<start>\b\B)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))/dv.exec('ab!')}finally{marker=3}marker===3&&m===null",
    );
    check(
        r"var r=/(?<start>\B)(?<whole>(?<part>[\q{\u{10428}|\u{10428}\n}])|(?<empty>))(?<end>^)/dvgim,m=r.exec('𐐀\n');m[0]==='𐐀\n'&&m.groups.start===''&&m.groups.whole===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.end[0]===3&&r.lastIndex===3",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<start>\b)(?<whole>(?<part>[\q{a|ab}])|(?<empty>))(?<end>\b)/dvg,clone=new RegExp(r),m=r.exec('ab!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"var n=clone.exec('z');m.groups.start===''&&m.groups.whole==='ab'&&m.groups.part==='ab'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.end[0]===2&&n.groups.start===''&&n.groups.whole===''&&n.groups.part===undefined&&n.groups.empty===''&&n.groups.end===''&&n.indices.groups.start===n.indices[1]&&n.indices.groups.end===n.indices[5]&&clone.source===r.source&&r.lastIndex===2"),Ok(Value::Boolean(true)));
    check(
        r"var r=/(?<start>\B)(?<whole>(?<part>[\q{|a|ab}])|(?<empty>))(?<end>\B)/dvg;r.lastIndex=2;var m=r.exec('😀a😀!');m.index===5&&m.groups.start===''&&m.groups.whole===''&&m.groups.part===''&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start[0]===5&&m.indices.groups.end[0]===5&&r.lastIndex===5",
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(?:([\\q{a|ab}])|()){1}(\\b)",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[0,2],[0,2],\"<undefined>\",[2,2]],\"index\":0},2],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",{\"values\":[\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[0,2],[0,2],\"<undefined>\",[2,2]],\"index\":0},2],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",{\"values\":[\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="(?:([\\q{a|ab}])|())+(\\b)",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[false,\"ababa\",{\"values\":[\"ababa\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[false,\"\\nab\\n\",{\"values\":[\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[1,3],[1,3],\"<undefined>\",[3,3]],\"index\":1},3],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[true,\"ababa\",{\"values\":[\"ababa\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"ab\",\"ab\",\"<undefined>\",\"\"],\"indices\":[[1,3],[1,3],\"<undefined>\",[3,3]],\"index\":1},3],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    assert_eq!(Realm::default().eval(r####"var pattern="(\\b)?(([\\q{a|ab}])|())",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&m!==null&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"<undefined>\",\"ab\",\"ab\",\"<undefined>\"],[[0,2],\"<undefined>\",[0,2],[0,2],\"<undefined>\"],0,0]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="(([\\q{a|ab}])|())(\\b)?",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&m!==null&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"ab\",\"ab\",\"<undefined>\",\"<undefined>\"],[[0,2],[0,2],[0,2],\"<undefined>\",\"<undefined>\"],0,0]""####),Ok(Value::Boolean(true)));
    for pattern in [
        r"((\b)|())(([\q{a|ab}])|())",
        r"(?:([[\q{a|ab}]&&\p{RGI_Emoji}])|())+(\b)",
        r"(?:([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()){1}(\b)",
        r"(\b)(([\q{a|ab}])|()|())",
        r"(\b)(([\q{a|ab}])|([\q{}]))",
        r"(\b)(([\q{a|ab}])|\1)",
        r"(\b\B)(([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|())",
        r"(\b)(([\q{a|ab}])|())a",
        r"a(\b)(([\q{a|ab}])|())",
        r"(\b)(?:([\q{a|ab}])|())(?=a)",
        r"(([\q{a|ab}])|())|(\b)",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
#[test]
fn surrounding_asserted_string_class_deep_roles_long_members_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"{}(?<start>\b)(?<whole>(?<part>[\q{{a|ab}}])|(?<empty>))(?<end>\B){}",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('ab!');m.length===1006&&m[0]==='a'&&m[1]==='a'&&m[1000]==='a'&&m.groups.start===''&&m.groups.whole==='a'&&m.groups.part==='a'&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start===m.indices[1001]&&m.indices.groups.end===m.indices[1005]&&m.indices.groups.end[0]===1"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(
        format!(r"(?<start>\b)(?<whole>(?<part>[\q{{{word}}}])|(?<empty>))(?<end>\b)").as_str(),
    );
    let input = JsString::from(word.as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===20000&&m.groups.start===''&&m.groups.whole===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.groups.end===''&&m.indices.groups.start[0]===0&&m.indices.groups.end[0]===20000&&m.indices.groups.part===m.indices[3]"
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
    realm.eval(r"var r=/(^)([\q{}]|)/vg;r.lastIndex=1").unwrap();
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
    realm.eval(r"var r=/(^)(|[\q{}])/dvgy,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.index===0&&m[0]===''&&m.length===3&&m[1]===''&&m[2]===''&&m.indices[1][0]===0&&m.indices[2][1]===0&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}
