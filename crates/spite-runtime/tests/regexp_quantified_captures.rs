//! Capturing wrappers distinguish complete runs from their final repeated atom.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn inner_outer_and_nested_captures_distinguish_last_iteration_and_zero_runs() {
    check(
        "let a=/(a)+/d.exec('aaa'),b=/(a+)/d.exec('aaa'),c=/((a)+)/d.exec('aaa');a[1]==='a'&&a.indices[1][0]===2&&b[1]==='aaa'&&b.indices[1][0]===0&&c[1]==='aaa'&&c[2]==='a'&&c.indices[2][0]===2",
    );
    check(
        "let a=/(a)*/d.exec(''),b=/(a*)/d.exec(''),c=/((a)*)/d.exec('');a[1]===undefined&&a.indices[1]===undefined&&b[1]===''&&b.indices[1][0]===0&&b.indices[1][1]===0&&c[1]===''&&c[2]===undefined&&Object.hasOwn(c,'2')&&Object.hasOwn(c.indices,'2')",
    );
    check(
        "let a=/((a))+/d.exec('aaa'),b=/((?:a)+)/.exec('aaa'),c=/(?:(a)+)/.exec('aaa');a[1]==='a'&&a[2]==='a'&&a.indices[1][0]===2&&b[1]==='aaa'&&c[1]==='a'&&/(a)??/.exec('a')[1]===undefined&&/(a??)/.exec('a')[1]===''",
    );
}

#[test]
fn continuations_select_capture_ranges_after_greedy_lazy_and_zero_repetition() {
    check(
        "let a=/(([ab])*?)ab/d.exec('abab'),b=/(([ab])+)ab/d.exec('abab');a[1]===''&&a[2]===undefined&&a.indices[1][1]===0&&b[1]==='ab'&&b[2]==='b'&&b.indices[1][1]===2&&b.indices[2][0]===1",
    );
    check(
        "let a=/(a)+aa/d.exec('aaaa'),b=/(a+)aa/d.exec('aaaa');a[1]==='a'&&a.indices[1][0]===1&&b[1]==='aa'&&b.indices[1][1]===2&&/(a)*x/.exec('x')[1]===undefined&&/(a*)x/.exec('x')[1]===''&&/(a+)(?:)/.exec('aaa')[1]==='aaa'",
    );
}

#[test]
fn anchors_alternatives_and_global_sticky_state_preserve_capture_slots() {
    check(
        r"let m=/^((a)+?)$/md.exec('x\naaa\ny');m.index===2&&m[1]==='aaa'&&m[2]==='a'&&m.indices[1][0]===2&&m.indices[2][0]===4",
    );
    check(
        "let r=/([x])|((a)+)b|([y])/dg,a=r.exec('aaby'),b=r.exec('aaby');a.length===5&&a[1]===undefined&&a[2]==='aa'&&a[3]==='a'&&a[4]===undefined&&a.indices[3][0]===1&&b[4]==='y'&&b[2]===undefined&&r.lastIndex===4&&r.exec('aaby')===null&&r.lastIndex===0",
    );
    check(
        "let r=/((a)+)b/y;r.lastIndex=1;let m=r.exec('xaab');m[1]==='aa'&&m[2]==='a'&&m.index===1&&r.lastIndex===4&&r.exec('xaab')===null&&r.lastIndex===0",
    );
}

#[test]
fn generic_consumers_callbacks_and_empty_advance_observe_dynamic_captures() {
    check(
        "'aa'.replace(/(a)+/,'[$1]')==='[a]'&&'aa'.replace(/(a+)/,'[$1]')==='[aa]'&&'x'.replace(/(a)*x/,'[$1]')==='[]'&&'xaa'.split(/(a)+/).join(',')==='x,a,'",
    );
    check(
        "let seen;let s='aaa'.replace(/((a)+)/,(whole,outer,inner,index,input)=>{seen=[whole,outer,inner,index,input];return 'x'});s==='x'&&seen.join(',')==='aaa,aaa,a,0,aaa'",
    );
    check(
        "let r=/((a)+)/dg,all=[...'aaa aa'.matchAll(r)];all.length===2&&all[0][1]==='aaa'&&all[0][2]==='a'&&all[0].indices[2][0]===2&&all[1].indices[2][0]===5&&r.lastIndex===0&&'a'.match(/(a)??/g).length===2",
    );
}

#[test]
fn intrinsic_capture_arrays_undefined_slots_and_strict_writes_bypass_setters() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(a)*/d.exec(''),b=/(a+)/d.exec('aaa'),p=Object.getOwnPropertyDescriptor(a,'1'),q=Object.getOwnPropertyDescriptor(a.indices,'1');calls===0&&p.value===undefined&&p.writable&&p.enumerable&&p.configurable&&q.value===undefined&&b[1]==='aaa'&&b.indices[1][1]===3&&b.groups===undefined&&b.indices.groups===undefined",
    );
    check(
        "let r=/(a)+/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aaa')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn flags_and_surrogate_units_capture_original_input_units() {
    check(
        r"let m=/(µ)+/i.exec('Μµ'),n=/(.)+/s.exec('a\nb');m[1]==='µ'&&m[0]==='Μµ'&&n[1]==='b'&&n[0]==='a\nb'&&/(ſ)+/i.exec('s')===null",
    );
    check(
        r"let m=/([^])+/d.exec('\uD800\uDC00'),n=/([^]+)/d.exec('\uD800\uDC00');m[0].length===2&&m[1]==='\uDC00'&&m.indices[1][0]===1&&n[1].length===2&&n.indices[1][0]===0",
    );
}

#[test]
fn deeply_nested_layouts_and_copied_plans_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let deep=new RegExp('('.repeat(100000)+'a'+')'.repeat(100000)+'+'),r=new RegExp('('.repeat(1000)+'a+'+')'.repeat(1000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec('aaa');deep.source.length===200002&&m.length===1001&&m.every(x=>x==='aaa')&&m.indices.length===1001&&m.indices.every(x=>x[0]===0&&x[1]===3)"),Ok(Value::Boolean(true)));
    realm
        .eval("let inside=new RegExp('('.repeat(1000)+'a'+')'.repeat(1000)+'*','d')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m2=inside.exec('aaa'),m3=inside.exec('');m2[1000]==='a'&&m2.indices[1000][0]===2&&m3.length===1001&&m3[1000]===undefined&&Object.hasOwn(m3.indices,'1000')"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_host_aborts_precede_capture_results_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/(\d)+/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/((a)+)/y,flag=0,s='a'.repeat(8000)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unsupported_group_bodies_and_captured_suffixes_remain_host_failures() {
    for source in [
        r"/(?:(?:(a[b]|c)+)|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:(a+[b]+)){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:a+([b]+)){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?<x>a)+\k<x>){2})|){2,3}/.test('a')",
        r"/(?:(?:(a+)+)|){2,3}/.test('a')",
        "/(a)+/u.test('a')",
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
