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
