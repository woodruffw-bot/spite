//! Equal-width literal choices retain fixed repeated-group semantics.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn literal_choices_greedy_lazy_and_bounded_endpoints_keep_final_iteration() {
    check(
        r"let a=/(a|b)+(b)/d.exec('abab'),b=/(a|b)+?(b)/d.exec('abab');a[0]==='abab'&&a[1]==='a'&&a.indices[1][0]===2&&a.indices[2][0]===3&&b[0]==='ab'&&b[1]==='a'&&b.indices[1][0]===0&&b.indices[2][0]===1",
    );
    check(
        r"let a=/(a|b){1,2}(b)/d.exec('aaaab');a.index===2&&a[0]==='aab'&&a.indices[1][0]===3&&a.indices[2][0]===4&&/(a|b){1,2}?(b)/.exec('aaaab').index===2",
    );
    check(
        r"let a=/(a|a|b|a)+(c)/d.exec('ababc');a[1]==='b'&&a.indices[1][0]===3&&a.indices[2][0]===4",
    );
}

#[test]
fn zero_iterations_partial_wrappers_and_noncapturing_choices_keep_slots() {
    check(
        r"let a=/((a|b)+)/d.exec('ab'),r=/(?:(?:a|b)+)/d; a[1]==='ab'&&a[2]==='b'&&a.indices[1][0]===0&&a.indices[1][1]===2&&a.indices[2][0]===1&&r.exec('ab').length===1&&r.source==='(?:(?:a|b)+)'",
    );
    check(
        r"let a=/(x)((a|b)*)(y)/d.exec('xy');a[1]==='x'&&a[2]===''&&a[3]===undefined&&a[4]==='y'&&a.indices[2][0]===1&&a.indices[2][1]===1&&a.indices[3]===undefined&&Object.hasOwn(a,'3')",
    );
    check(
        r"let a=/((x)(a|b)+)(y)/d.exec('xababy');a[1]==='xabab'&&a[2]==='x'&&a[3]==='b'&&a[4]==='y'&&a.indices[1][1]===5&&a.indices[3][0]===4",
    );
    check(
        r"let a=/x(?:a|b)+(y)/d.exec('xababy');a.length===2&&a[1]==='y'&&a.indices[1][0]===5&&/(a|b){0}/.exec('')[1]===undefined",
    );
    check(
        r"let a=/(((a|b)+))(c)/d.exec('ababc');a[1]==='abab'&&a[2]==='abab'&&a[3]==='b'&&a[4]==='c'&&a.indices[1][1]===4&&a.indices[3][0]===3",
    );
}

#[test]
fn assertions_flags_and_complete_input_neighbors_keep_choice_ranges() {
    check(
        r"let a=/^((a|b)+)($)(\n)/dm.exec('x\nabab\n');a.index===2&&a[1]==='abab'&&a[2]==='b'&&a.indices[2][0]===5&&a.indices[3][0]===6&&/^((a|b)+)($)(\n)/.exec('abab\n')===null",
    );
    check(
        r"let a=/\b((a|b)+)(c)\b/d.exec('ababc');a[1]==='abab'&&a[2]==='b'&&a.indices[1][1]===4&&a.indices[2][0]===3&&/\b((a|b)+)(c)\b/.exec('ababcd')===null",
    );
    check(
        r"let a=/(µ|Μ)+(x)/di.exec('µΜµx');a[1]==='µ'&&a.indices[1][0]===2&&a.indices[2][0]===3&&/(S|s)+/i.exec('ſ')===null&&/(ſ|S)+/i.exec('ſSs')[0]==='ſSs'",
    );
}

#[test]
fn decoded_literal_metacharacters_controls_and_surrogates_are_exact() {
    check(
        r"let a=/(\x61|\u0062)+(c)/d.exec('ababc'),b=/(\(|\))+(x)/d.exec('()()x');a[1]==='b'&&a.indices[1][0]===3&&b[1]===')'&&b.indices[2][0]===4",
    );
    check(
        r"let a=/(\0|1)+(x)/d.exec('\x0011x'),b=/(\x08|\t)+(x)/d.exec('\x08\tx');a[1]==='1'&&a.indices[1][0]===2&&b[1]==='\t'&&b.indices[2][0]===2",
    );
    check(
        r"let a=/(\uDCA9|a)+(b)/d.exec('💩aaab');a.index===1&&a[1]==='a'&&a.indices[1][0]===4&&a.indices[2][0]===5&&/(\uD800|\uDC00)+(c)/.exec('\uD800\uDC00c')[1]==='\uDC00'",
    );
}

