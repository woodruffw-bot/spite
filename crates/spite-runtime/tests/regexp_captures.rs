//! Ordinary literal captures and MatchIndices Arrays (22.2.2.7, 22.2.7.2).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn nested_and_empty_captures_follow_opening_parenthesis_order() {
    check(
        "let r=/a(b(c))()d/d,m=r.exec('xabcd');m.length===4 && m[0]==='abcd' && m[1]==='bc' && m[2]==='c' && m[3]==='' && m.index===1 && m.groups===undefined && m.indices.length===4 && m.indices[0].join(',')==='1,5' && m.indices[1].join(',')==='2,4' && m.indices[2].join(',')==='3,4' && m.indices[3].join(',')==='4,4' && m.indices.groups===undefined && r.source==='a(b(c))()d'",
    );
    check("let m=/(?:a)(b)(?:c)/.exec('abc');m.length===2 && m[1]==='b' && m.indices===undefined");
    check(r"let m=/(\(x\))/.exec('x(x)');m[0]==='(x)' && m[1]==='(x)' && m.index===1");
}

#[test]
fn captures_preserve_input_case_and_independent_surrogate_halves() {
    check("let m=/(a)(b)/i.exec('xAB');m[0]==='AB' && m[1]==='A' && m[2]==='B' && m.index===1");
    check(
        r"let m=/(\uD83D)(\uDCA9)/d.exec('x💩');m[0]==='💩' && m[1]==='\uD83D' && m[2]==='\uDCA9' && m.indices[1].join(',')==='1,2' && m.indices[2].join(',')==='2,3'",
    );
}

#[test]
fn capture_results_share_global_sticky_and_strict_lastindex_writes() {
    check(
        "let r=/(a)/gy;r.lastIndex=1;let m=r.exec('baa');m[1]==='a' && m.index===1 && r.lastIndex===2 && r.exec('baa')[1]==='a' && r.lastIndex===3 && r.exec('baa')===null && r.lastIndex===0",
    );
    check(
        "let r=/(a)/g;Object.defineProperty(r,'lastIndex',{writable:false});try{r.exec('a');false;}catch(e){e instanceof TypeError && r.lastIndex===0;}",
    );
    check(
        "let r=/()/g;r.lastIndex=2;let m=r.exec('ab');m[0]==='' && m[1]==='' && m.index===2 && r.lastIndex===2",
    );
}

#[test]
fn capture_arrays_use_intrinsics_own_properties_and_standard_attributes() {
    check(
        "let P=Array.prototype;Object.defineProperty(P,'1',{set(){throw 7;},configurable:true});let r=/(a)(b)/d;Array=function(){throw 8;};let m=r.exec('ab'),d=Object.getOwnPropertyDescriptor(m,'1');Object.getPrototypeOf(m)===P && Object.getPrototypeOf(m.indices)===P && Object.getPrototypeOf(m.indices[1])===P && d.value==='a' && d.writable && d.enumerable && d.configurable && m.indices[1][0]===0 && m.indices[1][1]===1",
    );
    check(
        "let m=/()()/.exec('');Object.getOwnPropertyNames(m).join(',')==='0,1,2,length,index,input,groups' && m.length===3 && m[1]==='' && m[2]===''",
    );
}

#[test]
fn replacement_split_match_and_lazy_matchall_consume_native_captures() {
    check("'xab'.replace(/(a)(b)/,'$2$1')==='xba' && 'abab'.replaceAll(/(a)(b)/g,'$2$1')==='baba'");
    check(
        "let seen='';let s='xab'.replace(/(a)(b)/,function(m,a,b,index,input){seen=[m,a,b,index,input,arguments.length].join(',');return b+a;});s==='xba' && seen==='ab,a,b,1,xab,5'",
    );
    check(
        "'a,b,'.split(/(,)/).join('|')==='a|,|b|,|' && 'ab'.match(/(a)(b)/)[2]==='b' && [...'abab'.matchAll(/(a)(b)/dg)].map(m=>m[1]+m.indices[2][0]).join(',')==='a1,a3'",
    );
    check("'ab'.match(/()/g).length===3 && [...'ab'.matchAll(/()/g)].every(m=>m[1]==='')");
}

#[test]
fn constructor_copies_preserve_capture_plans_and_original_text() {
    check(
        "let a=/(a(?:b)c)/dg,b=new RegExp(a);a.lastIndex=7;let m=b.exec('xabc');a!==b && b.source==='(a(?:b)c)' && b.lastIndex===4 && m[1]==='abc' && m.indices[1].join(',')==='1,4'",
    );
}

#[test]
fn many_flat_captures_and_plans_survive_collection_without_default_count_quota() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('()'.repeat(1000),'d');")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('');m.length===1001 && m.every(s=>s==='') && m.indices.length===1001 && m.indices.every(pair=>pair[0]===0 && pair[1]===0)"),Ok(Value::Boolean(true)));
}

#[test]
fn named_quantified_and_backreference_patterns_remain_unsupported() {
    for source in [
        "/(?<x>a)\\k<x>/.test('a')",
        "/(ab|c)*/.test('a')",
        "/a(a|bc)/.test('a')",
        "/([a]b|c)*/.test('a')",
        r"/(a)\1/.test('aa')",
        "/(a)/u.test('a')",
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
