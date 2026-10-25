//! Complete or zero captures in one nullable UnicodeSets group (22.2.2.3).
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
fn single_nullable_complete_layouts_keep_zero_unnamed_and_named_capture_shapes() {
    let huge = 2u128;
    for class in [
        r"[\q{a|b}]",
        r"[\q{a|b|ab}&&\p{Any}]",
        r"[\q{\(|\)|a}]",
        r"[\q{a|\(|\)|ab}&&\p{Any}]",
    ] {
        for empty_first in [false, true] {
            for layout in 0..6 {
                let character = if layout == 5 {
                    format!("(?:){class}(?:)")
                } else {
                    class.to_owned()
                };
                let empty = match layout {
                    0 => String::new(),
                    1 => "(?:)".to_owned(),
                    2 => "(?:)(?:)".to_owned(),
                    3 => format!("(?:){{0,{huge}}}?"),
                    4 => format!("(?:(?:){{{huge}}}|(?:))"),
                    _ => format!("(?:)?(?:){{{huge}}}"),
                };
                let branch = if empty_first {
                    format!("{empty}|{character}")
                } else {
                    format!("{character}|{empty}")
                };
                for (open, close, count, named) in [
                    ("", "", 0, false),
                    ("(", ")", 1, false),
                    ("(?<whole>", ")", 1, true),
                    ("(?<outer>(?<inner>", "))", 2, true),
                ] {
                    let pattern = JsString::from(format!("{open}(?:{branch}){close}").as_str());
                    let whole = if empty_first { "" } else { "a" };
                    let end = if empty_first { 1 } else { 2 };
                    check(&format!(
                        r#"
                    var r=new RegExp({pattern:?},'dvgy');r.lastIndex=1;var m=r.exec('aaa'),ok=true;
                    for(var slot=0;slot<m.length;slot++)ok=ok&&m[slot]==={whole:?}&&m.indices[slot][0]===1&&m.indices[slot][1]==={end};
                    if({named}){{var keys=Object.keys(m.groups);ok=ok&&keys.length==={count};for(var name of keys){{var slot=keys.indexOf(name)+1;ok=ok&&m.groups[name]==={whole:?}&&m.indices.groups[name]===m.indices[slot]}}}}
                    else ok=ok&&m.groups===undefined&&m.indices.groups===undefined;
                    ok&&m.length==={count}+1&&m.index===1&&r.lastIndex==={end}&&Object.hasOwn(m,'groups')&&Object.hasOwn(m.indices,'groups')&&r.source==={pattern:?}
                "#
                    ));
                }
            }
        }
    }
}

