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

#[test]
fn negative_word_assertion_keeps_named_undefined_slots_and_original_predicate_polarity() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['\\ba\\b','aa'],['\\ba\\B','a!'],['\\Ba\\B','aaa!'],['\\Ba\\b','aa!']]){
            var source='(?<outside>(?!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode),m=r.exec(row[1]);
            ok=ok&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===0&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_assertion_line_fold_property_and_backspace_context_remain_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?!(?<run>^\\ba\\b$))','dimy'+mode);r.lastIndex=1;ok=ok&&r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('!A!');ok=ok&&m.index===1&&m.groups.run===undefined;
            var r=new RegExp('(?!(\\bſ\\b))','diy'+mode);r.lastIndex=1;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
            var r=new RegExp('(?!(\\b\\p{ASCII}\\b))','diy'+mode);r.lastIndex=1;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
            var r=new RegExp('(?!(\\B[\\b]\\B))','dy'+mode);ok=ok&&r.exec('\b')===null;r.lastIndex=1;var m=r.exec('a\bb');ok=ok&&m.index===1&&m[1]===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_assertion_string_consumers_copies_and_collection_retain_undefined_values() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<outside>(?!(?<run>\\ba\\b)))',r=new RegExp(source,'dg'+mode),all=Array.from('a!'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===1&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
            ok=ok&&'a!'.replace(r,'<$<run>>')==='a<>!<>'&&JSON.stringify('a!'.split(new RegExp('(?!(\\ba\\b))',mode)))==='["a",null,"!"]';
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===1;
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('a!');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_assertion_search_retains_complete_characters_and_initial_pair_rewind() {
    // Independent specification expectation: V8 may select an interior pair.
    check(
        "var ok=true;for(var mode of ['u','v']){var r=new RegExp('(?!(?<run>\\\\B😀\\\\B))','dg'+mode),m=r.exec('😀😀');ok=ok&&m.index===4&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===4;r.lastIndex=1;var m=r.exec('😀😀');ok=ok&&m.index===4;r=new RegExp('(?!(?<run>\\\\B😀\\\\B))','dgy'+mode);r.lastIndex=1;ok=ok&&r.exec('😀😀')===null&&r.lastIndex===0}ok",
    );
}

#[test]
fn negative_q_class_assertion_keeps_original_source_names_and_undefined_operand_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[\\q{a|b}]','[[\\q{a|b}]--[x]]','[[\\q{a|b}]&&[ab]]','[x\\q{a|b}]']){
            var source='(?<outside>(?!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=2;var m=r.exec('ab!');
            ok=ok&&m.index===2&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===2&&m.indices.groups.outside[1]===2&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===2&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_q_class_assertion_retains_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var r=/(?!(?<run>^[\q{a|😀}]$))/dimyv;r.lastIndex=1;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('!A!');ok=ok&&m.index===1&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?!(?<run>[\q{ſ|k}]))/diyv;ok=ok&&r.exec(text)===null&&r.lastIndex===0}
        var r=/(?!(?<run>[\p{ASCII}\q{😀}]))/div,m=r.exec('ſK');ok=ok&&m.index===2&&m.groups.run===undefined;
        var r=/(?!(?<run>[\q{\uD800|x}]))/dgv,m=r.exec('😀');ok=ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined;r.lastIndex=0;var m=r.exec('\ud800x');ok=ok&&m.index===2&&m.groups.run===undefined;
        ok
    "#,
    );
}

#[test]
fn negative_q_class_assertion_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?!(?<run>[\\q{a|b}])))',r=new RegExp(source,'dgv'),all=Array.from('aba!'.matchAll(r));
        var ok=all.length===2&&all[0].index===3&&all[1].index===4&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'aba!'.replace(r,'<$<run>>')==='aba<>!<>'&&'aba!'.replaceAll(r,'<$<run>>')==='aba<>!<>'&&JSON.stringify('aba!'.split(new RegExp('(?!([\\q{a|b}]))','v')))==='["aba",null,"!"]';
        var offsets=[];ok=ok&&'aba!'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='aba!!!'&&JSON.stringify(offsets)==='[3,4]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!');m.index===0&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_q_class_assertion_deep_wrappers_keep_original_empty_and_undefined_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?!'+'('.repeat(1000)+'[\\q{a|b}]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgv'),m=r.exec('ab!');
        var ok=m.index===2&&m.length===2001&&m[0]===''&&r.lastIndex===2;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':undefined)&&(i<=1000?m.indices[i][0]===2&&m.indices[i][1]===2:m.indices[i]===undefined);ok
    "#,
    );
}

