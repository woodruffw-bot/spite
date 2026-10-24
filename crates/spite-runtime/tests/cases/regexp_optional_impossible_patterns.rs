//! Optional impossible operands stay empty and roll descendant captures back.

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
fn optional_impossible_operands_keep_required_outside_and_undefined_inside_masks() {
    check(
        r#"
        var rows=[
            ['(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?',[false,false]],
            ['(?<outside>(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?)+',[true,false,false]],
            ['(?im-s:(?<outside>(?<skipped>[\\p{RGI_Emoji}]+(?<!(?<zero>\\uD800{0})))*)){2,3}',[true,false,false]],
            ['(?<outside>(?<skipped>(?!(?!(?!(?<zero>[\\p{RGI_Emoji}]{0}))))|(?<other>[\\p{RGI_Emoji}]+)(?<!(?!(?!))))?)+',[true,false,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
            for(var name of Object.keys(m.groups)){ok=ok&&Object.hasOwn(m.indices.groups,name);if(m.groups[name]===undefined)ok=ok&&m.indices.groups[name]===undefined;else ok=ok&&m.groups[name]===''&&m.indices.groups[name][0]===0&&m.indices.groups[name][1]===0;}
        }
        for(var source of ['((?!)|[\\p{Not_A_Property}])?','((?!)(?<x>a)(?<x>b))?','((?!)\\k<missing>)?']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn optional_impossible_consumers_clones_and_collection_preserve_rolled_back_captures() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?)+/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[0].index===0&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.skipped===undefined;
        ok=ok&&'😀a'.replace(r,function(match,outside,skipped,zero,offset,input,groups){ok=ok&&outside===''&&skipped===undefined&&zero===undefined&&groups.outside===''&&groups.skipped===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,"😀","",null,null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m.indices[2]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.skipped===undefined&&after.indices.groups.zero===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_impossible_depth_masks_and_huge_upper_bounds_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'([\\p{RGI_Emoji}]+(?!(?<zero>\\uD800{0})))?'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+3&&m[depth]===''&&m[depth+1]===undefined&&m[depth+2]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}([\p{{RGI_Emoji}}]+(?!(?<zero>\uD800{{0}})))?{}",
            "(".repeat(depth),
            ")+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===''&&m[{}]===undefined&&m[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 3,
        depth,
        depth + 1,
        depth + 2
    ));
    let pattern = JsString::from(
        format!(
            r"([\p{{RGI_Emoji}}]+(?!(?<zero>\uD800{{0}}))){{0,{}}}",
            "9".repeat(20_000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===3&&m[1]===undefined&&m[2]===undefined&&m.groups.zero===undefined&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn optional_impossible_sequences_keep_interleaved_required_skipped_and_original_forward_bindings() {
    check(
        r#"
        var rows=[
            ['(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0})',[true,false,false,true]],
            ['(?<before>a{0})(?<skipped>(?!(?<zero>\\k<after>){0}))?(?<after>[\\p{RGI_Emoji}]{0})',[true,false,false,true]],
            ['(?<outside>(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?)+(?<after>b{0})',[true,false,false,true]],
            ['\\uD83D\\uDE00{0}(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0})',[false,false,true]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['((?!))?[\\p{Not_A_Property}]{0}','((?!))?(?<x>a{0})(?<x>b{0})','((?!))?\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn optional_impossible_sequence_consumers_clones_and_collection_keep_capture_order() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<before>a{0})(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0})/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,before,skipped,zero,after,offset,input,groups){ok=ok&&before===''&&skipped===undefined&&zero===undefined&&after===''&&groups.before===''&&groups.skipped===undefined&&groups.after==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,"","😀","",null,null,"","b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m[4]===''&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.before===''&&after.groups.skipped===undefined&&after.groups.after===''&&after.indices.groups.after[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn many_optional_impossible_sequence_terms_keep_full_interleaved_masks_without_default_quotas() {
    check(
        r#"
        var count=10,pattern='([\\p{RGI_Emoji}]+(?!(a{0})))?'.repeat(count)+'(?<tail>b{0})',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===count*2+2&&m[count*2]===undefined&&m[count*2+1]===''&&m.groups.tail===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let count = 1000;
    let pattern = JsString::from(
        (r"([\p{RGI_Emoji}]+(?!(a{0})))?".repeat(count) + r"(?<tail>b{0})").as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===undefined&&m[{}]===''&&m.indices[{}][0]===0&&m.groups.tail===''&&r.lastIndex===0&&r.source==={pattern:?}",
        count * 2 + 2,
        count * 2,
        count * 2 + 1,
        count * 2 + 1
    ));
}

#[test]
fn enclosing_optional_impossible_sequences_keep_required_optional_and_descendant_masks() {
    check(
        r#"
        var rows=[
            ['(?<outside>(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))+',[true,true,false,false,true]],
            ['(?<outside>(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))?',[false,false,false,false,false]],
            ['(?im-s:(?<outside>(?<before>a{0})(?<skipped>(?!(?<zero>\\k<outside>){0}))?(?<after>[\\p{RGI_Emoji}]{0}))){2,3}',[true,true,false,false,true]],
            ['(?<outer>(?<inner>(a{0})([\\p{RGI_Emoji}]+(?!(a{0})))?(b{0}))?)+',[true,false,false,false,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }ok
    "#,
    );
}

#[test]
fn enclosing_optional_impossible_sequence_consumers_clones_and_collection_keep_inner_masks() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<before>a{0})(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))+/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,outside,before,skipped,zero,after,offset,input,groups){ok=ok&&outside===''&&before===''&&skipped===undefined&&zero===undefined&&after===''&&groups.outside===''&&groups.before===''&&groups.skipped===undefined&&groups.after==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","","",null,null,"","😀","","",null,null,"","b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===''&&m[3]===undefined&&m[4]===undefined&&m[5]===''&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.before===''&&after.groups.skipped===undefined&&after.groups.after===''&&after.indices.groups.after[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn enclosing_optional_impossible_sequence_depth_and_huge_bounds_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'(a{0})([\\p{RGI_Emoji}]+(?!(a{0})))?(b{0})'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+5&&m[depth+1]===''&&m[depth+2]===undefined&&m[depth+3]===undefined&&m[depth+4]===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    for (depth, minimum, expected) in [
        (1000, "1".to_owned(), "''"),
        (1, "9".repeat(20_000), "''"),
        (1, "0".to_owned(), "undefined"),
    ] {
        let pattern = JsString::from(
            format!(
                r"{}(a{{0}})([\p{{RGI_Emoji}}]+(?!(a{{0}})))?(b{{0}}){}{{{minimum}}}",
                "(".repeat(depth),
                ")+".repeat(depth - 1) + ")"
            )
            .as_str(),
        );
        check(&format!(
            "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]==={expected}&&m[{}]==={expected}&&m[{}]===undefined&&m[{}]===undefined&&m[{}]==={expected}&&r.lastIndex===0&&r.source==={pattern:?}",
            depth + 5,
            depth,
            depth + 1,
            depth + 2,
            depth + 3,
            depth + 4
        ));
    }
}

#[test]
fn first_optional_impossible_choices_keep_first_masks_and_all_later_undefined_names() {
    check(
        r#"
        var rows=[
            ['(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<later>[\\p{RGI_Emoji}]+)',[false,false,false]],
            ['(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<later>[\\p{RGI_Emoji}]+)',[true,false,false,false]],
            ['(?<outside>(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))+|(?<later>[\\p{RGI_Emoji}]+)',[true,true,false,false,true,false]],
            ['(?<same>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<same>[\\p{RGI_Emoji}]+)',[false,false,false]],
            ['(?<before>a{0})(?<skipped>(?!(?<zero>\\k<later>){0}))?|(?<later>[\\p{RGI_Emoji}]+)',[true,false,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        var named=/(?<same>a{0})(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<same>[\p{RGI_Emoji}]+)/dgv,hit=named.exec('😀');
        ok=ok&&hit.groups.same===''&&hit.indices.groups.same[0]===0&&hit.indices.groups.same[1]===0&&hit.groups.skipped===undefined;
        for(var source of ['((?!))?|[\\p{Not_A_Property}]','((?!))?|(?<x>a)(?<x>b)','((?!))?|\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn first_optional_impossible_choice_consumers_clones_and_collection_keep_skipped_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<before>a{0})(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<later>[\p{RGI_Emoji}]+)/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,before,skipped,zero,later,offset,input,groups){ok=ok&&before===''&&skipped===undefined&&zero===undefined&&later===undefined&&groups.before===''&&groups.later===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,null,"😀","",null,null,null,"b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[4]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.before===''&&after.groups.later===undefined&&after.indices.groups.later===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn first_optional_impossible_choices_keep_large_later_inventories_without_default_quotas() {
    check(
        r#"
        var depth=10,pattern='([\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|'+'('.repeat(depth)+'[\\p{RGI_Emoji}]'+')'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+3&&m[depth+2]===undefined&&m.indices[depth+2]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"([\p{{RGI_Emoji}}]+(?!(?<zero>a{{0}})))?|{}[\p{{RGI_Emoji}}]{}",
            "(".repeat(depth),
            ")".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===undefined&&m.indices[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 3,
        depth + 2,
        depth + 2
    ));
}

#[test]
fn positive_optional_impossible_assertions_export_original_masks_names_and_directions() {
    check(
        r#"
        var rows=[
            ['(?=(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?)',[false,false]],
            ['(?<=(?<outside>(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?)+)',[true,false,false]],
            ['(?=(?<before>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))',[true,false,false,true]],
            ['(?=(?<same>a{0})(?<skipped>[\\p{RGI_Emoji}]+(?!(?<zero>a{0})))?|(?<same>[\\p{RGI_Emoji}]+))',[true,false,false,false]],
            ['(?<=(?<before>a{0})(?<skipped>(?!(?<zero>\\k<later>){0}))?|(?<later>[\\p{RGI_Emoji}]+))',[true,false,false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
            if(Object.hasOwn(m.groups,'same'))ok=ok&&m.groups.same===''&&m.indices.groups.same[0]===0&&m.indices.groups.same[1]===0;
        }
        for(var source of ['(?=((?!))?|[\\p{Not_A_Property}])','(?<=((?!))?(?<x>a)(?<x>b))','(?=((?!))?\\k<missing>)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn positive_optional_impossible_consumers_clones_and_collection_keep_exported_masks() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<=(?<before>a{0})(?<skipped>[\p{RGI_Emoji}]+(?!(?<zero>a{0})))?(?<after>b{0}))/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        ok=ok&&'😀a'.replace(r,function(match,before,skipped,zero,after,offset,input,groups){ok=ok&&before===''&&skipped===undefined&&zero===undefined&&after===''&&groups.before===''&&groups.skipped===undefined&&groups.after==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,"","😀","",null,null,"","b"]';
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m[4]===''&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.before===''&&after.groups.skipped===undefined&&after.groups.after===''&&after.indices.groups.after[0]===0&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn positive_optional_impossible_depth_masks_and_huge_upper_bounds_have_no_default_quotas() {
    check(
        r#"
        var depth=10,pattern='(?<='+'('.repeat(depth)+'(a{0})([\\p{RGI_Emoji}]+(?!(a{0})))?(b{0})'+')+'.repeat(depth)+')',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+5&&m[depth+1]===''&&m[depth+2]===undefined&&m[depth+3]===undefined&&m[depth+4]===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"(?<={}(a{{0}})([\p{{RGI_Emoji}}]+(?!(a{{0}})))?(b{{0}}){})",
            "(".repeat(depth),
            ")+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===''&&m[{}]===undefined&&m[{}]===undefined&&m[{}]===''&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 5,
        depth + 1,
        depth + 2,
        depth + 3,
        depth + 4
    ));
    let pattern = JsString::from(
        format!(
            r"(?=([\p{{RGI_Emoji}}]+(?!(?<zero>a{{0}}))){{0,{}}})",
            "9".repeat(20_000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===3&&m[1]===undefined&&m[2]===undefined&&m.groups.zero===undefined&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn optional_established_impossible_bodies_keep_required_prefix_and_all_skipped_descendants() {
    check(
        r#"
        var rows=[
            ['(?<skip>(?![\\p{RGI_Emoji}]*))?',[false]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+',[true,false]],
            ['(?i:(?<outside>(?<skip>(?!(?<slot>[\\p{RGI_Emoji}]+)?))*)){2,3}',[true,false,false]],
            ['(?<outside>(?<skip>(?!(?<ref>\\k<outside>)*))?)+',[true,false,false]],
            ['(?<skip>(?![\\p{RGI_Emoji}]*)|(?!(?<slot>[\\p{RGI_Emoji}]{0})))?',[false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');
            ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['((?![\\p{Not_A_Property}]*))?','((?!a*)(?<x>a)(?<x>b))?','((?!a*)\\k<missing>)?']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn optional_established_impossible_consumers_clones_and_collection_keep_rolled_back_captures() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+/dgv,ok=true,offsets=[];
        ok=ok&&JSON.stringify('😀a'.match(r))==='["","",""]';
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[0].index===0&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.skip===undefined;
        ok=ok&&'😀a'.replace(r,function(match,outside,skip,offset,input,groups){ok=ok&&outside===''&&skip===undefined&&groups.outside===''&&groups.skip===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,"😀","",null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m.indices[2]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.skip===undefined&&after.indices.groups.skip===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_established_impossible_depth_and_huge_upper_bounds_keep_full_masks_without_default_quotas()
 {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'(?<skipped>(?![\\p{RGI_Emoji}]*))?'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+2&&m[depth]===''&&m[depth+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}(?<skipped>(?![\p{{RGI_Emoji}}]*))?{}",
            "(".repeat(depth),
            ")+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===''&&m[{}]===undefined&&m.groups.skipped===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 2,
        depth,
        depth + 1
    ));
    let pattern = JsString::from(
        format!(
            r"(?<skipped>(?![\p{{RGI_Emoji}}]*)){{0,{}}}",
            "9".repeat(20_000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===2&&m[1]===undefined&&m.groups.skipped===undefined&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn optional_known_impossible_sequences_keep_original_interleaved_masks_bindings_and_errors() {
    check(
        r#"
        var rows=[
            ['(?<skip>(?![\\p{RGI_Emoji}]*))?(?<erased>(a)\\3){0}',[false,false,false]],
            ['(?<erased>(a)\\4){0}(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[b]{0})+',[false,false,true,false,true]],
            ['(?<left>(?![\\p{RGI_Emoji}]*)|(?!(?<inside>[\\p{RGI_Emoji}]{0})))?(?<right>(?![\\p{RGI_Emoji}]*))*',[false,false,false]],
            ['(?<skip>(?![\\p{RGI_Emoji}]*))?\\uD83D\\uDE00{0}',[false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['((?![\\p{RGI_Emoji}]*))?[\\p{Not_A_Property}]{0}','((?![\\p{RGI_Emoji}]*))?(?<x>a)(?<x>b){0}','((?![\\p{RGI_Emoji}]*))?\\k<missing>{0}']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn optional_known_impossible_sequence_consumers_clones_and_collection_keep_interleaved_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+/dgv,ok=true,offsets=[];
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[0].index===0&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.skip===undefined&&matches[1].groups.last==='';
        ok=ok&&'😀a'.replace(r,function(match,outside,skip,last,offset,input,groups){ok=ok&&outside===''&&skip===undefined&&last===''&&groups.last==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,"","😀","",null,"","b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===''&&m.indices[2]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.skip===undefined&&after.groups.last===''&&after.indices.groups.skip===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_known_impossible_sequences_keep_many_masks_without_default_quotas() {
    check(
        r#"
        var count=10,pattern='(((?![\\p{RGI_Emoji}]*))?)+((a)\\1){0}'.repeat(count),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===4*count+1&&m[1]===''&&m[2]===undefined&&m[4*count]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let count = 1000;
    let pattern = JsString::from(
        r"(((?![\p{RGI_Emoji}]*))?)+((a)\1){0}"
            .repeat(count)
            .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[1]===''&&m[2]===undefined&&m[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        4 * count + 1,
        4 * count
    ));
}

#[test]
fn enclosing_optional_known_sequences_keep_required_and_optional_prefix_masks() {
    check(
        r#"
        var body='(?<skip>(?![\\p{RGI_Emoji}]*))?(?<last>[a]{0})+',rows=[
            ['(?<outside>'+body+')+',[true,false,true]],
            ['(?<outside>'+body+')?',[false,false,false]],
            ['(?<outside>(?<rolled>'+body+')*)+',[true,false,false,false]],
            ['(?im-s:(?<outside>'+body+')){0001,0002}?',[true,false,true]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['(((?![\\p{RGI_Emoji}]*))?[\\p{Not_A_Property}]{0})+','(((?![\\p{RGI_Emoji}]*))?(?<x>a)(?<x>b){0})+','(((?![\\p{RGI_Emoji}]*))?\\k<missing>{0})+']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn enclosing_optional_known_sequence_consumers_clones_and_collection_keep_rolled_back_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<rolled>(?<skip>(?![\p{RGI_Emoji}]*))?(?<last>[a]{0})+)*)+/dgv,ok=true,offsets=[];
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.rolled===undefined&&matches[1].groups.last===undefined;
        ok=ok&&'😀a'.replace(r,function(match,outside,rolled,skip,last,offset,input,groups){ok=ok&&outside===''&&rolled===undefined&&skip===undefined&&last===undefined&&groups.outside==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,null,null,"😀","",null,null,null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[4]===undefined&&m.indices[4]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.last===undefined&&after.indices.groups.last===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn enclosing_optional_known_sequences_keep_deep_masks_and_huge_counts_without_default_quotas() {
    check(
        r#"
        var depth=10,body='(?<skip>(?![\\p{RGI_Emoji}]*))?(?<last>[a]{0})+',pattern='('.repeat(depth)+body+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+3&&m[depth]===''&&m[depth+1]===undefined&&m[depth+2]===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let body = r"(?<skip>(?![\p{RGI_Emoji}]*))?(?<last>[a]{0})+";
    let pattern =
        JsString::from(format!("{}{body}{}", "(".repeat(depth), ")+".repeat(depth)).as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===''&&m[{}]===undefined&&m[{}]===''&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 3,
        depth,
        depth + 1,
        depth + 2
    ));
    let pattern = JsString::from(format!("({body}){{{}}}", "9".repeat(20_000)).as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===''&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}

#[test]
fn first_optional_known_choices_keep_first_masks_duplicate_names_and_later_original_slots() {
    check(
        r#"
        var rows=[
            ['(?<skip>(?![\\p{RGI_Emoji}]*))?|(?<later>(a)\\1)+',[false,false,false]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)(c)*',[true,false,true,false,false]],
            ['(?<outside>(?<rolled>(?<skip>(?![\\p{RGI_Emoji}]*))?(?<last>[a]{0})+)*)+|(?=(b))c',[true,false,false,false,false]],
            ['(?<same>(?![\\p{RGI_Emoji}]*))?|(?<same>a)',[false,false]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var r=new RegExp(row[0],flags);r.lastIndex=1;var m=r.exec('😀a');ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===row[0];
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['((?![\\p{RGI_Emoji}]*))?|[\\p{Not_A_Property}]','((?![\\p{RGI_Emoji}]*))?|(?<x>a)(?<x>b)','((?![\\p{RGI_Emoji}]*))?|\\k<missing>']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn first_optional_known_choice_consumers_clones_and_collection_keep_unselected_arm_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)/dgv,ok=true,offsets=[];
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.skip===undefined&&matches[1].groups.last===''&&matches[1].groups.later===undefined;
        ok=ok&&'😀a'.replace(r,function(match,outside,skip,last,later,offset,input,groups){ok=ok&&outside===''&&skip===undefined&&last===''&&later===undefined&&groups.later===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,"",null,"😀","",null,"",null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===''&&m[4]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.last===''&&after.groups.later===undefined&&after.indices.groups.later===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn first_optional_known_choices_keep_many_unselected_slots_without_default_quotas() {
    check(
        r#"
        var count=10,pattern='(?<skip>(?![\\p{RGI_Emoji}]*))?|'+Array(count).fill('(a)').join('|'),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===count+2&&m[1]===undefined&&m[count+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let count = 1000;
    let pattern = JsString::from(
        format!(
            r"(?<skip>(?![\p{{RGI_Emoji}}]*))?|{}",
            std::iter::repeat_n("(a)", count)
                .collect::<Vec<_>>()
                .join("|")
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[1]===undefined&&m[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        count + 2,
        count + 1
    ));
}

#[test]
fn positive_optional_known_assertions_export_both_direction_masks_and_later_lexical_slots() {
    check(
        r#"
        var operands=[
            ['(?<skip>(?![\\p{RGI_Emoji}]*))?',[false]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+',[true,false,true]],
            ['(?<outside>(?<rolled>(?<skip>(?![\\p{RGI_Emoji}]*))?(?<last>[a]{0})+)*)+',[true,false,false,false]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b)',[true,false,true,false]]
        ],ok=true;
        for(var row of operands)for(var head of ['(?=','(?<='])for(var flags of ['dv','dgv','dgyv','dgimsv']){
            var source=head+row[0]+')',r=new RegExp(source,flags);r.lastIndex=1;var m=r.exec('😀a');ok=ok&&m.index===0&&m[0]===''&&m.length===row[1].length+1&&r.lastIndex===(r.global||r.sticky?0:1)&&r.source===source;
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===0&&m.indices[i+1][1]===0:m[i+1]===undefined&&m.indices[i+1]===undefined);
        }
        for(var source of ['(?=((?![\\p{RGI_Emoji}]*))?|[\\p{Not_A_Property}])','(?<=((?![\\p{RGI_Emoji}]*))?|(?<x>a)(?<x>b))','(?=((?![\\p{RGI_Emoji}]*))?|\\k<missing>)']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn positive_optional_known_assertion_consumers_clones_and_collection_keep_exported_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<=(?<outside>(?<skip>(?![\p{RGI_Emoji}]*))?)+(?<last>[a]{0})+|(?<later>b))/dgv,ok=true,offsets=[];
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[1].index===2&&matches[2].index===3&&matches[1].groups.outside===''&&matches[1].groups.skip===undefined&&matches[1].groups.last===''&&matches[1].groups.later===undefined;
        ok=ok&&'😀a'.replace(r,function(match,outside,skip,last,later,offset,input,groups){ok=ok&&outside===''&&skip===undefined&&last===''&&later===undefined&&groups.later===undefined;offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","",null,"",null,"😀","",null,"",null,"b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===''&&m[2]===undefined&&m[3]===''&&m[4]===undefined&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var after=copy.exec('😀');after.index===0&&after.groups.outside===''&&after.groups.last===''&&after.groups.later===undefined&&after.indices.groups.later===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn positive_optional_known_assertions_keep_many_slots_and_huge_bounds_without_default_quotas() {
    check(
        r#"
        var count=10,pattern='(?<=((?![\\p{RGI_Emoji}]*))?|'+Array(count).fill('(a)').join('|')+')',r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===count+2&&m[1]===undefined&&m[count+1]===undefined&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let count = 1000;
    let pattern = JsString::from(
        format!(
            r"(?<=((?![\p{{RGI_Emoji}}]*))?|{})",
            std::iter::repeat_n("(a)", count)
                .collect::<Vec<_>>()
                .join("|")
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[1]===undefined&&m[{}]===undefined&&r.lastIndex===0&&r.source==={pattern:?}",
        count + 2,
        count + 1
    ));
    let pattern = JsString::from(
        format!(
            r"(?=(?<outside>(?<skip>(?![\p{{RGI_Emoji}}]*))?){{{}}})",
            "9".repeat(20_000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===3&&m[1]===''&&m[2]===undefined&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}
