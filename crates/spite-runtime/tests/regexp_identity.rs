//! Ordinary IdentityEscape uses the pinned Unicode ID_Continue boundary.

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn escaped_punctuation_and_whitespace_match_as_original_literal_characters() {
    check(
        r"let r=/\!\#\%\&\,\:\;\<\=\>\@\`\~/dg,m=r.exec('x!#%&,:;<=>@`~');m[0]==='!#%&,:;<=>@`~' && m.index===1 && m.indices[0][1]===14 && r.lastIndex===14",
    );
    check(r"/(?:\!)(?:\ )/.exec('x! ')[0]==='! ' && /\$\-\/\\/.exec('x$-/\\')[0]==='$-/\\'");
    check(
        r"'a b'.split(/\ /).join(',')==='a,b' && 'a!b!'.replaceAll(/\!/g,'-')==='a-b-' && [...'!!'.matchAll(/\!/g)].map(m=>m.index).join(',')==='0,1'",
    );
}

#[test]
fn constructor_escapes_preserve_line_terminators_and_surrogate_code_units() {
    check(
        r"[9,10,13,160,8232,8233,9731,55296,56320].every(code=>{let s=String.fromCharCode(code),r=new RegExp('\\'+s,'d'),m=r.exec('x'+s);return m[0]===s && m.index===1 && m.indices[0][0]===1 && m.indices[0][1]===2;})",
    );
    check(
        r"let r=new RegExp('\\'+String.fromCharCode(56320),'d'),m=r.exec(String.fromCharCode(55296,56320));m.index===1 && m[0]===String.fromCharCode(56320) && m.indices[0][1]===2",
    );
}

#[test]
fn identifier_continue_and_unicode_identity_rules_still_reject_invalid_patterns() {
    check(
        r"['a','_',String.fromCharCode(0x200c),String.fromCharCode(0x200d),String.fromCharCode(0x301),String.fromCharCode(0x660)].every(s=>{try{new RegExp('\\'+s);return false;}catch(e){return e instanceof SyntaxError;}})",
    );
    check(
        r"['u','v'].every(flags=>{try{new RegExp('\\ ',flags);return false;}catch(e){return e instanceof SyntaxError;}})",
    );
    check(r"/\$/.test('$')");
}

#[test]
fn character_classes_assertions_and_backreferences_remain_distinct_from_identity_escapes() {
    for source in [
        r"/\d/.test('1')",
        r"/\w/.test('a')",
        r"/\s/.test(' ')",
        r"/\b/.test('a')",
        r"/\B/.test('a')",
        r"/(a)\1/.test('aa')",
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
