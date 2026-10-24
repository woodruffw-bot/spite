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

#[test]
fn concatenated_zero_spines_keep_interleaved_named_values_and_index_aliases() {
    check(
        r#"
        var cases=[
            ['(?<skip>[\\p{RGI_Emoji}]{0})?(?<keep>[\\p{RGI_Emoji}]{0})+(?<gone>(?=(?<inner>a))b){0}',[0,1,0,0],['skip','keep','gone','inner']],
            ['(?<left>(?<hidden>a|b){0})+(?<skip>\\p{RGI_Emoji}{0})?(?<right>\\p{RGI_Emoji}{0}){2,3}',[1,0,0,1],['left','hidden','skip','right']],
            ['(?<gone>(?<x>a)|(?<x>b)){0}(?<keep>\\p{RGI_Emoji}{0})+',[0,0,0,1],['gone','x','x','keep']],
            ['(?<before>\\p{RGI_Emoji}{0})+(?<erased>\\k<before>|(?<inside>a)\\k<after>){0}(?<after>\\p{RGI_Emoji}{0})+',[1,0,0,1],['before','erased','inside','after']]
        ],ok=true;
        for(var pair of cases)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pair[0],flags);r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
            if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            var point=global?start:0;if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===5&&m.indices.length===5&&r.lastIndex===(global?point:start)&&r.source===pair[0];
            ok=ok&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            for(var i=1;i<m.length;i++){
                var name=pair[2][i-1];ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&Object.hasOwn(m.groups,name)&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===m[i]&&m.indices.groups[name]===m.indices[i];
                ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
            }
        }
        ok
    "#,
    );
}

