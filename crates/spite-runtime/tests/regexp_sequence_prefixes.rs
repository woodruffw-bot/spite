//! Fixed ordinary character-set sequences before a single quantified body.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn set_prefixes_choose_earliest_candidates_before_repetition_lengths() {
    check(
        "let a=/[ab]a+a/.exec('baaaa'),b=/[ab]a+?a/.exec('baaaa');a.index===0&&a[0]==='baaaa'&&b.index===0&&b[0]==='baa'",
    );
    check(
        "let a=/([ab][ab])([ab])*ab/d.exec('aabab'),b=/([ab][ab])([ab])*?ab/d.exec('aabab');a[0]==='aabab'&&a[1]==='aa'&&a[2]==='b'&&a.indices[2][0]===2&&b[0]==='aabab'&&b[2]==='b'",
    );
    check(
        r"let m=/[ab]c{0,1}d/.exec('xacd');m.index===1&&m[0]==='acd'&&/([])a*/.exec('aa')===null&&/([^])a+/.exec('\naa')[1]==='\n'",
    );
}

#[test]
fn sequence_prefix_capture_slots_keep_body_suffix_and_empty_participation() {
    check(
        "let m=/()((a[bc]))()(d)+((e)())/d.exec('xabdde');m.index===1&&m.length===9&&m[1]===''&&m[2]==='ab'&&m[3]==='ab'&&m[4]===''&&m[5]==='d'&&m[6]==='e'&&m[7]==='e'&&m[8]===''&&m.indices[3][1]===3&&m.indices[5][0]===4&&m.indices[8][0]===6",
    );
    check(
        "let a=/([ab])(c*)()/d.exec('a'),b=/([ab])(c)*()/d.exec('a');a[1]==='a'&&a[2]===''&&b[1]==='a'&&b[2]===undefined&&a.indices[2][0]===1&&Object.hasOwn(b.indices,'2')&&b[3]===''",
    );
}

#[test]
fn global_sticky_failures_and_strict_writes_use_whole_fixed_prefixes() {
    check(
        "let r=/([ab])(c)+/dg,a=r.exec('xac bbccc'),b=r.exec('xac bbccc');a.index===1&&a[1]==='a'&&b.index===5&&b[1]==='b'&&b[2]==='c'&&r.lastIndex===9&&r.exec('xac bbccc')===null&&r.lastIndex===0",
    );
    check(
        "let r=/([ab])c+/y;r.lastIndex=1;let m=r.exec('xacc');m.index===1&&m[1]==='a'&&r.lastIndex===4&&r.exec('xacc')===null&&r.lastIndex===0",
    );
    check(
        "let r=/([ab])c+/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('acc')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn sequence_prefixes_compose_with_anchors_branches_and_whole_captures() {
    check(
        "let m=/([x])|([ab])(c)+(d)|([y])/d.exec('accd');m.length===6&&m[1]===undefined&&m[2]==='a'&&m[3]==='c'&&m[4]==='d'&&m[5]===undefined&&m.indices[3][0]===2",
    );
    check(
        "let m=/^(([ab])(c)+())$/d.exec('acc');m[1]==='acc'&&m[2]==='a'&&m[3]==='c'&&m[4]===''&&m.indices[3][0]===2&&m.indices[4][0]===3",
    );
    check(
        r"let r=/^([ab])(c)*?(cd)$/dmy;r.lastIndex=2;let m=r.exec('x\nacccd\ny');m.index===2&&m[1]==='a'&&m[2]==='c'&&m[3]==='cd'&&m.indices[2][0]===4&&m.indices[3][0]===5&&r.lastIndex===7",
    );
}

#[test]
fn prefix_sets_dot_flags_case_and_utf16_offsets_use_pinned_semantics() {
    check(
        r"let m=/(\d)(a)+()/d.exec('x1aa');m.index===1&&m[1]==='1'&&m[2]==='a'&&m.indices[1][1]===2&&m.indices[2][0]===3",
    );
    check(
        "let a=/([µ])(a)+/di.exec('ΜAA'),b=/([^µ])(a)+/di.exec('ΜA');a[1]==='Μ'&&a[2]==='A'&&b===null&&/([ſ])a+/i.exec('saa')===null",
    );
    check(
        r"let m=/([💩])(a)+/d.exec('💩aa');m.index===1&&m[1]==='\uDCA9'&&m.indices[1][0]===1&&m.indices[2][0]===3",
    );
    check(r"let m=/(.)(a)+/ds.exec('\naa');m[1]==='\n'&&m[2]==='a'&&/(.)(a)+/.exec('\na')===null");
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_share_sequence_prefix_plans() {
    check(
        "'accd'.replace(/([ab])(c)+(d)/,'$1-$2-$3')==='a-c-d'&&'accxacc'.split(/([ab])(c)+/).join(',')===',a,c,x,a,c,'",
    );
    check(
        "let seen;let s='accd'.replace(/([ab])(c)+(d)/,(whole,a,c,d,index,input)=>{seen=[whole,a,c,d,index,input];return 'z'});s==='z'&&seen.join(',')==='accd,a,c,d,0,accd'",
    );
    check(
        "let r=/([ab])(c)+/dg,m=[...'acc bc'.matchAll(r)];m.length===2&&m[1][1]==='b'&&m[1].indices[2][0]===5&&r.lastIndex===0",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let m=/([ab])(c)*()/d.exec('a');calls===0&&m[1]==='a'&&m[2]===undefined&&Object.hasOwn(m,'2')&&Object.hasOwn(m.indices,'2')&&m[3]===''",
    );
}

#[test]
fn repeated_set_prefixes_share_storage_and_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('[a]'.repeat(1000)+'(b)+()','dg'),copy=new RegExp(r),s='a'.repeat(1000)+'bb'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(s);m.index===0&&m[0].length===1002&&m[1]==='b'&&m[2]===''&&m.indices[1][0]===1001&&copy.lastIndex===1002&&r.lastIndex===0&&copy.source===r.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_search_budget_covers_all_prefix_atoms_and_keeps_unsupported_host_errors() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('[a]'.repeat(100)+'b+'),s='a'.repeat(2000),flag=0")
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
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/(\d)a+/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for source in [
        "/([ab])c+d+/.test('acd')",
        "/([ab]c|d)+/.test('ac')",
        "/([ab])c+[d]+/.test('acd')",
        "/([ab]|c)d+/.test('ad')",
        "/([ab])c+/u.test('ac')",
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
