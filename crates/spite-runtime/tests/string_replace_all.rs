//! String replaceAll: positions, substitutions, callbacks, and hook ordering.

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
fn nonoverlapping_searches_and_empty_searches_use_utf16_boundaries() {
    check(
        "'ababa'.replaceAll('ba','X')==='aXX' && 'aaa'.replaceAll('aa','X')==='Xa' && 'aaaa'.replaceAll('aa','X')==='XX' && 'abc'.replaceAll('x','X')==='abc'",
    );
    check(
        "'ab'.replaceAll('','X')==='XaXbX' && ''.replaceAll('','X')==='X' && ''.replaceAll('a','X')==='' && 'abc'.replaceAll('abc','')===''",
    );
    check(
        "'A\\uD834\\uDF06B'.replaceAll('', '_')==='_A_\\uD834_\\uDF06_B_' && '\\uD800a\\uD800'.replaceAll('\\uD800','\\uDC00')==='\\uDC00a\\uDC00'",
    );
    check(
        "'anullanull'.replaceAll(null,7)==='a7a7' && 'undefinedundefined'.replaceAll(undefined,true)==='truetrue' && String.prototype.replaceAll.call(121,1n,'x')==='x2x'",
    );
}

#[test]
fn substitutions_use_original_prefixes_and_suffixes_at_each_match() {
    check(
        r#"'aba'.replaceAll('a', '$`')==='bab' && 'aba'.replaceAll('a', "$'")==='bab' && 'ab'.replaceAll('', '$`')==='aabab' && 'ab'.replaceAll('', "$'")==='ababb'"#,
    );
    check(
        "'aba'.replaceAll('a','$$$&')==='$ab$a' && 'aba'.replaceAll('a','$$$$')==='$$b$$' && 'ab'.replaceAll('','$&')==='ab'",
    );
    check(
        "'aa'.replaceAll('a','$0$1$01$99$100$<name>$')==='$0$1$01$99$100$<name>$$0$1$01$99$100$<name>$'",
    );
    check(r#"'aba'.replaceAll('a',()=>"$$$&$`$'") === "$$$&$`$'b$$$&$`$'""#);
}

#[test]
fn callbacks_receive_three_arguments_and_fixed_strings_in_position_order() {
    check(
        "let calls=0;function f(m,p,s){'use strict';if(this!==undefined || arguments.length!==3 || m!=='a' || s!=='aba' || p!==calls*2)throw 7;calls++;return p;}let result='aba'.replaceAll('a',f);result==='0b2' && calls===2",
    );
    check(
        "let calls=0;function f(){if(this!==globalThis)throw 7;calls++;return 'X';}let result='aa'.replaceAll('a',f);result==='XX' && calls===2",
    );
    check(
        "let positions='';let result='\\uD834\\uDF06'.replaceAll('',(m,p,s)=>{if(m!=='' || s.length!==2)throw 7;positions+=p;return '-';});positions==='012' && result==='-\\uD834-\\uDF06-'",
    );
    check(
        "let log='',receiver={toString(){log+='s';return 'aba';}},search={toString(){log+='r';return 'a';}};let result=String.prototype.replaceAll.call(receiver,search,(m,p,s)=>{log+='f';receiver.toString=()=> 'changed';search.toString=()=> 'b';if(m!=='a' || s!=='aba')throw 7;return {toString(){log+='v';return p;}};});result==='0b2' && log==='srfvfv'",
    );
    check(
        "let calls=0,caught=false;try{'aaa'.replaceAll('a',(m,p)=>{calls++;if(p===1)throw 7;return 'x';});}catch(e){caught=e===7;}caught && calls===2",
    );
    check(
        "let calls=0,caught=false;try{'aaa'.replaceAll('a',()=>{calls++;return {toString(){throw 7;}};});}catch(e){caught=e===7;}caught && calls===1",
    );
    check(
        "let caught=false;try{'aa'.replaceAll('a',()=>Symbol());}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn regexp_check_and_flags_precede_hook_lookup_and_preserve_original_arguments() {
    check(
        "let log='',receiver={toString(){throw 7;}},replacement={toString(){throw 8;}},marker={},search={get [Symbol.match](){log+='m';return 1;},get flags(){log+='f';return {toString(){log+='s';return 'dog';}};},get [Symbol.replace](){log+='r';return function(o,v){'use strict';if(this!==search || o!==receiver || v!==replacement || arguments.length!==2)throw 9;log+='c';return marker;};},toString(){throw 10;}};String.prototype.replaceAll.call(receiver,search,replacement)===marker && log==='mfsrc'",
    );
    check(
        "let log='',search={get [Symbol.match](){log+='m';return false;},get flags(){throw 7;},get [Symbol.replace](){log+='r';return ()=> undefined;}};'aa'.replaceAll(search,'x')===undefined && log==='mr'",
    );
    check(
        "let reads=0,caught=false,search={get [Symbol.match](){reads++;throw 7;}};try{String.prototype.replaceAll.call(null,search);}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
    check(
        "let search={[Symbol.match]:true,flags:'g',[Symbol.replace]:null,toString(){return 'a';}};'aba'.replaceAll(search,'x')==='xbx'",
    );
}

#[test]
fn abrupt_match_flags_and_method_checks_precede_receiver_conversion() {
    for flags in ["undefined", "null", "''", "'G'", "'i'", "Symbol()"] {
        check(&format!(
            "let reads=0,caught=false,poison={{toString(){{reads++;throw 7;}}}},search={{[Symbol.match]:true,flags:{flags},get [Symbol.replace](){{reads++;throw 8;}}}};try{{String.prototype.replaceAll.call(poison,search,poison);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    for properties in [
        "get [Symbol.match](){throw 7;},get flags(){reads++;throw 8;}",
        "[Symbol.match]:true,get flags(){throw 7;}",
        "[Symbol.match]:true,flags:{toString(){throw 7;}}",
        "[Symbol.match]:false,get flags(){reads++;throw 8;},get [Symbol.replace](){throw 7;}",
    ] {
        check(&format!(
            "let reads=0,caught=false,poison={{toString(){{reads++;throw 8;}}}},search={{{properties}}};try{{String.prototype.replaceAll.call(poison,search,poison);}}catch(e){{caught=e===7;}}caught && reads===0"
        ));
    }
    check(
        "let reads=0,caught=false;try{String.prototype.replaceAll.call({toString(){reads++;throw 7;}},{[Symbol.replace]:3});}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
}

#[test]
fn primitive_searches_ignore_prototype_hooks_and_regexp_markers() {
    for (prototype, search, source, expected) in [
        ("String.prototype", "','", "a,b,c", "aXbXc"),
        ("Number.prototype", "1", "a1b1c", "aXbXc"),
        ("Boolean.prototype", "true", "atruebtruec", "aXbXc"),
        ("BigInt.prototype", "1n", "a1b1c", "aXbXc"),
    ] {
        check(&format!(
            "Object.defineProperty({prototype},Symbol.replace,{{get(){{throw 7;}}}});Object.defineProperty({prototype},Symbol.match,{{get(){{throw 8;}}}});'{source}'.replaceAll({search},'X')==='{expected}'"
        ));
    }
    check(
        "Object.defineProperty(Symbol.prototype,Symbol.replace,{get(){throw 7;}});let caught=false;try{'abc'.replaceAll(Symbol(),'X');}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "Number.prototype[Symbol.replace]=function(o,r){'use strict';return this.valueOf()+o.length+r;};'abc'.replaceAll(Object(1),2)===6",
    );
}

#[test]
fn fallback_converts_before_search_and_never_stringifies_callable_replacements() {
    check(
        "let log='',receiver={toString(){log+='s';return 'abc';}},search={get [Symbol.match](){log+='m';return false;},get [Symbol.replace](){log+='g';return null;},toString(){log+='r';return 'x';}},replace={toString(){log+='v';return 'X';}};String.prototype.replaceAll.call(receiver,search,replace)==='abc' && log==='mgsrv'",
    );
    check(
        "let reads=0,caught=false;try{String.prototype.replaceAll.call({toString(){throw 7;}},{toString(){reads++;throw 8;}},{toString(){reads++;throw 9;}});}catch(e){caught=e===7;}caught && reads===0",
    );
    check(
        "let reads=0,caught=false;try{'abc'.replaceAll({toString(){throw 7;}},{toString(){reads++;throw 8;}});}catch(e){caught=e===7;}caught && reads===0",
    );
    check(
        "let caught=false;try{'abc'.replaceAll('x',{toString(){throw 7;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "function f(){return 'X';}Object.defineProperty(f,'toString',{get(){throw 7;}});'aaa'.replaceAll('a',f)==='XXX' && 'aaa'.replaceAll('x',f)==='aaa'",
    );
}

#[test]
fn metadata_intrinsic_retention_and_large_outputs_have_no_default_quota() {
    check(
        "let d=Object.getOwnPropertyDescriptor(String.prototype,'replaceAll'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='replaceAll' && !n.writable && !n.enumerable && n.configurable && l.value===2 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check(
        "let caught=false;try{new String.prototype.replaceAll();}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let s='a'.repeat(10000);s.replaceAll('a','xx').length===20000 && s.replaceAll('','x').length===20001 && s.replaceAll('b','xx')===s",
    );
    let mut realm = Realm::default();
    realm.eval("let replaceAll=String.prototype.replaceAll;delete String.prototype.replaceAll;delete globalThis.String").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("'a'.replaceAll===undefined && replaceAll.call('aa','a','X')==='XX'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opted_in_limits_and_unsupported_callbacks_skip_pending_handlers() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(100),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,calls=0,source='aa',replacement='b'.repeat(100)")
        .unwrap();
    assert!(matches!(realm.eval("try{source.replaceAll('a',()=>{calls++;return replacement;});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    assert_eq!(
        realm.eval("flag===0 && calls===2"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,calls=0,source='a'.repeat(2000),search='a'.repeat(100)+'b'")
        .unwrap();
    assert!(matches!(realm.eval("try{source.replaceAll(search,()=>{calls++;return 'x';});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
    assert_eq!(
        realm.eval("flag===0 && calls===0"),
        Ok(Value::Boolean(true))
    );

    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{'aa'.replaceAll('a',()=>{Proxy;});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
