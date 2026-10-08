//! Fixed prefixes filter repeated-group starts without restarting complete search.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prefixes_select_earliest_complete_greedy_lazy_bounded_and_disconnected_runs() {
    check(
        r"let a=/x(ab)+(ab)/d.exec('xababab'),b=/x(ab)+?(ab)/d.exec('xababab');a.index===0&&a[0]==='xababab'&&a.indices[1][0]===3&&a.indices[2][0]===5&&b[0]==='xabab'&&b.indices[1][0]===1&&b.indices[2][0]===3",
    );
    check(
        r"let a=/(a)(aa){1,2}(ab)/d.exec('aaaaaaaab');a.index===2&&a[0]==='aaaaaab'&&a.indices[1][0]===2&&a.indices[2][0]===5&&a.indices[3][0]===7&&/(a)(aa){1,2}?(ab)/.exec('aaaaaaaab').index===2",
    );
    check(
        r"let a=/x(ab)*c/d.exec('xababxc');a.index===5&&a[0]==='xc'&&a[1]===undefined&&a.indices[1]===undefined&&/x(ab)+c/.exec('xababxc')===null",
    );
}

#[test]
fn prefix_iteration_sequel_empty_and_undefined_capture_slots_keep_exact_positions() {
    check(
        r"let a=/([ab])(([ab])([ab])()){2}([ab]b)()/d.exec('aaaaaab');a.length===8&&a[1]==='a'&&a[2]==='aa'&&a[3]==='a'&&a[4]==='a'&&a[5]===''&&a[6]==='ab'&&a[7]===''&&a.indices[1][0]===0&&a.indices[2][0]===3&&a.indices[5][0]===5&&a.indices[6][0]===5&&a.indices[7][0]===7",
    );
    check(
        r"let a=/()(ab)+(c)()/d.exec('ababc');a[1]===''&&a[2]==='ab'&&a[3]==='c'&&a[4]===''&&a.indices[1][0]===0&&a.indices[2][0]===2&&a.indices[3][0]===4&&a.indices[4][0]===5",
    );
    check(
        r"let a=/(a)(ab){0}(b)()/d.exec('ab');a[1]==='a'&&a[2]===undefined&&a[3]==='b'&&a[4]===''&&Object.hasOwn(a,'2')&&Object.hasOwn(a.indices,'2')&&a.indices[2]===undefined&&a.indices[3][0]===1",
    );
}

#[test]
fn zero_width_repeated_bodies_preserve_required_prefix_and_sequel_semantics() {
    check(
        r"let a=/(a)()*(b)/d.exec('ab'),b=/(a)()+(b)/d.exec('ab'),c=/(a)(){999999999999999999999999999999}(b)/d.exec('ab');a[1]==='a'&&a[2]===undefined&&a[3]==='b'&&b[2]===''&&b.indices[2][0]===1&&c[2]===''&&c.indices[3][0]===1",
    );
    check(
        r"let a=/a(\b\B)*(b)/d.exec('ab');a[1]===undefined&&a[2]==='b'&&/a(\b\B)+(b)/.exec('ab')===null&&/a(^){2}b/.exec('ab')===null&&/a()+/.exec('a')[1]===''",
    );
    check(
        r"let a=/(\b)(a)(a\B)+b/d.exec('aaab');a[1]===''&&a[2]==='a'&&a[3]==='a'&&a.indices[1][0]===0&&a.indices[2][0]===0&&a.indices[3][0]===2",
    );
}

#[test]
fn inner_prefix_and_outer_assertions_use_complete_multiline_and_word_neighbors() {
    check(
        r"let a=/(^)(a)(a())+($)(\n)/dm.exec('x\naaaa\n');a.index===2&&a[0]==='aaaa\n'&&a[1]===''&&a[2]==='a'&&a.indices[3][0]===5&&a.indices[4][0]===6&&a.indices[5][0]===6&&a.indices[6][0]===6",
    );
    check(
        r"let a=/^((a)(aa)+(ab))$/dm.exec('x\u2028aaaaaab\u2029y');a.index===2&&a[1]==='aaaaaab'&&a.indices[2][0]===2&&a.indices[3][0]===5&&a.indices[4][0]===7&&/^((a)(aa)+(ab))$/.exec('aaaaaab\n')===null",
    );
    check(
        r"let a=/\b(a)(aa)*\b/d.exec('aaaaa'),b=/\b(a)(aa)*?\b/d.exec('aaaaa');a[0]==='aaaaa'&&a.indices[2][0]===3&&b[0]==='aaaaa'&&/\b(a)(aa)*\b/.exec('aaaa')===null",
    );
    check(
        r"let a=/^((\b(ab)+\b))$/d.exec('abab');a[1]==='abab'&&a[2]==='abab'&&a[3]==='ab'&&a.indices[1][0]===0&&a.indices[1][1]===4&&a.indices[2][1]===4&&a.indices[3][0]===2",
    );
}

