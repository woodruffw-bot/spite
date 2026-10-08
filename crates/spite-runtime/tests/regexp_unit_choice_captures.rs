//! Complete captures inside repeated literal-unit choices retain the last iteration.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn every_inner_wrapper_captures_the_final_successful_unit() {
    check(
        r"let a=/(((a|b)))+(c)/d.exec('ababc');a.length===5&&a[1]==='b'&&a[2]==='b'&&a[3]==='b'&&a[4]==='c'&&a.indices[1][0]===3&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices[4][0]===4",
    );
    check(
        r"let a=/((?:a|b))+(c)/d.exec('ababc'),b=/(?:(a|b))+(c)/d.exec('ababc'),c=/(?:(?:a|b))+(c)/d.exec('ababc');a.length===3&&b.length===3&&c.length===2&&a[1]==='b'&&b[1]==='b'&&c[1]==='c'",
    );
}

#[test]
fn enclosing_partial_runs_and_zero_iterations_keep_distinct_slots() {
    check(
        r"let a=/(x)(((a|b))+)(y)/d.exec('xababy');a[1]==='x'&&a[2]==='abab'&&a[3]==='b'&&a[4]==='b'&&a[5]==='y'&&a.indices[2][0]===1&&a.indices[2][1]===5&&a.indices[3][0]===4&&a.indices[4][0]===4",
    );
    check(
        r"let a=/(x)(((a|b))*)(y)/d.exec('xy');a[1]==='x'&&a[2]===''&&a[3]===undefined&&a[4]===undefined&&a[5]==='y'&&a.indices[2][0]===1&&a.indices[2][1]===1&&a.indices[3]===undefined&&a.indices[4]===undefined",
    );
    check(
        r"let a=/(((a|b))){0}(c)/d.exec('c');a[1]===undefined&&a[2]===undefined&&a[3]===undefined&&a[4]==='c'&&Object.hasOwn(a,'2')&&Object.hasOwn(a.indices,'2')&&a.indices[4][0]===0",
    );
}

#[test]
fn greedy_lazy_bounded_and_asserted_endpoints_preserve_iteration_ranges() {
    check(
        r"let a=/((a|b))+(b)/d.exec('abab'),b=/((a|b))+?(b)/d.exec('abab');a[1]==='a'&&a[2]==='a'&&a.indices[1][0]===2&&a.indices[2][0]===2&&a.indices[3][0]===3&&b[0]==='ab'&&b.indices[1][0]===0&&b.indices[2][0]===0",
    );
    check(
        r"let a=/((a|b)){1,2}(b)/d.exec('aaaab');a.index===2&&a[1]==='a'&&a[2]==='a'&&a.indices[1][0]===3&&a.indices[2][0]===3&&a.indices[3][0]===4",
    );
    check(
        r"let a=/^(((a|b))+)($)(\n)/dm.exec('x\nabab\n');a.index===2&&a[1]==='abab'&&a[2]==='b'&&a[3]==='b'&&a.indices[1][1]===6&&a.indices[2][0]===5&&a.indices[3][0]===5&&a.indices[4][0]===6&&/^(((a|b))+)($)(\n)/.exec('abab\n')===null",
    );
}

#[test]
fn alternatives_global_sticky_and_copies_preserve_wrapper_layouts_and_source() {
    check(
        r"let a=/((a|b))+a|((a|b))+b/d.exec('aba'),b=/((a|b))+a|((a|b))+b/d.exec('ab');a[1]==='b'&&a[2]==='b'&&a[3]===undefined&&a[4]===undefined&&b[1]===undefined&&b[2]===undefined&&b[3]==='a'&&b[4]==='a'&&b.indices[3][0]===0&&b.indices[4][0]===0",
    );
    check(
        r"let r=/(x)(((a|b))+)(y)/dg,copy=new RegExp(r),a=r.exec('xay xababy'),b=r.exec('xay xababy');a.indices[3][0]===1&&a.indices[4][0]===1&&b.index===4&&b.indices[2][0]===5&&b.indices[3][0]===8&&b.indices[4][0]===8&&r.lastIndex===10&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let r=/x((a|b))+(y)/dy;r.lastIndex=1;let a=r.exec('yxababy');a.index===1&&a[1]==='b'&&a[2]==='b'&&a.indices[1][0]===5&&a.indices[2][0]===5&&a.indices[3][0]===6&&r.lastIndex===7&&r.exec('yxababy')===null&&r.lastIndex===0",
    );
}