#[test]
fn alternatives_global_sticky_copies_and_original_source_keep_capture_offsets() {
    check(
        r"let a=/(a|b)+a|(a|b)+b/d.exec('aba'),b=/(a|b)+a|(a|b)+b/d.exec('ab');a[0]==='aba'&&a[1]==='b'&&a[2]===undefined&&b[1]===undefined&&b[2]==='a'&&b.indices[2][0]===0",
    );
    check(
        r"let r=/(x)((a|b)+)(y)/dg,copy=new RegExp(r),a=r.exec('xay xababy'),b=r.exec('xay xababy');a.indices[3][0]===1&&b.index===4&&b.indices[2][0]===5&&b.indices[2][1]===9&&b.indices[3][0]===8&&r.lastIndex===10&&copy.source===r.source&&copy.lastIndex===0&&r.exec('xay xababy')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/x(a|b)+(y)/dy;r.lastIndex=1;let a=r.exec('yxababy');a.index===1&&a[1]==='b'&&a.indices[1][0]===5&&a.indices[2][0]===6&&r.lastIndex===7&&r.exec('yxababy')===null&&r.lastIndex===0",
    );
}

#[test]
fn generic_consumers_intrinsic_arrays_and_callback_arguments_keep_choices() {
    check(
        r"'xababy'.replace(/(x)((a|b)+)(y)/,'<$1,$2,$3,$4>')==='<x,abab,b,y>'&&'qxababyz'.split(/(x)((a|b)+)(y)/).join(',')==='q,x,abab,b,y,z'&&'qxababy'.search(/x(a|b)+y/)===1",
    );
    check(
        r"let r=/(x)((a|b)+)(y)/dg,a=[...'xay xababy'.matchAll(r)];a.length===2&&a[1][2]==='abab'&&a[1][3]==='b'&&a[1].indices[3][0]===8&&r.lastIndex===0",
    );
    check(
        r"let seen;let s='qxababy'.replace(/(x)((a|b)+)(y)/,(whole,prefix,run,last,sequel,index,input)=>{seen=[whole,prefix,run,last,sequel,index,input];return 'z'});s==='qz'&&seen[2]==='abab'&&seen[3]==='b'&&seen[5]===1&&seen[6]==='qxababy'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'3',{set(){calls++},configurable:true});let a=/(x)((a|b){0})(y)/d.exec('xy');calls===0&&a[2]===''&&a[3]===undefined&&a.indices[3]===undefined&&Object.hasOwn(a,'3')&&Object.hasOwn(a.indices,'3')",
    );
}

#[test]
fn deep_partial_captures_long_runs_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('a'+'('.repeat(100000)+'(a|b)+'+')'.repeat(100000)+'(ab)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100003&&a[1].length===199998&&a[100000]===a[1]&&a[100001]==='a'&&a[100002]==='ab'&&a.indices[100000][0]===1&&a.indices[100000][1]===199999&&a.indices[100001][0]===199998&&a.indices[100002][0]===199999&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_aborts_and_other_repeated_choices_remain_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/x(a|b)+(y)/y,s='x'+'a'.repeat(20000)+'y',flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(ab|a)+/.test('aba')",
        r"/(a|)+/.test('a')",
        r"/((a)|(b))+/.test('ab')",
        r"/((a|bc))+/.test('ab')",
        r"/(a|[^b]c)+/.test('ab')",
        r"/(a|[^]c)+/.test('ab')",
        r"/(a|b)+(c)+/.test('abc')",
        r"/(?<n>a|b)+/.test('ab')",
        r"/(a|b)+\1/.test('aa')",
        r"/(a|b)+/u.test('ab')",
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
