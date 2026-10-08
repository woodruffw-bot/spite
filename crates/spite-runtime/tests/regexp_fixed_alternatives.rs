//! Fixed ordinary alternatives preserve ordering, captures and original slots.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn earliest_positions_and_source_order_ties_precede_match_length() {
    check(
        "/[ab]|a[ab]/.exec('ab')[0]==='a'&&/a[ab]|[ab]/.exec('ab')[0]==='ab'&&/xx|[ab]/.exec('axxx').index===0",
    );
    check(
        "/[]|a/.exec('a')[0]==='a'&&/[^]|a/.exec('a')[0]==='a'&&/()|([a])/.exec('a')[0]===''&&/([a])|()/.exec('a')[0]==='a'",
    );
}

#[test]
fn selected_class_branches_keep_global_capture_and_indices_slots() {
    check(
        "let m=/([ab])([a])|([a])/d.exec('ba'),n=/([ab])([a])|([a])/d.exec('a');m.length===4&&m[1]==='b'&&m[2]==='a'&&m[3]===undefined&&Object.hasOwn(m,'3')&&m.indices[1][0]===0&&m.indices[2][0]===1&&m.indices[3]===undefined&&Object.hasOwn(m.indices,'3')&&n[1]===undefined&&n[2]===undefined&&n[3]==='a'&&n.indices[3][0]===0&&n.indices[3][1]===1",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'3',{set(){calls++},configurable:true});let m=/([ab])([a])|([a])/d.exec('ba'),p=Object.getOwnPropertyDescriptor(m,'3'),q=Object.getOwnPropertyDescriptor(m.indices,'3');calls===0&&p.value===undefined&&p.writable&&p.enumerable&&p.configurable&&q.value===undefined&&q.writable&&q.enumerable&&q.configurable",
    );
}

#[test]
fn global_and_sticky_state_follow_the_selected_branch() {
    check(
        "let r=/[ab]a|b/g,a=r.exec('ba b'),b=r.exec('ba b');a[0]==='ba'&&a.index===0&&b[0]==='b'&&b.index===3&&r.lastIndex===4&&r.exec('ba b')===null&&r.lastIndex===0",
    );
    check(
        "let r=/[ab]a|b/y;r.lastIndex=1;let m=r.exec('xba');m[0]==='ba'&&m.index===1&&r.lastIndex===3&&r.exec('xba')===null&&r.lastIndex===0",
    );
}

#[test]
fn per_branch_anchors_keep_complete_input_and_multiline_boundaries() {
    check(
        r"let r=/^([a])$|([b])$/dmi,a=r.exec('x\na\nyb'),b=r.exec('xb');a.index===2&&a[1]==='a'&&a[2]===undefined&&b.index===1&&b[1]===undefined&&b[2]==='b'&&b.indices[2][0]===1&&b.indices[2][1]===2",
    );
    check(r"/^a|[b]$/.exec('xa b').index===3&&/^([a])|([b])$/my.exec('x\na')===null");
}

#[test]
fn dotall_case_rules_surrogate_units_and_class_delimiters_stay_independent() {
    check(
        r"/([a])|./s.exec('\n')[1]===undefined&&/[µ]|a/i.exec('Μ')[0]==='Μ'&&/[ſ]|a/i.exec('s')===null&&/([\uD800])|./.exec('\uD800\uDC00')[1]==='\uD800'",
    );
    check(
        r"/[(|)]|x/.test('|')&&/[(|)]|x/.test('(')&&/[\]]|x/.test(']')&&/[[a]|b/.test('[')&&/(?:[|])|(a)/d.exec('|')[1]===undefined",
    );
}

#[test]
fn generic_string_consumers_preserve_unselected_slots() {
    check(
        "'ab'.replace(/([a])|([b])/g,'$1-$2')==='a--b'&&'axb'.split(/([x])|([y])/).length===4&&'axb'.split(/([x])|([y])/)[2]===undefined&&'ab'.search('([z])|([b])')===1",
    );
    check(
        "let seen=[];let result='ab'.replace(/([a])|([b])/g,(whole,a,b,index)=>{seen.push(a===undefined,b===undefined,index);return whole});result==='ab'&&seen.join(',')==='false,true,0,true,false,1'",
    );
    check(
        "let all=[...'ab'.matchAll(/([a])|([b])/dg)];all.length===2&&all[0][1]==='a'&&all[0][2]===undefined&&all[1][1]===undefined&&all[1][2]==='b'&&all[1].indices[2][0]===1",
    );
}

#[test]
fn copied_plans_and_capture_layouts_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=/([a])|([b])/d,copy=new RegExp(r)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('b'),n=copy.exec('a');m[1]===undefined&&m[2]==='b'&&n[1]==='a'&&n[2]===undefined&&m.indices[2][1]===1"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_construction_and_search_aborts_remain_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{/\d|[a]/}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[a]a|^b$/g,flag=0,s='x'.repeat(8000)")
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
fn nested_choices_quantifiers_backreferences_and_unicode_remain_explicit_gaps() {
    for source in [
        r"/(?:(?:(?:(?:a([^a]|bc)){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:((?:(?:[a]+[b]+|b){2}){2}){2})|){2,3}/.test('b')",
        r"/(?:(?:((?:(?:a|[b]*[a]+){2}){2}){2})|){2,3}/.test('a')",
        r"/(?:(?:(?:([a])(?:\1)+|b){2})|){2,3}/.test('b')",
        "/[a]|b/u.test('a')",
        "/[a]|b/v.test('a')",
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
