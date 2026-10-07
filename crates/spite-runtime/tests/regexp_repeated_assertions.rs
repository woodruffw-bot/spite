//! Every consuming iteration checks word/input/line assertions against complete input.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn consuming_word_assertions_choose_earliest_greedy_lazy_and_bounded_iterations() {
    check(
        r"let a=/(\Ba)+/d.exec('aaa'),b=/(\ba)+/d.exec('aaa');a.index===1&&a[0]==='aa'&&a.indices[1][0]===2&&b.index===0&&b[0]==='a'&&b.indices[1][0]===0",
    );
    check(
        r"let a=/(a\B)+/d.exec('aaaa'),b=/(a\B){2,3}?/d.exec('aaaa');a[0]==='aaa'&&a.indices[1][0]===2&&b[0]==='aa'&&b.indices[1][0]===1&&/(a\B){4}/.exec('aaaa')===null",
    );
    check(
        r"let a=/(a\b)+/d.exec('aa aa');a.index===1&&a[0]==='a'&&a.indices[1][0]===1&&/(\b\Ba)+/.exec('a')===null&&/(\b\Ba)*/.exec('a')[1]===undefined",
    );
}

#[test]
fn consuming_single_unit_groups_keep_empty_and_undefined_last_iteration_captures() {
    check(
        r"let a=/(a())+/d.exec('aaa'),b=/(()a())+/d.exec('aaa');a[0]==='aaa'&&a[1]==='a'&&a[2]===''&&a.indices[1][0]===2&&a.indices[2][0]===3&&b[1]==='a'&&b.indices[2][0]===2&&b.indices[3][0]===3",
    );
    check(
        r"let a=/((a())*)/d.exec('aaa'),b=/((a())*)/d.exec('x');a[1]==='aaa'&&a[2]==='a'&&a[3]===''&&a.indices[2][0]===2&&b[0]===''&&b[1]===''&&b[2]===undefined&&b[3]===undefined&&b.indices[1][0]===0&&b.indices[2]===undefined",
    );
    check(
        r"let a=/(()a())*?/d.exec('aaa');a[0]===''&&a[1]===undefined&&a[2]===undefined&&a[3]===undefined&&Object.hasOwn(a,'3')&&Object.hasOwn(a.indices,'3')",
    );
}

#[test]
fn multiline_assertions_filter_each_complete_iteration_and_preserve_crlf_positions() {
    check(
        r"let a=/(^a$\n){2,3}/dm.exec('x\na\na\na\n'),b=/(^a$\n){2,3}?/dm.exec('x\na\na\na\n');a.index===2&&a[0]==='a\na\na\n'&&a.indices[1][0]===6&&b[0]==='a\na\n'&&b.indices[1][0]===4&&/(^a$\n)+/.exec('a\n')===null",
    );
    check(
        r"let a=/((a)($)(\r)($)(\n))+/dm.exec('a\r\na\r\n');a[0]==='a\r\na\r\n'&&a[1]==='a\r\n'&&a[2]==='a'&&a[3]===''&&a[5]===''&&a.indices[1][0]===3&&a.indices[3][0]===4&&a.indices[5][0]===5&&a.indices[6][0]===5",
    );
    check(
        r"let a=/(^a$\u2028)+/dm.exec('a\u2028a\u2028');a[0].length===4&&a.indices[1][0]===2&&/(a$)+/m.exec('aa\n').index===1&&/(a^)+/m.exec('aaa')===null",
    );
}

#[test]
fn enclosing_captures_and_top_level_alternatives_keep_complete_and_iteration_slots() {
    check(
        r"let a=/((a\B)+)|(a\b)+/d.exec('aaa');a[0]==='aa'&&a[1]==='aa'&&a[2]==='a'&&a[3]===undefined&&a.indices[1][0]===0&&a.indices[2][0]===1&&a.indices[3]===undefined",
    );
    check(
        r"let a=/((a\B)+)|(a\b)+/d.exec(' a'),b=/(\ba)+|a+/d.exec('aaa');a.index===1&&a[1]===undefined&&a[2]===undefined&&a[3]==='a'&&b[0]==='a'&&b[1]==='a'",
    );
    check(
        r"let a=/(((^)(a)($)(\n))+)/dm.exec('a\na\n');a[1]==='a\na\n'&&a[2]==='a\n'&&a[3]===''&&a[4]==='a'&&a[5]===''&&a[6]==='\n'&&a.indices[2][0]===2&&a.indices[3][0]===2&&a.indices[5][0]===3&&a.indices[6][0]===3",
    );
}

