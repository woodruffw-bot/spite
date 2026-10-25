//! Single-character negative lookbehind discards all operand captures.

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn negative_predecessor_preserves_named_outside_and_undefined_operand_properties() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var atom of ['a','[a-z]','\\d','.','😀','\\uD800']){
            var source='(?<outside>(?<!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+mode),m=r.exec('');
            ok=ok&&m.index===0&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===0&&m.indices.groups.outside[1]===0&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===0&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_predecessor_search_sticky_end_and_pair_candidates_remain_exact() {
    // Independent expectation: search visits complete Unicode candidates.
    check(
        "var ok=true,input='😀😀a';for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var start=0;start<=input.length+1;start++){var r=new RegExp('(?<!(?<run>😀))','d'+state+mode);r.lastIndex=start;var m=r.exec(input),active=r.global||r.sticky,point=active?start:0;if(point===1||point===3)point--;if(point>input.length||r.sticky&&(point===2||point===4)){ok=ok&&m===null&&r.lastIndex===0;continue}if(point===2||point===4)point=5;ok=ok&&m.index===point&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===(active?point:start)}ok",
    );
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?<!(a))','dgy'+mode);r.lastIndex=1;ok=ok&&r.exec('aaa')===null&&r.lastIndex===0;r.lastIndex=1;ok=ok&&r.exec('b').index===1;
            var r=new RegExp('(?<!(a))','dg'+mode);r.lastIndex=1;ok=ok&&r.exec('aaa')===null&&r.lastIndex===0;
        }ok
    "#,
    );
}

#[test]
fn negative_predecessor_line_dot_property_and_lone_surrogate_context_stay_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?<!(?<run>^a$))','dimy'+mode);r.lastIndex=2;ok=ok&&r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
            var r=new RegExp('(?<!(.))','dg'+mode);r.lastIndex=1;var m=r.exec('a\n');ok=ok&&m.index===2&&m[1]===undefined;
            var r=new RegExp('(?<!(.))','dsy'+mode);r.lastIndex=2;ok=ok&&r.exec('a\n')===null&&r.lastIndex===0;
            var r=new RegExp('(?<!(\\p{ASCII}))','diy'+mode);r.lastIndex=1;ok=ok&&r.exec('ſK!')===null&&r.lastIndex===0;
            var r=new RegExp('(?<!(\\uDE00))','dy'+mode);r.lastIndex=2;var m=r.exec('😀');ok=ok&&m.index===2&&m[1]===undefined;r.lastIndex=3;ok=ok&&r.exec('\ud800x\ude00')===null&&r.lastIndex===0;
        }ok
    "#,
    );
}

#[test]
fn negative_predecessor_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<outside>(?<!(?<run>a)))',r=new RegExp(source,'dg'+mode),all=Array.from('aba'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===0&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
            ok=ok&&'aba'.replace(r,'<$<run>>')==='<>ab<>a'&&'aba'.replaceAll(r,'<$<run>>')==='<>ab<>a'&&JSON.stringify('aba'.split(new RegExp('(?<!(a))',mode)))==='["ab",null,"a"]';
            var offsets=[];ok=ok&&'aba'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&match===''&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='!ab!a'&&JSON.stringify(offsets)==='[0,2]';
            r.lastIndex=1;ok=ok&&'aba'.search(r)===0&&r.lastIndex===1;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('b').groups.run===undefined;
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('b');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_predecessor_preserves_named_undefined_slots_and_boundary_polarity() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v'])for(var state of ['','g','gy'])for(var row of [['\\ba\\b','aa'],['\\ba\\B','a!'],['\\Ba\\B','a!'],['\\Ba\\b','aa']]){
            var source='(?<outside>(?<!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+mode);r.lastIndex=1;var m=r.exec(row[1]),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_predecessor_line_fold_property_and_backspace_context_stay_original() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var r=new RegExp('(?<!(?<run>^\\ba\\b$))','dimy'+mode);r.lastIndex=2;ok=ok&&r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
            var r=new RegExp('(?<!(\\bſ\\b))','diy'+mode);r.lastIndex=2;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
            var r=new RegExp('(?<!(\\b\\p{ASCII}\\b))','diy'+mode);r.lastIndex=2;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
            var r=new RegExp('(?<!(\\B[\\b]\\B))','dy'+mode);r.lastIndex=1;ok=ok&&r.exec('\b')===null;r.lastIndex=2;var m=r.exec('a\bb');ok=ok&&m.index===2&&m[1]===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_predecessor_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='(?<outside>(?<!(?<run>\\ba\\b)))',r=new RegExp(source,'dg'+mode),all=Array.from('a!'.matchAll(r));
            ok=ok&&all.length===2&&all[0].index===0&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
            ok=ok&&'a!'.replace(r,'<$<run>>')==='<>a!<>'&&JSON.stringify('a!'.split(new RegExp('(?<!(\\ba\\b))',mode)))==='["a!"]';
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===0;
        }ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a!');m.index===2&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_predecessor_search_retains_complete_characters_and_initial_pair_rewind() {
    // Independent expectations exclude native interior-pair search candidates.
    check(
        "var ok=true;for(var mode of ['u','v']){var r=new RegExp('(?<!(?<run>\\\\B😀\\\\B))','dg'+mode);r.lastIndex=2;ok=ok&&r.exec('😀😀')===null&&r.lastIndex===0;r.lastIndex=1;var m=r.exec('😀😀');ok=ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===0;r=new RegExp('(?<!(?<run>\\\\B.\\\\b))','dgy'+mode);r.lastIndex=5;ok=ok&&r.exec('a😀ab!')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('a😀ab!');ok=ok&&m.index===1&&m.groups.run===undefined&&r.lastIndex===1}ok",
    );
}