#[test]
fn concatenated_zero_spine_string_consumers_and_collection_preserve_every_slot() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<skip>\p{RGI_Emoji}{0})?(?<keep>\p{RGI_Emoji}{0})+(?<gone>(a|b)){0}/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===5&&m[1]===undefined&&m[2]===''&&m[3]===undefined&&m[4]===undefined&&m.indices.groups.keep===m.indices[2];
        var calls=[];var value='ab'.replace(r,function(m,skip,keep,gone,inner,at,input,groups){calls.push(at);return arguments.length===8&&m===''&&skip===undefined&&keep===''&&gone===undefined&&inner===undefined&&input==='ab'&&groups.skip===undefined&&groups.keep===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===undefined&&m[2]===''&&m[3]===undefined&&m[4]===undefined
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('x');m.groups.keep===''&&m.groups.skip===undefined&&m.groups.gone===undefined&&Object.hasOwn(m.indices,'4')&&m.indices.groups.keep===m.indices[2]"),Ok(Value::Boolean(true)));
}

#[test]
fn large_concatenated_zero_spines_and_symbolic_required_counts_have_no_default_quotas() {
    check(
        r#"
        var pattern='([\\p{RGI_Emoji}]{0})?([\\p{RGI_Emoji}]{0})+'.repeat(1000),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&r.lastIndex===0&&m.length===2001;
        for(var i=1;i<m.length;i++)ok=ok&&(i%2===0?m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0:m[i]===undefined&&m.indices[i]===undefined);
        var count='9'.repeat(20000);var huge=new RegExp('(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0}){'+count+'}(?<gone>(a|b)){0}','dgv').exec('x');
        ok&&huge.length===5&&huge[1]===undefined&&huge[2]===''&&huge[3]===undefined&&huge[4]===undefined&&huge.indices.groups.keep===huge.indices[2]
    "#,
    );
}

#[test]
fn sequence_erasure_still_requires_complete_original_early_error_validation() {
    check(
        r#"
        var ok=true;for(var source of ['(?<x>\\p{RGI_Emoji}{0})?(?<x>\\p{RGI_Emoji}{0})+','[\\p{Not_A_Property}]{0}\\p{RGI_Emoji}{0}','(\\k<missing>){0}\\p{RGI_Emoji}{0}','[a]{0}([b]{3,2}){0}']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
    for source in [
        r"new RegExp('([\\p{RGI_Emoji}])*','v').test('')",
        r"new RegExp('[\\p{RGI_Emoji}]+[\\p{RGI_Emoji}]{0}','v').test('x')",
        r"new RegExp('[\\p{RGI_Emoji}]{0}[\\p{RGI_Emoji}]+','v').test('x')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn unconditional_first_alternatives_preserve_selected_and_unselected_capture_slots() {
    check(
        r#"
        var cases=[
            ['|(?<never>[\\p{RGI_Emoji}]+)',[0],['never']],
            ['(?<keep>\\p{RGI_Emoji}{0})+|(?<never>a(?<inner>b))',[1,0,0],['keep','never','inner']],
            ['(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+|(?<never>a)(?<inner>b)',[0,1,0,0],['skip','keep','never','inner']],
            ['(?<keep>(?<erased>\\k<later>){0})+|(?<later>a)\\k<keep>',[1,0,0],['keep','erased','later']]
        ],ok=true;
        for(var pair of cases)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pair[0],flags);r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
            if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            var point=global?start:0;if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===pair[1].length+1&&r.source===pair[0]&&r.lastIndex===(global?point:start)&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            for(var i=1;i<m.length;i++){
                var name=pair[2][i-1];ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&Object.hasOwn(m.groups,name)&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===m[i]&&m.indices.groups[name]===m.indices[i];
                ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
            }
        }
        var r=/(?<same>\p{RGI_Emoji}{0})+|(?<same>[\p{RGI_Emoji}]+)/dgv,m=r.exec('abc');
        ok&&m.length===3&&m[1]===''&&m[2]===undefined&&Object.hasOwn(m,'2')&&Object.hasOwn(m.indices,'2')&&m.groups.same===''&&m.indices.groups.same===m.indices[1]&&m.indices[2]===undefined
    "#,
    );
}

#[test]
fn first_empty_alternative_string_consumers_copies_and_gc_never_select_later_arms() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<skip>\p{RGI_Emoji}{0})?(?<keep>\p{RGI_Emoji}{0})+|(?<never>a)(?<inner>b)/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===5&&m[1]===undefined&&m[2]===''&&m[3]===undefined&&m[4]===undefined&&m.indices.groups.keep===m.indices[2];
        var calls=[];var value='ab'.replace(r,function(m,skip,keep,never,inner,at,input,groups){calls.push(at);return arguments.length===8&&m===''&&skip===undefined&&keep===''&&never===undefined&&inner===undefined&&input==='ab'&&groups.never===undefined&&groups.keep===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===undefined&&m[2]===''&&m[3]===undefined&&m[4]===undefined
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('ab');m.groups.keep===''&&m.groups.never===undefined&&m.groups.inner===undefined&&Object.hasOwn(m.indices,'4')&&m.indices.groups.keep===m.indices[2]"),Ok(Value::Boolean(true)));
}

#[test]
fn deeply_unselected_alternatives_keep_all_slots_and_validate_original_early_errors() {
    check(
        r#"
        var depth=1000,pattern='(?<keep>\\p{RGI_Emoji}{0})+|'+'('.repeat(depth)+'[\\p{RGI_Emoji}]+'+')'.repeat(depth),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&r.lastIndex===0&&m.length===depth+2&&m[1]===''&&m.indices.groups.keep===m.indices[1];
        for(var i=2;i<m.length;i++)ok=ok&&m[i]===undefined&&m.indices[i]===undefined&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i);
        for(var source of ['\\p{RGI_Emoji}{0}|[\\p{Not_A_Property}]','|(?<x>a)(?<x>b)','|(\\k<missing>)','[a]{0}|([b]{3,2})']){
            var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught;
        }ok
    "#,
    );
}

#[test]
fn mandatory_enclosing_sequence_and_choice_captures_keep_full_named_layouts() {
    check(
        r#"
        var cases=[
            ['(?<outside>(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+)',[1,0,1],['outside','skip','keep']],
            ['(?<outside>|(?<never>[\\p{RGI_Emoji}]+))',[1,0],['outside','never']],
            ['(?<outside>(?<keep>\\p{RGI_Emoji}{0})+|(?<never>(?<inner>a)))',[1,1,0,0],['outside','keep','never','inner']],
            ['(?<outside>(?<keep>(?<erased>\\k<outside>|\\k<later>){0})+|(?<later>[\\p{RGI_Emoji}]+))',[1,1,0,0],['outside','keep','erased','later']]
        ],ok=true;
        for(var pair of cases)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pair[0],flags);r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
            if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            var point=global?start:0;if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===pair[1].length+1&&r.source===pair[0]&&r.lastIndex===(global?point:start)&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            for(var i=1;i<m.length;i++){
                var name=pair[2][i-1];ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&Object.hasOwn(m.groups,name)&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===m[i]&&m.indices.groups[name]===m.indices[i];
                ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
            }
        }
        var m=/(?<outside>(?<same>\p{RGI_Emoji}{0})+|(?<same>[\p{RGI_Emoji}]+))/dgv.exec('ab');
        ok&&m.length===4&&m[1]===''&&m[2]===''&&m[3]===undefined&&m.groups.same===''&&m.indices.groups.outside===m.indices[1]&&m.indices.groups.same===m.indices[2]&&Object.hasOwn(m.indices,'3')&&m.indices[3]===undefined
    "#,
    );
}

