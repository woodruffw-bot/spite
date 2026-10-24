//! Mandatory line predicates at either endpoint of a single string class.
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
fn exact_single_string_class_layouts_keep_actual_endpoint_and_capture_ranges() {
    let huge = 10u128.pow(30);
    for (class, input, words) in [
        (r"[\q{a|ab|abc}]", "\na!", [None, None, None]),
        (r"[\q{ab}\q{a|abc}]", "\na!", [None, None, None]),
        (r"[x\q{a|ab}]", "\na!", [None, None, None]),
        (r"[a-z\q{ab|abc}]", "\na!", [None, None, None]),
        (r"[[\q{a|ab}][x\q{abc}]]", "\na!", [None, None, None]),
        (r"[\q{a|ab|abc}--\q{ab}]", "\na!", [None, None, None]),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", "\na!", [None, None, None]),
        (r"[\q{a|b|ab}--[b-z]]", "\na!", [None, None, None]),
        (r"[\q{😀|😀a|a}]", "\na!", [None, None, None]),
        (r"[\q{\u{10428}|\u{10428}a}]", "\na!", [None, None, None]),
        (r"[\q{s|sa|k|ka}]", "\na!", [None, None, None]),
        (r"[\q{ab}--\q{ab}]", "\na!", [None, None, None]),
        (r"[\q{|a|ab}--\q{}]", "\na!", [None, None, None]),
        (r"[\q{a|ab}&&\q{ab}]", "\na!", [None, None, None]),
        (r"[\q{|a|ab}]", "\na!", [Some(""), Some(""), Some("")]),
        (r"[\q{ab}\q{}]", "\na!", [Some(""), Some(""), Some("")]),
        (r"[x\q{|ab}]", "\na!", [Some(""), Some(""), Some("")]),
        (r"[a-z\q{|ab}]", "\na!", [Some(""), Some(""), Some("")]),
        (
            r"[[\q{|ab}][x\q{a}]]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (
            r"[\q{|a|ab}--\q{a}]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (
            r"[[\q{|a|ab}x]&&[\q{|ab|x}]]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (
            r"[\q{|a|ab}--[a-z]]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (r"[\q{|😀|😀a}]", "\na!", [Some(""), Some(""), Some("")]),
        (
            r"[\q{|\u{10428}|\u{10428}a}]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (r"[\q{|s|sa|k|ka}]", "\na!", [Some(""), Some(""), Some("")]),
        (r"[\q{}]", "\na!", [Some(""), Some(""), Some("")]),
        (r"[\q{|ab}--\q{ab}]", "\na!", [Some(""), Some(""), Some("")]),
        (
            r"[\q{a|\n|\na}]",
            "\na!",
            [Some("\n"), Some("\n"), Some("\na")],
        ),
        (
            r"[\q{|a|\n|\na}]",
            "\na!",
            [Some("\n"), Some("\n"), Some("\na")],
        ),
        (
            r"[\n\q{a|\na}]",
            "\na!",
            [Some("\n"), Some("\n"), Some("\na")],
        ),
        (
            r"[\q{|a|a\n}--\q{a}]",
            "\na!",
            [Some(""), Some(""), Some("")],
        ),
        (
            r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]",
            "\na!",
            [Some("\n"), Some("\n"), Some("\na")],
        ),
        (r"[\q{a|a\n}]", "a\n!", [None, Some("a\n"), None]),
    ] {
        for (mode, (head, tail)) in [("$", "^"), ("", "^"), ("$", "")].into_iter().enumerate() {
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

                    let pattern = pattern.replacen(class, &format!("{class}{{0001,1}}?"), 1);
                    let body = format!("(?<head>{head}){pattern}(?<tail>{tail})");
                    let pattern = match frame {
                        0 => body,
                        1 => format!("(?<enclosing>{body})"),
                        _ => format!("(?<enclosing>(?:(?<inner>{body})))"),
                    };
                    let pattern = format!("(?:{pattern}){{1}}");
                    let pattern = JsString::from(pattern.as_str());
                    let text = JsString::from(input);
                    let word = words[mode];
                    let matched = word.is_some();
                    let whole = JsString::from(word.unwrap_or(""));
                    let end = whole.len();
                    check(&format!(
                        r#"
var r=new RegExp({pattern:?},'dvgym'),m=r.exec({text:?}),ok=true;
if({matched}){{
 var keys=Object.keys(m.groups),characters='{characters}'.split(','),before='{before}'.split(','),after='{after}'.split(','),never='{never}'.split(',');
 for(var name of keys){{var slot=keys.indexOf(name)+1,expected=undefined,from=-1,to=-1;
  if(characters.indexOf(name)!==-1||['enclosing','inner'].indexOf(name)!==-1){{expected={whole:?};from=0;to={end}}}
  if(before.indexOf(name)!==-1||name==='head'){{expected='';from=0;to=0}}
  if(after.indexOf(name)!==-1||name==='tail'){{expected='';from={end};to={end}}}
  if(never.indexOf(name)!==-1)expected=undefined;
  ok=ok&&m[slot]===expected&&m.groups[name]===expected&&m.indices.groups[name]===m.indices[slot]&&Object.hasOwn(m.indices.groups,name);
  if(expected===undefined)ok=ok&&m.indices[slot]===undefined;else ok=ok&&m.indices[slot][0]===from&&m.indices[slot][1]===to;
 }}
 ok=ok&&m.length===keys.length+1&&m[0]==={whole:?}&&m.index===0&&m.indices[0][1]==={end}&&r.lastIndex==={end};
}}else ok=m===null&&r.lastIndex===0;
ok&&r.source==={pattern:?}
"#
                    ));
                }
            }
        }
    }
}
#[test]
fn exact_single_string_class_consumers_ranges_and_scope_keep_original_slots() {
    check(
        r"var m=/($)([\q{\n|\na}]{1})(^)/dvm.exec('\na!');m.length===4&&m[0]==='\n'&&m[1]===''&&m[2]==='\n'&&m[3]===''&&m.indices[2][1]===1&&m.indices[3][0]===1",
    );
    check(
        r"var m=/($){2}(\B){999999999999999999999999999999}([\q{\n|\na}]{1})(^){1,3}/dvm.exec('\na!');m.length===5&&m[0]==='\n'&&m[1]===''&&m[2]===''&&m[3]==='\n'&&m[4]===''&&m.indices[4][0]===1",
    );
    check(
        r"var m=/($$)([\q{\n|\na}]{1})(^^)/dvm.exec('\na!');m[0]==='\n'&&m[1]===''&&m[2]==='\n'&&m[3]===''&&m.indices[3][0]===1",
    );
    check(
        r"var m=/(^$)([\q{|\n}]{1})(^$)/dvm.exec('\n\n');m[0]==='\n'&&m[1]===''&&m[2]==='\n'&&m[3]===''&&m.indices[3][0]===1",
    );
    check(
        r"var r=/([\q{|a|ab}]{1})(^)/dvg,m=r.exec('ab'),a=Array.from('ab'.matchAll(r));m[0]===''&&m[1]===''&&m[2]===''&&m.index===0&&r.lastIndex===0&&a.length===1&&a[0].index===0",
    );
    check(
        r"var r=/($)([\q{|a|ab}]{1})/dv;r.lastIndex=999;var m=r.exec('ab');m.index===2&&m[0]===''&&m[1]===''&&m[2]===''&&m.indices[2][0]===2&&r.lastIndex===999",
    );
    check(
        r"var marker=0,m;try{m=/($\b\B)([\q{|a|ab}]{1})(^)/dvm.exec('\n')}finally{marker=3}marker===3&&m===null",
    );
    check(
        r"var r=/([\q{|😀|😀a}]{1})(^)/dvgy;r.lastIndex=1;var m=r.exec('😀a!');m.index===0&&m[0]===''&&m[1]===''&&m[2]===''&&m.indices[1][0]===0&&r.lastIndex===0",
    );
    check(
        r"var m=/(?<\u{1D49C}>)(?<part>[\q{s\n|sa\n|k\n|ka\n}]{1})(?<tail>^)/dvim.exec('ſa\n!');m[0]==='ſa\n'&&m.groups.𝒜===''&&m.groups.part===m[0]&&m.groups.tail===''&&Object.keys(m.groups).join(',')==='𝒜,part,tail'&&m.indices.groups.part===m.indices[2]&&m.indices[3][0]===3",
    );
    check(
        r"var r=/(?<head>)(?<part>[\q{a|a\n}]{1})(?<tail>^)/dvgm;var a=Array.from('a\nab\na\n!'.matchAll(r));a.length===2&&a.every(m=>m.groups.head===''&&m.groups.part===m[0]&&m.groups.tail==='')&&a[0][0]==='a\n'&&a[1].index===5&&a[1].indices[2][1]===7&&r.lastIndex===0",
    );
    check(
        r"var calls=0;var out='a\nab\na\n!'.replace(/(?<head>)(?<part>[\q{a|a\n}]{1})(?<tail>^)/vgm,function(all,h,p,t,offset,input,groups){calls++;return arguments.length===7&&all===p&&h===''&&t===''&&groups.part===p&&input==='a\nab\na\n!'?'#':'bad'});out==='#ab\n#!'&&calls===2",
    );
    check(
        r"var r=/(?<head>)(?<part>[\q{a|a\n}]{1})(?<tail>^)/vgm;r.lastIndex=9;var a='a\nb'.split(r);'a\nb'.search(r)===0&&r.lastIndex===9&&a.length===5&&a[0]===''&&a[1]===''&&a[2]==='a\n'&&a[3]===''&&a[4]==='b'",
    );
    check(
        r"var r=/(?<before>)(?<part>[\q{a|ab}]){1}(?<after>)/dv,m=r.exec('ab!');m[0]==='ab'&&m.groups.before===''&&m.groups.part==='ab'&&m.groups.after===''&&m.indices.groups.part===m.indices[2]&&m.indices.groups.before[0]===0&&m.indices.groups.after[0]===2",
    );
    check(
        r"var m=/(?<before>)(?<part>[\q{|a|ab}]){1}(?<after>)/dv.exec('!');m[0]===''&&m.groups.before===''&&m.groups.part===''&&m.groups.after===''&&m.indices.groups.part===m.indices[2]&&m.indices.groups.after[0]===0",
    );
    check(
        r"var m=/(?<before>\b)(?<part>[\q{a|ab}]){0001,1}?(?<after>\B)/dv.exec('ab!');m[0]==='a'&&m.groups.before===''&&m.groups.part==='a'&&m.groups.after===''&&m.indices.groups.part===m.indices[2]&&m.indices.groups.after[0]===1",
    );
    check(
        r"var r=/(?:(?<before>^)(?<part>[\q{|a|ab}])(?<after>$)){0001,01}?/dv,m=r.exec('ab');m[0]==='ab'&&m.groups.before===''&&m.groups.part==='ab'&&m.groups.after===''&&m.indices.groups.part===m.indices[2]&&m.indices.groups.after[0]===2&&r.source.indexOf('0001,01')!==-1",
    );
    check(
        r"var m=/(?<before>\b)(?<\u{1D49C}>[\q{s|sa|k|ka}]){1}(?<after>\B)/dvi.exec('ſa!');m[0]==='ſ'&&m.groups.before===''&&m.groups.𝒜==='ſ'&&m.groups.after===''&&m.indices.groups.𝒜===m.indices[2]&&m.indices.groups.after[0]===1",
    );
    check(
        r"var r=/()([\q{|a|ab}]){1}/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('!')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"var m=/(?<before>\B)(?<part>[\q{\u{10428}|\u{10428}\n}]){1}(?<after>^)/dvim.exec('𐐀\n!');m[0]==='𐐀\n'&&m.groups.before===''&&m.groups.part===m[0]&&m.groups.after===''&&m.indices.groups.part===m.indices[2]&&m.indices.groups.after[0]===3",
    );
    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<head>$)(?<part>[\q{|\n|\na}]{1})(?<tail>^)/dvgm,clone=new RegExp(r),m=r.exec('\na!')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"m.groups.head===''&&m.groups.part==='\n'&&m.groups.tail===''&&m.indices.groups.part===m.indices[2]&&clone.exec('')[2]===''&&clone.source===r.source"),Ok(Value::Boolean(true)));
    let pattern = JsString::from(r"(?<part>[\q{|a|ab}]{1})(^)");
    check(&format!(
        "var r=new RegExp({pattern:?},'dvgm');r.lastIndex=2;var m=r.exec('😀a😀!\\n');m.index===7&&m.groups.part===''&&m.indices.groups.part[0]===7&&r.lastIndex===7"
    ));
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="($)([\\q{a|ab}]|)(^)",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",null,0],[false,\"\",{\"values\":[\"\",\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",null,0],[false,\"x\\nab\",null,0],[true,\"ab!\",null,0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="($\\b\\B)([\\q{a|ab}]{1})+",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",null,0],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",null,0],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"($)?([\q{a|ab}]{1})",
        r"($){0}([\q{a|ab}]{1})",
        r"($|)([\q{a|ab}]{1})",
        r"(|$)([\q{a|ab}]{1})",
        r"([\q{a|ab}]{1})(^)?",
        r"($\b\B)([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]{1})",
        r"($\b\B)([\q{a|ab}]{1})\1",
        r"($\b\B)([[\q{a|ab}]&&\p{RGI_Emoji}]{1})+",
        r"($)([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]{1}|)(^)",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ab')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
#[test]
fn exact_single_string_class_deep_points_long_members_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            "{}($)([\\q{{|\\n|\\na\\n}}])(^){}",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dvm'),m=r.exec('\\na\\n'),e=r.exec('');m.length===1004&&m[1]==='\\na\\n'&&m[1000]===m[0]&&m[1001]===''&&m[1002]===m[0]&&m[1003]===''&&e[1]===''&&e[1000]===''&&e[1001]===''&&e[1002]===''&&e[1003]===''"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(
        format!(r"(?<head>$)(?<part>[\q{{\n|\n{word}\n}}]{{1}})(?<tail>^)").as_str(),
    );
    let input = JsString::from(format!("\n{word}\n!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dvm').exec({input:?});m.index===0&&m[0].length===20002&&m.groups.head===''&&m.groups.part===m[0]&&m.groups.tail===''&&m.indices.groups.part[1]===20002"
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
    realm.eval(r"var r=/^[\q{}]{1}/vg;r.lastIndex=1").unwrap();
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
    realm.eval(r"var r=/^[\q{}]{1}/dvgy,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===1&&m.index===0&&m[0]===''&&m.indices[0][0]===0&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}
