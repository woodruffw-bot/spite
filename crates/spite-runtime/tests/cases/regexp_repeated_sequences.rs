//! Fixed class/dot groups repeat with complete iterations and last-iteration captures.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn repeated_fixed_groups_choose_earliest_greedy_lazy_and_bounded_runs() {
    check(
        r"let a=/(a[bc])+/d.exec('xababac'),b=/(a[bc])+?/d.exec('xababac');a.index===1&&a[0]==='ababac'&&a[1]==='ac'&&a.indices[1][0]===5&&b.index===1&&b[0]==='ab'&&b.indices[1][0]===1",
    );
    check(
        r"let a=/(a[bc]){2,3}/d.exec('abxabacacab'),b=/(a[bc]){2,3}?/d.exec('abxabacacab');a.index===3&&a[0]==='abacac'&&a[1]==='ac'&&a.indices[1][0]===7&&b[0]==='abac'&&b.indices[1][0]===5",
    );
    check(
        r"/([ab][ab])+/.exec('abababab')[0]==='abababab'&&/([ab][ab]){3}/.exec('aaaaa')===null&&/([ab][ab]){3}/.exec('aaaaaa')[0]==='aaaaaa'&&/(a[ab]){2}/.exec('baabaaba')[0]==='abaa'",
    );
}

#[test]
fn nested_fixed_groups_capture_last_iteration_and_empty_groups_keep_exact_slots() {
    check(
        r"let a=/(((a)([bc])())+)/d.exec('xabac');a[0]==='abac'&&a[1]==='abac'&&a[2]==='ac'&&a[3]==='a'&&a[4]==='c'&&a[5]===''&&a.indices[1][0]===1&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices[4][0]===4&&a.indices[5][0]===5",
    );
    check(
        r"let a=/((a[b])*)/d.exec('xab'),b=/((a)([bc])())*?/d.exec('abac');a[0]===''&&a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[2]===undefined&&b[0]===''&&b[1]===undefined&&b[2]===undefined&&b[3]===undefined&&b[4]===undefined",
    );
    check(r"let a=/(a()[bc])+/d.exec('abac');a[1]==='ac'&&a[2]===''&&a.indices[2][0]===3");
    check(
        r"let a=/([]a)*/d.exec('aaa'),b=/([]a)+/d.exec('aaa');a[0]===''&&a[1]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a.indices,'1')&&b===null",
    );
}

#[test]
fn complete_branch_captures_and_alternatives_keep_source_order_and_global_offsets() {
    check(
        r"let a=/((a[b]){2,3})|(a[bc])+?/d.exec('abababab');a[0]==='ababab'&&a[1]==='ababab'&&a[2]==='ab'&&a[3]===undefined&&a.indices[1][0]===0&&a.indices[2][0]===4&&a.indices[3]===undefined",
    );
    check(
        r"let a=/((a[b]){2,3})|(a[bc])+?/d.exec('xac');a.index===1&&a[1]===undefined&&a[2]===undefined&&a[3]==='ac'&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(a[b])*|(a[bc])+/d.exec('xabab'),b=/(a[b])+|x/d.exec('xabab');a[0]===''&&a[1]===undefined&&a[2]===undefined&&b[0]==='x'&&b[1]===undefined",
    );
}

#[test]
fn global_sticky_exec_and_empty_iterations_preserve_last_index_rules() {
    check(
        r"let r=/(a[bc])+/dg,a=r.exec('xabac ab'),b=r.exec('xabac ab'),n=r.exec('xabac ab');a.index===1&&a[0]==='abac'&&a[1]==='ac'&&a.indices[1][0]===3&&b.index===6&&b[0]==='ab'&&n===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a[bc]){2}/dy;r.lastIndex=1;let a=r.exec('xabac ab');a[0]==='abac'&&r.lastIndex===5&&r.exec('xabac ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a[b])*?/gy;r.lastIndex=2;let a=r.exec('xxabab');a.index===2&&a[0]===''&&a[1]===undefined&&r.lastIndex===2",
    );
    check(r"let r=/(a[b])+/;r.lastIndex=99;let a=r.exec('abab');a.index===0&&r.lastIndex===99");
}

