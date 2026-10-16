//! Fixed input/line assertions constrain one quantified atom before endpoint selection.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn line_assertion_continuations_select_earliest_greedy_lazy_and_bounded_repetitions() {
    check(
        r"let a=/(a)+($)/dm.exec('aaa\nx'),b=/(a)+?($)/dm.exec('aaaa\n');a[0]==='aaa'&&a[1]==='a'&&a[2]===''&&a.indices[1][0]===2&&a.indices[2][0]===3&&b[0]==='aaaa'&&b.indices[1][0]===3",
    );
    check(
        r"let a=/(a)+($)/d.exec('aaa\naa'),b=/a{1,2}($)/d.exec('aaaa');a.index===4&&a[0]==='aa'&&a.indices[2][0]===6&&b.index===2&&b[0]==='aa'&&b.indices[1][0]===4",
    );
    check(
        r"let a=/(a)*?(^)/d.exec('aaa');a[0]===''&&a[1]===undefined&&a[2]===''&&a.indices[1]===undefined&&a.indices[2][0]===0",
    );
}

#[test]
fn assertion_only_prefixes_filter_starts_without_losing_empty_captures() {
    check(
        r"let a=/(^)(a)+($)/dm.exec('x\naaa\nx');a.index===2&&a[0]==='aaa'&&a[1]===''&&a[2]==='a'&&a[3]===''&&a.indices[1][0]===2&&a.indices[2][0]===4&&a.indices[3][0]===5",
    );
    check(
        r"/(^)(a)+($)/.exec('x\naaa')===null&&/(^)(a)+($)/.exec('aaa')[1]===''&&/(^\b)(a)+(\b$)/.exec('aaa')[0]==='aaa'",
    );
    check(
        r"let a=/(a$)(\n^)(b)+($)/dm.exec('xa\nbbb\ny');a.index===1&&a[0]==='a\nbbb'&&a[1]==='a'&&a[2]==='\n'&&a[3]==='b'&&a[4]===''&&a.indices[3][0]===5&&a.indices[4][0]===6",
    );
}

#[test]
fn consuming_line_continuations_keep_source_order_and_exact_crlf_positions() {
    check(
        r"let a=/(a)+($)(\n)(^)(b)/dm.exec('xaaa\nb');a.index===1&&a[0]==='aaa\nb'&&a[1]==='a'&&a[2]===''&&a[3]==='\n'&&a[4]===''&&a[5]==='b'&&a.indices[1][0]===3&&a.indices[4][0]===5",
    );
    check(
        r"let a=/(a)+($)(\r)($)(\n)/dm.exec('aaa\r\n');a[2]===''&&a[4]===''&&a.indices[2][0]===3&&a.indices[4][0]===4&&a.indices[5][1]===5",
    );
    check(
        r"let a=/(a)+($)(\u2028)(^)(b)/dm.exec('aa\u2028b');a[3]==='\u2028'&&a[4]===''&&a[5]==='b'&&a.indices[5][0]===3",
    );
}

#[test]
fn outer_assertions_complete_groups_and_alternatives_share_constrained_endpoints() {
    check(
        r"let a=/^((^a+))$/d.exec('aaa'),b=/^((a+($)))$/d.exec('aaa');a[1]==='aaa'&&a[2]==='aaa'&&a.indices[2][1]===3&&b[1]==='aaa'&&b[2]==='aaa'&&b[3]===''&&b.indices[3][0]===3",
    );
    check(
        r"let a=/(a+($))|(a+)/dm.exec('aaa\n');a[1]==='aaa'&&a[2]===''&&a[3]===undefined&&Object.hasOwn(a,'3')&&a.indices[3]===undefined",
    );
    check(
        r"let a=/(a+($))|(a+)/d.exec('aaa\n');a[1]===undefined&&a[2]===undefined&&a[3]==='aaa'&&a.indices[3][0]===0",
    );
}

#[test]
fn global_sticky_flags_and_surrogate_offsets_preserve_complete_input_boundaries() {
    check(
        r"let r=/(^)(a)+($)/dmy;r.lastIndex=2;let a=r.exec('x\naaa\nx');a.index===2&&r.lastIndex===5&&a.indices[1][0]===2&&r.exec('x\naaa\nx')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)+($)/dg,a=r.exec('xaa aa'),b=r.exec('xaa aa');a.index===4&&a[0]==='aa'&&r.lastIndex===0&&b===null",
    );
    check(
        r"let a=/([µ]$)(\n^)(a)+/dmi.exec('Μ\nAAA');a[1]==='Μ'&&a[2]==='\n'&&a[3]==='A'&&a.indices[3][0]===4",
    );
    check(
        r"let a=/(\uDCA9$)(\n^)(a)+/dm.exec('💩\naa');a.index===1&&a[1]==='\uDCA9'&&a[2]==='\n'&&a[3]==='a'&&a.indices[3][0]===4",
    );
    check(
        r"/[^]+?(^)(a)/m.exec('x\nay')[0]==='x\na'&&/.+?(^)(a)/ms.exec('x\nay')[0]==='x\na'&&/.+?(^)(a)/m.exec('x\nay')===null&&/[^]+?(^)(a)/.exec('x\nay')===null",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_results_keep_line_assertion_slots() {
    check(
        r"'xaaa\nb'.replace(/(a)+($)(\n)(^)(b)/m,'$1-$5')==='xa-b'&&'aa\nbxaa\nb'.split(/(a)+($)(\n)(^)(b)/m).join(',')===',a,,\n,,b,x,a,,\n,,b,'",
    );
    check(
        r"let seen;let s='aaa\nb'.replace(/(a)+($)(\n)(^)(b)/m,(whole,a,end,nl,start,b,index,input)=>{seen=[whole,a,end,nl,start,b,index,input];return 'x'});s==='x'&&seen[1]==='a'&&seen[2]===''&&seen[4]===''&&seen[6]===0&&seen[7]==='aaa\nb'",
    );
    check(
        r"let r=/(^)(a)+($)/dgm,a=[...'aa\na'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].indices[1][0]===3&&a[1].indices[2][0]===3&&r.lastIndex===0",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(^)(a)*($)/dm.exec('\n');calls===0&&a[1]===''&&a[2]===undefined&&Object.hasOwn(a,'2')&&a.indices[2]===undefined&&Object.hasOwn(a.indices,'2')&&a[3]===''",
    );
}

#[test]
fn flat_prefix_and_suffix_assertions_copies_and_long_runs_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('+'^'.repeat(100000)+')(a)+('+'$'.repeat(100000)+')','dgm'),copy=new RegExp(r),s='x\\n'+'a'.repeat(100000)+'\\nx'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a.index===2&&a[0].length===100000&&a[1]===''&&a[2]==='a'&&a[3]===''&&a.indices[1][0]===2&&a.indices[2][0]===100001&&a.indices[3][0]===100002&&copy.lastIndex===100002&&r.lastIndex===0&&r.source===copy.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_counts_full_sticky_suffixes_and_unsupported_repeated_atoms() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(^)(a)+($)/my,s='a'.repeat(20000),flag=0")
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
        r"/(?:(?:(?:(?:a+($)b+){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(a$|b)+)|){2,3}/m.test('a')",
        r"/(?:(?:(a^|b)+)|){2,3}/m.test('a')",
        r"/a+(?<=(b+))/.test('ab')",
        r"/(?:(?:(?:a+(?<n>b)\k<n>){2})|){2,3}/.test('ab')",
        r"/a+($)/u.test('a')",
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
