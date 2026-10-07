//! Partial enclosing captures around one quantified character or set atom.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn partial_literal_class_and_sequel_captures_keep_actual_endpoints() {
    check(
        r"let a=/(x(a+))(y)/d.exec('xaaay');a.length===4&&a[1]==='xaaa'&&a[2]==='aaa'&&a[3]==='y'&&a.indices[1][1]===4&&a.indices[2][0]===1&&a.indices[3][0]===4",
    );
    check(
        r"let a=/(x)((a+)(y))/d.exec('xaaay');a[1]==='x'&&a[2]==='aaay'&&a[3]==='aaa'&&a[4]==='y'&&a.indices[2][0]===1&&a.indices[2][1]===5&&a.indices[3][1]===4",
    );
    check(
        r"let a=/(x([ab]+))(y)/d.exec('xababy');a[1]==='xabab'&&a[2]==='abab'&&a[3]==='y'&&a.indices[2][0]===1&&a.indices[2][1]===5",
    );
    check(
        r"let a=/(())(a*)()b()/d.exec('b');a.length===6&&a[1]===''&&a[2]===''&&a[3]===''&&a[4]===''&&a[5]===''&&a.indices[3][0]===0&&a.indices[4][0]===0&&a.indices[5][0]===1",
    );
}

#[test]
fn greedy_lazy_bounded_zero_minimum_and_huge_bounds_keep_order() {
    check(
        r"let a=/(a+)a/d.exec('aaaa'),b=/(a+?)a/d.exec('aaaa');a[0]==='aaaa'&&a[1]==='aaa'&&a.indices[1][1]===3&&b[0]==='aa'&&b[1]==='a'&&b.indices[1][1]===1",
    );
    check(
        r"let a=/(x(a{1,2}?))(a)/d.exec('xaaaa');a[0]==='xaa'&&a[1]==='xa'&&a[2]==='a'&&a[3]==='a'&&a.indices[2][0]===1&&a.indices[2][1]===2",
    );
    check(
        r"let a=/(x(a*))(y)/d.exec('xy'),b=/(x(a{0}))(y)/d.exec('xy');a[1]==='x'&&a[2]===''&&a.indices[2][0]===1&&a.indices[2][1]===1&&b[2]===''&&b.indices[3][0]===1",
    );
    check(
        r"let a=/(a{0,999999999999999999999999999999})(b)/d.exec('aaab');a[1]==='aaa'&&a.indices[2][0]===3&&/(a{999999999999999999999999999999})(b)/.exec('aaab')===null",
    );
}

#[test]
fn whole_wrappers_alternatives_copies_global_and_sticky_preserve_partial_slots() {
    check(
        r"let a=/a(a+b)|x/d.exec('aab'),b=/a(a+b)|x/d.exec('x');a[1]==='ab'&&a.indices[1][0]===1&&a.indices[1][1]===3&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        r"let a=/((x(a+))(y))|(b)+/d.exec('xaaay');a[1]==='xaaay'&&a[2]==='xaaa'&&a[3]==='aaa'&&a[4]==='y'&&a[5]===undefined&&a.indices[3][0]===1&&a.indices[4][0]===4",
    );
    check(
        r"let r=/(x(a+))(y)/dg,copy=new RegExp(r),a=r.exec('xay xaaay'),b=r.exec('xay xaaay');a.indices[2][0]===1&&a.indices[2][1]===2&&b.index===4&&b.indices[2][0]===5&&b.indices[2][1]===8&&r.lastIndex===9&&copy.source===r.source&&copy.lastIndex===0&&r.exec('xay xaaay')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/x(a+)(y)/dy;r.lastIndex=1;let a=r.exec('yxaaay');a.index===1&&a.indices[1][0]===2&&a.indices[1][1]===5&&a.indices[2][0]===5&&r.lastIndex===6&&r.exec('yxaaay')===null&&r.lastIndex===0",
    );
}

#[test]
fn assertions_and_set_atoms_use_complete_multiline_and_word_neighbors() {
    check(
        r"let a=/^a(a+b)$/d.exec('aab');a[0]==='aab'&&a[1]==='ab'&&a.indices[1][0]===1&&a.indices[1][1]===3&&/^a(a+b)$/.exec('aab\n')===null",
    );
    check(
        r"let a=/^((a)(a+))($)(\n)/dm.exec('x\naaaa\n');a.index===2&&a[1]==='aaaa'&&a[2]==='a'&&a[3]==='aaa'&&a.indices[3][0]===3&&a.indices[3][1]===6&&a.indices[4][0]===6&&/^((a)(a+))($)(\n)/.exec('aaaa\n')===null",
    );
    check(
        r"let a=/(\b(a+))(b)\b/d.exec('aaab');a[1]==='aaa'&&a[2]==='aaa'&&a.indices[2][1]===3&&a.indices[3][0]===3&&/(\b(a+))(b)\b/.exec('aaabc')===null",
    );
    check(
        r"let a=/(x(\d+))(y)/d.exec('x123y'),b=/(x(\s{1,2}))(y)/d.exec('x  y');a[2]==='123'&&a.indices[2][0]===1&&a.indices[3][0]===4&&b[2]==='  '&&b.indices[2][1]===3",
    );
}