#[test]
fn dotall_classes_pinned_case_and_surrogates_use_each_iterations_actual_width() {
    check(
        r"let a=/(\x61[b])+/di.exec('xABab'),b=/([µ]b)+/di.exec('ΜBµb');a[0]==='ABab'&&a[1]==='ab'&&a.indices[1][0]===3&&b[0]==='ΜBµb'&&b[1]==='µb'&&b.indices[1][0]===2",
    );
    check(
        r"/([K]b)+/i.exec('kb')===null&&/([ſ]b)+/i.exec('sb')===null&&/(\d\w)+/.exec('1a2b')[0]==='1a2b'&&/(\s\S)+/.exec(' a\nb')[0]===' a\nb'",
    );
    check(
        r"let a=/([\uD83D][\uDCA9])+/d.exec('x💩💩'),b=/([\uD800]a)+/d.exec('\uD800a\uD800a');a[0].length===4&&a[1].length===2&&a.indices[1][0]===3&&b[0].length===4&&b.indices[1][0]===2",
    );
    check(
        r"/(.[ab]){2}/.exec('\nabb')===null&&/(.[ab]){2}/s.exec('\nabb')[0]==='\nabb'&&/(a[^])+/.exec('a\na\r')[0]==='a\na\r'&&/(a[b]){999999999999999999999999999999}/.exec('abab')===null&&/(a[b]){1,999999999999999999999999999999}/.exec('abab')[0]==='abab'",
    );
}

#[test]
fn generic_callbacks_consumers_and_intrinsic_arrays_keep_fixed_group_capture_slots() {
    check(
        r"'xabacz'.replace(/((a)([bc])())+/,'$1-$2-$3-$4')==='xac-a-c-z'&&'abacxab'.split(/(a[bc])+/).join(',')===',ac,x,ab,'",
    );
    check(
        r"let seen;let s='xabacz'.replace(/((a)([bc])())+/,(whole,last,a,b,empty,index,input)=>{seen=[whole,last,a,b,empty,index,input];return 'q'});s==='xqz'&&seen[0]==='abac'&&seen[1]==='ac'&&seen[2]==='a'&&seen[3]==='c'&&seen[4]===''&&seen[5]===1&&seen[6]==='xabacz'",
    );
    check(
        r"let r=/(a[bc])+/dg,a=[...'abac xab'.matchAll(r)];a.length===2&&a[0].indices[1][0]===2&&a[1].index===6&&r.lastIndex===0&&'xab'.search(/(a[b])+/)===1&&'abac xab'.match(/(a[bc])+/g).join(',')==='abac,ab'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(a[b])*/d.exec('x');calls===0&&a[1]===undefined&&Object.hasOwn(a,'1')&&a.indices[1]===undefined&&Object.hasOwn(a.indices,'1')",
    );
}

#[test]
fn large_minimums_shared_sets_copies_and_captures_survive_collection_with_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval("let r=/((a)([bc])()){100000}/dg,copy=new RegExp(r),s='ab'.repeat(99999)+'x'+'ac'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a.index===199999&&a[0].length===200000&&a[1]==='ac'&&a[2]==='a'&&a[3]==='c'&&a[4]===''&&a.indices[1][0]===399997&&a.indices[4][0]===399999&&copy.lastIndex===399999&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_work_covers_full_sticky_runs_and_remaining_group_features_stay_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(300000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(a[bc])+/y,s='ac'.repeat(100000),flag=0")
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
        r"/(?:(?:^(a[b]|c)+$)|){2,3}/.test('ab')",
        r"/(?:(?:(a[b]|a)+c)|){2,3}/.test('abc')",
        r"/(?:(?:c(a[b]|c)+)|){2,3}/.test('cab')",
        r"/(?:(?:(ab|cd)+)|){2,3}/.test('ab')",
        r"/(?:(?:(a\bb|c)+)|){2,3}/.test('ab')",
        r"/(?:(?:(a[b]$|c)+)|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:(a[b])+(cd)+){2}){2})|){2,3}/.test('abcd')",
        r"/(?:(?:(?:(?<n>a[b])+\k<n>){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:(a[b])(?:\1)+){2})|){2,3}/.test('abab')",
        r"/(a[b])+/u.test('ab')",
        r"/(|a){2,3}/.test('')",
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
