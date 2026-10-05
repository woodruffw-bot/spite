//! Ordered tagged calls and frozen template objects cached by site and realm.

mod common;
use common::REALM_ENTRIES;
use spite_core::JsString;
use spite_parser::parse_script;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn tag_getters_receivers_and_substitution_arguments_are_ordered_and_unconverted() {
    check(
        "let log='',original={},symbol=Symbol(),receiver,values,obj={get tag(){log+='g';return function(){'use strict';receiver=this;values=arguments;log+='c';return 7;};}};let result=obj.tag`head${(log+='1',original)}${(log+='2',symbol)}tail`;log==='g12c' && result===7 && receiver===obj && values.length===3 && values[1]===original && values[2]===symbol && values[0].join('|')==='head||tail'",
    );
    check(
        "let context,tag=function(){'use strict';context=this;return 7;};let result=tag`x`;result===7 && context===undefined",
    );
    check("let context,tag=function(){context=this;};tag`x`;context===globalThis");
    check(
        "let o={tag:function(){'use strict';return this;}},first=(o.tag)`x`,second=(0,o.tag)`x`;first===o && second===undefined",
    );
    check(
        "let log='',o={tag:()=>7},first=o.tag`x${(o.tag=()=>8,log+='v')}`,second=o.tag`x`;first===7 && second===8 && log==='v'",
    );
}

#[test]
fn callable_checks_follow_substitutions_and_abrupt_getters_or_values_stop_later_work() {
    check(
        "let log='',caught=false;try{(7)`x${(log+='1',1)}${(log+='2',2)}`;}catch(e){caught=e instanceof TypeError;}caught && log==='12'",
    );
    check(
        "let sentinel={},log='',caught=false;try{(7)`x${(log+='1',1)}${(()=>{throw sentinel;})()}${(log+='2',2)}`;}catch(e){caught=e===sentinel;}caught && log==='1'",
    );
    check(
        "let sentinel={},log='',o={get tag(){log+='g';throw sentinel;}},caught=false;try{o.tag`x${(log+='1',1)}`;}catch(e){caught=e===sentinel;}caught && log==='g'",
    );
    check(
        "let sentinel={},flag=0,tag=()=>{throw sentinel;},caught=false;try{tag`x`;}catch(e){caught=e===sentinel;}finally{flag=7;}caught && flag===7",
    );
}

#[test]
fn raw_and_cooked_strings_preserve_utf16_and_invalid_escapes_become_undefined() {
    check(
        r"let t=(x=>x)`a\n${7}\ud800\u{1f4a9}`;t[0]==='a\n' && t.raw[0]==='a\\n' && t[1].charCodeAt(0)===55296 && t[1].length===3 && t.raw[1]==='\\ud800\\u{1f4a9}'",
    );
    for escape in [
        r"\01",
        r"\1",
        r"\8",
        r"\9",
        r"\xg",
        r"\xAg",
        r"\u0",
        r"\u00g",
        r"\u{g",
        r"\u{0",
        r"\u{110000}",
    ] {
        check(&format!(
            "let t=(x=>x)`{escape}${{7}}tail`;t[0]===undefined && t.raw[0]===String.raw`{escape}` && t[1]==='tail' && t.raw[1]==='tail'"
        ));
    }
    assert_eq!(
        Realm::default().eval("String.raw`a\\n${7}b`"),
        Ok(Value::String(JsString::from("a\\n7b")))
    );
    check("let t=(x=>x)`a\r\nb\rc`;t[0]==='a\\nb\\nc' && t.raw[0]==='a\\nb\\nc'");
}

