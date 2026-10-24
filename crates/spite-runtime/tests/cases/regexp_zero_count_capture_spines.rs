//! Required and skipped empty iterations preserve original lexical slots.

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
fn quantified_zero_spines_keep_defined_prefixes_and_undefined_suffixes() {
    check(
        r#"
        var cases=[
            ['(?<outer>(?<inner>[\\p{RGI_Emoji}]{0})?)+',[1,0]],
            ['(?<outer>(?<inner>\\p{RGI_Emoji}{0})+)?',[0,0]],
            ['(?i:(?<outer>(?<inner>[\\p{RGI_Emoji}]){0})+)',[1,0]],
            ['(?<outer>(?<inner>\\p{RGI_Emoji}{0})?){2,3}',[1,0]],
            ['(?<outer>(?<inner>[\\p{RGI_Emoji}]{0}){1,2})+?',[1,1]],
            ['(?-i:(?<outer>(?<inner>\\p{RGI_Emoji}){0}){2,3}?)',[1,0]]
        ],ok=true;
        for(var pair of cases)for(var extras of ['d','dg','dgy','dgimsv']){
            var flags=extras.includes('v')?extras:extras+'v',r=new RegExp(pair[0],flags);
            for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
                r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
                if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
                var point=global?start:0;
                if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
                ok=ok&&m.index===point&&m[0]===''&&m.length===3&&m.indices.length===3&&r.lastIndex===(global?point:start)&&r.source===pair[0];
                for(var i=1;i<m.length;i++){
                    ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i);
                    ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
                }
                ok=ok&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null&&Object.hasOwn(m.groups,'inner')&&Object.hasOwn(m.indices.groups,'inner')&&m.groups.outer===m[1]&&m.groups.inner===m[2]&&m.indices.groups.outer===m.indices[1]&&m.indices.groups.inner===m.indices[2];
            }
        }
        ok
    "#,
    );
}

