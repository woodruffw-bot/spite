//! Partial enclosing capture endpoints compose with one fixed repetition.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn partial_enclosing_prefix_and_sequel_captures_keep_source_order() {
    check(
        r"let a=/((x)(ab)+)(c)/d.exec('xababc');a.length===5&&a[1]==='xabab'&&a[2]==='x'&&a[3]==='ab'&&a[4]==='c'&&a.indices[1][1]===5&&a.indices[3][0]===3&&a.indices[4][0]===5",
    );
    check(
        r"let a=/(x)((ab)+(c))/d.exec('xababc');a[1]==='x'&&a[2]==='ababc'&&a[3]==='ab'&&a[4]==='c'&&a.indices[2][0]===1&&a.indices[2][1]===6&&a.indices[3][0]===3",
    );
    check(
        r"let a=/(())((ab)+)()c()/d.exec('ababc');a.length===7&&a[1]===''&&a[2]===''&&a[3]==='abab'&&a[4]==='ab'&&a[5]===''&&a[6]===''&&a.indices[1][0]===0&&a.indices[5][0]===4&&a.indices[6][0]===5",
    );
}

#[test]
fn greedy_lazy_bounded_and_overlapping_endpoints_preserve_partial_ranges() {
    check(
        r"let a=/((ab)+)ab/d.exec('ababab'),b=/((ab)+?)ab/d.exec('ababab');a[1]==='abab'&&a[2]==='ab'&&a.indices[1][1]===4&&a.indices[2][0]===2&&b[1]==='ab'&&b.indices[1][1]===2&&b[0]==='abab'",
    );
    check(
        r"let a=/((a)(aa){1,2})(ab)/d.exec('aaaaaaaab'),b=/((a)(aa){1,2}?)(ab)/d.exec('aaaaaaaab');a.index===2&&b.index===2&&a[1]==='aaaaa'&&a.indices[1][0]===2&&a.indices[1][1]===7&&a.indices[3][0]===5&&a.indices[4][0]===7",
    );
    check(
        r"let a=/x((ab)*)(c)/d.exec('xababxc');a.index===5&&a[1]===''&&a[2]===undefined&&a[3]==='c'&&a.indices[1][0]===6&&a.indices[1][1]===6&&a.indices[2]===undefined&&Object.hasOwn(a,'2')",
    );
}

#[test]
fn empty_required_repetitions_distinguish_enclosing_and_iteration_captures() {
    check(
        r"let a=/x(()*)(a)/d.exec('xa'),b=/x(()+)(a)/d.exec('xa'),c=/x((){999999999999999999999999999999})(a)/d.exec('xa');a[1]===''&&a[2]===undefined&&a.indices[1][0]===1&&b[1]===''&&b[2]===''&&b.indices[2][0]===1&&c[1]===''&&c[2]===''&&c.indices[3][0]===1",
    );
    check(
        r"let a=/x((\b\B)*)(a)/d.exec('xa');a[1]===''&&a[2]===undefined&&/x((\b\B)+)(a)/.exec('xa')===null&&/x((^){2})(a)/.exec('xa')===null",
    );
    check(
        r"let a=/x((ab){0})(c)/d.exec('xc');a[1]===''&&a[2]===undefined&&a[3]==='c'&&a.indices[1][0]===1&&a.indices[3][0]===1",
    );
}

#[test]
fn internal_outer_line_and_word_assertions_use_complete_input_neighbors() {
    check(
        r"let a=/^((a)(a())+)($)(\n)/dm.exec('x\naaaa\n');a.index===2&&a[1]==='aaaa'&&a[2]==='a'&&a[3]==='a'&&a[4]===''&&a[5]===''&&a[6]==='\n'&&a.indices[1][1]===6&&a.indices[3][0]===5&&a.indices[5][0]===6&&/^((a)(a())+)($)(\n)/.exec('aaaa\n')===null",
    );
    check(
        r"let a=/\b((a)(aa)+)(b)\b/d.exec('aaaaab');a[1]==='aaaaa'&&a[2]==='a'&&a[3]==='aa'&&a[4]==='b'&&a.indices[1][1]===5&&a.indices[3][0]===3&&/\b((a)(aa)+)(b)\b/.exec('aaaaabc')===null",
    );
    check(
        r"let a=/(x)((a\B)+)(b)/d.exec('xaaab');a[2]==='aaa'&&a[3]==='a'&&a.indices[2][0]===1&&a.indices[2][1]===4&&a.indices[3][0]===3",
    );
}

