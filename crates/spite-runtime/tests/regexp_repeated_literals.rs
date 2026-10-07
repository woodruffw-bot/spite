//! Literal groups repeat without expanding counts or losing last-iteration captures.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn complete_literal_groups_choose_earliest_greedy_lazy_and_bounded_runs() {
    check(
        r"let a=/(ab)+/d.exec('xaabababz'),b=/(ab)+?/d.exec('xaabababz');a.index===2&&a[0]==='ababab'&&a[1]==='ab'&&a.indices[1][0]===6&&b.index===2&&b[0]==='ab'&&b.indices[1][0]===2",
    );
    check(
        r"let a=/(ab){2,3}/d.exec('abxabababab'),b=/(ab){2,3}?/d.exec('abxabababab');a.index===3&&a[0]==='ababab'&&a.indices[1][0]===7&&b[0]==='abab'&&b.indices[1][0]===5",
    );
    check(
        r"/(aba){2}/.exec('abaabaaba')[0]==='abaaba'&&/(abab){2,}/.exec('abababababab')[0]==='abababababab'&&/(aa){3}/.exec('aaaaa')===null",
    );
    check(
        r"let a=/((a)(a)())*/d.exec('aaaaaa');a[0]==='aaaaaa'&&a[1]==='aa'&&a.indices[1][0]===4&&a.indices[4][0]===6&&/(aa)+/.exec('aaaaa')[0]==='aaaa'&&/(abab)+/.exec('abababab')[0]==='abababab'",
    );
}

#[test]
fn nested_groups_capture_last_iteration_and_zero_iterations_leave_undefined() {
    check(
        r"let a=/(((a)(b)())+)/d.exec('xabab');a[0]==='abab'&&a[1]==='abab'&&a[2]==='ab'&&a[3]==='a'&&a[4]==='b'&&a[5]===''&&a.indices[1][0]===1&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices[4][0]===4&&a.indices[5][0]===5",
    );
    check(
        r"let a=/((ab)*)/d.exec('xab'),b=/((a)(b)())*?/d.exec('abab');a[0]===''&&a[1]===''&&a[2]===undefined&&a.indices[1][0]===0&&a.indices[2]===undefined&&b[0]===''&&b[1]===undefined&&b[2]===undefined&&b[3]===undefined&&b[4]===undefined",
    );
    check(
        r"let a=/(a()b)+/d.exec('abab'),b=/(()ab())+/d.exec('abab');a[1]==='ab'&&a[2]===''&&a.indices[2][0]===3&&b.indices[2][0]===2&&b.indices[3][0]===4",
    );
    check(
        r"let a=/(ab){0}/d.exec('ab'),b=/(ab)??/d.exec('ab');a[0]===''&&a[1]===undefined&&b[0]===''&&b[1]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a.indices,'1')",
    );
}

#[test]
fn source_order_alternatives_keep_whole_branch_and_iteration_capture_offsets() {
    check(
        r"let a=/((ab){2,3})|(ab)+?/d.exec('abababab');a[0]==='ababab'&&a[1]==='ababab'&&a[2]==='ab'&&a[3]===undefined&&a.indices[1][0]===0&&a.indices[2][0]===4&&a.indices[3]===undefined",
    );
    check(
        r"let a=/((ab){2,3})|(ab)+?/d.exec('xab');a.index===1&&a[1]===undefined&&a[2]===undefined&&a[3]==='ab'&&a.indices[3][0]===1",
    );
    check(
        r"let a=/(ab)*|(ab)+/d.exec('xabab'),b=/(ab)+|x/d.exec('xabab');a[0]===''&&a[1]===undefined&&a[2]===undefined&&b[0]==='x'&&b[1]===undefined",
    );
}

#[test]
fn global_and_sticky_exec_keep_last_index_and_empty_match_rules() {
    check(
        r"let r=/(ab)+/dg,a=r.exec('xabab ab'),b=r.exec('xabab ab'),n=r.exec('xabab ab');a.index===1&&a[0]==='abab'&&a.indices[1][0]===3&&b.index===6&&b[0]==='ab'&&n===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(ab){2}/dy;r.lastIndex=1;let a=r.exec('xabab ab');a[0]==='abab'&&r.lastIndex===5&&r.exec('xabab ab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(ab)*?/gy;r.lastIndex=2;let a=r.exec('xxabab');a.index===2&&a[0]===''&&a[1]===undefined&&r.lastIndex===2",
    );
    check(r"let r=/(ab)+/;r.lastIndex=99;let a=r.exec('abab');a.index===0&&r.lastIndex===99");
}

