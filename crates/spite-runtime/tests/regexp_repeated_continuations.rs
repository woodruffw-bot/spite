//! Fixed sequels participate in repeated-group endpoint selection.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn greedy_lazy_and_bounded_groups_shorten_before_matching_fixed_sequels() {
    check(
        r"let a=/(ab)+(ab)/d.exec('ababab'),b=/(ab)+?(ab)/d.exec('ababab');a[0]==='ababab'&&a.indices[1][0]===2&&a.indices[2][0]===4&&b[0]==='abab'&&b.indices[1][0]===0&&b.indices[2][0]===2",
    );
    check(
        r"let a=/([ab][ab]){1,2}([ab]b)/d.exec('aaaaab'),b=/([ab][ab]){1,2}?([ab]b)/d.exec('aaaaab'),c=/([ab][ab]){0,1}([ab]b)/d.exec('aaaaab');a.index===0&&a[0]==='aaaaab'&&a.indices[1][0]===2&&a.indices[2][0]===4&&b[0]==='aaaaab'&&c.index===2&&c[0]==='aaab'",
    );
    check(
        r"let a=/(ab)*?(ab)/d.exec('abab'),b=/(ab)*(ab)/d.exec('abab');a[0]==='ab'&&a[1]===undefined&&a[2]==='ab'&&b[0]==='abab'&&b[1]==='ab'&&b.indices[2][0]===2",
    );
}

#[test]
fn iteration_empty_and_fixed_sequel_captures_remain_distinct() {
    check(
        r"let a=/(([ab])([ab])()){2}([ab]b)()/d.exec('aaaaab');a.length===7&&a[1]==='aa'&&a[2]==='a'&&a[3]==='a'&&a[4]===''&&a[5]==='ab'&&a[6]===''&&a.indices[1][0]===2&&a.indices[4][0]===4&&a.indices[5][0]===4&&a.indices[6][0]===6",
    );
    check(
        r"let a=/(ab){0}(a)()/d.exec('a');a[1]===undefined&&a[2]==='a'&&a[3]===''&&a.indices[1]===undefined&&a.indices[2][0]===0&&a.indices[3][0]===1&&Object.hasOwn(a,'1')&&Object.hasOwn(a.indices,'1')",
    );
    check(
        r"let a=/()*(a)/d.exec('a'),b=/()+(a)/d.exec('a'),c=/(){999999999999999999999999999999}(a)/d.exec('a');a[1]===undefined&&a[2]==='a'&&b[1]===''&&b[2]==='a'&&c[1]===''&&c.indices[1][0]===0",
    );
}

#[test]
fn outer_and_internal_assertions_filter_sequel_positions_with_explicit_multiline() {
    check(
        r"let a=/^((^a$\n){2}(a))$/dm.exec('x\na\na\na\n');a.index===2&&a[1]==='a\na\na'&&a.indices[2][0]===4&&a.indices[3][0]===6&&/^((^a$\n){2}(a))$/.exec('a\na\na')===null",
    );
    check(
        r"let a=/(a())+($)(\n)/dm.exec('aaa\n');a[0]==='aaa\n'&&a[1]==='a'&&a[2]===''&&a[3]===''&&a[4]==='\n'&&a.indices[1][0]===2&&a.indices[2][0]===3&&a.indices[3][0]===3&&a.indices[4][0]===3",
    );
    check(
        r"let a=/(^){2}(a)/dm.exec('x\na'),b=/(\b\B)*(a)/d.exec('a');a.index===2&&a[1]===''&&a[2]==='a'&&b[1]===undefined&&b[2]==='a'&&/(\b\B)+(a)/.exec('a')===null&&/\b((ab)+(ab))\b/.exec('abab')[1]==='abab'",
    );
}

#[test]
fn enclosing_alternative_slots_and_source_order_use_the_complete_match() {
    check(
        r"let a=/((ab)+(ab))|(b+)/d.exec('ababab');a[1]==='ababab'&&a[2]==='ab'&&a[3]==='ab'&&a[4]===undefined&&a.indices[2][0]===2&&a.indices[3][0]===4",
    );
    check(
        r"let a=/(ab)+a|(ab)+ab/d.exec('ababab');a.index===0&&a[0]==='ababa'&&a[1]==='ab'&&a[2]===undefined&&a.indices[1][0]===2",
    );
    check(
        r"let a=/^((ab)+(ab))$/d.exec('ababab');a[1]==='ababab'&&a[2]==='ab'&&a[3]==='ab'&&a.indices[1][1]===6&&a.indices[2][0]===2&&a.indices[3][0]===4&&/^(ab)+ab$/.exec('ababa')===null",
    );
}

