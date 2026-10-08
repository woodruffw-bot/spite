//! Ordinary word assertions retain fixed UTF-16 offsets across consuming plans.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn assertions_inside_fixed_capturing_groups_preserve_consuming_and_empty_slots() {
    check(
        r"let a=/((a)(\B)(b))()/d.exec('xab');a.index===1&&a.length===6&&a[1]==='ab'&&a[2]==='a'&&a[3]===''&&a[4]==='b'&&a[5]===''&&a.indices[3][0]===2&&a.indices[5][0]===3",
    );
    check(
        r"let a=/-(\b)()a(\b)-/d.exec(' -a- ');a.index===1&&a[1]===''&&a[2]===''&&a[3]===''&&a.indices[1][0]===2&&a.indices[3][0]===3&&/a\bb/.exec('ab')===null",
    );
    check(
        r"let a=/([a]\B)([0-9_])/d.exec('a_');a[1]==='a'&&a[2]==='_'&&a.indices[2][0]===1&&/a(\b\B)b/.exec('ab')===null",
    );
}

#[test]
fn prefix_assertions_filter_overlapping_and_zero_width_prefix_candidates() {
    check(
        r"let a=/(a)(\B)(b)+(\b)/d.exec('aabb');a.index===1&&a[0]==='abb'&&a[1]==='a'&&a[2]===''&&a[3]==='b'&&a[4]===''&&a.indices[2][0]===2&&a.indices[3][0]===3&&a.indices[4][0]===4",
    );
    check(
        r"let a=/(\B)(a)+/d.exec('baaa');a.index===1&&a[0]==='aaa'&&a[1]===''&&a[2]==='a'&&a.indices[1][0]===1&&a.indices[2][0]===3&&/(\b)(a)+/.exec('baaa')===null",
    );
    check(
        r"let a=/(a\B)(b)+?/d.exec('abb');a[0]==='ab'&&a[1]==='a'&&a[2]==='b'&&a.indices[2][0]===1&&/a\bb+/.exec('ab')===null",
    );
}

#[test]
fn continuation_assertions_constrain_repetition_before_greedy_lazy_selection() {
    check(
        r"let a=/(a)+(\B)/d.exec('aaaa'),b=/(a)+?(\B)/d.exec('aaaa');a[0]==='aaa'&&a[1]==='a'&&a[2]===''&&a.indices[1][0]===2&&a.indices[2][0]===3&&b[0]==='a'&&b.indices[2][0]===1",
    );
    check(
        r"let a=/(a)+(\B)(a)/d.exec('aaaa'),b=/(a)+?(\B)(a)/d.exec('aaaa');a[0]==='aaaa'&&a[1]==='a'&&a[2]===''&&a[3]==='a'&&a.indices[2][0]===3&&b[0]==='aa'&&b.indices[2][0]===1",
    );
    check(
        r"let a=/(a*)(\b)/d.exec('aaa');a[1]==='aaa'&&a[2]===''&&a.indices[2][0]===3&&/(a)+(\b)b/.exec('aaab')===null",
    );
}

#[test]
fn outer_anchors_groups_branches_and_sticky_state_reuse_fixed_assertion_plans() {
    check(
        r"let a=/^((\ba+\b))$/d.exec('aaa');a[1]==='aaa'&&a[2]==='aaa'&&a.indices[2][0]===0&&a.indices[2][1]===3",
    );
    check(
        r"let a=/^((a\Bb))$/d.exec('ab');a[1]==='ab'&&a[2]==='ab'&&a.indices[2][1]===2&&/^((\ba\b))$/.test('a')",
    );
    check(
        r"let a=/(a\Bb)|(ab)/d.exec('xab');a.index===1&&a[1]==='ab'&&a[2]===undefined&&Object.hasOwn(a,'2')&&a.indices[2]===undefined",
    );
    check(
        r"let r=/(a\B)(b)+/dy;r.lastIndex=1;let a=r.exec('xabbb');a.index===1&&a[1]==='a'&&a[2]==='b'&&a.indices[2][0]===4&&r.lastIndex===5&&r.exec('xabbb')===null&&r.lastIndex===0",
    );
}

#[test]
fn word_assertion_offsets_keep_pinned_flags_and_individual_surrogate_units() {
    check(
        r"let a=/(µ\b)(a)/di.exec('ΜA');a[1]==='Μ'&&a[2]==='A'&&a.indices[2][0]===1&&/(ſ\b)(a)/i.exec('ſA')[1]==='ſ'&&/(K\b)(a)/i.exec('KA')[1]==='K'",
    );
    check(
        r"let a=/(\uD83D\B)(\uDCA9\b)(a)/d.exec('💩a');a[0].length===3&&a[1]==='\uD83D'&&a[2]==='\uDCA9'&&a[3]==='a'&&a.indices[2][0]===1&&a.indices[3][0]===2",
    );
    check(
        r"let a=/(.\b)(a)/ds.exec('\na');a[1]==='\n'&&a[2]==='a'&&/(.\b)(a)/.exec('\na')===null&&/([\b]\b)(a)/.exec('\ba')[1]==='\b'",
    );
}

#[test]
fn generic_consumers_and_intrinsic_results_keep_assertion_capture_indices() {
    check(
        r"'ab ab'.replace(/(a\B)(b)/g,'$2$1')==='ba ba'&&'abxab'.split(/(a\B)(b)/).join(',')===',a,b,x,a,b,'",
    );
    check(
        r"let seen;let s='aaaa'.replace(/(a)+(\B)/,(whole,a,empty,index,input)=>{seen=[whole,a,empty,index,input];return 'x'});s==='xa'&&seen.join(',')==='aaa,a,,0,aaaa'",
    );
    check(
        r"let r=/(a\B)(b)/dg,a=[...'ab ab'.matchAll(r)];a.length===2&&a[1].index===3&&a[1].indices[2][0]===4&&r.lastIndex===0",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(\B)(a)+/d.exec('ba');calls===0&&a[1]===''&&Object.hasOwn(a,'1')&&a.indices[1][0]===1",
    );
}

#[test]
fn duplicate_checks_shared_sets_and_thousand_capture_copies_survive_collection() {
    let mut realm = Realm::default();
    realm.eval(r"let flat=new RegExp('a'+'\\B'.repeat(100000)+'b'),r=new RegExp('([a]\\B)'.repeat(999)+'([a]\\b)','dg'),copy=new RegExp(r),s=' '+'a'.repeat(1000)+' '").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);flat.exec('xab').index===1&&a.length===1001&&a[1]==='a'&&a[1000]==='a'&&a.indices[1000][0]===1000&&copy.lastIndex===1001&&r.lastIndex===0&&r.source===copy.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_counts_assertions_in_fixed_prefixes_and_continuations() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(a)+(\B)/y,s='a'.repeat(20000),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=new RegExp('a'+'(\\B)'.repeat(100)+'b','y'),s='a'.repeat(1000)+'b'")
        .unwrap();
    assert_eq!(realm.eval("r.exec(s)===null"), Ok(Value::Boolean(true)));
    for source in [
        r"/(?:(?:((?:(?:a+^b+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(a\B|c)+)|){2,3}/.test('aa')",
        r"/(?:(?:((?:(?:a\Bb+c+){2}){2}){2})|){2,3}/.test('abc')",
        r"/(?:(?:(?:a\B(?<n>b)(?:\k<n>)+){2})|){2,3}/.test('ab')",
        r"/a\Bb/u.test('ab')",
        r"/(?:(?:^((\b(ab|a)+\b))$)|){2,3}/.test('ab')",
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