#[test]
fn escapes_case_canonicalization_and_surrogates_use_decoded_utf16_width() {
    check(
        r"let a=/(\x61\u0062)+/di.exec('xABab'),b=/(µb)+/di.exec('ΜBµb');a[0]==='ABab'&&a[1]==='ab'&&a.indices[1][0]===3&&b[0]==='ΜBµb'&&b[1]==='µb'&&b.indices[1][0]===2",
    );
    check(
        r"/(Kb)+/i.exec('kb')===null&&/(ſb)+/i.exec('sb')===null&&/(\(\))+/.exec('()()')[0]==='()()'&&/(a\|b)+/.exec('a|ba|b')[0]==='a|ba|b'",
    );
    check(
        r"let a=/(\uD83D\uDCA9)+/d.exec('x💩💩'),b=/(\uD800a)+/d.exec('\uD800a\uD800a');a[0].length===4&&a[1].length===2&&a.indices[1][0]===3&&b[0].length===4&&b.indices[1][0]===2",
    );
    check(
        r"/(\n\r)+/ms.exec('x\n\r\n\r')[0]==='\n\r\n\r'&&/(ab){999999999999999999999999999999}/.exec('abab')===null&&/(ab){1,999999999999999999999999999999}/.exec('abab')[0]==='abab'",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_results_share_repeated_capture_slots() {
    check(
        r"'xababz'.replace(/((a)(b)())+/,'$1-$2-$3-$4')==='xab-a-b-z'&&'ababxab'.split(/(ab)+/).join(',')===',ab,x,ab,'",
    );
    check(
        r"let seen;let s='xababz'.replace(/((a)(b)())+/,(whole,last,a,b,empty,index,input)=>{seen=[whole,last,a,b,empty,index,input];return 'q'});s==='xqz'&&seen[0]==='abab'&&seen[1]==='ab'&&seen[2]==='a'&&seen[3]==='b'&&seen[4]===''&&seen[5]===1&&seen[6]==='xababz'",
    );
    check(
        r"let r=/(ab)+/dg,a=[...'abab xab'.matchAll(r)];a.length===2&&a[0].indices[1][0]===2&&a[1].index===6&&r.lastIndex===0&&'xab'.search(/(ab)+/)===1&&'abab xab'.match(/(ab)+/g).join(',')==='abab,ab'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(ab)*/d.exec('x');calls===0&&a[1]===undefined&&Object.hasOwn(a,'1')&&a.indices[1]===undefined&&Object.hasOwn(a.indices,'1')",
    );
}

#[test]
fn cloned_plans_large_minimums_and_last_iteration_captures_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=/((a)(b)()){100000}/dg,copy=new RegExp(r),s='ab'.repeat(99999)+'x'+'ab'.repeat(100000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a.index===199999&&a[0].length===200000&&a[1]==='ab'&&a[2]==='a'&&a[3]==='b'&&a[4]===''&&a.indices[1][0]===399997&&a.indices[4][0]===399999&&copy.lastIndex===399999&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_work_covers_sticky_repetitions_and_remaining_groups_are_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(ab)+/y,s='ab'.repeat(30000),flag=0")
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
        r"/^(ab|a)+$/.test('ab')",
        r"/(ab)+c/.test('abc')",
        r"/c(ab)+/.test('cab')",
        r"/(a[bc]|d)+/.test('ab')",
        r"/(ab|cd)+/.test('ab')",
        r"/(a\bb|c)+/.test('ab')",
        r"/(ab$|c)+/.test('ab')",
        r"/(ab)+(cd)+/.test('abcd')",
        r"/(?<n>ab)+/.test('ab')",
        r"/(ab)\1/.test('abab')",
        r"/(ab)+/u.test('ab')",
        r"/()*/.test('')",
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
