//! String search predicates for the currently exposed value kinds.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prefix_suffix_and_substring_search_have_distinct_position_rules() {
    check("'abcabc'.includes('bc') && 'abcabc'.includes('bc',3) && !'abcabc'.includes('bc',5)");
    check(
        "'abcabc'.startsWith('ab') && 'abcabc'.startsWith('ab',3) && !'abcabc'.startsWith('ab',1)",
    );
    check("'abcabc'.endsWith('bc') && 'abcabc'.endsWith('bc',3) && !'abcabc'.endsWith('bc',4)");
    check(
        "'abc'.includes('a',-Infinity) && 'abc'.startsWith('a',-1) && !'abc'.endsWith('a',-1) && 'abc'.endsWith('a',1.9)",
    );
    check(
        "'abc'.includes('a',NaN) && 'abc'.startsWith('a',NaN) && !'abc'.endsWith('a',NaN) && 'abc'.endsWith('bc',undefined) && 'abc'.endsWith('bc',Infinity)",
    );
    check(
        "'abc'.startsWith('b','1.9') && 'abc'.endsWith('ab','2.9') && !'abc'.includes('a',Infinity) && !'abc'.startsWith('a',Infinity)",
    );
    check(
        "'undefined'.includes() && 'undefined'.startsWith() && 'undefined'.endsWith() && 'null'.includes(null)",
    );
}

#[test]
fn empty_needles_match_and_searches_preserve_utf16_units() {
    for name in ["includes", "startsWith", "endsWith"] {
        for position in [
            "undefined",
            "NaN",
            "null",
            "-Infinity",
            "Infinity",
            "-10",
            "100",
        ] {
            check(&format!(
                "''.{name}('',{position}) && 'abc'.{name}('',{position}) && !''.{name}('a',{position}) && !'a'.{name}('ab',{position})"
            ));
        }
    }
    check(
        "'💩'.includes('\\uDCA9') && '💩'.startsWith('\\uD83D') && '💩'.endsWith('\\uDCA9') && '💩'.startsWith('\\uDCA9',1) && '💩'.endsWith('\\uD83D',1)",
    );
    check(
        "'\\uD800x'.startsWith('\\uD800') && 'x\\uDC00'.endsWith('\\uDC00') && !'é'.includes('é')",
    );
}

#[test]
fn predicates_order_receiver_search_and_position_conversions() {
    for (name, position) in [("includes", 1), ("startsWith", 1), ("endsWith", 2)] {
        check(&format!(
            "let log='';let r={{toString:()=>{{log+='r';return 'abc';}}}},s={{toString:()=>{{log+='s';return 'b';}},valueOf:()=>{{throw 1;}}}},p={{valueOf:()=>{{log+='p';return {position};}}}};String.prototype.{name}.call(r,s,p) && log==='rsp'"
        ));
        check(&format!(
            "String.prototype.{name}.call(123n,2,{position}) && new String('abc').{name}('b',{position})"
        ));
        assert_eq!(Realm::default().eval(&format!("String.prototype.{name}.call({{toString:()=>{{throw 6;}}}},{{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}})")),Err(Error::Thrown(Value::Number(6.0))));
        assert_eq!(
            Realm::default().eval(&format!(
                "''.{name}({{toString:()=>{{throw 7;}}}},{{valueOf:()=>{{throw 8;}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        // Position conversion is still required for an empty search string.
        assert_eq!(
            Realm::default().eval(&format!("''.{name}('',{{valueOf:()=>{{throw 8;}}}})")),
            Err(Error::Thrown(Value::Number(8.0)))
        );
        assert!(matches!(
            Realm::default().eval(&format!("''.{name}('',0n)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn string_named_properties_do_not_substitute_for_symbol_match() {
    for name in ["includes", "startsWith", "endsWith"] {
        check(&format!(
            "let s={{toString:()=> 'x'}};Object.defineProperty(s,'@@match',{{get:()=>{{throw 1;}}}});Object.defineProperty(s,'Symbol.match',{{get:()=>{{throw 2;}}}});'x'.{name}(s)"
        ));
    }
}

#[test]
fn generic_methods_reject_nullish_receivers_and_have_standard_metadata() {
    for name in ["includes", "startsWith", "endsWith"] {
        for receiver in ["null", "undefined"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "String.prototype.{name}.call({receiver},{{toString:()=>{{throw 1;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===1 && !Object.hasOwn(d.value,'prototype')"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new String.prototype.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("delete globalThis.String").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("'abc'.includes('b') && 'abc'.startsWith('a') && 'abc'.endsWith('c')"),
        Ok(Value::Boolean(true))
    );
}