#[test]
fn negative_word_q_classes_keep_original_polarity_source_and_named_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{a|b}]\\b','aa'],['\\b[\\q{a|b}]\\B','a!'],['\\B[\\q{a|b}]\\B','aaa!'],['\\B[\\q{a|b}]\\b','aa!']]){
            var source='(?<outside>(?!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v'),m=r.exec(row[1]);
            ok=ok&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===0&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec(row[1]).groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_q_class_line_fold_property_and_backspace_context_stay_original() {
    check(
        r#"
        var r=/(?!(?<run>^\b[\q{a|😀}]\b$))/dimyv;r.lastIndex=1;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('!A!');ok=ok&&m.index===1&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?!(?<run>\b[\q{ſ|k}]\b))/diyv;r.lastIndex=1;ok=ok&&r.exec('!'+text+'!')===null&&r.lastIndex===0}
        var r=/(?!(?<run>\b[\p{ASCII}\q{😀}]\b))/diyv;r.lastIndex=1;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
        var r=/(?!(?<run>\B[\q{\u0008|a}]\B))/dyv;ok=ok&&r.exec('\b')===null;r.lastIndex=1;var m=r.exec('a\bb');ok=ok&&m.index===1&&m.groups.run===undefined;ok
    "#,
    );
}

#[test]
fn negative_word_q_class_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?!(?<run>\\b[\\q{a|b}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('a!'.matchAll(r));
        var ok=all.length===2&&all[0].index===1&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'a!'.replace(r,'<$<run>>')==='a<>!<>'&&JSON.stringify('a!'.split(new RegExp('(?!(\\b[\\q{a|b}]\\b))','v')))==='["a",null,"!"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===1;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('a!');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_q_class_search_preserves_complete_unicode_candidates_and_initial_pair_rewind() {
    check(
        "var r=new RegExp('(?!(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgv'),m=r.exec('😀😀');var ok=m.index===4&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===4;r=new RegExp('(?!(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgyv');r.lastIndex=1;ok&&r.exec('😀😀')===null&&r.lastIndex===0",
    );
}

#[test]
fn negative_scalar_v_set_assertion_keeps_original_source_names_and_undefined_operand_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[[a-z]&&[ab]]','[[[a-z]&&[ab]]--[x]]','[[[a-z]&&[ab]]&&[ab]]','[[a][b]]']){
            var source='(?<outside>(?!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=2;var m=r.exec('ab!');
            ok=ok&&m.index===2&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===2&&m.indices.groups.outside[1]===2&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===2&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_scalar_v_set_assertion_retains_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var r=/(?!(?<run>^[[a😀]--[x]]$))/dimyv;r.lastIndex=1;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('!A!');ok=ok&&m.index===1&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?!(?<run>[[ſk]--[x]]))/diyv;ok=ok&&r.exec(text)===null&&r.lastIndex===0}
        var r=/(?!(?<run>[[\p{ASCII}]--[x]]))/div,m=r.exec('ſK');ok=ok&&m.index===2&&m.groups.run===undefined;
        var r=/(?!(?<run>[[\uD800x]--[q]]))/dgv,m=r.exec('😀');ok=ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined;r.lastIndex=0;var m=r.exec('\ud800x');ok=ok&&m.index===2&&m.groups.run===undefined;
        ok
    "#,
    );
}

#[test]
fn negative_scalar_v_set_assertion_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?!(?<run>[[a-z]&&[ab]])))',r=new RegExp(source,'dgv'),all=Array.from('aba!'.matchAll(r));
        var ok=all.length===2&&all[0].index===3&&all[1].index===4&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'aba!'.replace(r,'<$<run>>')==='aba<>!<>'&&'aba!'.replaceAll(r,'<$<run>>')==='aba<>!<>'&&JSON.stringify('aba!'.split(new RegExp('(?!([[a-z]&&[ab]]))','v')))==='["aba",null,"!"]';
        var offsets=[];ok=ok&&'aba!'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='aba!!!'&&JSON.stringify(offsets)==='[3,4]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('!');m.index===0&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_scalar_v_set_assertion_deep_wrappers_keep_original_empty_and_undefined_slots() {
    check(
        r#"
        var source='('.repeat(1000)+'(?!'+'('.repeat(1000)+'[[a-z]&&[ab]]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgv'),m=r.exec('ab!');
        var ok=m.index===2&&m.length===2001&&m[0]===''&&r.lastIndex===2;
        for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':undefined)&&(i<=1000?m.indices[i][0]===2&&m.indices[i][1]===2:m.indices[i]===undefined);ok
    "#,
    );
}