#[test]
fn negative_q_class_predecessors_keep_original_source_and_named_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[\\q{a|b}]','[[\\q{a|b}]--[x]]','[[\\q{a|b}]&&[ab]]','[x\\q{a|b}]']){
            var source='(?<outside>(?<!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec('!'),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_q_class_predecessors_keep_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var r=/(?<!(?<run>^[\q{a|😀}]$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>[\q{ſ|k}]))/diyv;r.lastIndex=1;ok=ok&&r.exec(text)===null&&r.lastIndex===0}
        var r=/(?<!(?<run>[\p{ASCII}\q{😀}]))/diyv;r.lastIndex=1;ok=ok&&r.exec('ſ')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>[\q{\uD800|x}]))/dyv;r.lastIndex=2;var m=r.exec('😀');ok=ok&&m.index===2&&m.groups.run===undefined;r.lastIndex=1;ok=ok&&r.exec('\ud800x')===null&&r.lastIndex===0;ok
    "#,
    );
}

#[test]
fn negative_q_class_predecessor_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>[\\q{a|b}])))',r=new RegExp(source,'dgv'),all=Array.from('ab!a'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===3&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'ab!a'.replace(r,'<$<run>>')==='<>ab!<>a'&&'ab!a'.replaceAll(r,'<$<run>>')==='<>ab!<>a'&&JSON.stringify('ab!a'.split(new RegExp('(?<!([\\q{a|b}]))','v')))==='["ab!",null,"a"]';
        var offsets=[];ok=ok&&'ab!a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='!ab!!a'&&JSON.stringify(offsets)==='[0,3]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('!');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_q_class_predecessors_keep_original_polarity_source_and_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{a|b}]\\b','aa'],['\\b[\\q{a|b}]\\B','a!'],['\\B[\\q{a|b}]\\B','a!'],['\\B[\\q{a|b}]\\b','aa']]){
            var source='(?<outside>(?<!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec(row[1]),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_q_class_predecessor_line_fold_property_and_backspace_context_stay_original() {
    check(
        r#"
        var r=/(?<!(?<run>^\b[\q{a|😀}]\b$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>\b[\q{ſ|k}]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!'+text+'!')===null&&r.lastIndex===0}
        var r=/(?<!(?<run>\b[\p{ASCII}\q{😀}]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>\B[\q{\u0008|a}]\B))/dyv;r.lastIndex=1;ok=ok&&r.exec('\b')===null;r.lastIndex=2;var m=r.exec('a\bb');ok=ok&&m.index===2&&m.groups.run===undefined;ok
    "#,
    );
}

#[test]
fn negative_word_q_class_predecessor_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>\\b[\\q{a|b}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('a!'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'a!'.replace(r,'<$<run>>')==='<>a!<>'&&JSON.stringify('a!'.split(new RegExp('(?<!(\\b[\\q{a|b}]\\b))','v')))==='["a!"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===0;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a!');m.index===2&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_q_class_predecessor_search_keeps_complete_candidates_and_initial_pair_rewind() {
    check(
        "var r=new RegExp('(?<!(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgv');r.lastIndex=2;var ok=r.exec('😀😀')===null&&r.lastIndex===0;r=new RegExp('(?<!(?<run>\\\\B[\\\\q{😀|a}]\\\\B))','dgyv');r.lastIndex=1;var m=r.exec('😀😀');ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===0",
    );
}

#[test]
fn negative_scalar_v_set_predecessors_keep_original_source_and_named_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[[a-z]&&[ab]]','[[[a-z]&&[ab]]--[x]]','[[[a-z]&&[ab]]&&[ab]]','[[a][b]]']){
            var source='(?<outside>(?<!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec('!'),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_scalar_v_set_predecessors_keep_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var r=/(?<!(?<run>^[[a😀]--[x]]$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>[[ſk]--[x]]))/diyv;r.lastIndex=1;ok=ok&&r.exec(text)===null&&r.lastIndex===0}
        var r=/(?<!(?<run>[[\p{ASCII}][😀]]))/diyv;r.lastIndex=1;ok=ok&&r.exec('ſ')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>[[\uD800x]--[q]]))/dyv;r.lastIndex=2;var m=r.exec('😀');ok=ok&&m.index===2&&m.groups.run===undefined;r.lastIndex=1;ok=ok&&r.exec('\ud800x')===null&&r.lastIndex===0;ok
    "#,
    );
}