#[test]
fn complete_wrappers_alternatives_copies_global_and_sticky_keep_partial_slots() {
    check(
        r"let a=/(((x)(ab)+)(c))|(ab)+/d.exec('xababc');a[1]==='xababc'&&a[2]==='xabab'&&a[3]==='x'&&a[4]==='ab'&&a[5]==='c'&&a[6]===undefined&&a.indices[2][1]===5&&a.indices[4][0]===3",
    );
    check(
        r"let r=/(x)((ab)+)(c)/dg,copy=new RegExp(r),a=r.exec('xabc xababc'),b=r.exec('xabc xababc');a.indices[2][0]===1&&a.indices[2][1]===3&&b.index===5&&b.indices[2][0]===6&&b.indices[2][1]===10&&r.lastIndex===11&&copy.source===r.source&&copy.lastIndex===0&&r.exec('xabc xababc')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/x((ab)+)(c)/dy;r.lastIndex=1;let a=r.exec('yxababc');a.index===1&&a.indices[1][0]===2&&a.indices[1][1]===6&&a.indices[2][0]===4&&r.lastIndex===7&&r.exec('yxababc')===null&&r.lastIndex===0",
    );
    check(r"let a=/(?:x(?:ab)+)(c)/d.exec('xababc');a.length===2&&a[1]==='c'&&a.indices[1][0]===5");
}

#[test]
fn normalization_preserves_escape_boundaries_flags_and_utf16_offsets() {
    check(
        r"let a=/\0()1((ab)+)c/d.exec('\x001ababc');a[0]==='\x001ababc'&&a[1]===''&&a[2]==='abab'&&a[3]==='ab'&&a.indices[1][0]===1&&a.indices[2][0]===2&&a.indices[3][0]===4&&/\0()1((ab)+)c/.exec('\x01ababc')===null",
    );
    check(
        r"let a=/\((ab)+c\)/d.exec('(ababc)');a[0]==='(ababc)'&&a[1]==='ab'&&a.indices[1][0]===3&&a.indices[1][1]===5",
    );
    check(
        r"let a=/(µ)(([µ][µ])+)(x)/di.exec('µΜµx'),b=/(.)((.a)+)([ab])/ds.exec('\n\na\nab');a[2]==='Μµ'&&a.indices[2][0]===1&&a.indices[4][0]===3&&b[2]==='\na\na'&&b.indices[2][0]===1&&b.indices[2][1]===5",
    );
    check(
        r"let a=/💩((ab)+)(c)/d.exec('💩ababc'),b=/\uDCA9((ab)+)(\uDCA9)/d.exec('💩abab\uDCA9');a.indices[1][0]===2&&a.indices[1][1]===6&&a.indices[2][0]===4&&b.index===1&&b.indices[1][0]===2&&b.indices[3][0]===6",
    );
}

#[test]
fn generic_consumers_results_long_runs_and_deep_partial_captures_survive_collection() {
    check(
        r"'xababc'.replace(/(x)((ab)+)(c)/,'<$1,$2,$3,$4>')==='<x,abab,ab,c>'&&'yxababcq'.split(/(x)((ab)+)(c)/).join(',')==='y,x,abab,ab,c,q'&&'yxababc'.search(/x((ab)+)c/)===1",
    );
    check(
        r"let r=/(x)((ab)+)(c)/dg,a=[...'xabc xababc'.matchAll(r)];a.length===2&&a[1][2]==='abab'&&a[1].indices[2][0]===6&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"let seen;let s='yxababc'.replace(/(x)((ab)+)(c)/,(whole,prefix,run,last,sequel,index,input)=>{seen=[whole,prefix,run,last,sequel,index,input];return 'q'});s==='yq'&&seen[2]==='abab'&&seen[3]==='ab'&&seen[5]===1&&seen[6]==='yxababc'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let a=/x((ab){0})(c)/d.exec('xc');calls===0&&a[1]===''&&a[2]===undefined&&a.indices[2]===undefined&&Object.hasOwn(a,'2')&&Object.hasOwn(a.indices,'2')",
    );
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('a'+'('.repeat(100000)+'(aa)+'+')'.repeat(100000)+'(ab)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100003&&a[1].length===199998&&a[100000]===a[1]&&a[100001]==='aa'&&a[100002]==='ab'&&a.indices[100000][0]===1&&a.indices[100000][1]===199999&&a.indices[100001][0]===199997&&a.indices[100002][0]===199999&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_failures_and_remaining_variable_group_features_are_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/x((ab)+)(c)/y,s='x'+'ab'.repeat(10000)+'c',flag=0")
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
        r"/((ab|a)+)c/.test('abc')",
        r"/(?:(?:((ab)+)(c)+){2}){2}/.test('abc')",
        r"/((a*)+)b/.test('ab')",
        r"/(?:(?<n>(ab)+)c\k<n>){2}/.test('abc')",
        r"/(?:((ab)+)\1){2}/.test('abab')",
        r"/x((ab)+)c/u.test('xabc')",
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
