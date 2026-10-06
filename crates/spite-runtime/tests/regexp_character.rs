//! Ordinary CharacterSetMatcher atoms and native execution (22.2.2.7.1).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn classes_ranges_inversion_and_empty_sets_match_one_utf16_unit() {
    check("/[]/.exec('a')===null && /[^]/.exec('\\n')[0]==='\\n'");
    check(
        "/[a-c]/.exec('xd')===null && /[^a-c]/.exec('abdx')[0]==='d' && /[--a]/.exec('A')[0]==='A' && /[-a]/.test('-') && /[a-]/.test('-') && /[[a]/.test('[')",
    );
}

#[test]
fn canonicalization_precedes_outer_inversion_and_preserves_original_case() {
    check(
        "/[a-z]/i.exec('A')[0]==='A' && /[^a-z]/i.exec('A')===null && /[σ]/i.exec('ς')[0]==='ς' && /[µ]/i.exec('Μ')[0]==='Μ' && /[s]/i.exec('ſ')===null && /[k]/i.exec('K')===null && /[ſK]/i.exec('ſ')[0]==='ſ'",
    );
}

#[test]
fn six_class_escapes_and_class_backspace_use_ecmascript_sets() {
    check(
        r"/\d/.exec('x1')[0]==='1' && /\D/.exec('1x')[0]==='x' && /\d/.exec('\u0661')===null && /\w/.exec('σ_')[0]==='_' && /\W/.exec('_σ')[0]==='σ' && /\w/i.exec('ſK')===null",
    );
    check(
        r"/\s/.exec('\uFEFF')[0]==='\uFEFF' && /\s/.exec('\u180E')===null && /\s/.exec('\u200B')===null && /\S/.exec('\nA')[0]==='A' && /[\b]/.exec('\b')[0]==='\b' && /[^\d\s]/.exec('1\nx')[0]==='x'",
    );
}

#[test]
fn dotall_and_surrogate_halves_keep_ordinary_consuming_ranges() {
    check(
        "['\\n','\\r','\\u2028','\\u2029'].every(t=>/./.exec(t)===null && /./s.exec(t)[0]===t) && /./m.exec('\\n')===null",
    );
    check(
        r"let r=/[\uD800-\uDFFF]/dg,m=r.exec('x\uD800\uDC00'),n=r.exec('x\uD800\uDC00');m[0]==='\uD800' && n[0]==='\uDC00' && m.indices[0].join(',')==='1,2' && n.indices[0].join(',')==='2,3' && r.lastIndex===3",
    );
    check(r"/./.exec('\uD800\uDC00')[0]==='\uD800' && /[𐀀]/.exec('\uDC00')[0]==='\uDC00'");
}

#[test]
fn global_sticky_strict_state_and_intrinsic_results_remain_ordered() {
    check(
        r"let r=/\d/dy;r.lastIndex=1;let m=r.exec('x1');m.index===1 && m.input==='x1' && m.length===1 && m.groups===undefined && m.indices.length===1 && m.indices.groups===undefined && r.lastIndex===2 && r.exec('x1')===null && r.lastIndex===0",
    );
    check(
        "let r=/[a]/g;Object.defineProperty(r,'lastIndex',{writable:false});let threw=false;try{r.exec('a')}catch(e){threw=e instanceof TypeError}threw && r.lastIndex===0",
    );
}

#[test]
fn all_generic_consumers_and_string_fallbacks_use_the_same_set_matcher() {
    check(
        r"'a1b2'.match(/\d/g).join(',')==='1,2' && 'a1b2'.search(/\d/)===1 && 'a1b2'.replaceAll(/\d/g,'x')==='axbx' && 'a,b;c'.split(/[,;]/).join('|')==='a|b|c' && [...'a1b2'.matchAll('\\d')].map(m=>m.index).join(',')==='1,3'",
    );
    check("'ba'.match('[a]')[0]==='a' && 'ba'.search('[a]')===1");
}

#[test]
fn copied_large_class_plans_survive_collection_without_default_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('['+'a'.repeat(100000)+'b]','dg'),copy=new RegExp(r),s='x'.repeat(120000)+'b';RegExp=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(s);m[0]==='b' && m.index===120000 && m.indices[0].join(',')==='120000,120001' && copy.lastIndex===120001 && r.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_construction_and_search_work_abort_outside_javascript() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("let flag=0,t='';").unwrap();
    assert!(matches!(realm.eval(r"try{new RegExp({toString(){t+='s';return '\\d'}},{toString(){t+='f';return ''}})}catch{flag=1}finally{flag=2}"),Err(Error::Limit{..})));
    assert_eq!(realm.eval("flag===0 && t==='sf'"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(70_000),
        ..Limits::default()
    });
    realm.eval(r"let r=/\d/g,flag=0,s='';").unwrap();
    realm.eval("s='x'.repeat(40000);").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn class_quantifiers_choices_assertions_and_unicode_modes_remain_explicit_gaps() {
    for source in [
        "/a[a]+/.test('aa')",
        "/([a])*/.test('a')",
        "/[a]*/.test('a')",
        "/[a]+|b/.test('a')",
        "/[a]/u.test('a')",
        "/./v.test('a')",
        r"/\b/.test('a')",
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
