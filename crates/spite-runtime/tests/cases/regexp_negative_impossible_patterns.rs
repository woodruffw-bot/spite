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
