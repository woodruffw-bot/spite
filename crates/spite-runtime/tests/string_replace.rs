//! String replacement: string searches, uncaptured substitution, and object hooks.

mod common;

use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn string_searches_replace_only_the_first_match_and_preserve_utf16_units() {
    check(
        "'ababa'.replace('ba','X')==='aXba' && 'abc'.replace('x','X')==='abc' && 'abc'.replace('','X')==='Xabc' && ''.replace('','X')==='X' && ''.replace('x','X')===''",
    );
    check(
        "'anullb'.replace(null,7)==='a7b' && 'undefinedx'.replace(undefined,true)==='truex' && 'a1b1'.replace(1n,undefined)==='aundefinedb1'",
    );
    check(
        "String.prototype.replace.call(1231,1,'x')==='x231' && String.prototype.replace.call(false,'a','X')==='fXlse' && String.prototype.replace.call(123n,2,'X')==='1X3'",
    );
    check(
        "'A\\uD834\\uDF06B'.replace('\\uDF06','\\uD800')==='A\\uD834\\uD800B' && 'A\\uD834\\uDF06B'.replace('\\uD834\\uDF06','X')==='AXB' && 'a\\uD800b'.replace('\\uD800','\\uDC00')==='a\\uDC00b'",
    );
}

#[test]
fn uncaptured_substitution_expands_supported_dollar_patterns_once() {
    check(
        r#"'abc'.replace('b', "$$-$&-$`-$'-$0-$1-$01-$99-$<x>-$") === "a$-b-a-c-$0-$1-$01-$99-$<x>-$c""#,
    );
    check(
        "'abc'.replace('b','$$$&')==='a$bc' && 'abc'.replace('b','$$$$')==='a$$c' && 'abc'.replace('b','$&$&')==='abbc'",
    );
    check(
        r#"'abc'.replace('', "$'")==='abcabc' && 'abc'.replace('', '$`')==='abc' && 'abc'.replace('abc', '$`$&'+"$'")==='abc'"#,
    );
    check(
        "'abc'.replace('b','$&$0$00$01$10$99$100$<name>')==='ab$0$00$01$10$99$100$<name>c' && 'abc'.replace('b','$x$-')==='a$x$-c'",
    );
}

