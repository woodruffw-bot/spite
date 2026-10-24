//! Entire validated classes with max zero need no membership implementation.

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
fn erased_classes_match_native_empty_patterns_across_modes_flags_and_boundaries() {
    check(
        r#"
        var cases=[
            ['[\\p{RGI_Emoji}]','v'],
            ['[[\\q{ab}]&&\\p{RGI_Emoji}]','v'],
            ['[\\q{ab|cd}--\\p{RGI_Emoji}]','v'],
            ['[\\p{RGI_Emoji}\\q{|ab}]','v'],
            ['[\\q{\\[\\]|a}]','v'],
            ['[\\p{Script=Greek}]','u'],
            ['[()\\]]','u'],
            ['[[]','']
        ],ok=true;
        for(var pair of cases)for(var count of ['{0}','{0,0}','{0000}?','{000,000}?'])for(var extras of ['d','dg','dy','dgy','dgi','dgim','dgms','dgisy']) {
            var pattern=pair[0]+count,r=new RegExp(pattern,extras+pair[1]),control=new RegExp('',extras+pair[1]);
            for(var input of ['','ab','A😀!','\ud800x\udc00','ſa','a\r\nb'])for(var start=0;start<=input.length+1;start++) {
                r.lastIndex=start;control.lastIndex=start;var m=r.exec(input),n=control.exec(input);
                ok=ok&&r.source===pattern&&r.flags===control.flags&&r.lastIndex===control.lastIndex;
                if(m===null||n===null)ok=ok&&m===n;
                else ok=ok&&m.length===1&&m.indices.length===1&&m[0]===''&&m.index===n.index&&m.input===input&&Object.hasOwn(m,'groups')&&Object.hasOwn(m.indices,'groups')&&m.groups===undefined&&m.indices.groups===undefined&&JSON.stringify(m.indices)===JSON.stringify(n.indices);
            }
        }
        ok
    "#,
    );
}

