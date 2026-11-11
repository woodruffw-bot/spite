//! Quantified prefixes preserve continuation-driven repetition ordering.

use spite_runtime::Realm;

use crate::common::assert_true;

#[test]
fn greedy_lazy_continuations_choose_the_earliest_start_before_match_length() {
    assert_true(
        "/[ab]*ab/.exec('abab')[0]==='abab'&&/[ab]*?ab/.exec('abab')[0]==='ab'&&/a+aa/.exec('baaaa')[0]==='aaaa'&&/a+?aa/.exec('baaaa')[0]==='aaa'",
    );
    assert_true(
        "let m=/[ab]{2,3}b/.exec('aababb'),n=/[ab]{2,3}?b/.exec('aababb');m.index===0&&m[0]==='aab'&&n.index===0&&n[0]==='aab'&&/a{1,3}a/.exec('aaaaa')[0]==='aaaa'",
    );
    assert_true(
        "/[Nn]?evermore/.exec('Nevermore')[0]==='Nevermore'&&/[Nn]?evermore/.exec('evermore')[0]==='evermore'&&/(?:a)+?(?:aa)/.exec('aaaa')[0]==='aaa'",
    );
}

#[test]
fn empty_and_failed_prefixes_preserve_candidate_positions() {
    assert_true(
        "let m=/[]*x/.exec('aax');m.index===2&&m[0]==='x'&&/[]+x/.exec('aax')===null&&/a{0}a/.exec('baa').index===1&&/a+(?:)/.exec('baaa')[0]==='aaa'",
    );
}

#[test]
fn global_sticky_and_strict_lastindex_follow_the_complete_match() {
    assert_true(
        "let r=/\\d+x/g,a=r.exec('12x 3x'),b=r.exec('12x 3x');a[0]==='12x'&&a.index===0&&b[0]==='3x'&&b.index===4&&r.lastIndex===6&&r.exec('12x 3x')===null&&r.lastIndex===0",
    );
    assert_true(
        "let r=/[ab]*ab/y;r.lastIndex=1;let m=r.exec('xabab');m.index===1&&m[0]==='abab'&&r.lastIndex===5&&r.exec('xabab')===null&&r.lastIndex===0",
    );
    assert_true("let r=/a+b/y;r.lastIndex=1;r.exec('xaaacb')===null&&r.lastIndex===0");
    assert_true(
        "let r=/a+b/g;Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;try{r.exec('aab')}catch(e){ok=e instanceof TypeError}ok",
    );
}

#[test]
fn prefix_sets_suffix_case_dotall_and_utf16_units_use_pinned_semantics() {
    assert_true(
        r"/.+x/s.exec('a\nx')[0]==='a\nx'&&/.*x/.exec('a\nx')[0]==='x'&&/[µ]+x/i.exec('ΜµX')[0]==='ΜµX'&&/[ſ]+x/i.exec('sx')===null",
    );
    assert_true(
        r"let m=/[^]*x/d.exec('\uD800\uDC00x'),n=/\uD800+x/.exec('\uD800\uD800x');m[0].length===3&&m.indices[0][1]===3&&n[0].length===3",
    );
}

#[test]
fn intrinsic_results_have_only_the_whole_match_and_bypass_setters() {
    assert_true(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(){calls++},configurable:true});let m=/a+b/d.exec('xaab'),p=Object.getOwnPropertyDescriptor(m,'0');calls===0&&m.length===1&&m[0]==='aab'&&m.index===1&&m.input==='xaab'&&m.groups===undefined&&m.indices.length===1&&m.indices[0][0]===1&&m.indices[0][1]===4&&p.writable&&p.enumerable&&p.configurable",
    );
}

#[test]
fn generic_consumers_and_callbacks_share_continuation_ordering() {
    assert_true(
        "'abxab'.match(/[ab]*ab/g).join(',')==='ab,ab'&&'abxab'.replace(/[ab]*ab/g,'X')==='XxX'&&'abxab'.split(/[ab]*ab/).join(',')===',x,'&&'xaab'.search('a+b')===1",
    );
    assert_true(
        "[...'abab'.matchAll(/[ab]*?ab/dg)].length===2&&[...'abab'.matchAll(/[ab]*ab/g)].length===1",
    );
    assert_true(
        "let seen;let s='xaab'.replace(/a+b/,(...args)=>{seen=args;return 'z'});s==='xz'&&seen.length===3&&seen[0]==='aab'&&seen[1]===1&&seen[2]==='xaab'",
    );
}

#[test]
fn captures_sets_multiple_quantifiers_assertions_and_choices_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:a+([b]+)){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:a+[b]+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:a+b+){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:(?:(?:(a+[b]+)){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:^a+b[c]+$){2}){2}){2})|){2,3}/.test('ab')",
        r"/(?:(?:((?:(?:a+[b]+|b){2}){2}){2})|){2,3}/.test('b')",
        "/a+b/u.test('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