#[test]
fn complete_character_escapes_and_removed_group_boundaries_remain_distinct() {
    check(
        r"let a=/(\x61+)(b)/d.exec('aaab'),b=/(\u0061+)(b)/d.exec('aaab'),c=/(\cA+)(b)/d.exec('\x01\x01b');a[1]==='aaa'&&a.indices[2][0]===3&&b[1]==='aaa'&&c[1]==='\x01\x01'&&c.indices[2][0]===2",
    );
    check(
        r"let a=/\0()1(a+)b/d.exec('\x001aaab');a[1]===''&&a[2]==='aaa'&&a.indices[1][0]===1&&a.indices[2][0]===2&&a.indices[2][1]===5&&/\0()1(a+)b/.exec('\x01aaab')===null",
    );
    check(
        r"let a=/(\0+)()1/d.exec('\x00\x001');a[1]==='\x00\x00'&&a[2]===''&&a.indices[1][1]===2&&a.indices[2][0]===2&&/(\(+)(a)/.exec('((a')[1]==='(('",
    );
}

#[test]
fn dotall_pinned_case_and_surrogate_atoms_keep_actual_utf16_widths() {
    check(
        r"let a=/(µ+)(x)/di.exec('µΜµx'),b=/(.+)([ab])/ds.exec('\na\nab');a[1]==='µΜµ'&&a.indices[2][0]===3&&b[1]==='\na\na'&&b.indices[1][1]===4&&/(.+)([ab])/.exec('\na\nab').index===3",
    );
    check(
        r"let a=/💩(a+)(b)/d.exec('💩aaab'),b=/\uDCA9(a+)(\uDCA9)/d.exec('💩aaa\uDCA9');a.indices[1][0]===2&&a.indices[1][1]===5&&b.index===1&&b.indices[1][0]===2&&b.indices[2][0]===5",
    );
    check(
        r"let a=/(\uD83D\uDCA9+)(b)/d.exec('💩\uDCA9b');a[1]==='💩\uDCA9'&&a.indices[1][0]===0&&a.indices[1][1]===3&&a.indices[2][0]===3",
    );
}

#[test]
fn generic_consumers_intrinsic_results_deep_captures_and_gc_keep_behavior() {
    check(
        r"'xaaay'.replace(/(x(a+))(y)/,'<$1,$2,$3>')==='<xaaa,aaa,y>'&&'qxaaayz'.split(/(x(a+))(y)/).join(',')==='q,xaaa,aaa,y,z'&&'qxaaay'.search(/x(a+)y/)===1",
    );
    check(
        r"let r=/(x(a+))(y)/dg,a=[...'xay xaaay'.matchAll(r)];a.length===2&&a[1][2]==='aaa'&&a[1].indices[2][0]===5&&a[1].indices[2][1]===8&&r.lastIndex===0",
    );
    check(
        r"let seen;let s='qxaaay'.replace(/(x(a+))(y)/,(whole,prefix,run,sequel,index,input)=>{seen=[whole,prefix,run,sequel,index,input];return 'z'});s==='qz'&&seen[1]==='xaaa'&&seen[2]==='aaa'&&seen[4]===1&&seen[5]==='qxaaay'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let a=/(x(a{0}))(y)/d.exec('xy');calls===0&&a[2]===''&&a.indices[2][0]===1&&Object.hasOwn(a,'2')&&Object.hasOwn(a.indices,'2')",
    );
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('a'+'('.repeat(100000)+'a+'+')'.repeat(100000)+'(ab)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100002&&a[1].length===199998&&a[100000]===a[1]&&a[100001]==='ab'&&a.indices[100000][0]===1&&a.indices[100000][1]===199999&&a.indices[100001][0]===199999&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_aborts_and_remaining_variable_features_are_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/x(a+)(y)/y,s='x'+'a'.repeat(20000)+'y',flag=0")
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
        r"/(a+)(b)+/.test('ab')",
        r"/(a+b+)c/.test('abc')",
        r"/((a|b)+)c/.test('abc')",
        r"/(?<n>a+)b/.test('ab')",
        r"/(a+)\1/.test('aa')",
        r"/x(a+)y/u.test('xay')",
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
