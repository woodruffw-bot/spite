//! Single-character negative Unicode assertions retain undefined operand slots.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn negative_assertion_named_outside_and_undefined_operand_slots_retain_source() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var atom of ['a','[a-z]','\\d','.','😀','\\uD800']){
            var source='(?<outside>(?!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=1;var m=r.exec('');
            if(state!==''){ok=ok&&m===null&&r.lastIndex===0;r.lastIndex=0;m=r.exec('')}
            ok=ok&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m[1]===''&&m[2]===undefined&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run===undefined&&m.indices[2]===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.source===source&&r.lastIndex===(state===''?1:0);
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_assertion_search_sticky_end_and_unicode_pair_candidates_remain_exact() {
    check(
        "\n        var ok=true,input='😀a';\n        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var start=0;start<=input.length+1;start++){\n            var r=new RegExp('(?!(?<run>😀))','d'+state+mode);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;\n            if(point===1)point=0;\n            if(point>input.length||r.sticky&&point===0){ok=ok&&m===null&&r.lastIndex===0;continue}\n            if(point===0)point=2;\n            ok=ok&&m.index===point&&m[0]===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===(active?point:start);\n            var r=new RegExp('(?!(\\\\uDE00))','dgy'+mode);r.lastIndex=1;var m=r.exec('😀');ok=ok&&m.index===0&&m[1]===undefined&&r.lastIndex===0;\n        }ok\n    ",
    );
}

#[test]
fn negative_assertion_line_dot_property_fold_and_lone_surrogate_context_remain_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?!(?<run>^a$))','dimy'+mode);r.lastIndex=1;ok=ok&&r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('!A!');ok=ok&&m.index===1&&m.groups.run===undefined;
            var m=new RegExp('(?!(.))','d'+mode).exec('a\nb');ok=ok&&m.index===1&&m[1]===undefined;
            var m=new RegExp('(?!(.))','ds'+mode).exec('a\nb');ok=ok&&m.index===3&&m[1]===undefined;
            var m=new RegExp('(?!(\\p{ASCII}))','di'+mode).exec('ſK!');ok=ok&&m.index===3&&m[1]===undefined;
            var m=new RegExp('(?!(\\uD800))','d'+mode).exec('\ud800x\udc00');ok=ok&&m.index===1&&m[1]===undefined;
            var m=new RegExp('(?!(\\uD800))','d'+mode).exec('\ud800\udc00');ok=ok&&m.index===0&&m[1]===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_assertion_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<outside>(?!(?<run>a)))',r=new RegExp(source,'dg'+mode),all=Array.from('aba'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===1&&all[1].index===3&&all[0].groups.run===undefined&&all[0].groups.outside===''&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
            ok=ok&&'aba'.replace(r,'<$<run>>')==='a<>ba<>'&&'aba'.replaceAll(r,'<$<run>>')==='a<>ba<>'&&JSON.stringify('aba'.split(new RegExp('(?!(a))',mode)))==='["a",null,"ba"]';
            var offsets=[];ok=ok&&'aba'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&match===''&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='a!ba!'&&JSON.stringify(offsets)==='[1,3]';
            r.lastIndex=2;ok=ok&&'aba'.search(r)===1&&r.lastIndex===2;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('b').groups.run===undefined;
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('b');m.index===0&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_assertion_deep_wrappers_keep_all_original_capture_slots() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='('.repeat(1000)+'(?!'+'('.repeat(1000)+'[a-z]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'d'+mode),m=r.exec('!a');
            ok=ok&&m.index===0&&m.length===2001&&m[0]==='';
            for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':undefined)&&(i<=1000?m.indices[i][0]===0&&m.indices[i][1]===0:m.indices[i]===undefined);
        }ok
    "#,
    );
}

#[test]
fn negative_assertion_preserves_coercion_lastindex_errors_and_original_validation() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var events=[],r=new RegExp('(?!(?<run>a))','dgy'+mode);r.lastIndex={valueOf(){events.push('index');return 1}};
            var m=r.exec({toString(){events.push('input');return 'ab'}});ok=ok&&events.join(',')==='input,index'&&m.index===1&&m.groups.run===undefined&&r.lastIndex===1;
            Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('ab')}catch(e){caught=e instanceof TypeError}ok=ok&&caught;
            for(var source of ['(?!(a{2,1}))','(?!(?<x>a))\\k<missing>','(?!(\\p{Not_A_Property}))']){var caught=false;try{new RegExp(source,mode)}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        }ok
    "#,
    );
}

#[test]
fn negative_assertion_opted_in_construction_abort_does_not_publish_partial_state() {
    let pattern =
        spite_core::JsString::from(format!("(?!(?<run>[{}]))", "a".repeat(20_000)).as_str());
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'dgu')}}catch(e){{marker=1}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
}