#[test]
fn enclosing_alternatives_global_sticky_and_original_source_preserve_complete_ranges() {
    check(
        r"let a=/((x)(ab)+(c))|(ab)+/d.exec('xababc');a[1]==='xababc'&&a[2]==='x'&&a[3]==='ab'&&a[4]==='c'&&a[5]===undefined&&a.indices[3][0]===3&&a.indices[4][0]===5",
    );
    check(
        r"let r=/(x)(ab)+(c)/dg,a=r.exec('xabc xababc'),b=r.exec('xabc xababc');a.index===0&&a.indices[3][0]===3&&b.index===5&&b.indices[2][0]===8&&b.indices[3][0]===10&&r.lastIndex===11&&r.exec('xabc xababc')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/x(ab)+(c)/dy,copy=new RegExp(r);r.lastIndex=1;let a=r.exec('yxababc'),ok=a.index===1&&a.indices[1][0]===4&&a.indices[2][0]===6&&r.lastIndex===7;ok&&r.exec('yxababc')===null&&r.lastIndex===0&&copy.lastIndex===0&&copy.source===r.source",
    );
}

#[test]
fn escaped_class_delimiters_dotall_case_and_surrogates_keep_actual_prefix_width() {
    check(
        r"let a=/\((ab)+(c)/d.exec('(ababc'),b=/[(](ab)+(c)/d.exec('(ababc');a[1]==='ab'&&a.indices[1][0]===3&&a.indices[2][0]===5&&b[0]===a[0]&&/\x28(ab)+c/.exec('(ababc')[0]==='(ababc'",
    );
    check(
        r"let a=/(µ)([µ][µ])+(x)/di.exec('µΜµx'),b=/(.)(.a)+([ab])/ds.exec('\n\na\nab');a[1]==='µ'&&a[2]==='Μµ'&&a.indices[3][0]===3&&b[1]==='\n'&&b[2]==='\na'&&b.indices[3][0]===5&&/(.)(.a)+([ab])/.exec('\n\na\nab')===null",
    );
    check(
        r"let a=/💩(ab)+(c)/d.exec('💩ababc'),b=/\uDCA9(ab)+(\uDCA9)/d.exec('💩abab\uDCA9');a.indices[1][0]===4&&a.indices[2][0]===6&&b.index===1&&b.indices[1][0]===4&&b.indices[2][0]===6",
    );
}

#[test]
fn generic_consumers_intrinsic_results_long_runs_and_copies_survive_collection() {
    check(
        r"'xababc'.replace(/(x)(ab)+(c)/,'<$1,$2,$3>')==='<x,ab,c>'&&'yxababcq'.split(/(x)(ab)+(c)/).join(',')==='y,x,ab,c,q'&&'yxababc'.search(/x(ab)+c/)===1",
    );
    check(
        r"let r=/(x)(ab)+(c)/dg,a=[...'xabc xababc'.matchAll(r)];a.length===2&&a[1].indices[2][0]===8&&a[1].indices[3][0]===10&&r.lastIndex===0",
    );
    check(
        r"let seen;let s='yxababc'.replace(/(x)(ab)+(c)/,(whole,prefix,last,sequel,index,input)=>{seen=[whole,prefix,last,sequel,index,input];return 'q'});s==='yq'&&seen[0]==='xababc'&&seen[1]==='x'&&seen[2]==='ab'&&seen[3]==='c'&&seen[4]===1&&seen[5]==='yxababc'",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let a=/(a)(ab){0}(b)()/d.exec('ab');calls===0&&a[2]===undefined&&a.indices[2]===undefined&&Object.hasOwn(a,'2')&&Object.hasOwn(a.indices,'2')",
    );
    let mut realm = Realm::default();
    realm.eval("let s='a'.repeat(200000)+'b',r=new RegExp('('.repeat(100000)+'a'+')'.repeat(100000)+'(aa)+(ab)','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a[0]===s&&a.length===100003&&a[100000]==='a'&&a[100001]==='aa'&&a[100002]==='ab'&&a.indices[100000][0]===0&&a.indices[100001][0]===199997&&a.indices[100002][0]===199999&&copy.source===r.source&&/(a)(aa){1,2}(ab)/.exec(s).index===199994"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_host_work_aborts_and_remaining_variable_group_features_are_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(x)(ab)+(c)/y,s='x'+'ab'.repeat(10000)+'c',flag=0")
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
        r"/(?:(?:(?:(?:a+b(ab)+c){2}){2})|){2,3}/.test('ababc')",
        r"/(?:(?:x(ab|a)+c)|){2,3}/.test('xabc')",
        r"/(?:(?:(?:(?:x(ab)+(c)+){2}){2})|){2,3}/.test('xabc')",
        r"/(?:(?:(?:x(?<n>ab)+c\k<n>){2})|){2,3}/.test('xabc')",
        r"/(?:(?:(?:x(ab)+\1){2})|){2,3}/.test('xabab')",
        r"/(?:(?:x((ab|a)+)c)|){2,3}/.test('xabc')",
        r"/x(ab)+c/u.test('xabc')",
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