#[test]
fn decoded_characters_flags_words_and_surrogates_remain_exact() {
    check(
        r"let a=/((\x61|\u0062))+(c)/d.exec('ababc'),b=/((µ|Μ))+(x)/di.exec('µΜµx');a[1]==='b'&&a[2]==='b'&&a.indices[1][0]===3&&b[1]==='µ'&&b[2]==='µ'&&b.indices[1][0]===2&&b.indices[2][0]===2",
    );
    check(
        r"let a=/\b(((a|b))+)(c)\b/d.exec('ababc');a[1]==='abab'&&a[2]==='b'&&a[3]==='b'&&a.indices[2][0]===3&&a.indices[3][0]===3&&/\b(((a|b))+)(c)\b/.exec('ababcd')===null",
    );
    check(
        r"let a=/((\uDCA9|a))+(b)/d.exec('💩aaab');a.index===1&&a[1]==='a'&&a[2]==='a'&&a.indices[1][0]===4&&a.indices[2][0]===4&&a.indices[3][0]===5",
    );
}

#[test]
fn generic_consumers_intrinsic_arrays_and_callbacks_keep_inner_wrapper_values() {
    check(
        r"'xababy'.replace(/(x)(((a|b))+)(y)/,'<$1,$2,$3,$4,$5>')==='<x,abab,b,b,y>'&&'qxababyz'.split(/(x)(((a|b))+)(y)/).join(',')==='q,x,abab,b,b,y,z'&&'qxababy'.search(/x((a|b))+y/)===1",
    );
    check(
        r"let r=/(x)(((a|b))+)(y)/dg,a=[...'xay xababy'.matchAll(r)];a.length===2&&a[1][2]==='abab'&&a[1][3]==='b'&&a[1][4]==='b'&&a[1].indices[3][0]===8&&a[1].indices[4][0]===8&&r.lastIndex===0",
    );
    check(
        r"let seen;let s='qxababy'.replace(/(x)(((a|b))+)(y)/,(whole,prefix,run,outer,inner,sequel,index,input)=>{seen=[whole,prefix,run,outer,inner,sequel,index,input];return 'z'});s==='qz'&&seen[2]==='abab'&&seen[3]==='b'&&seen[4]==='b'&&seen[6]===1&&seen[7]==='qxababy'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'3',{set(){calls++},configurable:true});let a=/(x)(((a|b)){0})(y)/d.exec('xy');calls===0&&a[2]===''&&a[3]===undefined&&a[4]===undefined&&a.indices[3]===undefined&&Object.hasOwn(a,'3')&&Object.hasOwn(a.indices,'3')",
    );
}

#[test]
fn deep_inner_wrappers_long_runs_and_copies_survive_collection_with_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'c',r=new RegExp('('.repeat(100000)+'(a|b)'+')'.repeat(100000)+'+(c)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100003&&a[1]==='a'&&a[100000]==='a'&&a[100001]==='a'&&a[100002]==='c'&&a.indices[1][0]===199999&&a.indices[100000][0]===199999&&a.indices[100001][0]===199999&&a.indices[100002][0]===200000&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_aborts_and_branch_specific_captures_remain_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/x((a|b))+(y)/y,s='x'+'a'.repeat(20000)+'y',flag=0")
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
        r"/(?:(?:((a)|(b))+)|){2,3}/.test('ab')",
        r"/(?:(?:((a|bc)())+)|){2,3}/.test('ab')",
        r"/(?:(?:(x(a|bc))+)|){2,3}/.test('xab')",
        r"/(?:(?:((ab|a))+)|){2,3}/.test('aba')",
        r"/((a|)){2,3}/.test('a')",
        r"/(?:(?:((a|[^b]c))+)|){2,3}/.test('ab')",
        r"/(?:(?:((a|b))+(c)+)|){2,3}/.test('abc')",
        r"/(?:(?:(?<n>(a|b))+\k<n>)|){2,3}/.test('ab')",
        r"/(?:(?:((a|b))+\1)|){2,3}/.test('aa')",
        r"/((a|b))+/u.test('ab')",
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
