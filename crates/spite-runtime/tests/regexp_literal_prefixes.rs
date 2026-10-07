//! Fixed literal prefixes share linear quantified continuation and capture plans.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prefixes_choose_earliest_candidates_before_greedy_lazy_lengths() {
    check(
        "let a=/aa+a/.exec('aaaa'),b=/aa+?a/.exec('aaaa');a[0]==='aaaa'&&b[0]==='aaa'&&/ab+/.exec('xabb').index===1&&/ab{0,1}c/.exec('aabc').index===1",
    );
    check(
        "let a=/a([ab])*ab/d.exec('aabab'),b=/a([ab])*?ab/d.exec('aabab');a[0]==='aabab'&&a[1]==='b'&&a.indices[1][0]===2&&b[0]==='aab'&&b[1]===undefined&&b.indices[1]===undefined",
    );
    check(
        "let a=/ab{0,2}?b/.exec('abbb'),b=/ab{0,2}b/.exec('abbb');a[0]==='ab'&&b[0]==='abbb'&&/aa{0,1}b/.exec('aaab').index===1",
    );
}

#[test]
fn quantified_and_suffix_captures_exclude_the_literal_prefix() {
    check(
        "let m=/x(a)+(b)/d.exec('xaaab');m[0]==='xaaab'&&m[1]==='a'&&m[2]==='b'&&m.indices[1][0]===3&&m.indices[2][0]===4",
    );
    check(
        "let m=/a((a)+)b/d.exec('aaaaab');m[1]==='aaaa'&&m[2]==='a'&&m.indices[1][0]===1&&m.indices[1][1]===5&&m.indices[2][0]===4",
    );
    check(
        "let a=/x(a)*y/d.exec('xy'),b=/x(a*)y/d.exec('xy');a[1]===undefined&&a.indices[1]===undefined&&b[1]===''&&b.indices[1][0]===1&&b.indices[1][1]===1&&/a[]*/.exec('a')[0]==='a'",
    );
}

#[test]
fn global_sticky_and_strict_lastindex_writes_use_whole_prefix_ranges() {
    check(
        "let r=/ab+/dg,a=r.exec('xabb abbb'),b=r.exec('xabb abbb');a.index===1&&a[0]==='abb'&&a.indices[0][0]===1&&b.index===5&&b[0]==='abbb'&&r.lastIndex===9&&r.exec('xabb abbb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/ab+/y;r.lastIndex=1;let m=r.exec('xabb');m.index===1&&m[0]==='abb'&&r.lastIndex===4&&r.exec('xabb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/ab+/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('abb')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn prefixes_compose_with_anchors_choices_and_enclosing_captures() {
    check(
        "let m=/([x])|a(b)+(c)|([y])/d.exec('abbc');m.length===5&&m[1]===undefined&&m[2]==='b'&&m[3]==='c'&&m[4]===undefined&&m.indices[2][0]===2&&m.indices[3][0]===3",
    );
    check(
        "let a=/(ab+)|x/.exec('abbb'),b=/^((ab+))$/d.exec('abbb');a[1]==='abbb'&&b[1]==='abbb'&&b[2]==='abbb'&&b.indices[2][1]===4",
    );
    check(
        r"let a=/^a(b)*?(bc)$/d.exec('abbbc'),b=/^a(b)+(c)$/my;b.lastIndex=2;let m=b.exec('x\nabbc\ny');a[1]==='b'&&a[2]==='bc'&&a.indices[1][0]===2&&m.index===2&&m[1]==='b'&&m[2]==='c'&&b.lastIndex===6",
    );
}

#[test]
fn escaped_prefix_lengths_pinned_case_and_surrogate_units_keep_input_offsets() {
    check(
        r"let m=/\u0061(b)+/d.exec('abb');m[0]==='abb'&&m[1]==='b'&&m.indices[1][0]===2&&m.indices[1][1]===3",
    );
    check(
        r"let m=/\n(a)+/di.exec('x\nAAA');m.index===1&&m[0]==='\nAAA'&&m[1]==='A'&&m.indices[1][0]===4",
    );
    check("let m=/µ(a)+/i.exec('ΜAA');m[0]==='ΜAA'&&m[1]==='A'&&/ſa+/i.exec('saa')===null");
    check(
        r"let m=/💩+/.exec('\uD83D\uDCA9\uDCA9');m[0].length===3&&/x(.)+/s.exec('x\n')[1]==='\n'&&/x(.)+/.exec('x\n')===null",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_keep_body_capture_layout() {
    check(
        "'xaaab'.replace(/x(a)+(b)/,'$1-$2')==='a-b'&&'abbxabb'.split(/a(b)+/).join(',')===',b,x,b,'",
    );
    check(
        "let seen;let s='xaaab'.replace(/x(a)+(b)/,(whole,a,b,index,input)=>{seen=[whole,a,b,index,input];return 'z'});s==='z'&&seen.join(',')==='xaaab,a,b,0,xaaab'",
    );
    check(
        "let r=/a(b)+/dg,m=[...'abb ab'.matchAll(r)];m.length===2&&m[0][1]==='b'&&m[1][1]==='b'&&m[1].indices[1][0]===5&&r.lastIndex===0",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/x(a)*y/d.exec('xy');calls===0&&m[1]===undefined&&Object.hasOwn(m,'1')&&Object.hasOwn(m.indices,'1')&&m.indices[1]===undefined",
    );
}

#[test]
fn long_prefixes_overlap_failures_and_copies_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('a'.repeat(100000)+'b+','dg'),copy=new RegExp(r),input='a'.repeat(300000)+'bb',failed=/aaa+b/").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(input);copy.source===r.source&&m.index===200000&&m[0].length===100002&&m.indices[0][1]===300002&&copy.lastIndex===300002&&r.lastIndex===0&&failed.exec('a'.repeat(300000))===null"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_work_abort_and_complete_unsupported_prefixes_remain_distinct() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/a\d+x/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/ab+/y,s='a'+'b'.repeat(8000),flag=0")
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
        "/([a])b+c+/.test('ab')",
        "/(?:[a])b+c+/.test('ab')",
        "/a.b+c+/.test('axb')",
        "/a[b]c+d+/.test('abc')",
        "/ab+c+/.test('abc')",
        "/ab+(c|d)/.test('abc')",
        "/ab+[c]/.test('abc')",
        "/ab+/u.test('ab')",
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