#[test]
fn negative_scalar_v_set_predecessor_string_consumers_copies_and_collection_keep_undefined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>[[a-z]&&[ab]])))',r=new RegExp(source,'dgv'),all=Array.from('ab!a'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===3&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'ab!a'.replace(r,'<$<run>>')==='<>ab!<>a'&&'ab!a'.replaceAll(r,'<$<run>>')==='<>ab!<>a'&&JSON.stringify('ab!a'.split(new RegExp('(?<!([[a-z]&&[ab]]))','v')))==='["ab!",null,"a"]';
        var offsets=[];ok=ok&&'ab!a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='!ab!!a'&&JSON.stringify(offsets)==='[0,3]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('!');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_scalar_v_set_predecessors_keep_original_polarity_source_and_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[[a-z]&&[ab]]\\b','aa'],['\\b[[a-z]&&[ab]]\\B','a!'],['\\B[[a-z]&&[ab]]\\B','a!'],['\\B[[a-z]&&[ab]]\\b','aa']]){
            var source='(?<outside>(?<!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec(row[1]),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_scalar_v_set_predecessor_line_fold_property_and_backspace_context_stay_original() {
    check(
        r#"
        var r=/(?<!(?<run>^\b[[a😀]--[x]]\b$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>\b[[ſk]--[x]]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!'+text+'!')===null&&r.lastIndex===0}
        var r=/(?<!(?<run>\b[[\p{ASCII}][😀]]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>\B[[\u0008a]--[x]]\B))/dyv;r.lastIndex=1;ok=ok&&r.exec('\b')===null;r.lastIndex=2;var m=r.exec('a\bb');ok=ok&&m.index===2&&m.groups.run===undefined;ok
    "#,
    );
}

#[test]
fn negative_word_scalar_v_set_predecessor_string_consumers_copies_and_collection_keep_undefined_slots()
 {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>\\b[[a-z]&&[ab]]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('a!'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'a!'.replace(r,'<$<run>>')==='<>a!<>'&&JSON.stringify('a!'.split(new RegExp('(?<!(\\b[[a-z]&&[ab]]\\b))','v')))==='["a!"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===0;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a!');m.index===2&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_scalar_v_set_predecessor_search_keeps_complete_candidates_and_initial_pair_rewind()
{
    check(
        "var r=new RegExp('(?<!(?<run>\\\\B[[😀a]--[x]]\\\\B))','dgv');r.lastIndex=2;var ok=r.exec('😀😀')===null&&r.lastIndex===0;r=new RegExp('(?<!(?<run>\\\\B[[😀a]--[x]]\\\\B))','dgyv');r.lastIndex=1;var m=r.exec('😀😀');ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===0",
    );
}

#[test]
fn negative_character_result_predecessors_keep_original_source_and_named_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var atom of ['[\\q{a|b|ab|}&&\\p{Any}]','[[\\q{a|b|ab|}&&\\p{Any}]--[x]]','[[\\q{a|b|ab|}&&\\p{Any}]&&[ab]]','[[\\q{ab}&&a]ab]']){
            var source='(?<outside>(?<!(?<run>'+atom+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec('!'),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_character_result_predecessors_keep_original_line_fold_property_and_surrogate_context() {
    check(
        r#"
        var r=/(?<!(?<run>^[[\q{😀x|😀}&&\p{Any}]a]$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>[\q{ſ|k|sk|}&&\p{Any}]))/diyv;r.lastIndex=1;ok=ok&&r.exec(text)===null&&r.lastIndex===0}
        var r=/(?<!(?<run>[\q{S|K|sK|}&&\p{ASCII}]))/diyv;r.lastIndex=1;ok=ok&&r.exec('ſ')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>[\q{\uD800|x|ab|}&&\p{Any}]))/dyv;r.lastIndex=2;var m=r.exec('😀');ok=ok&&m.index===2&&m.groups.run===undefined;r.lastIndex=1;ok=ok&&r.exec('\ud800x')===null&&r.lastIndex===0;ok
    "#,
    );
}

