//! Negative assertions over impossible operands succeed empty without captures.

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
fn negative_impossible_assertions_keep_undefined_names_indices_and_unicode_boundaries() {
    check(
        r#"
        var patterns=[
            '(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0})))',
            '(?<!(?i:(?<outside>[\\p{RGI_Emoji}]+(?!(?<erased>\\k<outside>){0}))){2,3})',
            '(?<!((?!(?<left>[\\p{RGI_Emoji}]{0}))|(?<right>[\\p{RGI_Emoji}]+)(?<!a{0})))',
            '(?!(?<=[\\p{RGI_Emoji}]+(?!(?<zero>\\uD800{0}))))'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv']) {
            var r=new RegExp(pattern,flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m!==null&&m[0]===''&&m.index===0&&m.input==='😀a'&&m.indices[0][0]===0&&m.indices[0][1]===0&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===pattern;
            for(var i=1;i<m.length;i++)ok=ok&&m[i]===undefined&&m.indices[i]===undefined;
            for(var name of Object.keys(m.groups))ok=ok&&m.groups[name]===undefined&&m.indices.groups[name]===undefined&&Object.hasOwn(m.indices.groups,name);
            r.lastIndex=4;ok=ok&&(r.global||r.sticky?r.exec('😀a')===null&&r.lastIndex===0:r.exec('😀a').index===0&&r.lastIndex===4);
        }
        for(var source of ['(?!(?!)|[\\p{Not_A_Property}])','(?<!(?!)(?<x>a)(?<x>b))','(?!(?!)\\k<missing>)']) {
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_impossible_consumers_callbacks_clones_and_collection_keep_empty_results() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?!(?<inside>[\p{RGI_Emoji}]+)(?!(?<zero>a{0})))/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[0].index===0&&matches[1].index===2&&matches[2].index===3;
        ok=ok&&'😀a'.replace(r,function(match,inside,zero,offset,input,groups){ok=ok&&match===''&&inside===undefined&&zero===undefined&&groups.inside===undefined&&groups.zero===undefined&&input==='😀a';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&'a😀b'.search(r)===0&&JSON.stringify('a😀b'.split(r))==='["a",null,null,"😀",null,null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===undefined&&m.indices[1]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.inside===undefined&&after.indices.groups.zero===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_impossible_deep_operands_and_capture_masks_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='(?<!'+'('.repeat(depth)+'[\\p{RGI_Emoji}]+(?!(?<zero>\\uD800{0}))'+')+'.repeat(depth)+')',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+2&&m[depth+1]===undefined&&m.indices[depth+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    for (depth, minimum) in [(1000, "1".to_owned()), (1, "9".repeat(20_000))] {
        let pattern = JsString::from(
            format!(
                r"(?<!{}[\p{{RGI_Emoji}}]+(?!(?<zero>\uD800{{0}})){}{{{minimum}}})",
                "(".repeat(depth),
                ")+".repeat(depth - 1) + ")"
            )
            .as_str(),
        );
        check(&format!(
            "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===undefined&&m.indices[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
            depth + 2,
            depth + 1,
            depth + 1
        ));
    }
}

#[test]
fn enclosing_negative_impossible_assertions_keep_required_optional_and_named_capture_masks() {
    check(
        r#"
        var rows=[
            ['(?<outside>(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))+', [true,false,false]],
            ['(?<outside>(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))?', [false,false,false]],
            ['(?<outer>(?<inner>(?<!(?<inside>[\\p{RGI_Emoji}]+)(?<!a{0})))?){2,3}', [true,false,false]],
            ['(?i:(?<outside>(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>\\k<outside>){0})))){2,3}', [true,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']) {
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
            var key=Object.hasOwn(m.groups,'outside')?'outside':'outer';ok=ok&&m.groups[key]===m[1]&&m.indices.groups[key]===m.indices[1];
        }ok
    "#,
    );
}

#[test]
fn enclosing_negative_impossible_consumers_clones_and_collection_keep_capture_roles() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?!(?<inside>[\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))+/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,outside,inside,zero,offset,input,groups){ok=ok&&outside===''&&inside===undefined&&zero===undefined&&groups.outside==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,"😀","",null,null,"b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.inside===undefined&&after.indices.groups.outside[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn enclosing_negative_impossible_deep_masks_and_huge_counts_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'(?<![\\p{RGI_Emoji}]+(?!(?<zero>\\uD800{0})))'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+2&&m[depth]===''&&m[depth+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    for (depth, minimum, expected) in [
        (1000, "1".to_owned(), "''"),
        (1, "9".repeat(20_000), "''"),
        (1, "0".to_owned(), "undefined"),
    ] {
        let pattern = JsString::from(
            format!(
                r"{}(?<![\p{{RGI_Emoji}}]+(?!(?<zero>\uD800{{0}}))){}{{{minimum}}}",
                "(".repeat(depth),
                ")+".repeat(depth - 1) + ")"
            )
            .as_str(),
        );
        check(&format!(
            "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]==={expected}&&m[{}]===undefined&&m.indices[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
            depth + 2,
            depth,
            depth + 1,
            depth + 1
        ));
    }
}

#[test]
fn first_negative_impossible_choices_keep_first_captures_and_skip_later_named_slots() {
    check(
        r#"
        var rows=[
            ['(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0})))|(?<later>[\\p{RGI_Emoji}]+)', [false,false,false]],
            ['(?<outside>(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))+|(?<later>[\\p{RGI_Emoji}]+)', [true,false,false,false]],
            ['(?<outside>(?!(?<inside>[\\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))?|(?<later>[\\p{RGI_Emoji}]+)', [false,false,false,false]],
            ['(?!(?!(?<same>[\\p{RGI_Emoji}]{0})))|(?<same>[\\p{RGI_Emoji}]+)', [false,false]],
            ['(?<outside>(?!(?!(?<zero>\\k<later>){0})))|(?<later>[\\p{RGI_Emoji}]+)', [true,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']) {
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['(?!(?!))|[\\p{Not_A_Property}]','(?!(?!))|(?<x>a)(?<x>b)','(?!(?!))|\\k<missing>']) {
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn first_negative_impossible_choice_consumers_clones_and_collection_keep_skipped_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?!(?<inside>[\p{RGI_Emoji}]+)(?!(?<zero>a{0}))))+|(?<later>[\p{RGI_Emoji}]+)/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,outside,inside,zero,later,offset,input,groups){ok=ok&&outside===''&&inside===undefined&&zero===undefined&&later===undefined&&groups.outside===''&&groups.later===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,null,"😀","",null,null,null,"b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[4]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.later===undefined&&after.indices.groups.later===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn first_negative_impossible_choices_keep_large_later_inventories_without_default_quotas() {
    check(
        r#"
        var depth=10,pattern='(?!(?!(?<zero>[\\p{RGI_Emoji}]{0})))|'+'('.repeat(depth)+'[\\p{RGI_Emoji}]'+')'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+2&&m[depth+1]===undefined&&m.indices[depth+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"(?!(?!(?<zero>[\p{{RGI_Emoji}}]{{0}})))|{}[\p{{RGI_Emoji}}]{}",
            "(".repeat(depth),
            ")".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===undefined&&m.indices[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 2,
        depth + 1,
        depth + 1
    ));
}

#[test]
fn negative_impossible_sequences_keep_interleaved_capture_roles_and_original_bindings() {
    check(
        r#"
        var rows=[
            ['(?<before>a{0})(?!(?<inside>[\\p{RGI_Emoji}]+)(?!a{0}))(?<after>b{0})',[true,false,true]],
            ['(?<before>a{0})(?!(?!(?<erased>\\k<tail>){0}))(?<tail>[\\p{RGI_Emoji}]{0})',[true,false,true]],
            ['\\uD83D\\uDE00{0}(?<!(?<![\\p{RGI_Emoji}]{0}))(a{0})',[true]],
            ['(?i:(((?!(?!(a{0}))))?)+)(?<after>[\\p{RGI_Emoji}]{0})',[true,false,false,true]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']) {
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['(?!(?!))[\\p{Not_A_Property}]{0}','(?!(?!))(?<x>a{0})(?<x>b{0})','(?!(?!))\\k<missing>']) {
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn negative_impossible_sequence_consumers_clones_and_collection_keep_capture_order() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<before>a{0})(?!(?<inside>[\p{RGI_Emoji}]+)(?!a{0}))(?<after>b{0})/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,before,inside,after,offset,input,groups){ok=ok&&before===''&&inside===undefined&&after===''&&groups.before===''&&groups.inside===undefined&&groups.after==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,"","😀","",null,"","b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===''&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.before===''&&after.groups.inside===undefined&&after.groups.after===''&&after.indices.groups.after[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn many_negative_impossible_sequence_terms_keep_full_masks_without_default_quotas() {
    check(
        r#"
        var count=10,pattern='(?!(?!([\\p{RGI_Emoji}]{0})))'.repeat(count)+'(?<tail>a{0})',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===count+2&&m[1]===undefined&&m[count]===undefined&&m[count+1]===''&&m.groups.tail===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let count = 1000;
    let pattern =
        JsString::from((r"(?!(?!([\p{RGI_Emoji}]{0})))".repeat(count) + r"(?<tail>a{0})").as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===undefined&&m[{}]===''&&m.indices[{}][0]===0&&m.groups.tail===''&&r.lastIndex===0&&r.source==={pattern:?}",
        count + 2,
        count,
        count + 1,
        count + 1
    ));
}

#[test]
fn enclosing_negative_impossible_sequences_keep_required_optional_and_inner_capture_masks() {
    check(
        r#"
        var rows=[
            ['(?<outside>(?<before>a{0})(?!(?<inside>[\\p{RGI_Emoji}]+)(?!a{0}))(?<after>b{0}))+',[true,true,false,true]],
            ['(?<outside>(?<before>a{0})(?!(?<inside>[\\p{RGI_Emoji}]+)(?!a{0}))(?<after>b{0}))?',[false,false,false,false]],
            ['(?i:(?<outside>(?<before>a{0})(?!(?!(?<erased>\\k<outside>){0}))(?<after>[\\p{RGI_Emoji}]{0}))){2,3}',[true,true,false,true]],
            ['(?<outer>(?<inner>(a{0})(?!(?!([\\p{RGI_Emoji}]{0}))))?)+',[true,false,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']) {
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }ok
    "#,
    );
}

#[test]
fn enclosing_negative_impossible_sequence_consumers_clones_and_collection_keep_inner_masks() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<before>a{0})(?!(?<inside>[\p{RGI_Emoji}]+)(?!a{0}))(?<after>b{0}))+/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,outside,before,inside,after,offset,input,groups){ok=ok&&outside===''&&before===''&&inside===undefined&&after===''&&groups.outside===''&&groups.before===''&&groups.inside===undefined&&groups.after==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","","",null,"","😀","","",null,"","b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===''&&m[3]===undefined&&m[4]===''&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.before===''&&after.groups.inside===undefined&&after.groups.after===''&&after.indices.groups.after[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn enclosing_negative_impossible_sequence_depth_and_huge_bounds_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'(a{0})(?!(?!([\\p{RGI_Emoji}]{0})))'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+3&&m[depth+1]===''&&m[depth+2]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    for (depth, minimum, expected) in [
        (1000, "1".to_owned(), "''"),
        (1, "9".repeat(20_000), "''"),
        (1, "0".to_owned(), "undefined"),
    ] {
        let pattern = JsString::from(
            format!(
                r"{}(a{{0}})(?!(?!([\p{{RGI_Emoji}}]{{0}}))){}{{{minimum}}}",
                "(".repeat(depth),
                ")+".repeat(depth - 1) + ")"
            )
            .as_str(),
        );
        check(&format!(
            "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]==={expected}&&m[{}]==={expected}&&m[{}]===undefined&&m.indices[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
            depth + 3,
            depth,
            depth + 1,
            depth + 2,
            depth + 2
        ));
    }
}
