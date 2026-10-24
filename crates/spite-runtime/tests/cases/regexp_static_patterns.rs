//! Composed empty/impossible Pattern facts preserve every original lexical slot.

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
fn compositional_choices_select_first_successful_empty_arm_with_original_duplicate_names() {
    check(
        r#"
        var rows=[
            ['(?<same>(?![\\p{RGI_Emoji}]*))|(?<same>[\\p{RGI_Emoji}]{0})',[false,true]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))|(?<empty>[\\p{RGI_Emoji}]{0}))+',[true,false,true]],
            ['(?<outside>(?<skip>(?![\\p{RGI_Emoji}]*))|(?<empty>[\\p{RGI_Emoji}]{0}))*',[false,false,false]],
            ['(?<=(?<same>(?![\\p{RGI_Emoji}]*))|(?<same>[\\p{RGI_Emoji}]{0}))',[false,true]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(row[0],flags);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;
            if(active&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            if(point>0&&point<input.length&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===row[1].length+1&&r.source===row[0]&&r.lastIndex===(active?point:start);
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===point&&m.indices[i+1][1]===point:m[i+1]===undefined&&m.indices[i+1]===undefined);
            if('same'in m.groups)ok=ok&&m.groups.same===''&&m.indices.groups.same[0]===point;
        }ok
    "#,
    );
}

#[test]
fn compositional_impossible_arms_and_nested_assertions_preserve_source_flags_and_last_index() {
    check(
        r#"
        var patterns=[
            '(?<outside>(?!(?<slot>[\\p{RGI_Emoji}]*))|(?<!(?<zero>[\\p{RGI_Emoji}]{0})))+',
            '(?!(?<=(?<skip>(?![\\p{RGI_Emoji}]*))|(?<empty>[\\p{RGI_Emoji}]{0})))',
            '(?im-s:(?=(?<outside>(?!(?<slot>[\\p{RGI_Emoji}]*))|(?<!(?<zero>[\\p{RGI_Emoji}]{0}))))){2,3}'
        ],ok=true;
        for(var pattern of patterns)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pattern,flags);r.lastIndex=start;ok=ok&&r.exec(input)===null&&r.source===pattern&&r.hasIndices&&r.unicodeSets&&r.lastIndex===(r.global||r.sticky?0:start);
        }ok
    "#,
    );
}