#[test]
fn erased_class_string_consumers_and_clones_retain_empty_match_metadata() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/[\p{RGI_Emoji}]{0}/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===1&&m[0]===''&&m.groups===undefined&&m.indices.groups===undefined;
        var calls=[];var result='ab'.replace(r,function(m,at,input){calls.push(at);return arguments.length===3&&m===''&&input==='ab'?'!':'?'});
        ok=ok&&result==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.search(r)===0&&'ab'.split(r).join(',')==='a,b'&&'ab'.match(r).length===3;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');
        ok&&m.index===1&&m.length===1&&copy.source===r.source&&copy.flags==='dgv'&&copy.lastIndex===1
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("copy.lastIndex=0;copy.exec('x').length===1&&copy.source===r.source"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn syntax_errors_still_precede_class_erasure_and_other_shapes_stay_unsupported() {
    check(
        r#"
        var invalid=[
            ['[^\\p{RGI_Emoji}]{0}','v'],
            ['[\\P{RGI_Emoji}]{0}','v'],
            ['[\\q{ab}&&a--a]{0}','v'],
            ['[\\q{\\d}]{0}','v'],
            ['[z-a]{0}','u'],
            ['[\\p{NotAProperty}]{0}','v'],
            ['[\\q{ab}]{0}','u']
        ],ok=true;
        for(var pair of invalid){var caught=false;try{new RegExp(pair[0],pair[1])}catch(e){caught=e instanceof SyntaxError}ok=ok&&caught}
        ok
    "#,
    );
    for pattern in [r"[\p{RGI_Emoji}]{0,1}", r"[\p{RGI_Emoji}]{0}a"] {
        let pattern = JsString::from(pattern);
        assert!(
            matches!(
                Realm::default().eval(&format!("new RegExp({pattern:?},'v').test('')")),
                Err(Error::Unsupported { .. })
            ),
            "{pattern:?}"
        );
    }
}

#[test]
fn deep_classes_long_zero_counts_and_opt_in_aborts_preserve_full_validation() {
    check(
        r#"
        var pattern='['.repeat(1000)+'\\p{RGI_Emoji}'+']'.repeat(1000)+'{'+ '0'.repeat(1000)+','+'0'.repeat(1000)+'}?',r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀');m.length===1&&m.index===0&&m[0]===''&&m.indices[0][0]===0&&m.indices[0][1]===0&&r.lastIndex===0&&r.source===pattern
    "#,
    );
    let pattern =
        JsString::from(format!(r"[\p{{RGI_Emoji}}\q{{{}}}]{{0}}", "a".repeat(20_000)).as_str());
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
fn enclosing_class_captures_keep_named_aliases_and_normalized_empty_ranges() {
    check(
        r#"
        var cases=[['[\\p{RGI_Emoji}]','v'],['[[\\q{ab}]&&\\p{RGI_Emoji}]','v'],['[()\\]]','u'],['[[]','']],ok=true;
        for(var pair of cases)for(var count of ['{0}','{000,000}?'])for(var frame of [['(?<outside>',')',1],['(?<outside>(?:(?<second>',')))',2],['(?:',')',0]]){
            var pattern=frame[0]+pair[0]+count+frame[1],r=new RegExp(pattern,'dgy'+pair[1]),point=pair[1]===''?2:1;
            r.lastIndex=2;var m=r.exec('A😀!');
            ok=ok&&r.source===pattern&&m.index===point&&m[0]===''&&r.lastIndex===point&&m.length===frame[2]+1;
            for(var i=1;i<m.length;i++)ok=ok&&m[i]===''&&m.indices[i][0]===point&&m.indices[i][1]===point;
            if(frame[2]===0)ok=ok&&m.groups===undefined&&m.indices.groups===undefined;
            else ok=ok&&Object.getPrototypeOf(m.groups)===null&&Object.getPrototypeOf(m.indices.groups)===null&&m.groups.outside===''&&m.indices.groups.outside===m.indices[1];
            if(frame[2]>1)ok=ok&&m.groups.second===''&&m.indices.groups.second===m.indices[2];
        }
        var m=/(?<__proto__>(?<\u{1D49C}>[\p{RGI_Emoji}]{0}))/dv.exec('x');
        ok&&Object.getPrototypeOf(m.groups)===null&&Object.hasOwn(m.groups,'__proto__')&&m.groups.__proto__===''&&m.groups['𝒜']===''&&m.indices.groups.__proto__===m.indices[1]&&m.indices.groups['𝒜']===m.indices[2]
    "#,
    );
}

#[test]
fn enclosing_class_string_consumers_clones_and_collection_preserve_defined_slots() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        var r=/(?<outside>[\p{RGI_Emoji}]{0})/dgv,ms=Array.from('a😀'.matchAll(r)),ok=ms.length===3&&ms.map(m=>m.index).join(',')==='0,1,3';
        for(var m of ms)ok=ok&&m.length===2&&m[1]===''&&m.groups.outside===''&&m.indices.groups.outside===m.indices[1];
        var calls=[];var value='ab'.replace(r,function(m,outside,at,input,groups){calls.push(at);return arguments.length===5&&m===''&&outside===''&&input==='ab'&&groups.outside===''?'!':'?'});
        ok=ok&&value==='!a!b!'&&calls.join(',')==='0,1,2'&&'ab'.split(r).join(',')==='a,,b'&&'ab'.search(r)===0;
        var copy=new RegExp(r);copy.lastIndex=1;var m=copy.exec('ab');ok&&copy.source===r.source&&m.index===1&&m[1]===''&&copy.lastIndex===1
    "#),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("copy.lastIndex=0;var m=copy.exec('x');m.groups.outside===''&&m.indices.groups.outside===m.indices[1]"),Ok(Value::Boolean(true)));
}

#[test]
fn deep_enclosing_class_captures_remain_defined_without_class_membership_queries() {
    check(
        r#"
        var depth=1000,pattern='('.repeat(depth)+'['.repeat(depth)+'\\p{RGI_Emoji}'+']'.repeat(depth)+'{0}'+')'.repeat(depth),r=new RegExp(pattern,'dgv');
        r.lastIndex=1;var m=r.exec('😀'),ok=m.index===0&&m.length===1001&&m[0]===''&&r.lastIndex===0;
        for(var i=1;i<m.length;i++)ok=ok&&m[i]===''&&m.indices[i][0]===0&&m.indices[i][1]===0;
        ok
    "#,
    );
}
