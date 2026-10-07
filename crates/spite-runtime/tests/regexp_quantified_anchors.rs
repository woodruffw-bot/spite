//! Outer assertions constrain repetition order before a quantified match commits.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn end_assertions_choose_valid_lengths_in_greedy_and_lazy_order() {
    check(
        "/a*?$/.exec('aaa')[0]==='aaa'&&/a{1,3}?$/.exec('baaaa').index===2&&/a{1,3}$/.exec('baaaa')[0]==='aaa'&&/^a+$/.exec('aaa')[0]==='aaa'",
    );
    check(
        r"/[^]*?$/m.exec('aa\nbb')[0]==='aa'&&/[^]*$/m.exec('aa\nbb')[0]==='aa\nbb'&&/[^]{1,3}$/m.exec('a\nbc')[0]==='a'",
    );
    check(
        "/^a{0}$/.exec('')[0]===''&&/^[]*$/.exec('')[0]===''&&/^[]+$/.exec('')===null&&/^a{999999999999999999999999999}$/.exec('aaa')===null",
    );
}

#[test]
fn multiline_starts_and_sticky_state_use_complete_input_boundaries() {
    check(
        r"let r=/^a+$/my;r.lastIndex=2;let m=r.exec('x\naaa\n');m.index===2&&m[0]==='aaa'&&r.lastIndex===5&&r.exec('x\naaa\n')===null&&r.lastIndex===0",
    );
    check(
        "let r=/^a+$/y;r.lastIndex=1;r.exec('xaaa')===null&&r.lastIndex===0&&/^a+$/.exec('xaaa')===null",
    );
    check(
        r"let r=/^a*$/mg,a=r.exec('x\r\na\u2028\u2029'),b=r.exec('x\r\na\u2028\u2029');a.index===2&&a[0]===''&&b.index===2&&r.lastIndex===2&&/^a+$/m.exec('x\r\na\u2028').index===3",
    );
}

#[test]
fn literal_continuations_apply_end_assertions_before_repetition_selection() {
    check(
        "/[ab]*?ab$/.exec('abab')[0]==='abab'&&/^[ab]*ab$/.exec('abab')[0]==='abab'&&/^[Nn]?evermore$/.exec('evermore')[0]==='evermore'&&/^[Nn]?evermore$/.exec('Nevermore')[0]==='Nevermore'",
    );
    check(
        r"/[^]*?x$/m.exec('x\nx')[0]==='x'&&/[^]*x$/m.exec('x\nx')[0]==='x\nx'&&/^a+b$/.exec('aab\n')===null&&/^a+b$/m.exec('aab\n')[0]==='aab'",
    );
    check(r"/^a*\$$/.exec('aaa$')[0]==='aaa$'&&/^a*\\$/.exec('aaa\\')[0]==='aaa\\'");
}

#[test]
fn dotall_case_surrogates_and_transparent_groups_preserve_pinned_semantics() {
    check(
        r"/^.*?$/s.exec('a\nb')[0]==='a\nb'&&/^.*$/m.exec('a\nb')[0]==='a'&&/^[µ]+$/i.exec('Μµ')[0]==='Μµ'&&/^[ſ]+$/i.exec('s')===null",
    );
    check(
        r"let m=/^[^]*?$/d.exec('\uD800\uDC00'),n=/^\uD800+$/d.exec('\uD800\uD800');m[0].length===2&&m.indices[0][1]===2&&n[0].length===2",
    );
    check(
        "/^(?:a)+$/.exec('aaa')[0]==='aaa'&&/^(?:a+?)$/.exec('aaa')[0]==='aaa'&&/^(?:(?:a)+?)aa$/.exec('aaaa')[0]==='aaaa'",
    );
}

#[test]
fn alternative_capture_slots_results_and_strict_writes_remain_ordered() {
    check(
        r"let r=/^(?:a)+$|([b])/mdg,a=r.exec('aa\nb'),b=r.exec('aa\nb');a[0]==='aa'&&a[1]===undefined&&a.indices[1]===undefined&&Object.getOwnPropertyDescriptor(a,'1').value===undefined&&b[0]==='b'&&b[1]==='b'&&b.indices[1][0]===3&&r.lastIndex===4",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(){calls++},configurable:true});let m=/^a+$/d.exec('aaa');calls===0&&m.length===1&&m.indices.length===1&&m.indices[0][1]===3&&m.groups===undefined",
    );
    check(
        "let r=/^a+$/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aaa')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn generic_consumers_and_empty_advancement_share_anchored_repetition() {
    check(
        r"'aa\na'.match(/^a+$/gm).join(',')==='aa,a'&&'aa\na'.replace(/^a+$/gm,'x')==='x\nx'&&'x\naaa'.search('^a+$')===-1&&'x\naaa'.search(/^a+$/m)===2",
    );
    check(r"[...'\na\n'.matchAll(/^a*$/gm)].length===3&&'aa\na'.split(/^a+$/m).join(',')===',\n,'");
    check(
        "let seen;let s='aaa'.replace(/^a+$/,(...args)=>{seen=args;return 'x'});s==='x'&&seen.length===3&&seen[0]==='aaa'&&seen[1]===0&&seen[2]==='aaa'",
    );
}

#[test]
fn long_runs_failures_and_copies_survive_collection_with_unlimited_defaults() {
    let mut realm = Realm::default();
    realm
        .eval("let r=/^[^]*?$/,copy=new RegExp(r),s='a'.repeat(300000)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("r.exec(s)[0].length===300000&&copy.exec(s)[0].length===300000&&/^[a]*aaaaab$/.exec(s)===null"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_constructor_and_sticky_branch_work_failures_bypass_js_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/^\d+$/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for pattern in ["/^a*$/y", "/^a*$|([x])/y"] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(10000),
            ..Limits::default()
        });
        realm
            .eval(&format!("let r={pattern},flag=0,s='a'.repeat(8000)"))
            .unwrap();
        assert!(
            matches!(
                realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
                Err(Error::Limit { .. })
            ),
            "{pattern}"
        );
        assert_eq!(
            realm.eval("flag===0&&r.lastIndex===0"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn unsupported_bodies_reject_whole_plans_with_ordered_host_effects() {
    for source in [
        "/^a+[b]+$/.test('ab')",
        "/^(ab|a)+$/.test('a')",
        "/^a+b+$/.test('ab')",
        "/^(?:a+[b]+)$/.test('ab')",
        "/^a+$/u.test('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(
            "try{/^a+[b]+$/.exec({toString(){flag=1;return 'ab'}})}catch{flag=2}finally{flag=3}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
}