#[test]
fn functional_replacements_receive_exact_arguments_and_return_literal_text() {
    check(
        "let calls=0;function f(m,p,s){'use strict';if(this!==undefined || arguments.length!==3 || m!=='b' || p!==1 || s!=='aba')throw 7;calls++;return 3;}let result='aba'.replace('b',f);result==='a3a' && calls===1",
    );
    check(
        "let calls=0;function f(){if(this!==globalThis)throw 7;calls++;return 'X';}let result='aba'.replace('a',f);result==='Xba' && calls===1",
    );
    check(r#"'aba'.replace('b',()=>"$$$&$`$'") === "a$$$&$`$'a""#);
    check(
        "function f(){throw 7;}Object.defineProperty(f,'toString',{get(){throw 8;}});'abc'.replace('x',f)==='abc'",
    );
    check(
        "let log='',receiver={toString(){log+='s';return 'aba';}},search={toString(){log+='r';return 'b';}};let result=String.prototype.replace.call(receiver,search,(m,p,s)=>{log+='f';receiver.toString=()=> 'changed';if(m!=='b' || p!==1 || s!=='aba')throw 7;return {toString(){log+='v';return 'X';}};});result==='aXa' && log==='srfv'",
    );
    check(
        "let caught=false;try{'aba'.replace('b',()=>Symbol());}catch(e){caught=e instanceof TypeError;}caught",
    );
    check("let caught=false;try{'aba'.replace('b',()=>{throw 7;});}catch(e){caught=e===7;}caught");
}

#[test]
fn object_hooks_run_before_conversions_and_preserve_original_values() {
    check(
        "let log='',receiver={toString(){throw 7;}},replace={toString(){throw 8;}},search={get [Symbol.replace](){log+='g';return function(o,r){'use strict';if(this!==search || o!==receiver || r!==replace || arguments.length!==2)throw 9;log+='c';return 3;};},toString(){throw 10;}};String.prototype.replace.call(receiver,search,replace)===3 && log==='gc'",
    );
    check(
        "let marker={};'abc'.replace({[Symbol.replace](){return marker;}},'X')===marker && 'abc'.replace({[Symbol.replace](){return undefined;}},'X')===undefined",
    );
    check(
        "let reads=0,caught=false,search={get [Symbol.replace](){reads++;throw 7;}};try{String.prototype.replace.call(null,search);}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
    check(
        "let caught=false,receiver={toString(){throw 8;}};try{String.prototype.replace.call(receiver,{get [Symbol.replace](){throw 7;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "let reads=0,caught=false,receiver={toString(){reads++;throw 8;}};try{String.prototype.replace.call(receiver,{[Symbol.replace]:7});}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
    check("'abc'.replace({[Symbol.replace]:null,toString(){return 'b';}},'X')==='aXc'");
    check(
        "let calls=0;'abc'.replace({get [Symbol.match](){calls++;throw 7;},toString(){return 'b';}},'X')==='aXc' && calls===0",
    );
}

#[test]
fn edition_seventeen_ignores_primitive_prototype_replace_hooks() {
    for (prototype, search, source, expected) in [
        ("String.prototype", "','", "a,b,c", "aXb,c"),
        ("Number.prototype", "1", "a1b1c", "aXb1c"),
        ("Boolean.prototype", "true", "atruebtruec", "aXbtruec"),
        ("BigInt.prototype", "1n", "a1b1c", "aXb1c"),
    ] {
        check(&format!(
            "Object.defineProperty({prototype},Symbol.replace,{{get(){{throw 7;}}}});'{source}'.replace({search},'X')==='{expected}'"
        ));
    }
    check(
        "Object.defineProperty(Symbol.prototype,Symbol.replace,{get(){throw 7;}});let caught=false;try{'abc'.replace(Symbol(),'X');}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "Number.prototype[Symbol.replace]=function(o,r){'use strict';return this.valueOf()+o.length+r;};'abc'.replace(Object(1),2)===6",
    );
}

#[test]
fn fallback_conversions_precede_search_and_callable_replacements_skip_stringification() {
    check(
        "let log='',receiver={toString(){log+='s';return 'abc';}},search={get [Symbol.replace](){log+='g';return undefined;},toString(){log+='r';return 'x';}},replace={toString(){log+='v';return 'X';}};String.prototype.replace.call(receiver,search,replace)==='abc' && log==='gsrv'",
    );
    check(
        "let log='',receiver={toString(){log+='s';throw 7;}},search={toString(){log+='r';throw 8;}},replace={toString(){log+='v';throw 9;}},caught=false;try{String.prototype.replace.call(receiver,search,replace);}catch(e){caught=e===7;}caught && log==='s'",
    );
    check(
        "let reads=0,caught=false;try{'abc'.replace({toString(){throw 7;}},{toString(){reads++;throw 8;}});}catch(e){caught=e===7;}caught && reads===0",
    );
    check(
        "let caught=false;try{'abc'.replace('x',{toString(){throw 7;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "function f(){return 'X';}Object.defineProperty(f,'toString',{get(){throw 7;}});'abc'.replace('b',f)==='aXc'",
    );
}

#[test]
fn metadata_deletion_intrinsic_retention_and_large_default_outputs() {
    check(
        "let d=Object.getOwnPropertyDescriptor(String.prototype,'replace'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='replace' && !n.writable && !n.enumerable && n.configurable && l.value===2 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check(
        "let caught=false;try{new String.prototype.replace();}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let s='a'.repeat(10000),r='x'.repeat(20000);s.replace('a',r).length===29999 && s.replace('b',r)===s",
    );
    let mut realm = Realm::default();
    realm.eval("let replace=String.prototype.replace;delete String.prototype.replace;delete globalThis.String").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("'a'.replace===undefined && replace.call('abc','b','X')==='aXc'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opted_in_output_and_search_limits_skip_pending_handlers() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(100),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,calls=0,source='a'.repeat(100),replacement='b'.repeat(100)")
        .unwrap();
    assert!(matches!(realm.eval("try{source.replace('a',()=>{calls++;return replacement;});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    assert_eq!(
        realm.eval("flag===0 && calls===1"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,calls=0,source='a'.repeat(2000),search='a'.repeat(100)+'b'")
        .unwrap();
    assert!(matches!(
        realm.eval(
            "try{source.replace(search,()=>{calls++;return 'x';});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && calls===0"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{'a'.replace('a',()=>{Proxy;});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
