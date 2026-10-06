//! Top-level Disjunction capture participation and RegExpBuiltinExec (22.2.2.3, 22.2.7.2).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn source_order_selects_capture_participation_even_for_identical_matches() {
    check(
        "let m=/a|(a)/d.exec('a');m.length===2 && m[1]===undefined && Object.hasOwn(m,'1') && m.indices[1]===undefined && Object.hasOwn(m.indices,'1')",
    );
    check("let m=/(a)|a/d.exec('a');m.length===2 && m[1]==='a' && m.indices[1].join(',')==='0,1'");
    check(
        "let m=/(a)|(ab)/.exec('ab');m[0]==='a' && m[1]==='a' && m[2]===undefined && /(ab)|(a)/.exec('ab')[1]==='ab'",
    );
}

#[test]
fn selected_groups_keep_global_source_order_and_relative_utf16_ranges() {
    check(
        "let m=/((a)())|c(d)/d.exec('xcd');m.length===5 && m[0]==='cd' && m.index===1 && m[1]===undefined && m[2]===undefined && m[3]===undefined && m[4]==='d' && m.indices.length===5 && m.indices[0].join(',')==='1,3' && m.indices[1]===undefined && m.indices[2]===undefined && m.indices[3]===undefined && m.indices[4].join(',')==='2,3'",
    );
    check(
        "let m=/((a)())|c(d)/d.exec('xa');m[1]==='a' && m[2]==='a' && m[3]==='' && m[4]===undefined && m.indices[1].join(',')==='1,2' && m.indices[3].join(',')==='2,2' && m.indices[4]===undefined && m.groups===undefined && m.indices.groups===undefined",
    );
}

#[test]
fn empty_participating_groups_are_distinct_from_unselected_groups() {
    check(
        "let r=/()|(a)/dg,m=r.exec('a');m[0]==='' && m[1]==='' && m[2]===undefined && m.indices[1].join(',')==='0,0' && m.indices[2]===undefined && r.lastIndex===0",
    );
    check(
        "let all=[...'a'.matchAll(/()|(a)/g)];all.length===2 && all[0][1]==='' && all[0][2]===undefined && all[1].index===1 && all[1][1]==='' && all[1][2]===undefined",
    );
}

#[test]
fn original_case_surrogate_halves_and_sticky_state_survive_branch_selection() {
    check(
        "let r=/(a)|(σ)/diy;r.lastIndex=1;let m=r.exec('xς');m[1]===undefined && m[2]==='ς' && m.indices[2].join(',')==='1,2' && r.lastIndex===2 && r.exec('xς')===null && r.lastIndex===0",
    );
    check(
        r"let m=/(\uD800)|c(\uDC00)/d.exec('xc\uDC00');m[1]===undefined && m[2]==='\uDC00' && m.indices[2].join(',')==='2,3'",
    );
    check(
        r"let m=/(\uD800)|c(\uDC00)/d.exec('x\uD800\uDC00');m[1]==='\uD800' && m[2]===undefined && m.indices[1].join(',')==='1,2'",
    );
}

#[test]
fn global_results_do_not_retain_previous_branch_capture_state() {
    check(
        "let r=/(a)|(b)/dg,a=r.exec('ab'),b=r.exec('ab');a[1]==='a' && a[2]===undefined && b[1]===undefined && b[2]==='b' && a.indices[1].join(',')==='0,1' && b.indices[2].join(',')==='1,2' && a!==b && a.indices!==b.indices && r.lastIndex===2 && r.exec('ab')===null && r.lastIndex===0",
    );
    check(
        "let r=/(a)|(b)/g;Object.defineProperty(r,'lastIndex',{writable:false});let threw=false;try{r.exec('a')}catch(e){threw=e instanceof TypeError}threw && r.lastIndex===0",
    );
}

#[test]
fn absent_captures_are_own_data_properties_without_prototype_setter_calls() {
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let m=/(a)|(b)/d.exec('a'),p=Object.getOwnPropertyDescriptor(m,'2'),q=Object.getOwnPropertyDescriptor(m.indices,'2');calls===0 && p.value===undefined && p.writable && p.enumerable && p.configurable && q.value===undefined && q.writable && q.enumerable && q.configurable",
    );
}

#[test]
fn replacements_callbacks_and_split_preserve_undefined_capture_slots() {
    check(
        "let seen=[];let result='ab'.replace(/(a)|(b)/g,(whole,a,b,index,input)=>{seen.push(a===undefined,b===undefined,index,input);return whole;});result==='ab' && seen.join(',')==='false,true,0,ab,true,false,1,ab' && 'ab'.replace(/(a)|(b)/g,'$1-$2')==='a--b'",
    );
    check(
        "let parts='axb'.split(/(x)|(y)/);parts.length===4 && parts[0]==='a' && parts[1]==='x' && parts[2]===undefined && Object.hasOwn(parts,'2') && parts[3]==='b'",
    );
    check(
        "let all=[...'ab'.matchAll(/(a)|(b)/g)];all.length===2 && all[0][1]==='a' && all[0][2]===undefined && all[1][1]===undefined && all[1][2]==='b' && 'ab'.search('(z)|(b)')===1 && 'ab'.match('(z)|(b)')[2]==='b'",
    );
}

#[test]
fn large_capture_inventories_and_copied_plans_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('(a)|'.repeat(1000)+'(b)','dg'),copy=new RegExp(r);RegExp=null;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('xb'),n=copy.exec('a');m.length===1002 && m[1]===undefined && m[1000]===undefined && m[1001]==='b' && m.indices[1001].join(',')==='1,2' && m.indices[1]===undefined && n[1]==='a' && n[2]===undefined && n[1001]===undefined && copy.lastIndex===1 && r.lastIndex===2"),Ok(Value::Boolean(true)));
}
