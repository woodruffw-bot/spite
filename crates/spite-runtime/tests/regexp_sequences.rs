//! Fixed ordinary-mode sequences preserve RegExpBuiltinExec and consumer behavior.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn sequences_keep_earliest_start_global_state_and_sticky_failure() {
    check(
        "let r=/[Nn]evermore/g,s='x Nevermore nevermore';let a=r.exec(s),b=r.exec(s);a[0]==='Nevermore'&&a.index===2&&b[0]==='nevermore'&&b.index===12&&r.lastIndex===21&&r.exec(s)===null&&r.lastIndex===0",
    );
    check(
        "let r=/a[ab]b/y;r.lastIndex=1;let m=r.exec('xaabb');m[0]==='aab'&&m.index===1&&r.lastIndex===4&&r.exec('xaabb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/a[ab]b/;r.lastIndex=99;let m=r.exec('xxaabb');m.index===2&&m[0]==='aab'&&r.lastIndex===99",
    );
}

#[test]
fn nested_and_empty_groups_have_fixed_capture_and_indices_ranges() {
    check(
        "let m=/([a](b))()([c])/d.exec('xabc');m.length===5&&m[0]==='abc'&&m[1]==='ab'&&m[2]==='b'&&m[3]===''&&m[4]==='c'&&m.index===1&&m.groups===undefined&&m.indices.length===5&&m.indices[0][0]===1&&m.indices[0][1]===4&&m.indices[1][0]===1&&m.indices[1][1]===3&&m.indices[2][0]===2&&m.indices[2][1]===3&&m.indices[3][0]===3&&m.indices[3][1]===3&&m.indices[4][0]===3&&m.indices[4][1]===4&&m.indices.groups===undefined",
    );
    check(
        "let m=/(?:[a])([b])/d.exec('ab');m.length===2&&m[1]==='b'&&m.indices[1][0]===1&&m.indices[1][1]===2",
    );
}

#[test]
fn dot_class_escapes_and_ordinary_case_folding_apply_per_atom() {
    check(
        r"/[A]b/i.test('aB')&&/[µ]a/i.test('ΜA')&&!/[ſ]a/i.test('sA')&&!/[K]a/i.test('kA')&&/\d\w\s/.test('x1a ')&&/a[^b]c/.test('aac')&&!/a[^b]c/.test('abc')",
    );
    check(
        r"!/a.b/.test('a\nb')&&/a.b/s.test('a\nb')&&/a.b/s.test('a\u2028b')&&!/[]a/.test('a')&&/[^]a/.test('\na')",
    );
}

#[test]
fn ordinary_sequences_consume_surrogate_units_without_pair_reinterpretation() {
    check(
        r"let s='\uD800\uDC00',m=/([\uD800])([\uDC00])/d.exec(s);m[0]===s&&m[1].length===1&&m[2].length===1&&m.indices[1][0]===0&&m.indices[1][1]===1&&m.indices[2][0]===1&&m.indices[2][1]===2",
    );
    check(
        r"/[\uD800]./.test('\uD800\uDC00')&&/.[\uDC00]/.test('\uD800\uDC00')&&/[\uD800]x/.test('\uD800x')",
    );
}

#[test]
fn string_consumers_share_native_fixed_sequence_execution() {
    check(
        "'aa ab'.match(/[a][ab]/g).join(',')==='aa,ab'&&'xaa'.search(/[a]a/)===1&&'aa ab'.replace(/([a])([ab])/g,'$2$1')==='aa ba'&&'xaaYabZ'.split(/[a][ab]/).join(',')==='x,Y,Z'",
    );
    check(
        "let trace=[];let s='aa ab'.replace(/([a])([ab])/g,function(m,a,b,i){trace.push(m+':'+a+':'+b+':'+i);return b+a});s==='aa ba'&&trace.join(',')==='aa:a:a:0,ab:a:b:3'",
    );
    check(
        "let r=/([a])([ab])/dg,it='aa ab'.matchAll(r),a=it.next().value,b=it.next().value; a[1]==='a'&&a[2]==='a'&&a.indices[2][0]===1&&b.index===3&&b[2]==='b'&&b.indices[2][0]===4&&it.next().done&&r.lastIndex===0",
    );
}

#[test]
fn original_plans_survive_copy_collection_and_own_slot_shadowing() {
    let mut realm = Realm::default();
    realm
        .eval("let r=/([a])b/dg,c=new RegExp(r);Object.defineProperty(r,'source',{value:'wrong'});r.exec=function(){return null};")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=RegExp.prototype.exec.call(r,'ab'),b=c.exec('ab');a[1]==='a'&&b[1]==='a'&&a.indices[1][1]===1&&c.lastIndex===2"), Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_search_abort_keeps_state_and_skips_javascript_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[a]aaaaab/g,flag=0,s='x'.repeat(8000)")
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
fn opted_in_set_construction_abort_remains_a_host_failure() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval(r"try{new RegExp('[a]\\d')}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn opted_in_sticky_work_covers_one_candidate_instead_of_all_candidate_starts() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Limits::default()
    });
    realm
        .eval(&format!(
            "let r=/[a]{}/y,s='a'.repeat(201)",
            "a".repeat(200)
        ))
        .unwrap();
    assert_eq!(
        realm.eval("let m=r.exec(s);m[0]===s&&r.lastIndex===201"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn choices_quantifiers_assertions_backreferences_and_unicode_remain_explicit_gaps() {
    for source in [
        "/[a]+/.test('a')",
        "/[a]|b/.test('a')",
        "/^[a]/.test('a')",
        "/[a]b$/.test('ab')",
        r"/([a])\1/.test('aa')",
        "/(?<x>[a])/.test('a')",
        "/(?i:[a])/.test('a')",
        "/[a]b/u.test('ab')",
        "/[a]b/v.test('ab')",
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