#[test]
fn enclosing_empty_masks_preserve_string_consumer_slots_and_collection() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skip>\p{RGI_Emoji}{0})?(?<keep>\p{RGI_Emoji}{0})+)/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===''&&m.indices.groups.outside===m.indices[1]&&m.indices.groups.keep===m.indices[3]&&m.indices[1]!==m.indices[3];
        var calls=[];var value='ab'.replace(r,function(m,outside,skip,keep,at,input,groups){calls.push(at);return arguments.length===7&&m===''&&outside===''&&skip===undefined&&keep===''&&input==='ab'&&groups.outside===''&&groups.keep===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===''&&m[2]===undefined&&m[3]===''
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('ab');m.groups.outside===''&&m.groups.skip===undefined&&m.groups.keep===''&&Object.hasOwn(m.indices,'2')&&m.indices.groups.keep===m.indices[3]"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_mandatory_enclosing_masks_keep_defined_prefixes_and_every_inner_slot() {
    check(
        r#"
        var depth=1000,pattern='('.repeat(depth)+'([\\p{RGI_Emoji}]{0})?([\\p{RGI_Emoji}]{0})+'+')'.repeat(depth),r=new RegExp(pattern,'dgv');r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&r.lastIndex===0&&m.length===depth+3;
        for(var i=1;i<=depth;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        ok=ok&&m[depth+1]===undefined&&m.indices[depth+1]===undefined&&m[depth+2]===''&&m.indices[depth+2][0]===0;
        var choice='('.repeat(depth)+'|(?<never>(?=(a))[\\p{RGI_Emoji}]+)'+')'.repeat(depth),m=new RegExp(choice,'dgv').exec('ab');
        ok=ok&&m.length===depth+3&&m.groups.never===undefined&&m[depth+1]===undefined&&m[depth+2]===undefined&&Object.hasOwn(m.indices,''+(depth+2));
        for(var i=1;i<=depth;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        ok
    "#,
    );
}

#[test]
fn quantified_enclosing_masks_preserve_required_captures_and_roll_back_optional_ones() {
    check(
        r#"
        var cases=[
            ['(?<outside>(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+)+',[1,0,1],['outside','skip','keep']],
            ['(?<outside>(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+)*',[0,0,0],['outside','skip','keep']],
            ['(?i:(?<outside>(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+)){2,3}',[1,0,1],['outside','skip','keep']],
            ['(?i:(?<outside>(?<skip>\\p{RGI_Emoji}{0})?(?<keep>\\p{RGI_Emoji}{0})+)){0,2}',[0,0,0],['outside','skip','keep']],
            ['(?<outside>(?<keep>\\p{RGI_Emoji}{0})+|(?<never>a){0}){2,3}?',[1,1,0],['outside','keep','never']],
            ['(?-i:(?<outside>|(?<never>[\\p{RGI_Emoji}]+){0}))+',[1,0],['outside','never']]
        ],ok=true;
        for(var pair of cases)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pair[0],flags);r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
            if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            var point=global?start:0;if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===pair[1].length+1&&r.source===pair[0]&&r.lastIndex===(global?point:start)&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            for(var i=1;i<m.length;i++){
                var name=pair[2][i-1];ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&Object.hasOwn(m.groups,name)&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===m[i]&&m.indices.groups[name]===m.indices[i];
                ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
            }
        }
        var m=/(?i:(?<outside>(?<same>\p{RGI_Emoji}{0})+|(?<same>[\p{RGI_Emoji}]+){0})){2,3}/dgv.exec('ab');
        ok&&m.length===4&&m[1]===''&&m[2]===''&&m[3]===undefined&&m.groups.same===''&&m.indices.groups.outside===m.indices[1]&&m.indices.groups.same===m.indices[2]&&Object.hasOwn(m.indices,'3')&&m.indices[3]===undefined
    "#,
    );
}

#[test]
fn quantified_enclosing_empty_masks_keep_string_consumers_and_collection_consistent() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>(?<skip>\p{RGI_Emoji}{0})?(?<keep>\p{RGI_Emoji}{0})+)+/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===''&&m.indices.groups.outside===m.indices[1]&&m.indices.groups.keep===m.indices[3];
        var calls=[];var value='ab'.replace(r,function(m,outside,skip,keep,at,input,groups){calls.push(at);return arguments.length===7&&m===''&&outside===''&&skip===undefined&&keep===''&&input==='ab'&&groups.outside===''&&groups.keep===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===''&&m[2]===undefined&&m[3]===''
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('ab');m.groups.outside===''&&m.groups.skip===undefined&&m.groups.keep===''&&Object.hasOwn(m.indices,'2')&&m.indices.groups.keep===m.indices[3]"),Ok(Value::Boolean(true)));
}

#[test]
fn quantified_enclosing_depth_and_required_counts_have_no_default_quotas() {
    check(
        r#"
        var depth=8,pattern='('.repeat(depth)+'|(?<never>[\\p{RGI_Emoji}]+){0}'+'){2,3}'.repeat(depth),m=new RegExp(pattern,'dgv').exec('ab'),ok=m.length===depth+2&&m.groups.never===undefined;
        for(var i=1;i<=depth;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        ok&&m[depth+1]===undefined&&Object.hasOwn(m.indices,''+(depth+1))
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}([\p{{RGI_Emoji}}]{{0}})?([\p{{RGI_Emoji}}]{{0}})+{}",
            "(".repeat(depth),
            ")+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&r.lastIndex===0&&m.length===1003;for(var i=1;i<=1000;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;ok&&m[1001]===undefined&&m.indices[1001]===undefined&&m[1002]===''&&m.indices[1002][0]===0"
    ));
    let count = "9".repeat(20_000);
    let pattern = JsString::from(
        format!(
            r"(?<outside>(?<skip>\p{{RGI_Emoji}}{{0}})?(?<keep>\p{{RGI_Emoji}}{{0}})+){{{count}}}"
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dgv').exec('x');m.length===4&&m[1]===''&&m[2]===undefined&&m[3]===''&&m.groups.outside===''&&m.groups.skip===undefined&&m.groups.keep===''&&m.indices.groups.keep===m.indices[3]"
    ));
}

