//! Ordered object hooks, native RegExp creation and literal matching.

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
fn primitive_hooks_are_ignored_and_fallbacks_create_native_patterns() {
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
            realm.eval(&format!("let reads=0;Object.defineProperty({prototype},Symbol.{symbol},{{get(){{reads++;throw 7;}}}});RegExp.prototype[Symbol.{symbol}]=function(s){{return s;}};")).unwrap();
            let source = if prototype == "Symbol.prototype" {
                format!(
                    "let caught=false;try{{'x'.{method}({pattern});}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
                )
            } else {
                format!("'x'.{method}({pattern})==='x' && reads===0")
            };
            assert_eq!(
                realm.eval(&source),
                Ok(Value::Boolean(true)),
                "{method}/{pattern}"
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
            realm
                .eval(&format!(
                    "let log='';RegExp.prototype[Symbol.{symbol}]=function(s){{return s;}};"
                ))
                .unwrap();
            assert_eq!(realm.eval(&format!("String.prototype.{method}.call({{toString(){{log+='s';return 'x';}}}},{pattern})==='x' && log==='s'")),Ok(Value::Boolean(true)));
        }
        check(&format!(
            "let caught=false;try{{String.prototype.{method}.call({{toString(){{throw 7;}}}},null);}}catch(e){{caught=e===7;}}caught"
        ));
    }
}

#[test]
fn fallback_conversion_creates_fresh_native_objects_and_invokes_live_symbol_methods() {
    for (method, flags) in [("match", ""), ("matchAll", "g"), ("search", "")] {
        check(&format!(
            "let t='',result={{}},seen,receiver={{[Symbol.toPrimitive](h){{t+='r'+h;return 'input';}}}},pattern={{get constructor(){{throw 7;}},get source(){{throw 8;}},get flags(){{throw 9;}},toString(){{t+='p';return 'a';}}}};Object.defineProperty(pattern,Symbol.{method},{{get(){{t+='k';return undefined;}}}});Object.defineProperty(RegExp.prototype,Symbol.{method},{{get(){{t+='h';seen=this;return function(s){{'use strict';t+='c';if(this!==seen || s!=='input' || arguments.length!==1)throw 10;return result;}};}}}});let a=String.prototype.{method}.call(receiver,pattern);a===result && t==='krstringphc' && seen!==pattern && seen instanceof RegExp && seen.source==='a' && seen.flags==='{flags}' && seen.lastIndex===0"
        ));
        check(&format!(
            "let first;RegExp.prototype[Symbol.{method}]=function(s){{if(first===undefined){{first=this;return this.source;}}return this!==first && this.source==='(?:)' && this.flags==='{flags}';}};'x'.{method}(undefined)==='(?:)' && 'x'.{method}(undefined)"
        ));
    }
}

#[test]
fn direct_creation_bypasses_regexp_identity_native_copy_and_repeated_match_lookups() {
    check(
        "let t='',pattern={get [Symbol.match](){t+='m';return undefined;},get constructor(){throw 7;},get source(){throw 8;},get flags(){throw 9;},toString(){t+='p';return 'a';}};RegExp.prototype[Symbol.match]=function(s){return this.source;};'x'.match(pattern)==='a' && t==='mp'",
    );
    check(
        "let t='',pattern={get [Symbol.match](){t+='m';return false;},get [Symbol.matchAll](){t+='a';return undefined;},get source(){throw 7;},get flags(){throw 8;},get constructor(){throw 9;},toString(){t+='p';return 'a';}};RegExp.prototype[Symbol.matchAll]=function(s){return this.source;};'x'.matchAll(pattern)==='a' && t==='map'",
    );
    check(
        r"let r=new RegExp('a','g');r[Symbol.match]=null;r.toString=function(){return 'custom';};RegExp.prototype[Symbol.match]=function(s){return this.source;};'x'.match(r)==='custom'",
    );
    check(
        "let r=new RegExp('a','g');r[Symbol.search]=undefined;RegExp.prototype[Symbol.search]=function(s){return this.source;};let expected=String.fromCharCode(92)+'/a'+String.fromCharCode(92)+'/g';'x'.search(r)===expected",
    );
}

