use super::*;

#[test]
fn string_search_predicates_observe_receiver_match_search_and_position_in_order() {
    for method in ["includes", "startsWith", "endsWith"] {
        let mut realm = realm_with_symbols();
        realm.eval("let log='',receiver={toString:()=>{log+='r';return 'x';}},search={toString:()=>{log+='s';return 'x';}},position={valueOf:()=>{log+='p';return 0;}};Object.defineProperty(search,matcher,{get:()=>{log+='m';return false;}})").unwrap();
        realm
            .eval(&format!(
                "String.prototype.{method}.call(receiver,search,position)"
            ))
            .unwrap();
        check(&mut realm, "log==='rmsp'");
        realm.eval("log=''").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "String.prototype.{method}.call(null,search,position)"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        check(&mut realm, "log===''");
        assert_eq!(
            realm.eval(&format!(
                "String.prototype.{method}.call({{toString:()=>{{throw 7;}}}},search,position)"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        check(&mut realm, "log===''");
    }
}

#[test]
fn truthy_match_markers_reject_without_calling_or_converting_them() {
    for method in ["includes", "startsWith", "endsWith"] {
        for marker in [
            "true",
            "1",
            "'x'",
            "s",
            "1n",
            "{}",
            "()=>{throw 8;}",
            "{[convert]:()=>{throw 9;}}",
        ] {
            let mut realm = realm_with_symbols();
            realm.eval(&format!("let called=false,search={{[matcher]:{marker},toString:()=>{{called=true;return '';}}}},position={{valueOf:()=>{{called=true;return 0;}}}};")).unwrap();
            assert!(
                matches!(
                    realm.eval(&format!("''.{method}(search,position)")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}, {marker}"
            );
            check(&mut realm, "!called");
        }
    }
}

#[test]
fn falsey_match_markers_allow_search_and_plain_objects_have_no_regexp_brand() {
    for method in ["includes", "startsWith", "endsWith"] {
        for marker in ["undefined", "null", "false", "0", "NaN", "''", "0n"] {
            let mut realm = realm_with_symbols();
            check(
                &mut realm,
                &format!("'x'.{method}({{[matcher]:{marker},toString:()=> 'x'}})"),
            );
        }
        check(
            &mut realm_with_symbols(),
            &format!("'x'.{method}({{toString:()=> 'x'}})"),
        );
    }
}

#[test]
fn inherited_match_getters_keep_receiver_and_abrupt_completions() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let seen,p={},search={__proto__:p,toString:()=> 'x'};Object.defineProperty(p,matcher,{get:function(){seen=this;return null;}});'x'.includes(search) && seen===search",
    );
    assert_eq!(realm.eval("let abrupt={};Object.defineProperty(abrupt,matcher,{get:()=>{throw s;}});'x'.includes(abrupt)"),Err(Error::Thrown(realm.eval("s").unwrap())));
    // IsRegExp does not box a primitive to look for a match property.
    check(
        &mut realm,
        "Object.defineProperty(String.prototype,matcher,{get:()=>{throw 8;}});'x'.includes('x') && 'x'.startsWith('x') && 'x'.endsWith('x')",
    );
    check(
        &mut realm,
        "let matchCalls=0,searchAgain={toString:()=> 'x'};Object.defineProperty(searchAgain,matcher,{get:()=>{matchCalls++;return false;}});'x'.indexOf(searchAgain)===0 && 'x'.lastIndexOf(searchAgain)===0 && matchCalls===0",
    );
}

#[test]
fn recursive_match_getters_use_the_host_recursion_limit() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        for method in ["includes","startsWith","endsWith"] {
            let mut realm=realm_with_symbols();
            realm.eval(&format!("let search={{}},flag=0;Object.defineProperty(search,matcher,{{get:()=> ''.{method}(search)}})")).unwrap();
            assert!(matches!(realm.eval(&format!("try{{''.{method}(search);}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
            check(&mut realm,"flag===0 && 'x'.includes({[matcher]:false,toString:()=> 'x'})");
        }
    }).unwrap().join().unwrap();
}