#[test]
fn compositional_string_consumers_copies_and_collection_keep_first_successful_arm_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<same>(?![\p{RGI_Emoji}]*))|(?<same>[\p{RGI_Emoji}]{0})/dgv,ok=true,offsets=[];
        var matches=Array.from('😀a'.matchAll(r));ok=ok&&matches.length===3&&matches[1].index===2&&matches[2].index===3&&matches[1][1]===undefined&&matches[1][2]===''&&matches[1].groups.same==='';
        ok=ok&&'😀a'.replace(r,function(match,failed,empty,offset,input,groups){ok=ok&&failed===undefined&&empty===''&&groups.same==='';offsets.push(offset);return '!';})==='!😀!a!'&&JSON.stringify(offsets)==='[0,2,3]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a",null,"","😀",null,"","b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m[1]===undefined&&m[2]===''&&m.groups.same===''&&m.indices.groups.same[0]===0&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('😀');m.index===0&&m.groups.same===''&&m[1]===undefined&&m[2]===''&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn compositional_depth_and_symbolic_counts_keep_masks_without_default_quotas() {
    check(
        r#"
        var depth=10,body='(?<skip>(?![\\p{RGI_Emoji}]*))|(?<empty>[\\p{RGI_Emoji}]{0})',pattern='('.repeat(depth)+body+')+'.repeat(depth),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+3&&m[depth]===''&&m[depth+1]===undefined&&m[depth+2]===''&&m.groups.empty===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let body = r"(?<skip>(?![\p{RGI_Emoji}]*))|(?<empty>[\p{RGI_Emoji}]{0})";
    let pattern =
        JsString::from(format!("{}{body}{}", "(".repeat(depth), ")+".repeat(depth)).as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[{}]===''&&m[{}]===undefined&&m[{}]===''&&m.groups.empty===''&&r.lastIndex===0&&r.source==={pattern:?}",
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
fn compositional_proofs_preserve_original_errors_coercions_and_opt_in_abort_state() {
    check(
        r#"
        var invalid=['(?<skip>(?!a*))|[\\p{Not_A_Property}]{0}','(?<skip>(?!a*))|(?<x>a)(?<x>b){0}','(?<skip>(?!a*))|\\k<missing>{0}'],ok=true;
        for(var source of invalid){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        var events=[],r=/(?<skip>(?![\p{RGI_Emoji}]*))|(?<empty>[\p{RGI_Emoji}]{0})/dgyv;r.lastIndex={valueOf(){events.push('index');return 2}};
        var m=r.exec({toString(){events.push('input');return 'a😀'}});ok&&m.index===1&&m.groups.empty===''&&m.groups.skip===undefined&&r.lastIndex===1&&events.join(',')==='input,index'
    "#,
    );
    let body = r"(?<skip>(?![\p{RGI_Emoji}]*))|(?<empty>[\p{RGI_Emoji}]{0})";
    let pattern = JsString::from(format!("({body}){{{}}}", "9".repeat(20_000)).as_str());
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'dgv')}}catch(e){{marker=1}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
}

#[test]
fn capture_free_positive_operands_compose_as_empty_without_exporting_consumed_data() {
    check(
        r#"
        var rows=[
            ['(?=[\\p{RGI_Emoji}]*)',[]],
            ['(?<outside>(?=[\\p{RGI_Emoji}]*))+',[true]],
            ['(?<outside>(?<=[\\q{a|ab}]*))?',[false]],
            ['(?im-s:(?<outside>(?=a*b*))+(?<=[\\p{RGI_Emoji}]*))',[true]],
            ['(?<outside>(?=(?<=[\\p{RGI_Emoji}]*)))+',[true]]
        ],ok=true;
        for(var row of rows)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(row[0],flags);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;
            if(active&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            if(point>0&&point<input.length&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===row[1].length+1&&r.source===row[0]&&r.lastIndex===(active?point:start);
            for(var i=0;i<row[1].length;i++)ok=ok&&(row[1][i]?m[i+1]===''&&m.indices[i+1][0]===point&&m.indices[i+1][1]===point:m[i+1]===undefined&&m.indices[i+1]===undefined);
            if(row[1].length===0)ok=ok&&m.groups===undefined&&m.indices.groups===undefined;
            else ok=ok&&(row[1][0]?m.groups.outside===''&&m.indices.groups.outside[0]===point:m.groups.outside===undefined&&m.indices.groups.outside===undefined);
        }ok
    "#,
    );
}

#[test]
fn capture_free_positive_extension_preserves_capturing_assertions_and_original_errors() {
    check(
        r#"
        var consuming=/(?=(?<value>a*))/d,m=consuming.exec('aaa'),zero=/(?=(?<empty>[\p{RGI_Emoji}]{0}))/dv,z=zero.exec('😀');
        var ok=m[0]===''&&m.groups.value==='aaa'&&m.indices.groups.value[0]===0&&m.indices.groups.value[1]===3&&z[0]===''&&z.groups.empty===''&&z.indices.groups.empty[0]===0&&z.indices.groups.empty[1]===0;
        for(var source of ['(?=[\\p{Not_A_Property}]*)','(?=(?<x>a)(?<x>b)*)','(?=\\k<missing>*)']){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        ok
    "#,
    );
}

#[test]
fn capture_free_positive_consumers_clones_and_collection_keep_only_outside_empty_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?=[\p{RGI_Emoji}]*))+/dgv,ok=true,offsets=[];
        var matches=Array.from('a😀b'.matchAll(r));ok=ok&&matches.length===4&&matches[1].index===1&&matches[2].index===3&&matches[3].index===4&&matches[1][1]===''&&matches[1].groups.outside==='';
        ok=ok&&'a😀b'.replace(r,function(match,outside,offset,input,groups){ok=ok&&outside===''&&groups.outside==='';offsets.push(offset);return '!';})==='!a!😀!b!'&&JSON.stringify(offsets)==='[0,1,3,4]';
        ok=ok&&JSON.stringify('a😀b'.split(r))==='["a","","😀","","b"]'&&''.split(r).length===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('😀');ok&&m.index===0&&m.length===2&&m[1]===''&&m.groups.outside===''&&m.indices.groups.outside[0]===0&&copy.lastIndex===0&&copy.source===r.source&&copy.flags===r.flags
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('😀');m.index===0&&m.length===2&&m.groups.outside===''&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn capture_free_positive_depth_and_symbolic_minima_keep_flat_masks_without_default_quotas() {
    check(
        r#"
        var depth=10,pattern='('.repeat(depth)+'(?='.repeat(depth)+'[\\p{RGI_Emoji}]*'+')'.repeat(depth)+')+'.repeat(depth),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===depth+1&&m[1]===''&&m[depth]===''&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}{}[\p{{RGI_Emoji}}]*{}{}",
            "(".repeat(depth),
            "(?=".repeat(depth),
            ")".repeat(depth),
            ")+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length==={}&&m[1]===''&&m[{}]===''&&r.lastIndex===0&&r.source==={pattern:?}",
        depth + 1,
        depth
    ));
    let pattern = JsString::from(
        format!(
            r"(?<outside>(?=[\p{{RGI_Emoji}}]*)){{{}}}",
            "9".repeat(20_000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀');m.index===0&&m.length===2&&m[1]===''&&m.groups.outside===''&&r.lastIndex===0&&r.source==={pattern:?}"
    ));
}