#[test]
fn global_sticky_flags_dotall_pinned_case_and_surrogates_keep_complete_input_neighbors() {
    check(
        r"let r=/(a\B)+/dg,a=r.exec('aaaa aa'),b=r.exec('aaaa aa'),n=r.exec('aaaa aa');a.index===0&&a[0]==='aaa'&&a.indices[1][0]===2&&b.index===5&&b[0]==='a'&&n===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(\Ba)+/dy;r.lastIndex=1;let a=r.exec('aaa');a.index===1&&a[0]==='aa'&&r.lastIndex===3&&r.exec('aaa')===null&&r.lastIndex===0&&r.exec('aaa')===null",
    );
    check(
        r"let r=/(\ba)*?/gy;r.lastIndex=1;let a=r.exec('aaa');a.index===1&&a[0]===''&&a[1]===undefined&&r.lastIndex===1",
    );
    check(
        r"let a=/([µ]\B)+/di.exec('Μµ '),b=/(\uDCA9\B)+/d.exec('💩 ');a[0]==='Μµ'&&a[1]==='µ'&&a.indices[1][0]===1&&b.index===1&&b[1]==='\uDCA9'&&b.indices[1][0]===1&&/(.$\n)+/m.exec('\n\n')===null&&/(.$\n)+/ms.exec('\n\n')[0]==='\n\n'",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_results_keep_assertion_capture_slots() {
    check(
        r"'aaa'.replace(/(a\B)+/,'<$1>')==='<a>a'&&'aa aa'.split(/(a\b)+/).join(',')==='a,a, a,a,'",
    );
    check(
        r"let seen;let s='xaaa'.replace(/(a())+/,(whole,last,empty,index,input)=>{seen=[whole,last,empty,index,input];return 'q'});s==='xq'&&seen[0]==='aaa'&&seen[1]==='a'&&seen[2]===''&&seen[3]===1&&seen[4]==='xaaa'",
    );
    check(
        r"let r=/(a\B)+/dg,a=[...'aaaa aa'.matchAll(r)];a.length===2&&a[0].indices[1][0]===2&&a[1].indices[1][0]===5&&r.lastIndex===0&&'xaaa'.search(/(a())+/)===1",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(a())*/d.exec('x');calls===0&&a[1]===undefined&&a[2]===undefined&&Object.hasOwn(a,'2')&&a.indices[2]===undefined&&Object.hasOwn(a.indices,'2')",
    );
}

#[test]
fn collapsed_assertion_captures_copies_and_long_multiline_runs_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('+'(^)'.repeat(100000)+'a$\\n)+','dm'),copy=new RegExp(r),s='a\\n'.repeat(20000)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0].length===40000&&a.length===100002&&a[1]==='a\\n'&&a[2]===''&&a[100001]===''&&a.indices[1][0]===39998&&a.indices[2][0]===39998&&a.indices[100001][0]===39998&&r.source===copy.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_assertion_work_aborts_bypass_handlers_and_remaining_group_features_are_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(^a$\\n)+/my,s='a\\n'.repeat(10000),flag=0")
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
        r"/^(a()|b)+$/.test('aaa')",
        r"/\b(ab|a)+\b/.test('ab')",
        r"/(a$|b)+b/.test('ab')",
        r"/b(a$|c)+/.test('ba')",
        r"/(a$|b)+/.test('a')",
        r"/(a\B)+b+/.test('aaab')",
        r"/(?<n>a$)+\k<n>/.test('a')",
        r"/(?:(a$)(?:\1)+){2}/.test('aa')",
        r"/(a$)+/u.test('a')",
        r"/(^|$)*/.test('')",
        r"/($|^)+/.test('')",
        r"/(\b|\B){2}/.test('')",
        r"/(|a)*/.test('')",
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