#[test]
fn template_and_raw_arrays_have_exact_frozen_descriptors_and_bypass_public_hooks() {
    check(
        "let p=Array.prototype,called=false;Object.defineProperty(p,'0',{set(){called=true;throw 7;},configurable:true});Object.defineProperty(p,'constructor',{get(){throw 8;}});globalThis.Array=function(){throw 9;};let t=(x=>x)`a${7}b`,d=Object.getOwnPropertyDescriptors(t),r=Object.getOwnPropertyDescriptors(t.raw);Object.getPrototypeOf(t)===p && Object.getPrototypeOf(t.raw)===p && t.length===2 && t.raw.length===2 && !called && Object.isFrozen(t) && Object.isFrozen(t.raw) && Object.getOwnPropertyNames(t).join(',')==='0,1,length,raw' && Object.keys(t).join(',')==='0,1' && d[0].value==='a' && !d[0].writable && d[0].enumerable && !d[0].configurable && !d.length.writable && !d.length.enumerable && !d.length.configurable && d.raw.value===t.raw && !d.raw.writable && !d.raw.enumerable && !d.raw.configurable && !r[0].writable && r[0].enumerable && !r[0].configurable && !r.length.writable && !r.length.enumerable && !r.length.configurable",
    );
    check(
        "let t=(x=>x)`a`;t[0]='b';t.raw[0]='c';t.extra=7;!Reflect.set(t,'length',0) && !Reflect.deleteProperty(t,'raw') && !Reflect.defineProperty(t,'x',{value:7}) && t[0]==='a' && t.raw[0]==='a' && t.extra===undefined",
    );
    check(
        "'use strict';let t=(x=>x)`a`,caught=0;try{t[0]='b';}catch(e){if(e instanceof TypeError)caught++;}try{t.raw[0]='c';}catch(e){if(e instanceof TypeError)caught++;}try{t.raw=[];}catch(e){if(e instanceof TypeError)caught++;}caught===3 && t[0]==='a' && t.raw[0]==='a'",
    );
}

#[test]
fn call_member_and_constructor_chains_preserve_precedence_and_return_values() {
    check("let seen='',tag=t=>{seen+=t[0];return tag;};tag`a``b``c`===tag && seen==='abc'");
    check(
        "let seen='',tag=t=>{seen+=t[0];return x=>({value:x});};tag`a`(7).value===7 && seen==='a'",
    );
    check(
        "let seen='',arg;function C(x){arg=x;}let tag=t=>{seen+=t[0];return C;};let a=new tag`a`,b=new tag`b`(7);a instanceof C && b instanceof C && seen==='ab' && arg===7",
    );
    check("function C(){return t=>t[0];}new C()`a`==='a'");
    check("let t=(x=>x)`a${(x=>x)`b${7}c`}d`;t[0]==='a' && t[1]==='d'");
}

#[test]
fn one_site_is_shared_across_calls_and_cloned_functions_but_identical_sites_are_distinct() {
    check(
        "let tag=x=>x;function run(tag){return tag`same${7}`;}let first=run(tag);run(x=>x)===first && first!==tag`same${7}` && first!==tag`same${7}`",
    );
    check("function factory(){return ()=>((x=>x)`same`);}factory()()===factory()()");
    check("let a=[];for(let n=0;n<3;n++)a.push((x=>x)`same`);a[0]===a[1] && a[1]===a[2]");
    let script = parse_script("(x=>x)`same`").unwrap();
    let mut realm = Realm::default();
    let first = realm.evaluate(&script).unwrap();
    assert_eq!(realm.evaluate(&script.clone()), Ok(first.clone()));
    assert_ne!(realm.eval("(x=>x)`same`").unwrap(), first);
    assert_ne!(Realm::default().evaluate(&script).unwrap(), first);
}

#[test]
fn cached_template_and_raw_remain_rooted_without_user_bindings_or_live_syntax() {
    let mut realm = Realm::default();
    let script = parse_script("(x=>x)`same`").unwrap();
    let Value::Object(template) = realm.evaluate(&script).unwrap() else {
        panic!("template")
    };
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 2);
    assert!(realm.inspect_object(&template).is_ok());
    assert_eq!(realm.evaluate(&script), Ok(Value::Object(template)));
    drop(script);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 2);
    let Value::Object(template) = realm.eval("(x=>x)`other`").unwrap() else {
        panic!("template")
    };
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 4);
    assert!(realm.inspect_object(&template).is_ok());
}

#[test]
fn large_default_templates_and_opt_in_host_failures_preserve_prior_effects() {
    let source = format!(
        "let tag=function(){{return arguments.length;}},result=tag`{}`;result===1001",
        "${7}".repeat(1000)
    );
    check(&source);
    for limits in [
        Limits {
            max_string_units: Some(3),
            ..Limits::default()
        },
        Limits {
            max_arguments: Some(1),
            ..Limits::default()
        },
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("let flag=0,tag=()=>{flag=3;};").unwrap();
        assert!(matches!(
            realm.eval("try{tag`\\u0061${7}`;}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{(()=>Function('class C{field;}'))`x${flag=7}`;}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(7.0)));
}
