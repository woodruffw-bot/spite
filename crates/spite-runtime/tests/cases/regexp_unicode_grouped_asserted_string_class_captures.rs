//! Complete wrapper captures around asserted string-class/empty choices.
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
fn grouped_asserted_string_class_layouts_preserve_selected_branch_and_lexical_ranges() {
    let huge = 10u128.pow(30);
    for (class, input, words, empty_matches) in [
        (
            r"[\q{a|ab|abc}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("abc")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{a|ab|abc}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("abc")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{ab}\q{a|abc}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[x\q{a|ab}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[x\q{a|ab}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("abc")],
            [true, false, true, true, true],
        ),
        (
            r"[a-z\q{ab|abc}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("abc")],
            [true, false, true, true, true],
        ),
        (
            r"[[\q{a|ab}][x\q{abc}]]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "abc!",
            [None, Some("a"), Some("a"), None, Some("abc")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{a|ab|abc}--\q{ab}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[[\q{a|ab}x]&&[\q{ab|x}]]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{a|b|ab}--[b-z]]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "😀a!",
            [None, None, None, None, Some("😀a")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{😀|😀a|a}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "𐐨a!",
            [None, None, None, None, Some("𐐨a")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{\u{10428}|\u{10428}a}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "sa!",
            [None, Some("s"), Some("s"), None, Some("sa")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{s|sa|k|ka}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{ab}--\q{ab}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|a|ab}--\q{}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{a|ab}&&\q{ab}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|ab}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|a|ab}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{ab}\q{}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{ab}\q{}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[x\q{|ab}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[x\q{|ab}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[a-z\q{|ab}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[a-z\q{|ab}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "abc!",
            [None, Some("ab"), Some("ab"), None, Some("ab")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|😀|😀a}]",
            "😀a!",
            [None, None, Some(""), None, Some("😀a")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|😀|😀a}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "𐐨a!",
            [None, None, Some(""), None, Some("𐐨a")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "sa!",
            [None, Some("s"), Some("s"), None, Some("sa")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|s|sa|k|ka}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{}]",
            "abc!",
            [None, None, None, None, Some("")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "abc!",
            [None, None, None, None, Some("")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|ab}--\q{ab}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{a|\n|\na}]",
            "\na!",
            [None, None, None, Some("\n"), Some("\na")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{a|\n|\na}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "\na!",
            [Some(""), None, Some(""), Some("\n"), Some("\na")],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[\n\q{a|\na}]",
            "\na!",
            [None, None, None, Some("\n"), Some("\na")],
            [true, true, true, false, false],
        ),
        (
            r"[\n\q{a|\na}]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "a\n!",
            [None, Some("a\n"), Some("a\n"), None, Some("a\n")],
            [true, false, true, true, true],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "!",
            [None, None, Some(""), None, Some("")],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "\na!",
            [None, None, None, Some("\n"), Some("\na")],
            [true, true, true, false, false],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "!",
            [None, None, None, None, None],
            [true, true, true, false, false],
        ),
    ] {
        for (mode, (cp, cs, ep, es)) in [
            ("^", "$", "", ""),
            (r"\b", r"\B", r"\B", ""),
            ("", r"\B", "^", ""),
            ("$", "^", r"\b", ""),
            ("", "", r"\b", ""),
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
                        let pattern = JsString::from(
                            match frame {
                                0 => format!("(?<enclosing>{source})"),
                                1 => format!("(?:(?<enclosing>{source}))"),
                                _ => format!("(?<enclosing>(?:(?<inner>{source})))"),
                            }
                            .as_str(),
                        );
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
fn grouped_asserted_string_class_consumers_preserve_defined_prefixes_and_conditional_branches() {
    check(
        r"var m=/(?<enclosing>^(?<part>[\q{a|ab}])$|(?<empty>\b))/dv.exec('ab!');m.index===0&&m[0]===''&&m.groups.part===undefined&&m.groups.empty===''&&m.indices[2]===undefined&&m.indices.groups.empty===m.indices[3]&&m.indices[3][0]===0",
    );
    check(
        r"var m=/(?<enclosing>^(?<part>[\q{a|ab}])$|(?<empty>\b))/dv.exec('ab');m[0]==='ab'&&m.groups.part==='ab'&&m.groups.empty===undefined&&m.indices.groups.part===m.indices[2]&&m.indices[2][1]===2",
    );
    check(
        r"var m=/(?<enclosing>(?<empty>\b)|(?<part>[\q{a|ab}])\B)/dv.exec('ab!');m[0]===''&&m.groups.empty===''&&m.groups.part===undefined&&Object.keys(m.groups).join(',')==='enclosing,empty,part'&&m.indices.groups.empty===m.indices[2]",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{|a|ab}])\B|(?<empty>^))/dv,a=r.exec('!'),b=r.exec('a');a[0]===''&&a.groups.part===''&&a.groups.empty===undefined&&a.indices.groups.part[0]===0&&b[0]===''&&b.groups.part===undefined&&b.groups.empty===''&&b.indices.groups.empty[0]===0",
    );
    check(
        r"var r=/(?<enclosing>(?<empty>^)|(?<before>)(?<part>[\q{|a|ab}])(?<after>))/dvg,a=r.exec('ab!');r.lastIndex=1;var b=r.exec('ab!');a[0]===''&&a.groups.empty===''&&a.groups.part===undefined&&b.index===1&&b.groups.empty===undefined&&b.groups.before===''&&b.groups.part===''&&b.groups.after===''&&b.indices.groups.part===b.indices[4]&&b.indices[4][0]===1&&r.lastIndex===1",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{a|ab}])\B|(?<empty>^))/dvg;r.lastIndex=1;var m=r.exec('!ab');m.index===1&&m[0]==='a'&&m.groups.part==='a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===2&&r.lastIndex===2",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{😀|😀a}])\b|(?<empty>^))/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m.index===0&&m[0]==='😀a'&&m.groups.part==='😀a'&&m.groups.empty===undefined&&m.indices.groups.part[1]===3&&r.lastIndex===3",
    );
    check(
        r"var r=/(?<enclosing>(?<\u{1D49C}>[\q{s|sa|k|ka}])\B|(?<empty>^))/dvi,m=r.exec('ſa!');m[0]==='ſ'&&m.groups.𝒜==='ſ'&&m.groups.empty===undefined&&m.indices.groups.𝒜===m.indices[2]&&m.indices[2][1]===1&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{|a|ab}])\B|(?<empty>\b))/dvg,a=Array.from('ab!'.matchAll(r));a.length===4&&a.map(m=>m.index+':'+(m.groups.part===undefined?'E':'C')+':'+m[0]).join('|')==='0:C:a|1:C:|2:E:|3:C:'&&a[2].indices.groups.empty===a[2].indices[3]&&r.lastIndex===0",
    );
    check(
        r"var calls=0,out='ab!'.replace(/(?<enclosing>(?<part>[\q{a|ab}])\B|(?<empty>\b))/vg,function(all,outer,p,e,offset,input,groups){calls++;return arguments.length===7&&outer===all&&groups.part===p&&groups.empty===e&&input==='ab!'?(p===undefined?'<E>':'<C:'+p+'>'):'bad'});out==='<C:a>b<E>!'&&calls===2",
    );
    check(
        r"var r=/(?<enclosing>(?<part>[\q{a|ab}])\B|(?<empty>\b))/vg;r.lastIndex=9;var a='ab!'.split(r);'!ab'.search(r)===1&&r.lastIndex===9&&a.length===9&&a[0]===''&&a[1]==='a'&&a[2]==='a'&&a[3]===undefined&&a[4]==='b'&&a[5]===''&&a[6]===undefined&&a[7]===''&&a[8]==='!'",
    );
    check(
        r"var r=/(?<enclosing>([\q{|a|ab}])\B|(^))/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"var m=/(?<enclosing>(?:(?<inner>^(?<part>[\q{a|ab}])$|(?<empty>\b))))/dv.exec('ab!');m.groups.enclosing===''&&m.groups.inner===''&&m.groups.part===undefined&&m.groups.empty===''&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.inner===m.indices[2]&&m.indices.groups.empty===m.indices[4]&&m.indices[1]!==m.indices[0]&&m.indices[1]!==m.indices[2]",
    );
    check(
        r"var m=/(?:(?:^(?<part>[\q{a|ab}])$|(?<empty>\b)))/dv.exec('ab!');m.length===3&&m.groups.part===undefined&&m.groups.empty===''&&m.indices.groups.empty===m.indices[2]&&m.indices[1]===undefined&&m.indices[2][0]===0",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<enclosing>(?<part>[\q{|a|ab}])\B|(?<empty>^))/dvg,clone=new RegExp(r),m=r.exec('ab!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('a');m.groups.enclosing==='a'&&m.groups.part==='a'&&m.groups.empty===undefined&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.part===m.indices[2]&&n.groups.enclosing===''&&n.groups.part===undefined&&n.groups.empty===''&&n.indices.groups.empty===n.indices[3]&&clone.source===r.source&&r.lastIndex===1"),Ok(Value::Boolean(true)));
    let pattern = JsString::from(r"(?<enclosing>(?<part>[\q{|a|ab}])\B|(?<empty>$))");
    check(&format!(
        "var r=new RegExp({pattern:?},'dvg');r.lastIndex=2;var m=r.exec('😀a😀!');m.index===5&&m.groups.enclosing===''&&m.groups.part===''&&m.groups.empty===undefined&&m.indices.groups.enclosing[0]===5&&m.indices.groups.part===m.indices[2]&&r.lastIndex===5"
    ));
    for pattern in [
        r"^((?:([\q{a|ab}])|()))",
        r"((^([\q{a|ab}])|()))$",
        r"(^([\q{a|ab}])|())()",
        r"()(^([\q{a|ab}])|())",
        r"(^([\q{a|ab}])|())?",
        r"(^([\q{a|ab}])|()|())",
        r"(^([\q{a|ab}])|([\q{}]))",
        r"(^([\q{a|ab}])|(\b)?)",
        r"(^([\q{a|ab}])|\1)",
        r"(\b\B([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|(^))",
        r"(^([\q{a|ab}])+|(^))",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
#[test]
fn grouped_asserted_string_class_deep_prefixes_members_first_branch_work_and_host_aborts_stay_flat()
{
    let pattern = JsString::from(
        format!(
            "{}([\\q{{|a|ab}}])\\B|(^){}",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('ab!'),e=r.exec('a');m.length===1003&&m[0]==='a'&&m[1]==='a'&&m[1000]==='a'&&m[1001]==='a'&&m[1002]===undefined&&e[0]===''&&e[1]===''&&e[1000]===''&&e[1001]===undefined&&e[1002]===''&&e.indices[1000][0]===0&&e.indices[1002][0]===0"
    ));
    let word = "x".repeat(20_000);
    let pattern =
        JsString::from(format!(r"(?<enclosing>(?<part>[\q{{{word}}}])\B|(?<empty>^))").as_str());
    let input = JsString::from(format!("{word}x").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===20000&&m.groups.enclosing===m[0]&&m.groups.part===m[0]&&m.groups.empty===undefined&&m.indices.groups.enclosing===m.indices[1]&&m.indices.groups.part[1]===20000"
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
        .eval(r"var r=/(\b\B[\q{}]|\b\B)/vg;r.lastIndex=1")
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
    for (pattern, class_selected) in [
        (r"(|(\b\B[\q{ab}]))", false),
        (r"(([\q{|a|ab}])|\b\B)", true),
    ] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(100_000),
            ..Limits::default()
        });
        realm
            .eval(&format!(
                "var r=new RegExp({:?},'dvg'),marker=0;r.lastIndex=1",
                JsString::from(pattern)
            ))
            .unwrap();
        realm
            .eval(&format!("var input='{}'", "x".repeat(40_000)))
            .unwrap();
        assert_eq!(realm.eval(&format!("var m;try{{m=r.exec(input)}}finally{{marker=3}}marker===3&&m.index===1&&m[0]===''&&m[1]===''&&({class_selected}?m[2]==='':m[2]===undefined)&&r.lastIndex===1")),Ok(Value::Boolean(true)));
    }
}