#[test]
fn global_sticky_dotall_case_and_surrogates_keep_actual_utf16_sequel_offsets() {
    check(
        r"let r=/(ab)+(ab)/dg,a=r.exec('abab xababab'),b=r.exec('abab xababab');a.index===0&&a.indices[2][0]===2&&b.index===6&&b.indices[1][0]===8&&b.indices[2][0]===10&&r.lastIndex===12&&r.exec('abab xababab')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(ab)+(ab)/dy;r.lastIndex=1;let a=r.exec('xababab'),ok=a.index===1&&a.indices[1][0]===3&&a.indices[2][0]===5&&r.lastIndex===7;ok&&r.exec('xababab')===null&&r.lastIndex===0",
    );
    check(
        r"let a=/([µ][µ])+(x)/di.exec('Μµx'),b=/(.a)+([ab])/ds.exec('\na\nab'),c=/(\uD83D\uDCA9)+(\uDCA9)/d.exec('💩💩\uDCA9');a[1]==='Μµ'&&a.indices[2][0]===2&&b[1]==='\na'&&b.indices[2][0]===4&&/(.a)+([ab])/.exec('\na\nab')===null&&c[1]==='💩'&&c.indices[1][0]===2&&c.indices[2][0]===4",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_results_keep_iteration_and_sequel_slots() {
    check(
        r"'ababab'.replace(/(ab)+(ab)/,'<$1,$2>')==='<ab,ab>'&&'xabababy'.split(/(ab)+(ab)/).join(',')==='x,ab,ab,y'&&'xababab'.search(/(ab)+(ab)/)===1",
    );
    check(
        r"let seen;let s='xababab'.replace(/(ab)+(ab)/,(whole,last,sequel,index,input)=>{seen=[whole,last,sequel,index,input];return 'q'});s==='xq'&&seen[0]==='ababab'&&seen[1]==='ab'&&seen[2]==='ab'&&seen[3]===1&&seen[4]==='xababab'",
    );
    check(
        r"let r=/(ab)+(ab)$/dgm,a=[...'abab\nabababab'.matchAll(r)];a.length===2&&a[0].indices[2][0]===2&&a[1].indices[1][0]===9&&a[1].indices[2][0]===11&&r.lastIndex===0",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(ab){0}(a)()/d.exec('a');calls===0&&a[1]===undefined&&a[2]==='a'&&a[3]===''&&Object.hasOwn(a.indices,'1')&&a.indices[1]===undefined",
    );
}

#[test]
fn large_runs_and_captures_use_unlimited_defaults_and_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('('+'('.repeat(100000)+'a'+')'.repeat(100000)+')+(b)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100003&&a[100001]==='a'&&a[100002]==='b'&&a.indices[100001][0]===199999&&a.indices[100002][0]===200000&&copy.source===r.source&&/(aa)+(ab)/.exec(s).index===1&&/(aa){1,2}(ab)/.exec(s).index===199995"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_full_suffix_work_aborts_and_other_group_compositions_stay_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(ab)+(c)/y,s='ab'.repeat(10000)+'c',flag=0")
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
        r"/(?:(?:x(ab|a)+c)|)*/.test('xababc')",
        r"/(?:(?:(ab|a)+c)|)*/.test('abc')",
        r"/(?:(?:(?:(?:(ab)+(a)+){2}){2})|)*/.test('aba')",
        r"/(?:(?:(?:(?:(ab)+a+){2}){2})|)*/.test('aba')",
        r"/(?:(?:((ab|a)+)c)|)*/.test('abc')",
        r"/(?:(?:(?:(?<n>ab)+c\k<n>){2})|)*/.test('abc')",
        r"/(?:(?:(?:(ab)+\1){2})|)*/.test('abab')",
        r"/(ab)+c/u.test('abc')",
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
