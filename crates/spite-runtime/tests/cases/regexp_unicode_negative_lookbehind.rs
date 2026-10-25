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
fn negative_predecessor_deep_wrappers_keep_all_original_empty_and_undefined_slots() {
    check(
        r#"
        var ok=true;
        for(var mode of ['u','v']){
            var source='('.repeat(1000)+'(?<!'+'('.repeat(1000)+'[a-z]'+')'.repeat(1000)+')'+')'.repeat(1000),r=new RegExp(source,'dgy'+mode);r.lastIndex=1;var m=r.exec('!a');
            ok=ok&&m.index===1&&m.length===2001&&m[0]==='';
            for(var i=1;i<=2000;i++)ok=ok&&m[i]===(i<=1000?'':undefined)&&(i<=1000?m.indices[i][0]===1&&m.indices[i][1]===1:m.indices[i]===undefined);
        }ok
    "#,
    );
}
