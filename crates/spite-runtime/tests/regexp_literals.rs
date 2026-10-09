//! Literal evaluation uses intrinsic RegExpCreate and fresh native slots.

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn every_evaluation_creates_a_fresh_branded_object_with_zero_lastindex() {
    check(
        "function f(){return /a/g;}let a=f(),b=f();a.lastIndex=7;a!==b && b.lastIndex===0 && a.source==='a' && a.flags==='g' && Object.prototype.toString.call(a)==='[object RegExp]' && Object.getPrototypeOf(a)===RegExp.prototype",
    );
    check(
        "let out=[];for(let i=0;i<3;i++)out.push(/a/g);out[0]!==out[1] && out[1]!==out[2] && out.every(r=>r.lastIndex===0)",
    );
    check(
        "let r=/a/;let d=Object.getOwnPropertyDescriptor(r,'lastIndex');d.value===0 && 1/d.value===Infinity && d.writable && !d.enumerable && !d.configurable",
    );
}

#[test]
fn public_constructor_and_symbol_hooks_cannot_intercept_literal_creation() {
    check(
        "let P=RegExp.prototype;Object.defineProperty(P,Symbol.match,{get(){throw 7;}});RegExp=function(){throw 8;};let r=/a/g;Object.getPrototypeOf(r)===P && r.source==='a' && r.flags==='g' && r.exec('ba').index===1",
    );
    check(
        "let P=RegExp.prototype;delete globalThis.RegExp;let r=/a/;Object.getPrototypeOf(r)===P && r.exec('ba')[0]==='a'",
    );
    check(
        "let P=RegExp.prototype;{let RegExp=7;let r=/a/;if(Object.getPrototypeOf(r)!==P || r.exec('a')[0]!=='a')throw 8;}true",
    );
}

#[test]
fn literals_preserve_source_flags_escape_spelling_and_utf16_units() {
    check(r"let r=/a\/b/ig;r.source==='a\\/b' && r.flags==='gi' && r.exec('xA/b')[0]==='A/b'");
    check(
        r"let r=/\uD800/d,m=r.exec('x\uD800');r.source==='\\uD800' && m.index===1 && m[0]==='\uD800' && m.indices[0][0]===1 && m.indices[0][1]===2",
    );
    check("let r=/💩/d,m=r.exec('x💩');m[0].length===2 && m.index===1 && m.indices[0][1]===3");
    check(
        "/(?:)/.source==='(?:)' && /(?<x>a)|(?<x>b)/du.flags==='du' && /[\\q{ab|c}]/v.unicodeSets && /\\p{ASCII}/u.unicode",
    );
}

#[test]
fn arrow_destructuring_and_template_literals_execute_with_correct_goals() {
    check("let f=(r=/a/g)=>r;let a=f(),b=f();a!==b && a.exec('ba').index===1 && b.lastIndex===0");
    check("let x;({x=/a/}={});let y;[y=/b/]=[];x.exec('a')[0]==='a' && y.exec('b')[0]==='b'");
    check("`${/a/.exec('ba')[0]}:${12/4}`==='a:3'");
    check("let r=/=x/;r.exec('a=x')[0]==='=x' && 12/4===3");
}

#[test]
fn all_generic_consumers_share_literal_matching_and_lastindex_state() {
    check(
        "'aba'.match(/a/g).join(',')==='a,a' && 'aba'.search(/b/)===1 && 'aba'.replace(/a/g,'x')==='xbx' && 'a,b,'.split(/,/).join('|')==='a|b|' && [...'aba'.matchAll(/a/g)].length===2",
    );
    check(
        "let r=/a/gy;r.lastIndex=1;let a=r.exec('baa'),b=r.exec('baa'),c=r.exec('baa');a.index===1 && b.index===2 && c===null && r.lastIndex===0",
    );
}

#[test]
fn valid_unimplemented_matchers_fail_only_when_execution_reaches_matching() {
    for source in [
        r"/(?:(?:(?:(?:a(a|bc)){2}){2})|){2,3}/.exec('a')",
        r"/(?:(?:((?:(?:.a*b*){2}){2}){2})|){2,3}/.test('a')",
        "/\\u{61}/u.exec('a')",
        "/\\u{61}/v.exec('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(
        "let r=/(a)/;r.exec=function(s){return {0:s,index:2,length:1};};r.test('x') && r[Symbol.search]('x')===2",
    );
}

#[test]
fn literal_instances_and_original_intrinsics_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=/a/dg;RegExp=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=r.exec('ba');m.index===1 && m.indices[0][1]===2 && r.lastIndex===2 && /b/.test('b')"),Ok(Value::Boolean(true)));
}

#[test]
fn recursive_literal_input_coercions_use_existing_native_stack_guards() {
    for source in [
        "function f(){return /a/.exec({toString(){return f();}});}f()",
        "function f(){return /a/.test({toString(){return f();}});}f()",
    ] {
        assert!(
            matches!(Realm::default().eval(source), Err(Error::Limit { .. })),
            "{source}"
        );
    }
}