#[test]
fn negative_character_result_predecessor_string_consumers_copies_and_collection_keep_undefined_slots()
 {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>[\\q{a|b|ab|}&&\\p{Any}])))',r=new RegExp(source,'dgv'),all=Array.from('ab!a'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===3&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'ab!a'.replace(r,'<$<run>>')==='<>ab!<>a'&&'ab!a'.replaceAll(r,'<$<run>>')==='<>ab!<>a'&&JSON.stringify('ab!a'.split(new RegExp('(?<!([\\q{a|b|ab|}&&\\p{Any}]))','v')))==='["ab!",null,"a"]';
        var offsets=[];ok=ok&&'ab!a'.replace(r,function(match,outside,run,offset,input,groups){offsets.push(offset);ok=ok&&outside===''&&run===undefined&&groups.run===undefined;return '!';})==='!ab!!a'&&JSON.stringify(offsets)==='[0,3]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('!').groups.run===undefined;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('!');m.index===1&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===1"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_character_result_predecessors_keep_original_polarity_source_and_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var state of ['','g','gy'])for(var row of [['\\b[\\q{a|b|ab|}&&\\p{Any}]\\b','aa'],['\\b[\\q{a|b|ab|}&&\\p{Any}]\\B','a!'],['\\B[\\q{a|b|ab|}&&\\p{Any}]\\B','a!'],['\\B[\\q{a|b|ab|}&&\\p{Any}]\\b','aa']]){
            var source='(?<outside>(?<!(?<run>'+row[0]+')))',r=new RegExp(source,'d'+state+'v');r.lastIndex=1;var m=r.exec(row[1]),point=state===''?0:1;
            ok=ok&&m.index===point&&m[0]===''&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.outside[0]===point&&m.indices.groups.outside[1]===point&&m.indices.groups.run===undefined&&Object.hasOwn(m.groups,'run')&&Object.hasOwn(m.indices.groups,'run')&&r.lastIndex===1&&r.source===source;
            var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('').groups.run===undefined;
        }ok
    "#,
    );
}

#[test]
fn negative_word_character_result_predecessor_line_fold_property_and_backspace_context_stay_original()
 {
    check(
        r#"
        var r=/(?<!(?<run>^\b[[\q{😀x|😀}&&\p{Any}]a]\b$))/dimyv;r.lastIndex=2;var ok=r.exec('\nA\n')===null&&r.lastIndex===0;r.lastIndex=2;var m=r.exec('!A!');ok=ok&&m.index===2&&m.groups.run===undefined;
        for(var text of ['S','ſ','K','K']){var r=/(?<!(?<run>\b[\q{ſ|k|sk|}&&\p{Any}]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!'+text+'!')===null&&r.lastIndex===0}
        var r=/(?<!(?<run>\b[\q{S|K|sK|}&&\p{ASCII}]\b))/diyv;r.lastIndex=2;ok=ok&&r.exec('!ſ!')===null&&r.lastIndex===0;
        var r=/(?<!(?<run>\B[\q{\u0008|a|ab|}&&\p{Any}]\B))/dyv;r.lastIndex=1;ok=ok&&r.exec('\b')===null;r.lastIndex=2;var m=r.exec('a\bb');ok=ok&&m.index===2&&m.groups.run===undefined;ok
    "#,
    );
}

#[test]
fn negative_word_character_result_predecessor_string_consumers_copies_and_collection_keep_undefined_slots()
 {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var source='(?<outside>(?<!(?<run>\\b[\\q{a|b|ab|}&&\\p{Any}]\\b)))',r=new RegExp(source,'dgv'),all=Array.from('a!'.matchAll(r));
        var ok=all.length===2&&all[0].index===0&&all[1].index===2&&all[0].groups.outside===''&&all[0].groups.run===undefined&&all[0].indices.groups.run===undefined&&r.lastIndex===0;
        ok=ok&&'a!'.replace(r,'<$<run>>')==='<>a!<>'&&JSON.stringify('a!'.split(new RegExp('(?<!(\\b[\\q{a|b|ab|}&&\\p{Any}]\\b))','v')))==='["a!"]';
        var copy=new RegExp(r);ok=ok&&copy.source===source&&copy.flags===r.flags&&copy.exec('a!').index===0;ok
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=1;var m=copy.exec('a!');m.index===2&&m.groups.outside===''&&m.groups.run===undefined&&m.indices.groups.run===undefined&&copy.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn negative_word_character_result_predecessor_search_keeps_complete_candidates_and_initial_pair_rewind()
 {
    check(
        "var r=new RegExp('(?<!(?<run>\\\\B[\\\\q{😀|a|😀x|}&&\\\\p{Any}]\\\\B))','dgv');r.lastIndex=2;var ok=r.exec('😀😀')===null&&r.lastIndex===0;r=new RegExp('(?<!(?<run>\\\\B[\\\\q{😀|a|😀x|}&&\\\\p{Any}]\\\\B))','dgyv');r.lastIndex=1;var m=r.exec('😀😀');ok&&m.index===0&&m.groups.run===undefined&&m.indices.groups.run===undefined&&r.lastIndex===0",
    );
}
