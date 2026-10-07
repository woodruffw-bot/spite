//! Top-level choices compose quantified consuming branches and fixed captures.

use spite_runtime::{Error, Limits, Realm, Value};
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn earliest_positions_and_source_order_keep_per_branch_repetition_order() {
    check(
        "/a|a+/.exec('aaa')[0]==='a'&&/a+|a/.exec('aaa')[0]==='aaa'&&/a+?|a+/.exec('aaa')[0]==='a'&&/[ab]*ab|[ab]*?ab/.exec('abab')[0]==='abab'&&/[ab]*?ab|[ab]*ab/.exec('abab')[0]==='ab'",
    );
    check(
        "/([x])|[ab]+/.exec('aaax').index===0&&/a*|([b])/.exec('b')[0]===''&&/[]+|a/.exec('a')[0]==='a'",
    );
}

#[test]
fn variable_branches_retain_global_undefined_slots_and_empty_fixed_captures() {
    check(
        "let r=/([x])()|[ab]+|([y])/d,m=r.exec('bbb'),n=r.exec('x'),p=r.exec('y');m.length===4&&m[1]===undefined&&m[2]===undefined&&m[3]===undefined&&Object.hasOwn(m,'1')&&Object.hasOwn(m.indices,'2')&&m.indices[1]===undefined&&m.indices[0][1]===3&&n[1]==='x'&&n[2]===''&&n.indices[2][0]===1&&n.indices[2][1]===1&&p[3]==='y'&&p[1]===undefined",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let m=/([x])|a+/d.exec('aaa'),p=Object.getOwnPropertyDescriptor(m,'1'),q=Object.getOwnPropertyDescriptor(m.indices,'1');calls===0&&p.value===undefined&&p.writable&&p.enumerable&&p.configurable&&q.value===undefined&&q.writable",
    );
}

#[test]
fn global_sticky_and_strict_lastindex_follow_the_selected_complete_branch() {
    check(
        "let r=/[ab]+|([x])/g,a=r.exec('aaa xbb'),b=r.exec('aaa xbb'),c=r.exec('aaa xbb');a[0]==='aaa'&&a[1]===undefined&&b[1]==='x'&&c[0]==='bb'&&r.lastIndex===7&&r.exec('aaa xbb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/[ab]*ab|([x])/y;r.lastIndex=1;let m=r.exec('xabab');m[0]==='abab'&&m[1]===undefined&&r.lastIndex===5&&r.exec('xabab')===null&&r.lastIndex===0",
    );
    check(
        "let r=/a+|b/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aaa')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn multiline_fixed_branches_dotall_case_and_surrogate_units_stay_independent() {
    check(
        r"let r=/^([a])$|b+/m,a=r.exec('x\na\ny'),b=r.exec('xbb');a.index===2&&a[1]==='a'&&b.index===1&&b[1]===undefined",
    );
    check(
        r"/.+|([a])/s.exec('a\nb')[0]==='a\nb'&&/[µ]+x|a/i.exec('ΜµX')[0]==='ΜµX'&&/[ſ]+|a/i.exec('s')===null&&/\uD800+|a/.exec('\uD800\uD800\uDC00')[0].length===2",
    );
}

#[test]
fn generic_consumers_callbacks_and_empty_advancement_keep_global_slots() {
    check(
        "'xaa'.replace(/([x])|a+/g,'$1-')==='x--'&&'xaa'.split(/([x])|a+/).length===5&&'xxa'.search('[ab]+|([x])')===0",
    );
    check(
        "let seen=[];let s='xaa'.replace(/([x])|a+/g,(whole,cap,index)=>{seen.push(cap===undefined,index);return whole});s==='xaa'&&seen.join(',')==='false,0,true,1'",
    );
    check(
        "let all=[...'xaa'.matchAll(/([x])|a+/dg)];all.length===2&&all[0][1]==='x'&&all[1][1]===undefined&&all[1].indices[1]===undefined&&'aa'.match(/a*?|([b])/g).length===3",
    );
}

#[test]
fn copied_variable_plans_and_global_capture_layouts_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=/([x])|[a]*aaaaab/d,copy=new RegExp(r),s='a'.repeat(100000)+'b'")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec(s),n=copy.exec('x');m[0].length===100001&&m[1]===undefined&&m.indices[0][1]===100001&&n[1]==='x'"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_constructor_and_sticky_search_aborts_remain_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/\d+|a/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm
        .eval("let r=/a+|([b])/y,flag=0,s='a'.repeat(8000)")
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
fn nested_choices_quantified_captures_and_unsupported_branches_reject_the_whole_plan() {
    for source in [
        "/a+[b]+|a/.test('a')",
        "/(ab)+|b/.test('b')",
        "/a(a+|b)/.test('a')",
        "/a|^b+[a]+/.test('a')",
        "/a+|b/u.test('a')",
        r"/(a)\1|b+/.test('b')",
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
