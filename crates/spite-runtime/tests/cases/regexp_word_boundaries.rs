//! Outer ordinary word assertions constrain complete match candidates (22.2.2.4).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn word_assertions_use_ascii_word_characters_even_with_ordinary_ignore_case() {
    check(
        r"let m=/\bfoo\b/d.exec(' foo ');m[0]==='foo'&&m.index===1&&m.indices[0][1]===4&&/\bfoo\b/.exec('xfooy')===null&&/\Bfoo\B/.exec('xfooy').index===1",
    );
    check(
        r"/\b_0Az\b/.test('_0Az')&&/\ba\b/i.exec('ſA').index===1&&/\ba\b/i.exec('KA').index===1&&/\bſ\b/i.exec('ſ')===null&&/\bK\b/i.exec('K')===null&&/\bµ\b/i.exec('µ')===null",
    );
    check(
        r"/\Bµ\B/i.exec('Μ')[0]==='Μ'&&/\b\b/.exec('a').index===0&&/\b\B/.exec('a')===null&&/\B/.exec('').index===0&&/^$\B/.test('')",
    );
}

#[test]
fn greedy_lazy_repetition_selects_only_endpoints_satisfying_assertions() {
    check(
        r"let a=/a*\b/.exec('aaa'),b=/a*?\b/.exec('aaa');a[0]==='aaa'&&b[0]===''&&a.index===0&&b.index===0&&/a+\b/.exec('aaab')===null&&/\Ba+/.exec('baaa').index===1",
    );
    check(
        r"let a=/\b([ab])+([ab])\b/d.exec(' abba '),b=/\b([ab])+?([ab])\b/d.exec(' abba ');a[0]==='abba'&&b[0]==='abba'&&a[1]==='b'&&a[2]==='a'&&a.indices[1][0]===3&&b.indices[2][0]===4",
    );
    check(
        r"let a=/\B([x])([ab])*?([ab])\b/d.exec('yxaaab ');a.index===1&&a[0]==='xaaab'&&a[1]==='x'&&a[2]==='a'&&a[3]==='b'&&a.indices[2][0]===4&&a.indices[3][0]===5",
    );
}

#[test]
fn capture_slots_empty_participation_and_branch_order_remain_observable() {
    check(
        r"let a=/\b((a*)())\b/d.exec('aaa'),b=/\b(a)*()\b/d.exec(' ');a[1]==='aaa'&&a[2]==='aaa'&&a[3]===''&&a.indices[3][0]===3&&b===null",
    );
    check(
        r"let a=/\b(a)*()\b/d.exec('b');a[0]===''&&a[1]===undefined&&a[2]===''&&Object.hasOwn(a,'1')&&a.indices[1]===undefined&&a.indices[2][0]===0",
    );
    check(
        r"let a=/(\b(a+)\b)|(a+)/d.exec('aaa');a.length===4&&a[1]==='aaa'&&a[2]==='aaa'&&a[3]===undefined&&a.indices[1][0]===0&&a.indices[2][1]===3",
    );
    check(
        r"let a=/((\b(a+)\b))/d.exec(' aaa ');a[1]==='aaa'&&a[2]==='aaa'&&a[3]==='aaa'&&a.indices[3][0]===1",
    );
}

#[test]
fn input_line_assertions_compose_without_reinterpreting_sticky_start() {
    check(
        r"/^^a$$/.test('a')&&/a^/.exec('a')===null&&/$a^/m.exec('a')===null&&/^\b(a+)\b$/m.exec('x\naaa\ny').index===2",
    );
    check(
        r"let r=/^\ba+\b$/my;r.lastIndex=2;let a=r.exec('x\naaa\ny');a.index===2&&a[0]==='aaa'&&r.lastIndex===5&&r.exec('x\naaa\ny')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/\ba+\b/y;r.lastIndex=1;r.exec('aaa')===null&&r.lastIndex===0&&/\b.\b/s.exec('a')[0]==='a'&&/\b.\b/s.exec('\n')===null",
    );
}

#[test]
fn zero_width_global_sticky_matches_keep_utf16_positions_and_last_index() {
    check(
        r"let r=/\b/dg,a=r.exec('💩a');a.index===2&&a[0]===''&&a.indices[0][0]===2&&r.lastIndex===2&&r.exec('💩a').index===2&&r.lastIndex===2",
    );
    check(
        r"let r=/\B/dy;r.lastIndex=1;let a=r.exec('💩a');a.index===1&&a.indices[0][1]===1&&r.lastIndex===1",
    );
    check(r"let r=/\b/g;r.lastIndex=10;r.exec('a')===null&&r.lastIndex===0");
    check(
        r"let r=/\b/g;Object.defineProperty(r,'lastIndex',{writable:false});let failed=false;try{r.exec('a')}catch(e){failed=e instanceof TypeError}failed",
    );
}

#[test]
fn generic_consumers_advance_empty_results_and_preserve_callback_slots() {
    check(
        r"let m=[...'💩a'.matchAll(/\b/dg)],n=[...'💩a'.matchAll(/\B/dg)];m.length===2&&m[0].index===2&&m[1].index===3&&n.length===2&&n[0].index===0&&n[1].index===1",
    );
    check(
        r"' foo '.replace(/\b(foo)\b/,'<$1>')===' <foo> '&&' foo '.search(/\bfoo\b/)===1&&'ab cd'.split(/\b/).join(',')==='ab, ,cd'",
    );
    check(
        r"let seen;let s=' aa '.replace(/\b(a+)\b/,(whole,a,index,input)=>{seen=[whole,a,index,input];return 'x'});s===' x '&&seen.join(',')==='aa,aa,1, aa '",
    );
    check(
        r"let calls=0;Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/\b(a)*()\b/d.exec('b');calls===0&&a[1]===undefined&&Object.hasOwn(a,'1')&&Object.hasOwn(a.indices,'1')&&a.indices[1]===undefined",
    );
}

#[test]
fn optional_work_and_remaining_unsupported_assertion_forms_keep_host_ordering() {
    for source in [
        r"/(?:(?:((?:(?:a+^b+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:\b(ab|a)+\b)|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:\ba+b+\b){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:\b(?<n>a)\b(?:\k<n>)+){2})|){2,3}/.test('a')",
        r"/\ba+\bb/u.test('a')",
        r"/(?:(?:^((\b(ab|a)+\b))$)|){2,3}/.test('ab')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}
