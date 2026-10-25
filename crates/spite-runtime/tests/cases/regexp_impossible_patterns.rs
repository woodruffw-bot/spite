//! Assertions with independently proved outcomes block complete patterns.

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
fn impossible_root_choices_preserve_every_arm_and_validate_original_early_errors() {
    check(
        r#"
        var patterns=[
            '(?!(?<left>[\\p{RGI_Emoji}]{0}))|[\\p{RGI_Emoji}]+(?<!(?<right>a{0}))',
            '((?<outer>[\\p{RGI_Emoji}]+(?!(?<x>a{0}))))|(?<!\\uD800{0})',
            '(?<before>\\p{RGI_Emoji})+(?!(?<zero>(?<erased>\\k<later>){0}))(?<later>c)|(?<before>[\\p{RGI_Emoji}]+)(?<!a{0})'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!)|[\\p{Not_A_Property}]','(?!)|(?<x>a)(?<x>b)','(?!)|\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn impossible_root_choice_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<left>[\p{RGI_Emoji}]{0}))|[\p{RGI_Emoji}]+(?<!(?<right>a{0}))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn many_impossible_alternatives_keep_duplicate_name_inventories_without_default_quotas() {
    check(
        r#"
        var pattern=Array(5).fill('(?!(?<same>[\\p{RGI_Emoji}]{0}))').join('|'),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let pattern = JsString::from(
        std::iter::repeat_n(r"(?!(?<same>[\p{RGI_Emoji}]{0}))", 1000)
            .collect::<Vec<_>>()
            .join("|")
            .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn required_impossible_groups_keep_modifiers_counts_bindings_and_original_errors() {
    check(
        r#"
        var patterns=[
            '(?<outer>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))+',
            '(?im-s:(?<outer>[\\p{RGI_Emoji}]+(?<!(?<zero>\\uD800{0})))){0002,0003}?',
            '(?i:(?<outer>[\\p{RGI_Emoji}]+(?!(?<zero>(?<erased>\\k<outer>){0})))){1,}',
            '((?!(?<left>[\\p{RGI_Emoji}]{0}))|(?<right>[\\p{RGI_Emoji}]+)(?<!a{0})){2}'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?!)|[\\p{Not_A_Property}])+','((?!)|(?<x>a)(?<x>b))+','((?!)|\\k<missing>)+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn required_impossible_consumers_clones_and_collection_preserve_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?i:(?<outside>[\p{RGI_Emoji}]+(?!(?<x>a{0})))){2,3}/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn positive_impossible_assertions_keep_directions_original_bindings_and_early_errors() {
    check(
        r#"
        var patterns=[
            '[\\p{RGI_Emoji}]+(?=(?!(?<zero>a{0})))(?<tail>b)',
            '(?<=(?<!\\uD800{0})(?<inside>[\\p{RGI_Emoji}]+))(?<tail>b)',
            '(?=(?i:(?<outside>[\\p{RGI_Emoji}]+(?!(?<erased>\\k<outside>){0}))){2,3})',
            '(?<=((?!(?<left>[\\p{RGI_Emoji}]{0}))|(?<right>[\\p{RGI_Emoji}]+)(?<!a{0})))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?=(?!)|[\\p{Not_A_Property}])','(?<=(?!)(?<x>a)(?<x>b))','(?=(?!)\\k<missing>)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn positive_impossible_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/[\p{RGI_Emoji}]+(?<=(?!(?<x>a{0})))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn composite_negative_assertions_keep_original_bindings_inventories_and_early_errors() {
    check(
        r#"
        var patterns=[
            '[\\p{RGI_Emoji}]+(?!(?!(?!(?<zero>a{0}))))(?<tail>b)',
            '(?<!(?<before>a{0})(?!(?!(?<inside>[\\p{RGI_Emoji}]{0}))))(?<tail>b)',
            '(?!(?i:(?<outside>(a{0})(?!(?!(?<erased>\\k<outside>){0})))){2,3})',
            '(?!(?!(?!(?<same>[\\p{RGI_Emoji}]{0})))|(?<same>[\\p{RGI_Emoji}]+))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!(?!(?!))|[\\p{Not_A_Property}])','(?<!(?!(?!))(?<x>a)(?<x>b))','(?!(?!(?!))\\k<missing>)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn composite_negative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/[\p{RGI_Emoji}]+(?<!(?!(?!(?<x>a{0}))))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀';
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn wrapped_composite_negative_assertions_keep_required_counts_modifiers_and_original_bindings() {
    check(
        r#"
        var patterns=[
            '(?<outside>[\\p{RGI_Emoji}]+(?!(?!(?!(?<zero>a{0})))))+',
            '(?im-s:(?<outside>(a{0})(?<!(?!(?!(?<zero>\\uD800{0})))))){0002,0003}?',
            '(?<outer>(?!(?<before>a{0})(?!(?!(?<erased>\\k<outer>){0})))){1,}'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?!(?!(?!)))|[\\p{Not_A_Property}])+','((?!(?!(?!)))(?<x>a)(?<x>b))+','((?!(?!(?!)))\\k<missing>)+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn wrapped_composite_negative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?i:(?<outside>[\p{RGI_Emoji}]+(?!(?!(?!(?<x>a{0})))))){2,3}/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn composite_negative_root_alternatives_keep_original_cross_arm_and_duplicate_name_bindings() {
    check(
        r#"
        var patterns=[
            '(?!(?!(?!(?<same>[\\p{RGI_Emoji}]{0}))))|(?<same>[\\p{RGI_Emoji}]+)(?<!(?!(?!)))',
            '(?i:(?<outside>[\\p{RGI_Emoji}]+(?!(?!(?!))))){2,3}|(?<!(?!(?!)))',
            '(?!(?!(?!(?<erased>\\k<later>){0})))|(?<later>[\\p{RGI_Emoji}]+)(?!(?!(?!)))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!(?!(?!)))|[\\p{Not_A_Property}]','(?!(?!(?!)))|(?<x>a)(?<x>b)','(?!(?!(?!)))|\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn composite_negative_root_alternative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?!(?!(?<same>[\p{RGI_Emoji}]{0}))))|(?<same>[\p{RGI_Emoji}]+)(?<!(?!(?!)))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn many_composite_negative_root_alternatives_keep_duplicate_name_inventories_without_default_quotas()
 {
    check(
        r#"
        var pattern=Array(5).fill('(?!(?!(?!(?<same>[\\p{RGI_Emoji}]{0}))))').join('|'),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let pattern = JsString::from(
        std::iter::repeat_n(r"(?!(?!(?!(?<same>[\p{RGI_Emoji}]{0}))))", 1000)
            .collect::<Vec<_>>()
            .join("|")
            .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn enclosing_composite_alternatives_keep_required_counts_modifiers_and_original_cross_arm_bindings()
{
    check(
        r#"
        var patterns=[
            '(?<outer>(?!(?!(?!(?<same>[\\p{RGI_Emoji}]{0}))))|(?<same>[\\p{RGI_Emoji}]+)(?<!(?!(?!))))+',
            '(?im-s:(?<outer>(?!(?!(?!(?<erased>\\k<outer>){0})))|[\\p{RGI_Emoji}]+(?<!(?!(?!))))){0002,0003}?',
            '((?!(?!(?!(?<erased>\\k<later>){0})))|(?<later>[\\p{RGI_Emoji}]+)(?!(?!(?!)))){1,}'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?!(?!(?!)))|[\\p{Not_A_Property}]){2,3}','((?!(?!(?!)))|(?<x>a)(?<x>b)){2,3}','((?!(?!(?!)))|\\k<missing>)+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn enclosing_composite_alternative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outer>(?!(?!(?!(?<same>[\p{RGI_Emoji}]{0}))))|(?<same>[\p{RGI_Emoji}]+)(?<!(?!(?!))))+/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_nullable_group_spines_keep_original_bindings_even_when_operands_could_consume() {
    check(
        r#"
        var patterns=[
            '(?!(?<many>[\\p{RGI_Emoji}]+(?<tail>a)?){0,3})',
            '(?<outside>[\\p{RGI_Emoji}]+)(?!(?<skipped>\\k<outside>){0,3})(?<tail>b)',
            '(?<!(?im-s:((?<letter>[\\p{RGI_Emoji}]+)(?<part>ab)?)*?){2,3})',
            '(?!(?:(?<word>[\\q{ab}]+)|(?<other>[\\p{RGI_Emoji}]+))*)'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!([\\p{Not_A_Property}])*)','(?!((?<x>a)(?<x>b))*)','(?!(\\k<missing>)*)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_nullable_group_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<many>[\p{RGI_Emoji}]+(?<tail>a)?){0,3})/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_nullable_atoms_keep_modes_wrappers_and_original_full_inventories() {
    check(
        r#"
        var patterns=[
            '(?<prefix>a{0})(?![\\p{RGI_Emoji}]*)(?<tail>b)',
            '(?<!\\p{RGI_Emoji}{0,3})(?<tail>a)',
            '(?!(?im-s:(?<outside>([\\q{ab}]*))){2,3})',
            '(?!\\uD83D\\uDE00*)(?<after>[\\p{RGI_Emoji}]+)'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!\\p{Not_A_Property}*)','(?!a*)(?<x>a)(?<x>b)','(?!a*)\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_nullable_atom_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?![\p{RGI_Emoji}]*)/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_nullable_sequences_keep_original_references_counts_and_mode_aware_term_boundaries() {
    check(
        r#"
        var patterns=[
            '(?!(?<maybe>a)?[\\p{RGI_Emoji}]*[\\q{ab}]?)',
            '(?<outside>a{0})(?!(?<inside>\\k<outside>)?[\\p{RGI_Emoji}]*)',
            '(?<!(?im-s:((?<many>[\\p{RGI_Emoji}]+)?)(?<part>ab)*[\\q{ab}]?){2,3})',
            '(?!(?:\\x28*[\\p{RGI_Emoji}]*\\u007C?))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!a*[\\p{Not_A_Property}]*)','(?!a*b*)(?<x>a)(?<x>b)','(?!a*b*)\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_nullable_sequence_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<maybe>a)?[\p{RGI_Emoji}]*[\q{ab}]?)/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_nullable_choices_keep_all_original_slots_while_only_one_arm_must_succeed() {
    check(
        r#"
        var patterns=[
            '(?!(?<word>[\\p{RGI_Emoji}]+)|(?<maybe>a)?)',
            '(?<outside>a{0})(?!(?<inside>\\k<outside>)+|[\\p{RGI_Emoji}]*)',
            '(?<!(?im-s:((?<first>[\\p{RGI_Emoji}]+)|(?<maybe>a)?)){2,3})',
            '(?!(?<same>[\\p{RGI_Emoji}]+)|(?<same>[\\q{ab}])*)',
            '(?![\\p{RGI_Emoji}]+|)'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!\\p{Not_A_Property}|)','(?!(?<x>a)(?<x>b)|)','(?!(\\k<missing>)|)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_nullable_choice_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<word>[\p{RGI_Emoji}]+)|(?<maybe>a)?)/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_successful_positive_assertions_keep_both_directions_and_original_variable_capture_bindings()
 {
    check(
        r#"
        var patterns=[
            '(?!(?=(?<maybe>[\\p{RGI_Emoji}]*)))',
            '(?<!(?<=(?<maybe>[\\p{RGI_Emoji}]*)))',
            '(?<outside>a{0})(?!(?i:((?=(?<inside>\\k<outside>)*|[\\p{RGI_Emoji}]+))){2,3})',
            '(?!(?=(?<same>[\\p{RGI_Emoji}]+)|(?<same>[\\q{ab}])?))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!(?=\\p{Not_A_Property}*))','(?!(?=(?<x>a)(?<x>b)))','(?!(?=\\k<missing>*))']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_successful_positive_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?=(?<maybe>[\p{RGI_Emoji}]*)))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn wrapped_nullable_negative_failures_keep_required_counts_modifiers_and_full_original_bindings() {
    check(
        r#"
        var patterns=[
            '(?<outside>(?!(?<many>[\\p{RGI_Emoji}]+)?))+',
            '(?im-s:(?<outside>(?![\\p{RGI_Emoji}]*))){0002,0003}?',
            '(?<outside>(?!(?<maybe>a)?[\\p{RGI_Emoji}]*)){1,}',
            '(?<outside>(?!(?<many>[\\p{RGI_Emoji}]+)|(?<maybe>a)?))+',
            '(?<outside>(?!(?=(?<maybe>[\\p{RGI_Emoji}]*))))+',
            '(?im-s:(?<outside>(a{0})(?!(?<inside>\\k<outside>)?[\\p{RGI_Emoji}]*))){2,3}'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?![\\p{Not_A_Property}]*))+','((?!a*)(?<x>a)(?<x>b))+','((?!a*)\\k<missing>)+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn wrapped_nullable_negative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?im-s:(?<outside>(?![\p{RGI_Emoji}]*))){2,3}/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn known_impossible_alternatives_mix_old_and_nullable_guards_with_original_bindings() {
    check(
        r#"
        var patterns=[
            '(?![\\p{RGI_Emoji}]*)|(?!(?<zero>[\\p{RGI_Emoji}]{0}))',
            '(?<same>(?!(?<inside>[\\p{RGI_Emoji}]{0})))|(?<same>(?!(?=[\\p{RGI_Emoji}]*)))+',
            '(?=([\\p{RGI_Emoji}]+)(?!(a{0})))|(?!a*[\\p{RGI_Emoji}]*)',
            '(?!(?<erased>\\k<later>){0})|(?<later>(?![\\p{RGI_Emoji}]*))(?!(b{0}))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!a*)|[\\p{Not_A_Property}]','(?!a*)|(?<x>a)(?<x>b)','(?!a*)|\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn known_impossible_alternative_consumers_clones_and_collection_keep_no_match_state() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?![\p{RGI_Emoji}]*)|(?!(?<zero>[\p{RGI_Emoji}]{0}))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!/'})==='a😀'&&called===0&&'a😀'.search(r)===-1&&'a😀'.split(r).length===1&&'a😀'.split(r)[0]==='a😀'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn many_known_impossible_alternatives_keep_full_inventories_without_default_quotas() {
    check(
        r#"
        var pattern=Array(5).fill('(?![\\p{RGI_Emoji}]*)|(?!(?<same>[\\p{RGI_Emoji}]{0}))').join('|'),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let pattern = JsString::from(
        std::iter::repeat_n(r"(?![\p{RGI_Emoji}]*)|(?!(?<same>[\p{RGI_Emoji}]{0}))", 500)
            .collect::<Vec<_>>()
            .join("|")
            .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;r.exec('a😀')===null&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn required_established_impossible_operands_keep_mixed_choices_counts_and_original_bindings() {
    check(
        r#"
        var patterns=[
            '((?![\\p{RGI_Emoji}]*)|(?!(?<slot>[\\p{RGI_Emoji}]{0})))+',
            '(?im-s:((?!(\\4){0})|((?![\\p{RGI_Emoji}]*))(?!(b{0})))){2,3}',
            '((?=([\\p{RGI_Emoji}]+)(?!(a{0})))|(?!a*[\\p{RGI_Emoji}]*)){0001,0002}?',
            '(?<outside>(?<same>(?![\\p{RGI_Emoji}]*))|(?<same>(?!(?<slot>[\\p{RGI_Emoji}]{0}))))+'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;
            ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?!a*)|[\\p{Not_A_Property}])+','((?!a*)|(?<x>a)(?<x>b))+','((?!a*)|\\k<missing>)+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn required_established_impossible_consumers_clones_and_collection_preserve_no_match() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/((?![\p{RGI_Emoji}]*)|(?!(?<slot>[\p{RGI_Emoji}]{0})))+/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!';})==='a😀'&&called===0&&'a😀'.search(r)===-1&&JSON.stringify('a😀'.split(r))==='["a😀"]'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_established_optional_empty_operands_keep_full_original_bindings_and_inventories() {
    check(
        r#"
        var operands=[
            '(?<skip>(?![\\p{RGI_Emoji}]*))?',
            '(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+',
            '(?<outside>(?<rolled>(?<skip>(?![\\p{RGI_Emoji}]*))?(?<last>[a]{0})+)*)+',
            '(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)',
            '(?<=(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b))',
            '(\\3){0}(?<skip>(?![\\p{RGI_Emoji}]*))?|((a)\\1)+'
        ],ok=true;
        for(var operand of operands)for(var head of ['(?!','(?<!'])for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var pattern='(a|ab)*'+head+operand+')(z)*',r=new RegExp(pattern,flags);r.lastIndex=start;ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['(?!((?![\\p{RGI_Emoji}]*))?|[\\p{Not_A_Property}])','(?<!((?![\\p{RGI_Emoji}]*))?|(?<x>a)(?<x>b))','(?!((?![\\p{RGI_Emoji}]*))?|\\k<missing>)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_established_optional_empty_consumers_clones_and_collection_keep_no_match() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<=(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)))/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!';})==='a😀'&&called===0&&'a😀'.search(r)===-1&&JSON.stringify('a😀'.split(r))==='["a😀"]'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}

#[test]
fn required_negative_established_optional_empty_spines_keep_counts_original_bindings_and_inventory()
{
    check(
        r#"
        var patterns=[
            '((?!(?<skip>(?![\\p{RGI_Emoji}]*))?))+',
            '(?im-s:((a|ab)*(?!(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+)(z)*)){0001,0002}?',
            '((?<!(?<=(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)))){2,3}',
            '((?!(\\3){0}(?<skip>(?![\\p{RGI_Emoji}]*))?|((a)\\1)+))+'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }
        for(var source of ['((?!((?![\\p{RGI_Emoji}]*))?|[\\p{Not_A_Property}]))+','((?<!((?![\\p{RGI_Emoji}]*))?|(?<x>a)(?<x>b)))+','((?!((?![\\p{RGI_Emoji}]*))?|\\k<missing>))+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn required_negative_established_optional_empty_consumers_clones_and_collection_preserve_no_match()
{
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/((?!(?<=(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b))))+/dgv,called=0,ok='a😀'.match(r)===null&&Array.from('a😀'.matchAll(r)).length===0;
        ok=ok&&'a😀'.replace(r,function(){called++;return '!';})==='a😀'&&called===0&&'a😀'.search(r)===-1&&JSON.stringify('a😀'.split(r))==='["a😀"]'&&''.split(r).length===1;
        var copy=new RegExp(r);copy.lastIndex=1;ok&&copy.exec('a😀')===null&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=2;copy.exec('😀')===null&&copy.lastIndex===0&&copy.hasIndices&&copy.unicodeSets"),Ok(Value::Boolean(true)));
}