#[test]
fn single_nullable_complete_ranges_keep_utf16_folding_gates_and_first_success() {
    check(
        r"var r=/(?<outer>(?<inner>(?:[\q{😀}]|)))/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]==='😀'&&m.index===0&&m.groups.outer==='😀'&&m.groups.inner==='😀'&&m.indices.groups.outer[0]===0&&m.indices.groups.inner[1]===2&&r.lastIndex===2",
    );
    check(
        r"var r=/(?<whole>(?:|[\q{😀}]))/dvgy;r.lastIndex=1;var m=r.exec('😀😀!');m[0]===''&&m.index===0&&m.groups.whole===''&&m.indices.groups.whole[0]===0&&m.indices.groups.whole[1]===0&&r.lastIndex===0",
    );
    check(
        r"var r=/(?<\u{1D49C}>(?:[\q{s|k}]|))/dvgi;var m=r.exec('ſK'),n=r.exec('ſK');m.groups.𝒜==='ſ'&&n.groups.𝒜==='K'&&n.indices.groups.𝒜[0]===1&&r.source.indexOf('1D49C')!==-1",
    );
    check(
        r"var m=/(?:[^\q{ab}&&a]|)/dv.exec('\ud800');m.length===1&&m[0]==='\ud800'&&m.indices[0][1]===1&&m.groups===undefined&&m.indices.groups===undefined",
    );
    for branch in [r"[\q{a}]|", r"|[\q{a}]"] {
        check(&format!(
            r"var m=/^\b(?<whole>(?:{branch}))\b$/dvm.exec('!\r\na\u2028!');m[0]==='a'&&m.index===3&&m.groups.whole==='a'&&m.indices.groups.whole[0]===3&&m.indices.groups.whole[1]===4"
        ));
    }
    check(
        r"var m=/\b(?:[\q{a}]|)\b/dv.exec(' aa');m.length===1&&m[0]===''&&m.index===1&&m.indices[0][0]===1&&m.indices[0][1]===1",
    );
    check(
        r"var r=/(?:|[\q{a}])/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
}

#[test]
fn single_nullable_complete_consumers_keep_zero_capture_callbacks_clones_and_collection() {
    check(
        r"var r=/(?:[\q{a|b}]|)/dvg;var all=Array.from('ab!'.matchAll(r));all.length===4&&all.map(m=>m[0]).join('|')==='a|b||'&&all.every(m=>m.length===1&&m.groups===undefined&&m.indices.groups===undefined)&&all.map(m=>m.index).join(',')==='0,1,2,3'&&r.lastIndex===0",
    );
    check(
        r"var all=Array.from('😀a'.matchAll(/(?<whole>(?:|[\q{😀}]))/dvg));all.length===3&&all.map(m=>m.index).join(',')==='0,2,3'&&all.every(m=>m[0]===''&&m.groups.whole==='')",
    );
    check(
        r"var calls=0;var out='aa!'.replace(/(?:[\q{a}]|)/vg,function(whole,offset,input){calls++;return arguments.length===3&&input==='aa!'?'#':'bad'});out==='###!#'&&calls===4",
    );
    check(
        r"var calls=0;var out='aa!'.replace(/(?<whole>(?:[\q{a}]|))/vg,function(match,whole,offset,input,groups){calls++;return arguments.length===5&&match===whole&&groups.whole===whole&&input==='aa!'?'#':'bad'});out==='###!#'&&calls===4",
    );
    check(
        r"var r=/(?:[\q{a}]|)/vg;r.lastIndex=9;var parts='ab'.split(r);'baa'.search(r)===0&&r.lastIndex===9&&parts.length===2&&parts[0]===''&&parts[1]==='b'&&'aa!'.match(r).join('|')==='a|a||'&&'aa!'.replace(/(?<whole>(?:[\q{a}]|))/vg,'<$<whole>>')==='<a><a><>!<>'",
    );
    let mut realm = Realm::default();
    realm
        .eval(r"var r=/(?<whole>(?:[\q{a}]|))/dvg;var clone=new RegExp(r);var m=r.exec('aaa')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.whole==='a'&&m.indices.groups.whole[1]===1&&clone.exec('aa').groups.whole==='a'&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn single_nullable_complete_proofs_finish_finalizers_and_keep_root_scope_and_unknown_terms_closed()
{
    check(
        r"var caught=false;try{new RegExp('(?:[\\q{a}]|)\\1','v')}catch(e){caught=e instanceof SyntaxError}caught",
    );
    check(
        r"var marker=0,m;try{m=/(?:[\q{a}]|)/dv.exec('a')}finally{marker=3}marker===3&&m.length===1&&m[0]==='a'&&m.indices[0][1]===1&&m.groups===undefined",
    );
    check(
        r"var marker=0,m;try{m=/((?:[\q{a}]|))/dv.exec('!')}finally{marker=3}marker===3&&m[0]===''&&m[1]===''&&m.indices[1][0]===0",
    );
    check(
        r"var marker=0,m;try{m=/(?:[\q{ab}&&\q{cd}]|)/dv.exec('ab')}finally{marker=3}marker===3&&m[0]===''&&m.length===1&&m.groups===undefined&&m.indices.groups===undefined&&m.indices[0][0]===0&&m.indices[0][1]===0",
    );
    for pattern in [
        r"^[\q{a}]|",
        r"[\q{a}]|$",
        r"(?:[\q{a}]|)x",
        r"((?:[\q{a}]|))\1",
        r"(?:[\q{a}]|\b)",
        r"(?:[\q{a}][\q{b}]|)",
        r"(?:[\q{ab}&&\q{cd}&&\p{RGI_Emoji}]|)",
        r"(?:[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]|)",
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
        "var yes=0;for(var p of ['(?<x>(?:[a]|))(?<x>)','(?:[\\q{ab}&&a--a]|)']){try{new RegExp(p,'v')}catch(e){if(e instanceof SyntaxError)yes++}}yes===2",
    );
}
