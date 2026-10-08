//! Outer assertions constrain repeated fixed groups before endpoint selection.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn earliest_overlapping_starts_precede_greedy_lazy_and_bounded_endpoint_selection() {
    check(
        r"let a=/([ab][ab])+$/.exec('aaaaa'),b=/([ab][ab])*$/.exec('aaaaa');a.index===1&&a[0]==='aaaa'&&b.index===1&&b[0]==='aaaa'",
    );
    check(
        r"let a=/([ab][ab]){1,2}$/d.exec('aaaaaa'),b=/([ab][ab]){1,2}?$/d.exec('aaaaaa');a.index===2&&a[0]==='aaaa'&&a.indices[1][0]===4&&b.index===2&&b[0]==='aaaa'",
    );
    check(
        r"let a=/([ab][ab])*?$/d.exec('aaaa');a.index===0&&a[0]==='aaaa'&&a.indices[1][0]===2&&/^(ab)+$/.exec('abab')[1]==='ab'&&/^(ab)+$/.exec('ababa')===null",
    );
}

#[test]
fn word_assertions_choose_complete_iterations_and_preserve_earliest_empty_matches() {
    check(
        r"let a=/\b([ab][ab])*\b/d.exec('aaaaa'),b=/\b([ab][ab])*?\b/d.exec('aaaa');a.index===0&&a[0]===''&&a[1]===undefined&&a.indices[1]===undefined&&b[0]===''",
    );
    check(
        r"let a=/\B([ab][ab])+\b/d.exec('aaaaa'),b=/\b([ab][ab])+\B/d.exec('aaaaa');a.index===1&&a[0]==='aaaa'&&a.indices[1][0]===3&&b.index===0&&b[0]==='aaaa'&&/\b\B(ab)*$/.exec('ab')===null",
    );
    check(
        r"let a=/\b(a\B)*\B/d.exec('aaaa');a.index===0&&a[0]==='aaa'&&a.indices[1][0]===2&&/\b(\Ba)+\b/.exec('aaa')===null",
    );
}

#[test]
fn multiline_outer_and_iteration_assertions_keep_crlf_and_complete_input_positions() {
    check(
        r"let a=/^((^a$\n)+)$/dm.exec('x\na\na\n');a.index===2&&a[1]==='a\na\n'&&a[2]==='a\n'&&a.indices[2][0]===4&&/^((^a$\n)+)$/.exec('a\n')===null",
    );
    check(
        r"let a=/(a$\r$\n)+$/dm.exec('a\r\na\r\n');a[0]==='a\r\na\r\n'&&a[1]==='a\r\n'&&a.indices[1][0]===3",
    );
    check(
        r"let a=/^(ab)+$/dm.exec('x\u2028abab\u2029y');a.index===2&&a[0]==='abab'&&a.indices[1][0]===4&&/^(ab)+$/.exec('abab\n')===null",
    );
}

#[test]
fn enclosing_alternative_and_zero_iteration_captures_keep_specified_slots() {
    check(
        r"let a=/^((a())*)$/d.exec('aaa'),b=/^((a())*)$/d.exec('');a[1]==='aaa'&&a[2]==='a'&&a[3]===''&&a.indices[2][0]===2&&b[1]===''&&b[2]===undefined&&b[3]===undefined&&Object.hasOwn(b,'3')&&Object.hasOwn(b.indices,'3')",
    );
    check(
        r"let a=/((ab)+$)|(a+)$/d.exec('abab'),b=/((ab)+$)|(a+)$/d.exec('aaa');a[1]==='abab'&&a[2]==='ab'&&a[3]===undefined&&a.indices[2][0]===2&&b[1]===undefined&&b[2]===undefined&&b[3]==='aaa'",
    );
    check(
        r"let a=/(ab)+$|b+$/d.exec('abb'),b=/b+$|(ab)+$/d.exec('abab');a.index===1&&a[0]==='bb'&&a[1]===undefined&&b.index===0&&b[0]==='abab'&&b[1]==='ab'",
    );
}

#[test]
fn global_sticky_original_flags_and_utf16_positions_survive_asserted_group_search() {
    check(
        r"let r=/(ab)+$/dgm,a=r.exec('abab\nab'),b=r.exec('abab\nab');a.index===0&&a.indices[1][0]===2&&b.index===5&&r.lastIndex===7&&r.exec('abab\nab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/([ab][ab])+$/dy;r.lastIndex=1;let a=r.exec('aaaaa');a.index===1&&a.indices[1][0]===3&&r.lastIndex===5&&r.exec('aaaaa')===null&&r.lastIndex===0&&r.exec('aaaaa')===null",
    );
    check(
        r"let a=/^([µ][µ])+$/di.exec('Μµ'),b=/(\uDCA9\B)+$/d.exec('💩');a[0]==='Μµ'&&a[1]==='Μµ'&&b.index===1&&b.indices[1][0]===1&&/^(.\n)+$/.exec('\n\n')===null&&/^(.\n)+$/s.exec('\n\n')[0]==='\n\n'",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_use_final_iteration_captures() {
    check(
        r"'abab'.replace(/(ab)+$/,'<$1>')==='<ab>'&&'xabab'.search(/(ab)+$/)===1&&'abab'.split(/(ab)+$/).join(',')===',ab,'",
    );
    check(
        r"let seen;let s='xaaa'.replace(/^(a())+$/m,(whole,last,empty,index,input)=>{seen=[whole,last,empty,index,input];return 'q'});s==='xaaa'&&seen===undefined;let t='x\naaa'.replace(/^(a())+$/m,(whole,last,empty,index,input)=>{seen=[whole,last,empty,index,input];return 'q'});t==='x\nq'&&seen[0]==='aaa'&&seen[1]==='a'&&seen[2]===''&&seen[3]===2&&seen[4]==='x\naaa'",
    );
    check(
        r"let r=/(ab)+$/dgm,a=[...'abab\nab'.matchAll(r)];a.length===2&&a[0].indices[1][0]===2&&a[1].indices[1][0]===5&&r.lastIndex===0",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/^((a())*)$/d.exec('');calls===0&&a[1]===''&&a[2]===undefined&&a[3]===undefined&&Object.hasOwn(a,'3')&&Object.hasOwn(a.indices,'3')",
    );
}

#[test]
fn unlimited_defaults_preserve_long_asserted_runs_deep_captures_copies_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000),bad=s+'x',r=new RegExp('^'+'('.repeat(100000)+'(a())+'+')'.repeat(100000)+'$','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0].length===200000&&a.length===100003&&a[100000]===s&&a[100001]==='a'&&a[100002]===''&&a.indices[100001][0]===199999&&a.indices[100002][0]===200000&&r.source===copy.source&&/([ab][ab])*?$/.exec(bad).index===200001&&/([ab][ab]){1,2}$/.exec(s).index===199996"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_work_aborts_and_remaining_group_compositions_stay_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(ab)+$/y,s='ab'.repeat(10000),flag=0")
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
        r"/^(ab|a)+b$/.test('abb')",
        r"/^b(ab|a)+$/.test('bab')",
        r"/^(ab|a)+$/.test('ab')",
        r"/^(ab)+(a)+$/.test('aba')",
        r"/(?:^(?<n>ab)+$\k<n>){2}/.test('ab')",
        r"/(?:^(ab)(?:\1)+$){2}/.test('abab')",
        r"/^(ab)+$/u.test('ab')",
        r"/^(|a)*$/.test('')",
        r"/^(^|$)+$/.test('')",
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
