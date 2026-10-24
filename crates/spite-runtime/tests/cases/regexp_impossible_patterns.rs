//! Negative empty assertions prove complete root sequences cannot match.

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
fn impossible_sequences_keep_original_source_flags_names_and_last_index_rules() {
    check(
        r#"
        var patterns=[
            '(?!(?<x>[\\p{RGI_Emoji}]{0}))',
            '[\\p{RGI_Emoji}]+(?<!(?<x>a{0}))(?<tail>b)',
            '(?<prefix>a|ab)*(?!(?=(?<ahead>\\uD800{0}))(?<=(?<behind>\\uDC00{0})))(?<suffix>[\\p{RGI_Emoji}]+)',
            '(?<before>\\p{RGI_Emoji})+(?!(?<zero>(?<erased>\\k<later>){0}))(?<later>c)'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;var m=r.exec(input);
            ok=ok&&m===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start)&&r.test(input)===false;
        }ok
    "#,
    );
}

#[test]
fn impossible_string_consumers_copies_and_collection_skip_replacement_callbacks() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/[\p{RGI_Emoji}]+(?!(?<x>a{0}))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1&&''.split(r)[0]==='';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_impossible_prefixes_and_long_assertion_spellings_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'a|b'+')'.repeat(depth)+'(?!(?<zero>[\\p{RGI_Emoji}]{0}))',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}a|b{}(?!(?<zero>[\p{{RGI_Emoji}}]{{0}}))",
            "(".repeat(depth),
            ")".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
    let zeros = "0".repeat(20_000);
    let pattern =
        JsString::from(format!(r"[\p{{RGI_Emoji}}]+(?!(?<zero>\u{{{zeros}D800}}{{0}}))").as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.exec('😀')===null&&r.source==={pattern:?}"
    ));
}

#[test]
fn impossible_proofs_preserve_early_errors_coercions_and_opt_in_aborts() {
    check(
        r#"
        var invalid=['[\\p{Not_A_Property}]+(?!)','(?<x>a)(?<x>b)(?!)','(?!)\\k<missing>','(?!)\\u{110000}'],ok=true;
        for(var source of invalid){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        var events=[],r=/[\p{RGI_Emoji}]+(?!(a{0}))/gv;r.lastIndex={valueOf(){events.push('index');return 1}};
        var result=r.exec({toString(){events.push('input');return '😀'}});ok&&result===null&&r.lastIndex===0&&events.join(',')==='input,index'
    "#,
    );
    let zeros = "0".repeat(20_000);
    let pattern =
        JsString::from(format!(r"[\p{{RGI_Emoji}}]+(?!(\u{{{zeros}D800}}{{0}}))").as_str());
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
}

#[test]
fn enclosing_impossible_sequences_keep_original_outside_and_inside_bindings() {
    check(
        r#"
        var patterns=[
            '(?<outside>[\\p{RGI_Emoji}]+(?!(?<x>a{0})))',
            '(?:(?<outside>(?<prefix>a|ab)*(?<!(?<x>\\uD800{0}))(?<suffix>[\\p{RGI_Emoji}]+)))',
            '(?<outside>(?!(?<zero>(?<erased>\\k<outside>|\\k<later>){0}))(?<later>[\\p{RGI_Emoji}]+))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }ok
    "#,
    );
}

#[test]
fn enclosing_impossible_consumers_clones_and_collection_keep_no_match_behavior() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>[\p{RGI_Emoji}]+(?!(?<x>a{0})))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_enclosing_impossible_groups_keep_full_inventories_without_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'[\\p{RGI_Emoji}]+(?!(?<zero>\\uD800{0}))'+')'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}[\p{{RGI_Emoji}}]+(?!(?<zero>\uD800{{0}})){}",
            "(".repeat(depth),
            ")".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}
