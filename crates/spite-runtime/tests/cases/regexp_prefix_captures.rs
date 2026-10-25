//! Ordinary literal-prefix groups retain fixed and repeated capture source order.

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prefix_captures_precede_repeated_and_suffix_groups() {
    check(
        "let m=/()(a)()(b)+((c)())/d.exec('xabbc');m.index===1&&m.length===8&&m[1]===''&&m[2]==='a'&&m[3]===''&&m[4]==='b'&&m[5]==='c'&&m[6]==='c'&&m[7]===''&&m.indices[1][0]===1&&m.indices[2][1]===2&&m.indices[3][0]===2&&m.indices[4][0]===3&&m.indices[7][0]===5",
    );
    check(
        "let m=/((a))(b*)()/d.exec('a');m[1]==='a'&&m[2]==='a'&&m[3]===''&&m[4]===''&&m.indices[3][0]===1&&m.indices[3][1]===1",
    );
    check(
        "let m=/(a)(b)*()/d.exec('a');m[1]==='a'&&m[2]===undefined&&m[3]===''&&Object.hasOwn(m,'2')&&Object.hasOwn(m.indices,'2')&&m.indices[2]===undefined&&m.indices[3][0]===1",
    );
}

#[test]
fn empty_prefix_groups_keep_participation_and_earliest_body_positions() {
    check(
        "let m=/()([ab])+()/d.exec('xaa');m.index===1&&m[0]==='aa'&&m[1]===''&&m[2]==='a'&&m[3]===''&&m.indices[1][0]===1&&m.indices[2][0]===2&&m.indices[3][0]===3",
    );
    check(
        "let m=/()([ab])*?()/d.exec('aa');m[0]===''&&m[1]===''&&m[2]===undefined&&m[3]===''&&m.indices[1][0]===0&&m.indices[3][0]===0",
    );
    check(
        "let m=/(?:)a+/.exec('xaa');m.index===1&&m[0]==='aa'&&m.length===1&&/()[]*/.exec('x')[1]===''&&/()[]+/.exec('x')===null",
    );
}

#[test]
fn global_sticky_empty_progress_and_strict_writes_remain_ordered() {
    check(
        "let r=/(a)(b)+/dgy;r.lastIndex=1;let m=r.exec('xabb');m.index===1&&m[1]==='a'&&m[2]==='b'&&m.indices[2][0]===3&&r.lastIndex===4&&r.exec('xabb')===null&&r.lastIndex===0",
    );
    check(
        "let r=/()a*?/dg,m=r.exec('aa'),all=[...'aa'.matchAll(r)];m[0]===''&&m[1]===''&&r.lastIndex===0&&all.length===3&&all[2].index===2&&all[2].indices[1][0]===2",
    );
    check(
        "let r=/(a)b+/g;Object.defineProperty(r,'lastIndex',{writable:false});let fail=false;try{r.exec('abb')}catch(e){fail=e instanceof TypeError}fail",
    );
}

#[test]
fn prefix_groups_compose_with_branch_outer_captures_and_anchor_selection() {
    check(
        "let m=/([x])|(a)(b)+(c)|([y])/d.exec('abbc');m.length===6&&m[1]===undefined&&m[2]==='a'&&m[3]==='b'&&m[4]==='c'&&m[5]===undefined&&m.indices[3][0]===2",
    );
    check(
        "let m=/^((a)(b)+())$/d.exec('abb');m.length===5&&m[1]==='abb'&&m[2]==='a'&&m[3]==='b'&&m[4]===''&&m.indices[1][1]===3&&m.indices[3][0]===2",
    );
    check(
        r"let r=/^()(a)*?(ab)$/dmy;r.lastIndex=2;let m=r.exec('x\naaaab\ny');m.index===2&&m[1]===''&&m[2]==='a'&&m[3]==='ab'&&m.indices[2][0]===4&&m.indices[3][0]===5&&r.lastIndex===7",
    );
    check(
        "let a=/^()a*$/d.exec('aa'),b=/^()a*?$/d.exec('aa');a[0]==='aa'&&b[0]==='aa'&&b[1]===''&&b.indices[1][0]===0",
    );
}

#[test]
fn decoded_prefix_captures_keep_original_utf16_ranges_and_flags() {
    check(
        r"let m=/(\u0061)(b)+/d.exec('abb');m[1]==='a'&&m[2]==='b'&&m.indices[1][1]===1&&m.indices[2][0]===2",
    );
    check(
        "let m=/(µ)(a)+/di.exec('ΜAA');m[1]==='Μ'&&m[2]==='A'&&m.indices[1][1]===1&&m.indices[2][0]===2&&/(ſ)a+/i.exec('saa')===null",
    );
    check(
        "let m=/(💩)(a)+/d.exec('x💩aa');m.index===1&&m[1]==='💩'&&m.indices[1][1]===3&&m.indices[2][0]===4",
    );
    check(
        r"let m=/(\n)(.)+/ds.exec('x\n\n');m.index===1&&m[1]==='\n'&&m[2]==='\n'&&m.indices[2][0]===2",
    );
}

#[test]
fn generic_consumers_callbacks_and_intrinsic_arrays_keep_prefix_slots() {
    check(
        "'abbc'.replace(/(a)(b)+(c)/,'$1-$2-$3')==='a-b-c'&&'abbxabb'.split(/(a)(b)+/).join(',')===',a,b,x,a,b,'",
    );
    check(
        "let seen;let s='abbc'.replace(/()(a)(b)+(c)/,(whole,empty,a,b,c,index,input)=>{seen=[whole,empty,a,b,c,index,input];return 'z'});s==='z'&&seen.join(',')==='abbc,,a,b,c,0,abbc'",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'2',{set(){calls++},configurable:true});let m=/(a)(b)*()/d.exec('a');calls===0&&m[1]==='a'&&m[2]===undefined&&Object.hasOwn(m,'2')&&m.indices[2]===undefined&&Object.hasOwn(m.indices,'2')&&m[3]===''",
    );
}

#[test]
fn optional_work_abort_and_remaining_nonliteral_prefixes_stay_host_failures() {
    for source in [
        r"/(?:(?:(?:(?:([a])b+c+){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:(a|bc)c+){2}){2})|){2,3}/.test('ac')",
        r"/(?:(?:(?:(?:(a)b+c+){2}){2})|){2,3}/.test('abc')",
        r"/(?:(?:(?:(?:(a)b+[c]+){2}){2})|){2,3}/.test('abc')",
        "/(a)b+/u.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}