#[test]
fn fallback_errors_follow_receiver_pattern_and_live_method_order() {
    for method in ["match", "matchAll", "search"] {
        check(&format!(
            "let reads=0,caught=false;Object.defineProperty(RegExp.prototype,Symbol.{method},{{get(){{reads++;throw 8;}}}});try{{'x'.{method}('(');}}catch(e){{caught=e instanceof SyntaxError;}}caught && reads===0"
        ));
        check(&format!(
            "let t='',pattern={{toString(){{t+='p';throw 7;}}}};Object.defineProperty(RegExp.prototype,Symbol.{method},{{get(){{t+='h';throw 8;}}}});try{{String.prototype.{method}.call({{toString(){{t+='r';return 'x';}}}},pattern);}}catch(e){{t+=e;}}t==='rp7'"
        ));
        for value in ["undefined", "null", "1", "{}"] {
            check(&format!(
                "RegExp.prototype[Symbol.{method}]={value};let caught=false;try{{'x'.{method}('a');}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
        check(&format!(
            "let marker={{}},caught=false;Object.defineProperty(RegExp.prototype,Symbol.{method},{{get(){{throw marker;}}}});try{{'x'.{method}('a');}}catch(e){{caught=e===marker;}}caught"
        ));
    }
}

#[test]
fn matchall_creation_is_lazy_and_custom_exec_observes_the_new_native_matcher() {
    check(
        "let n=0,seen,result={0:'x',index:0,length:1};RegExp.prototype.exec=function(s){n++;seen=this;if(this.source!=='a' || this.flags!=='g' || s!=='input')throw 7;return n===1?result:null;};let it='input'.matchAll('a');let lazy=n===0,a=it.next(),b=it.next(),c=it.next();lazy && a.value===result && !a.done && b.done && c.done && n===2 && seen instanceof RegExp",
    );
    check(
        "let n=0;RegExp.prototype.exec=function(s){n++;return null;};let iterator='x'.matchAll();n===0 && Object.getPrototypeOf(iterator)[Symbol.toStringTag]==='RegExp String Iterator' && iterator.next().done && n===1",
    );
    let mut realm = Realm::default();
    realm.eval("let it='x'.matchAll('a(a|b)'),flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{it.next();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("it.next()"),
        Err(Error::Unsupported { .. })
    ));
}

#[test]
fn fallback_intrinsics_and_large_inputs_survive_global_replacement_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let calls=0,input='a'.repeat(120000);RegExp.prototype.exec=function(s){calls++;return null;};let it=input.matchAll('a');RegExp=function(){throw 7;};String=function(){throw 8;};input=null;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("calls===0 && it.next().done && calls===1"),
        Ok(Value::Boolean(true))
    );
    for method in ["match", "matchAll", "search"] {
        check(&format!(
            "let f=String.prototype.{method};RegExp.prototype[Symbol.{method}]=function(s){{return this.source==='a' && s==='x';}};RegExp=function(){{throw 7;}};f.call('x','a')"
        ));
    }
}

#[test]
fn recursive_creation_conversions_and_fallback_methods_use_native_stack_guards() {
    for source in [
        "let p={toString(){return 'x'.match(p);}};'x'.match(p)",
        "let p={toString(){return 'x'.matchAll(p);}};'x'.matchAll(p)",
        "RegExp.prototype[Symbol.search]=function(s){return s.search('a');};'x'.search('a')",
    ] {
        assert!(
            matches!(Realm::default().eval(source), Err(Error::Limit { .. })),
            "{source}"
        );
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
        assert!(matches!(realm.eval(&format!("try{{'x'.{method}({{[Symbol.{method}](){{Function('function* gap(){{}}');}}}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Unsupported {..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