#[test]
fn quantified_enclosing_erasure_preserves_original_early_errors_and_host_aborts() {
    check(
        r#"
        var invalid=['(?i:(?<x>[\\p{Not_A_Property}]{0})?[a]{0})+','(?:(?<x>a)(?<x>b)|){0,2}','(?:(\\k<missing>)|){2,3}'],ok=true;
        for(var source of invalid){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}ok
    "#,
    );
    for source in [
        "new RegExp('(|a){2,3}').exec('a')",
        r"new RegExp('(?-i:(?<outside>|(?<never>[\\p{RGI_Emoji}]+)))+','v').exec('😀')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let count = "9".repeat(20_000);
    let pattern = JsString::from(
        format!(r"(([\p{{RGI_Emoji}}]{{0}})?([\p{{RGI_Emoji}}]{{0}})+){{{count}}}").as_str(),
    );
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
fn arbitrary_empty_trees_keep_named_branch_masks_and_original_erased_bindings() {
    check(
        r#"
        var cases=[
            ['(?<outer>(?<left>a{0})?|(?<right>b{0})+)+(?<tail>[\\p{RGI_Emoji}]{0})',[1,0,0,1],['outer','left','right','tail']],
            ['(?i:(?<outer>(?<left>a{0})|(?<right>b{0})))*(?<tail>\\p{RGI_Emoji}{0})',[0,0,0,1],['outer','left','right','tail']],
            ['(?<first>(?<erased>(?=(?<hidden>a))\\k<later>){0})(?<later>c{0}|d{0})',[1,0,0,1],['first','erased','hidden','later']],
            ['(?<outer>😀{0}|(?<never>[\\p{RGI_Emoji}]+){0})(?<tail>.\\d){0}',[1,0,0],['outer','never','tail']]
        ],ok=true;
        for(var pair of cases)for(var flags of ['dv','dgv','dgyv','dgimsv'])for(var input of ['','ab','A😀!','a\r\nb'])for(var start=0;start<=input.length+1;start++){
            var r=new RegExp(pair[0],flags);r.lastIndex=start;var m=r.exec(input),global=r.global||r.sticky;
            if(global&&start>input.length){ok=ok&&m===null&&r.lastIndex===0;continue}
            var point=global?start:0;if(point>0&&input.charCodeAt(point)>=0xdc00&&input.charCodeAt(point)<=0xdfff&&input.charCodeAt(point-1)>=0xd800&&input.charCodeAt(point-1)<=0xdbff)point--;
            ok=ok&&m.index===point&&m[0]===''&&m.length===pair[1].length+1&&r.source===pair[0]&&r.lastIndex===(global?point:start)&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null;
            for(var i=1;i<m.length;i++){
                var name=pair[2][i-1];ok=ok&&Object.hasOwn(m,''+i)&&Object.hasOwn(m.indices,''+i)&&Object.hasOwn(m.groups,name)&&Object.hasOwn(m.indices.groups,name)&&m.groups[name]===m[i]&&m.indices.groups[name]===m.indices[i];
                ok=ok&&(pair[1][i-1]?m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point:m[i]===undefined&&m.indices[i]===undefined);
            }
        }
        var m=/(?<outer>(?<same>a{0})|(?<same>b{0}))+/dgv.exec('ab');
        ok&&m.length===4&&m[1]===''&&m[2]===''&&m[3]===undefined&&m.groups.same===''&&m.indices.groups.outer===m.indices[1]&&m.indices.groups.same===m.indices[2]&&Object.hasOwn(m.indices,'3')&&m.indices[3]===undefined
    "#,
    );
}

#[test]
fn empty_tree_string_consumers_copies_and_gc_keep_branch_and_erasure_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outer>(?<left>a{0})?|(?<right>b{0})+)+(?<tail>[\p{RGI_Emoji}]{0})/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===5&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m[4]===''&&m.indices.groups.outer===m.indices[1]&&m.indices.groups.tail===m.indices[4]&&m.indices[1]!==m.indices[4];
        var calls=[];var value='ab'.replace(r,function(m,outer,left,right,tail,at,input,groups){calls.push(at);return arguments.length===8&&m===''&&outer===''&&left===undefined&&right===undefined&&tail===''&&input==='ab'&&groups.outer===''&&groups.tail===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,,,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&m.index===1&&copy.source===r.source&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&m[4]===''
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('ab');m.groups.outer===''&&m.groups.left===undefined&&m.groups.right===undefined&&m.groups.tail===''&&Object.hasOwn(m.indices,'2')&&m.indices.groups.tail===m.indices[4]"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_empty_trees_and_required_counts_have_no_default_quotas() {
    check(
        r#"
        var depth=5,pattern='('.repeat(depth)+'(?<leaf>[\\p{RGI_Emoji}]{0})'+'|(a){0})+'.repeat(depth),m=new RegExp(pattern,'dgv').exec('ab'),ok=m.length===2*depth+2&&m.groups.leaf==='';
        for(var i=1;i<=depth+1;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        for(var i=depth+2;i<m.length;i++)ok=ok&&m[i]===undefined&&m.indices[i]===undefined;
        ok
    "#,
    );
    let depth = 1000;
    let pattern = JsString::from(
        format!(
            r"{}(?<leaf>[\p{{RGI_Emoji}}]{{0}}){}",
            "(".repeat(depth),
            "|(a){0})+".repeat(depth)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dgv');r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&r.lastIndex===0&&m.length===2002&&m.groups.leaf==='';for(var i=1;i<=1001;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;for(var i=1002;i<m.length;i++)ok=ok&&m[i]===undefined&&m.indices[i]===undefined;ok"
    ));
    let count = "9".repeat(20_000);
    let pattern = JsString::from(
        format!(r"(?<outer>(?<a>a{{0}})(?<b>b{{0}})|(?<later>[\p{{RGI_Emoji}}]{{0}})){{{count}}}")
            .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dgv').exec('x');m.length===5&&m[1]===''&&m[2]===''&&m[3]===''&&m[4]===undefined&&m.groups.outer===''&&m.groups.a===''&&m.groups.b===''&&m.groups.later===undefined&&m.indices.groups.b===m.indices[3]"
    ));
}

#[test]
fn tree_erasure_keeps_original_early_errors_and_opt_in_host_aborts() {
    check(
        r#"
        var invalid=['(?<x>[\\P{RGI_Emoji}]{0})a{0}','(?<same>a{0})(?<same>b{0})','(?:(\\k<missing>)){0}a{0}','(?:(a){3,2}){0}b{0}'],ok=true;
        for(var source of invalid){var caught=false;try{new RegExp(source,'v')}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}ok
    "#,
    );
    let count = "9".repeat(20_000);
    let pattern =
        JsString::from(format!(r"((a{{0}})(b{{0}})|([\p{{RGI_Emoji}}]{{0}})){{{count}}}").as_str());
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
