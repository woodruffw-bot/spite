//! TrimString's exact whitespace set and generic receiver semantics.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn all_ecmascript_whitespace_and_line_terminators_are_trimmed() {
    for unit in [
        0x0009, 0x000b, 0x000c, 0x0020, 0x00a0, 0xfeff, 0x1680, 0x2000, 0x2001, 0x2002, 0x2003,
        0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200a, 0x202f, 0x205f, 0x3000, 0x000a,
        0x000d, 0x2028, 0x2029,
    ] {
        check(&format!(
            "let w='\\u{unit:04x}',s=w+'a'+w+'b'+w;s.trim()==='a'+w+'b' && s.trimStart()==='a'+w+'b'+w && s.trimEnd()===w+'a'+w+'b' && w.trim()==='' && w.trimStart()==='' && w.trimEnd()===''"
        ));
    }
    check(r"let s=' \t\r\n\u00a0\uFEFF';s.trim()==='' && s.trimStart()==='' && s.trimEnd()===''");
    check("''.trim()==='' && ''.trimStart()==='' && ''.trimEnd()==='' && 'plain'.trim()==='plain'");
}

#[test]
fn non_whitespace_and_surrogates_stop_trimming() {
    for unit in [
        0x0000, 0x0008, 0x000e, 0x0085, 0x180e, 0x200b, 0x2060, 0xffff, 0xd800, 0xdc00,
    ] {
        check(&format!(
            "let c='\\u{unit:04x}';c.trim()===c && (' '+c+' ').trim()===c && (c+' x '+c).trim()===c+' x '+c"
        ));
    }
    check("' 💩 '.trim()==='💩' && ' 💩 '.trimStart()==='💩 ' && ' 💩 '.trimEnd()===' 💩'");
    check("' \\uD800 x \\uDC00 '.trim()==='\\uD800 x \\uDC00'");
}

#[test]
fn generic_conversion_runs_once_and_ignores_extra_arguments() {
    for (name, expected) in [("trim", "a"), ("trimStart", "a "), ("trimEnd", " a")] {
        check(&format!(
            "let log='';let r={{toString:()=>{{log+='s';return ' a ';}},valueOf:()=>{{throw 1;}}}},ignored={{toString:()=>{{throw 2;}}}};String.prototype.{name}.call(r,(log+='a',ignored))==='{expected}' && log==='as'"
        ));
        check(&format!(
            "new String(' a ').{name}()==='{expected}' && String.prototype.{name}.call(123n)==='123' && String.prototype.{name}.call(true)==='true'"
        ));
        check(&format!(
            "String.prototype.{name}.call({{toString:()=>({{}}),valueOf:()=>123}})==='123'"
        ));
        assert_eq!(
            Realm::default().eval(&format!(
                "String.prototype.{name}.call({{toString:()=>{{throw 7;}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        for receiver in ["null", "undefined", "{toString:()=>({}),valueOf:()=>({})}"] {
            assert!(matches!(
                Realm::default().eval(&format!("String.prototype.{name}.call({receiver})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
}

#[test]
fn mandatory_methods_have_standard_metadata_and_remain_rooted() {
    for name in ["trim", "trimStart", "trimEnd"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===0 && !Object.hasOwn(d.value,'prototype')"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new String.prototype.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    check("String.prototype.trimLeft===undefined && String.prototype.trimRight===undefined");
    let mut realm = Realm::default();
    realm.eval("delete globalThis.String").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("' x '.trim()==='x' && ' x '.trimStart()==='x ' && ' x '.trimEnd()===' x'"),
        Ok(Value::Boolean(true))
    );
}