#[test]
fn quantified_zero_spine_consumers_clones_and_gc_keep_both_capture_states() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outer>(?<inner>\p{RGI_Emoji}{0})?)+/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m[1]===''&&m[2]===undefined&&m.groups.outer===''&&m.groups.inner===undefined&&m.indices.groups.outer===m.indices[1]&&m.indices.groups.inner===undefined;
        var calls=[];var value='ab'.replace(r,function(m,outer,inner,at,input,groups){calls.push(at);return arguments.length===6&&m===''&&outer===''&&inner===undefined&&input==='ab'&&groups.outer===''&&groups.inner===undefined?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&m[1]===''&&m[2]===undefined&&copy.lastIndex===1&&copy.source===r.source
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('x');m.groups.outer===''&&m.groups.inner===undefined&&Object.hasOwn(m.indices,'2')"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_and_unrepresentable_required_counts_keep_capture_storage_flat() {
    check(
        r#"
        var depth=1000,pattern='('.repeat(depth)+'[\\p{RGI_Emoji}]{0}'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&m.length===1001&&r.lastIndex===0;
        for(var i=1;i<m.length;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        ok
    "#,
    );
    let count = "9".repeat(20_000);
    let pattern =
        JsString::from(format!(r"(?<outer>(?<inner>\p{{RGI_Emoji}}{{0}})?){{{count}}}").as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv'),m=r.exec('x');m.length===3&&m[1]===''&&m[2]===undefined&&m.groups.outer===''&&m.groups.inner===undefined&&m.indices.groups.outer===m.indices[1]&&m.indices.groups.inner===undefined"
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
}

#[test]
fn syntax_errors_and_unerased_membership_requirements_keep_distinct_failures() {
    check(
        r#"
        var invalid=['(?i:(?<x>[^\\p{RGI_Emoji}]){0})+','(?i:(\\P{RGI_Emoji})){0}','(([a]){2,1}){0}'],ok=true;
        for(var source of invalid){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        ok
    "#,
    );
    for source in [
        r"(?i:([\p{RGI_Emoji}])+)",
        r"([\p{RGI_Emoji}])*",
        r"([\p{RGI_Emoji}]{0})a",
    ] {
        let source = JsString::from(source);
        assert!(
            matches!(
                Realm::default().eval(&format!("new RegExp({source:?},'v').test('')")),
                Err(Error::Unsupported { .. })
            ),
            "{source:?}"
        );
    }
}

#[test]
fn arbitrary_erased_bodies_reserve_duplicate_names_references_and_lookaround_slots() {
    check(
        r#"
        var cases=[
            ['(?<outer>(?<erased>(?<x>a)|(?<x>b)(?<y>(?<=c))){0})+',[1,0,0,0,0]],
            ['(?i:(?<outer>(?<erased>(?=(?<x>a))\\k<outer>|[\\p{RGI_Emoji}]+){0}){2,3})',[1,0,0]],
            ['(?<outer>(?<erased>(a)(?<=b(c))|\\p{RGI_Emoji}+){0})?',[0,0,0,0]],
            ['(?-i:(?<outer>(?<erased>[\\q{\\(|\\)}]z?){0})+?)',[1,0]]
        ],ok=true;
        for(var pair of cases){var r=new RegExp(pair[0],'dgvy');r.lastIndex=2;var m=r.exec('A😀!');
            ok=ok&&m.index===1&&m[0]===''&&m.length===pair[1].length+1&&r.lastIndex===1&&r.source===pair[0];
            for(var i=1;i<m.length;i++)ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===1&&m.indices[i][1]===1:m[i]===undefined&&m.indices[i]===undefined);
            ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.groups.outer===m[1]&&m.groups.erased===undefined&&Object.hasOwn(m.groups,'erased')&&Object.hasOwn(m.indices.groups,'erased')&&m.indices.groups.erased===undefined;
            if(pair[1][0])ok=ok&&m.indices.groups.outer===m.indices[1];
            if(Object.hasOwn(m.groups,'x'))ok=ok&&m.groups.x===undefined&&Object.hasOwn(m.indices.groups,'x')&&m.indices.groups.x===undefined;
        }
        ok
    "#,
    );
}

#[test]
fn arbitrary_zero_group_consumers_and_collection_keep_erased_captures_undefined() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outer>(?<erased>(?<x>a)|\p{RGI_Emoji}+){0})+/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m.groups.outer===''&&m.groups.erased===undefined&&m.groups.x===undefined;
        var calls=[];var value='ab'.replace(r,function(m,outer,erased,x,at,input,groups){calls.push(at);return arguments.length===7&&m===''&&outer===''&&erased===undefined&&x===undefined&&input==='ab'&&groups.outer===''&&groups.x===undefined?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===''&&m[2]===undefined&&m[3]===undefined
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('x');m.groups.outer===''&&m.groups.erased===undefined&&m.groups.x===undefined&&Object.hasOwn(m.indices,'3')"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_arbitrary_erased_groups_keep_full_layout_with_huge_required_outside_counts() {
    check(
        r#"
        var depth=1000,pattern='('.repeat(depth)+'('.repeat(depth)+'a|(?=(b))[\\p{RGI_Emoji}]+'+')'.repeat(depth)+'{0}'+')+'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&m.length===2002&&r.lastIndex===0;
        for(var i=1;i<m.length;i++)ok=ok&&(i<=depth?m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0:m[i]===undefined&&m.indices[i]===undefined);
        ok
    "#,
    );
    let count = "9".repeat(20_000);
    let pattern = JsString::from(
        format!(r"(?<outer>(?<erased>(a)\p{{RGI_Emoji}}+){{0}}){{{count}}}").as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dgv').exec('x');m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m.groups.outer===''&&m.groups.erased===undefined&&m.indices.groups.outer===m.indices[1]"
    ));
}
