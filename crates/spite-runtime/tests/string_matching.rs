//! Ordered object hooks for matching/search; native RegExp fallback remains open.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn hooks_preserve_original_receivers_return_values_and_exact_call_arguments() {
    for (method, symbol) in [
        ("match", "match"),
        ("matchAll", "matchAll"),
        ("search", "search"),
    ] {
        check(&format!(
            "let log='',receiver={{toString(){{throw 7;}}}},result={{}},pattern={{get [Symbol.{symbol}](){{log+='g';return function(o){{'use strict';if(this!==pattern || o!==receiver || arguments.length!==1)throw 8;log+='c';return result;}};}},toString(){{throw 9;}}}};String.prototype.{method}.call(receiver,pattern)===result && log==='gc'"
        ));
        check(&format!(
            "let proto={{[Symbol.{symbol}](o){{return o;}}}},pattern=Object.create(proto);String.prototype.{method}.call(7,pattern)===7"
        ));
        for result in ["null", "undefined", "false", "7", "7n", "Symbol.iterator"] {
            check(&format!(
                "let result={result},pattern={{[Symbol.{symbol}](){{return result;}}}};'x'.{method}(pattern)===result"
            ));
        }
    }
}

#[test]
fn receiver_check_and_getmethod_failures_precede_conversion() {
    for (method, symbol) in [
        ("match", "match"),
        ("matchAll", "matchAll"),
        ("search", "search"),
    ] {
        check(&format!(
            "let reads=0,caught=false,pattern={{get [Symbol.{symbol}](){{reads++;throw 7;}}}};try{{String.prototype.{method}.call(null,pattern);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
        check(&format!(
            "let reads=0,caught=false,receiver={{toString(){{reads++;throw 8;}}}},pattern={{get [Symbol.{symbol}](){{throw 7;}}}};try{{String.prototype.{method}.call(receiver,pattern);}}catch(e){{caught=e===7;}}caught && reads===0"
        ));
        check(&format!(
            "let reads=0,caught=false,receiver={{toString(){{reads++;throw 8;}}}},pattern={{[Symbol.{symbol}]:3}};try{{String.prototype.{method}.call(receiver,pattern);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
        check(&format!(
            "let marker={{}},caught=false;try{{'x'.{method}({{[Symbol.{symbol}](){{throw marker;}}}});}}catch(e){{caught=e===marker;}}caught"
        ));
    }
}

#[test]
fn matchall_regexp_marker_and_global_flags_are_checked_before_the_hook() {
    check(
        "let log='',receiver={toString(){throw 7;}},pattern={get [Symbol.match](){log+='m';return true;},get flags(){log+='f';return {toString(){log+='s';return 'dog';}};},get [Symbol.matchAll](){log+='h';return function(o){if(o!==receiver)throw 8;log+='c';return 9;};}};String.prototype.matchAll.call(receiver,pattern)===9 && log==='mfshc'",
    );
    check(
        "let log='',pattern={get [Symbol.match](){log+='m';return false;},get flags(){throw 7;},get [Symbol.matchAll](){log+='h';return ()=>8;}};'x'.matchAll(pattern)===8 && log==='mh'",
    );
    for flags in ["undefined", "null", "''", "'G'", "'i'", "Symbol()"] {
        check(&format!(
            "let reads=0,caught=false,receiver={{toString(){{reads++;throw 7;}}}},pattern={{[Symbol.match]:true,flags:{flags},get [Symbol.matchAll](){{reads++;throw 8;}}}};try{{String.prototype.matchAll.call(receiver,pattern);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    for properties in [
        "get [Symbol.match](){throw 7;}",
        "[Symbol.match]:true,get flags(){throw 7;}",
        "[Symbol.match]:true,flags:{toString(){throw 7;}}",
    ] {
        check(&format!(
            "let reads=0,caught=false,pattern={{{properties},get [Symbol.matchAll](){{reads++;throw 8;}}}};try{{'x'.matchAll(pattern);}}catch(e){{caught=e===7;}}caught && reads===0"
        ));
    }
    // Match and search never inspect IsRegExp or the flags property.
    check(
        "let pattern={get flags(){throw 7;},[Symbol.match](){return 8;},[Symbol.search](){return 9;}};'x'.match(pattern)===8 && 'x'.search(pattern)===9",
    );
}

#[test]
fn primitive_hooks_are_ignored_and_native_fallbacks_stay_unsupported() {
    for (method, symbol) in [
        ("match", "match"),
        ("matchAll", "matchAll"),
        ("search", "search"),
    ] {
        for (prototype, pattern) in [
            ("String.prototype", "'x'"),
            ("Number.prototype", "3"),
            ("Boolean.prototype", "true"),
            ("BigInt.prototype", "3n"),
            ("Symbol.prototype", "Symbol()"),
        ] {
            let mut realm = Realm::default();
            realm.eval(&format!("let reads=0,flag=0;Object.defineProperty({prototype},Symbol.{symbol},{{get(){{reads++;throw 7;}}}});")).unwrap();
            assert!(
                matches!(
                    realm.eval(&format!(
                        "try{{'x'.{method}({pattern});}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Unsupported { .. })
                ),
                "{method}/{pattern}"
            );
            assert_eq!(
                realm.eval("reads===0 && flag===0"),
                Ok(Value::Boolean(true))
            );
        }
        for pattern in [
            "undefined",
            "null",
            "{}",
            &format!("{{[Symbol.{symbol}]:null}}"),
            &format!("{{[Symbol.{symbol}]:undefined}}"),
        ] {
            let mut realm = Realm::default();
            realm.eval("let log='',flag=0;").unwrap();
            assert!(matches!(realm.eval(&format!("try{{String.prototype.{method}.call({{toString(){{log+='s';return 'x';}}}},{pattern});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Unsupported {..})));
            assert_eq!(
                realm.eval("log==='s' && flag===0"),
                Ok(Value::Boolean(true))
            );
        }
        check(&format!(
            "let caught=false;try{{String.prototype.{method}.call({{toString(){{throw 7;}}}},null);}}catch(e){{caught=e===7;}}caught"
        ));
    }
}

#[test]
fn full_string_property_inventory_supports_reflection_copying_and_integrity() {
    // All edition-17 prototype names, excluding optional Annex B additions.
    check(
        "Object.getOwnPropertyNames(String.prototype).sort().join(',')==='at,charAt,charCodeAt,codePointAt,concat,constructor,endsWith,includes,indexOf,isWellFormed,lastIndexOf,length,localeCompare,match,matchAll,normalize,padEnd,padStart,repeat,replace,replaceAll,search,slice,split,startsWith,substring,toLocaleLowerCase,toLocaleUpperCase,toLowerCase,toString,toUpperCase,toWellFormed,trim,trimEnd,trimStart,valueOf'",
    );
    check(
        "let keys=Reflect.ownKeys(String.prototype),symbols=Object.getOwnPropertySymbols(String.prototype);keys.length===37 && keys[0]==='length' && keys[1]==='constructor' && keys[2]==='toString' && keys[3]==='valueOf' && keys[36]===Symbol.iterator && symbols.length===1 && symbols[0]===Symbol.iterator && Object.keys(String.prototype).length===0",
    );
    check(
        "let descriptors=Object.getOwnPropertyDescriptors(String.prototype);Reflect.ownKeys(descriptors).length===37 && descriptors.normalize.value===String.prototype.normalize && descriptors.matchAll.value===String.prototype.matchAll && !descriptors.length.writable && !descriptors.length.enumerable && !descriptors.length.configurable",
    );
    check(
        "let receiver={};Object.assign(receiver,String.prototype)===receiver && Object.keys(receiver).length===0 && Object.defineProperties(receiver,String.prototype)===receiver && Object.keys(receiver).length===0",
    );
    check(
        "let seen='';String.prototype.added=7;for(let key in 'x'){seen+=key+',';}seen==='0,added,'",
    );
    check(
        "let p=String.prototype;Object.freeze(p)===p && Object.isFrozen(p) && Object.isSealed(p) && !Object.isExtensible(p) && 'é'.normalize()==='é' && 'x'.match({[Symbol.match](){return 7;}})===7",
    );
    check(
        "let p=String.prototype;delete p.matchAll;Object.getOwnPropertyDescriptor(p,'matchAll')===undefined && !Object.hasOwn(p,'matchAll') && 'x'.matchAll===undefined && !Reflect.ownKeys(p).includes('matchAll')",
    );
}

#[test]
fn method_metadata_and_retention_are_standard() {
    for method in ["match", "matchAll", "search"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(String.prototype,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new String.prototype.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let match=String.prototype.match,all=String.prototype.matchAll,search=String.prototype.search;delete String.prototype.match;delete String.prototype.matchAll;delete String.prototype.search;delete globalThis.String;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let p={[Symbol.match](o){return o;},[Symbol.matchAll](o){return o;},[Symbol.search](o){return o;},flags:'g'};match.call(7,p)===7 && all.call(8,p)===8 && search.call(9,p)===9"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_and_unsupported_hook_failures_bypass_handlers() {
    for method in ["match", "matchAll", "search"] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        realm.eval("let flag=0;").unwrap();
        assert!(matches!(realm.eval(&format!("try{{'x'.{method}({{[Symbol.{method}](){{while(true){{}}}}}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit {..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        assert!(matches!(realm.eval(&format!("try{{'x'.{method}({{[Symbol.{method}](){{Function();}}}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Unsupported {..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
