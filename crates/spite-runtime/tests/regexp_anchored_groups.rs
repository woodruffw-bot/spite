//! Complete consuming groups inside outer anchors preserve selected capture ranges.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn grouped_quantified_continuations_preserve_whole_and_inner_ranges() {
    check(
        "let m=/^((a)+(b))$/d.exec('aaab');m.length===4&&m[1]==='aaab'&&m[2]==='a'&&m[3]==='b'&&m.indices[1][1]===4&&m.indices[2][0]===2&&m.indices[3][0]===3&&/^(?:a+b)$/.exec('aaab')[0]==='aaab'",
    );
    check(
        "let m=/^((a)+?(aa))$/d.exec('aaaa');m[1]==='aaaa'&&m[2]==='a'&&m[3]==='aa'&&m.indices[2][0]===1&&m.indices[3][0]===2&&/^(a+?aa)$/.exec('aaaa')[1]==='aaaa'",
    );
    check(
        "let a=/^(([ab])*?(ab))$/d.exec('abab'),b=/^(([ab])*?(ab))/.exec('abab');a[1]==='abab'&&a[2]==='b'&&a[3]==='ab'&&a.indices[3][0]===2&&b[1]==='ab'&&b[2]===undefined&&b[3]==='ab'",
    );
}

#[test]
fn multiline_boundaries_and_exact_input_ends_constrain_grouped_bodies() {
    for boundary in [r"\n", r"\r", r"\u2028", r"\u2029"] {
        check(&format!(
            "let m=/^((a)+(b))$/md.exec('x{boundary}aaab{boundary}y');m.index===2&&m[1]==='aaab'&&m[2]==='a'&&m[3]==='b'&&m.indices[3][0]===5"
        ));
    }
    check(
        r"let a=/^([ab]*?ab)$/m.exec('abab\nab'),b=/^((a)+(b))$/m.exec('x\r\naaab\r\ny');a[1]==='abab'&&b.index===3&&b[1]==='aaab'&&/^(a+b)$/.exec('aaab\n')===null&&/^(a+b)$/.exec('xaaab')===null",
    );
}

#[test]
fn empty_groups_and_zero_iterations_keep_capture_participation() {
    check(
        "let a=/^((a)*())$/d.exec(''),b=/^((a*)())$/d.exec('');a[1]===''&&a[2]===undefined&&a[3]===''&&a.indices[1][0]===0&&a.indices[2]===undefined&&b[1]===''&&b[2]===''&&b[3]===''&&b.indices[2][1]===0",
    );
    check(
        "let a=/^((?:))$/.exec(''),b=/^((?:a)*x)$/.exec('x');a[1]===''&&b[1]==='x'&&/^((?:))$/.exec('x')===null",
    );
}

#[test]
fn global_sticky_state_and_strict_writes_preserve_anchor_boundaries() {
    check(
        r"let r=/^(a+b)$/my;r.lastIndex=2;let m=r.exec('x\naaab\ny');m.index===2&&m[1]==='aaab'&&r.lastIndex===6&&r.exec('x\naaab\ny')===null&&r.lastIndex===0",
    );
    check(r"let r=/^(a+b)$/my;r.lastIndex=3;r.exec('x\naaab\ny')===null&&r.lastIndex===0");
    check(
        "let r=/^(a+b)$/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('aaab')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn grouped_anchor_bodies_compose_with_branch_and_outer_capture_prefixes() {
    check(
        r"let m=/([x])|^(a+b)$|([y])/md.exec('z\naaab\ny');m.index===2&&m[1]===undefined&&m[2]==='aaab'&&m[3]===undefined&&m.indices[2][0]===2",
    );
    check(
        "let m=/(^(a+b)$)|x/d.exec('aaab'),n=/(?:(^(a+b)$))/.exec('aaab');m[1]==='aaab'&&m[2]==='aaab'&&m.indices[2][1]===4&&n[1]==='aaab'&&n[2]==='aaab'",
    );
}

#[test]
fn original_source_pinned_flags_and_generic_consumers_preserve_captures() {
    check(
        "let r=new RegExp('^((µ)+(x))$','di'),copy=new RegExp(r),m=copy.exec('ΜΜX');copy.source===r.source&&r.source==='^((µ)+(x))$'&&m[1]==='ΜΜX'&&m[2]==='Μ'&&m[3]==='X'&&m.indices[3][0]===2&&/^(ſ+x)$/i.exec('ssx')===null",
    );
    check(
        r"let m=/^((.)+\uD800)$/sd.exec('a\n\uD800');m[1]==='a\n\uD800'&&m[2]==='\n'&&m.indices[2][0]===1",
    );
    check(
        "'aaab'.replace(/^((a)+(b))$/,'$1-$2-$3')==='aaab-a-b'&&'aaab'.split(/^(a+b)$/).join(',')===',aaab,'",
    );
    check(
        r"let r=/^(a+b)$/mg,m=[...'aaab\naab'.matchAll(r)];m.length===2&&m[0][1]==='aaab'&&m[1][1]==='aab'&&m[1].index===5&&r.lastIndex===0",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/^((a)*())$/d.exec('');calls===0&&m[1]===''&&m[2]===undefined&&Object.hasOwn(m,'2')&&Object.hasOwn(m.indices,'2')&&m.indices[2]===undefined",
    );
}

#[test]
fn deep_anchor_groups_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let deep=new RegExp('^'+'('.repeat(100000)+'a+b'+')'.repeat(100000)+'$'),r=new RegExp('^'+'('.repeat(1000)+'(a)+(b)'+')'.repeat(1000)+'$','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec('aaab');deep.source.length===200005&&copy.source===r.source&&m.length===1003&&m[1]==='aaab'&&m[1000]==='aaab'&&m[1001]==='a'&&m[1002]==='b'&&m.indices[1002][0]===3"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_aborts_and_complete_unsupported_bodies_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/^(\d+x)$/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/^(a+b)$/y,s='a'.repeat(8000)+'b',flag=0")
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
        "/(?:(?:^(a+[b]+)$){2}){2}/.test('ab')",
        "/^(a+b)*$/.test('ab')",
        "/(?:(?:^a(a+b+)$){2}){2}/.test('aab')",
        "/(?:(?:^(a|bc)$){2}){2}/.test('a')",
        "/(?:^(?<n>a)$(?:\\k<n>)+){2}/.test('a')",
        "/(?:(?:^((^a+b+))$){2}){2}/.test('ab')",
        "/^(a+b)$/u.test('ab')",
        r"/(?:^((a)(?:\1)+)$){2}/.test('aa')",
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
