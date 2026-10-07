//! Fixed empty iterations stop once RepeatMatcher's minimum is satisfied.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn mandatory_empty_iterations_capture_while_optional_iterations_leave_undefined() {
    check(
        r"let a=/()+/d.exec('a'),b=/()*/d.exec('a'),c=/(){2}/d.exec('a');a[0]===''&&a[1]===''&&a.indices[1][0]===0&&b[1]===undefined&&b.indices[1]===undefined&&Object.hasOwn(b,'1')&&c[1]===''",
    );
    check(
        r"let a=/(())*?/d.exec('a'),b=/(()){1,3}?/d.exec('a');a[1]===undefined&&a[2]===undefined&&b[1]===''&&b[2]===''&&b.indices[1]!==b.indices[2]",
    );
    check(
        r"let a=/(){999999999999999999999999999999}/d.exec('a'),b=/(){0,999999999999999999999999999999}/d.exec('a');a[1]===''&&b[1]===undefined&&/(?:)+/.exec('a').length===1",
    );
}

#[test]
fn mandatory_assertions_use_complete_input_while_zero_minimum_skips_failed_bodies() {
    check(
        r"let a=/($)+/d.exec('ab'),b=/(^){2}/d.exec('ab');a.index===2&&a[1]===''&&a.indices[1][0]===2&&b.index===0&&b[1]===''&&/(^$)+/.exec('ab')===null&&/(^$)*/.exec('ab')[1]===undefined",
    );
    check(
        r"let a=/(\b){2}/d.exec(' a '),b=/(\B)+/d.exec('aa');a.index===1&&a[1]===''&&b.index===1&&/(\b\B)+/.exec('a')===null&&/(\b\B)*/.exec('a')[1]===undefined",
    );
    check(
        r"let a=/\b((\B)*)\b/d.exec('a'),b=/^((^$)*)$/d.exec('');a.index===0&&a[1]===''&&a[2]===undefined&&b[1]===''&&b[2]===undefined&&/^((^$)+)$/.exec('a')===null",
    );
}

#[test]
fn enclosing_alternative_captures_and_source_order_preserve_empty_participation() {
    check(
        r"let a=/(()*)|a+/d.exec('aaa'),b=/a+|(()*)/d.exec('aaa');a[0]===''&&a[1]===''&&a[2]===undefined&&b[0]==='aaa'&&b[1]===undefined&&b[2]===undefined",
    );
    check(
        r"let a=/((($){2}))|(a)/d.exec('a'),b=/((($){2}))|(a)/d.exec('');a[0]==='a'&&a[1]===undefined&&a[4]==='a'&&b[0]===''&&b[1]===''&&b[2]===''&&b[3]===''&&b[4]===undefined",
    );
    check(
        r"let a=/^((())+)$/d.exec('');a.length===4&&a[1]===''&&a[2]===''&&a[3]===''&&a.indices[1][0]===0&&a.indices[2]!==a.indices[3]",
    );
}

#[test]
fn global_and_sticky_exec_preserve_zero_length_last_index_and_ordered_failure_reset() {
    check(
        r"let r=/()*/dg;r.lastIndex=1;let a=r.exec('ab'),b=r.exec('ab');a.index===1&&b.index===1&&r.lastIndex===1&&a[1]===undefined",
    );
    check(
        r"let r=/($)+/dgy;r.lastIndex=1;let n=r.exec('ab'),failed=n===null&&r.lastIndex===0;r.lastIndex=2;let a=r.exec('ab');failed&&a.index===2&&a[1]===''&&r.lastIndex===2",
    );
    check(
        r"let r=/(^)+/dgy;r.lastIndex=1;let n=r.exec('ab'),failed=n===null&&r.lastIndex===0&&r.exec('ab')[1]==='';r.lastIndex=3;failed&&r.exec('ab')===null&&r.lastIndex===0",
    );
}

#[test]
fn multiline_flags_crlf_ascii_word_rules_and_surrogate_offsets_remain_explicit() {
    check(
        r"let r=/(^){2}/dgm;r.lastIndex=1;let a=r.exec('a\r\nb');a.index===2&&a.indices[1][0]===2;r.lastIndex=3;let b=r.exec('a\r\nb');b.index===3&&r.lastIndex===3",
    );
    check(
        r"let r=/($){2}/dgm,a=r.exec('a\u2028b\u2029');a.index===1&&a[1]==='';r.lastIndex=2;let b=r.exec('a\u2028b\u2029');b.index===3&&b.indices[1][0]===3&&/($){2}/.exec('a\n').index===2",
    );
    check(
        r"let a=/(\B)+/di.exec('ſK'),b=/(\b)+/di.exec('µa'),r=/()+/dy;r.lastIndex=1;let c=r.exec('💩');a.index===0&&b.index===1&&c.index===1&&c.indices[1][0]===1&&r.lastIndex===1&&/(^$)+/s.exec('')!==null",
    );
}

#[test]
fn generic_consumers_advance_empty_matches_and_preserve_callback_capture_arguments() {
    check(
        r"'a'.replace(/()*/g,'x')==='xax'&&'ab'.split(/()+/).join(',')==='a,,b'&&'ab'.search(/($)+/)===2",
    );
    check(
        r"let seen=[],s='a'.replace(/()*/g,(whole,capture,index,input)=>{seen.push([whole,capture,index,input]);return 'x'});s==='xax'&&seen.length===2&&seen[0][0]===''&&seen[0][1]===undefined&&seen[0][2]===0&&seen[1][2]===1&&seen[1][3]==='a'",
    );
    check(
        r"let r=/()+/dg,a=[...'💩'.matchAll(r)];a.length===3&&a[0].index===0&&a[1].index===1&&a[2].index===2&&a[1][1]===''&&r.lastIndex===0&&'a'.match(/()*/g).length===2",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/()*/d.exec('a');calls===0&&a[1]===undefined&&a.indices[1]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a.indices,'1')",
    );
}

#[test]
fn huge_mandatory_counts_and_collapsed_assertion_captures_survive_collection_without_quotas() {
    let mut realm = Realm::default();
    realm.eval("let s='a\\n'.repeat(100000),r=new RegExp('('+'(^)'.repeat(100000)+'){999999999999999999999999999999}','dgm'),copy=new RegExp(r);copy.lastIndex=3").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec(s);a.index===4&&a[0]===''&&a.length===100002&&a[1]===''&&a[100001]===''&&a.indices[100001][0]===4&&a.indices[1]!==a.indices[100001]&&copy.lastIndex===4&&r.lastIndex===0&&r.source===copy.source"),Ok(Value::Boolean(true)));
}

#[test]
fn optional_assertion_search_aborts_bypass_handlers_and_other_group_features_stay_unsupported() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(100000),
        ..Limits::default()
    });
    realm
        .eval("let r=/($){2}/g,s='a'.repeat(20000),flag=0;r.lastIndex=3")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/(|b)*a/.test('a')",
        r"/a(|b)+/.test('a')",
        r"/(^|$)+/.test('a')",
        r"/(?<n>)+/.test('')",
        r"/()\1+/.test('')",
        r"/(a*)*/.test('a')",
        r"/()+/u.test('')",
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
