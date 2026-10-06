//! Generic RegExp replacement and captured GetSubstitution (22.2.6.11).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn receiver_and_argument_checks_preserve_conversion_order_even_without_matches() {
    check(
        r"
        let f=RegExp.prototype[Symbol.replace],t='',r={get flags(){t+='f';return '';},get exec(){t+='e';return function(s){t+='x';return this===r && s==='a' && arguments.length===1?null:7;};}};
        f.call(r,{toString(){t+='s';return 'a';}},{toString(){t+='r';return 'b';}})==='a' && t==='srfex'
    ",
    );
    check(
        r"
        let t='',r={get flags(){t+='f';throw 9;}};
        try{RegExp.prototype[Symbol.replace].call(r,{toString(){t+='s';return ''; }},{toString(){t+='r';throw 7;}});}catch(e){t+=e;}t==='sr7'
    ",
    );
    check(
        r"
        let f=function(){return 'x';};f[Symbol.toPrimitive]=function(){throw 7;};
        RegExp.prototype[Symbol.replace].call({flags:'',exec(){return null;}},'a',f)==='a'
    ",
    );
    for receiver in ["undefined", "null", "1", "'x'", "true", "1n", "Symbol()"] {
        assert!(matches!(Realm::default().eval(&format!("RegExp.prototype[Symbol.replace].call({receiver},{{toString(){{throw 7;}}}},{{toString(){{throw 8;}}}})")),Err(Error::Exception {kind:ExceptionKind::TypeError,..})),"{receiver}");
    }
    for result in ["undefined", "1", "true", "'x'", "1n", "Symbol()"] {
        assert!(matches!(Realm::default().eval(&format!("RegExp.prototype[Symbol.replace].call({{flags:'',exec(){{return {result};}}}},'','')")),Err(Error::Exception {kind:ExceptionKind::TypeError,..})),"{result}");
    }
}

#[test]
fn matching_finishes_before_live_result_reads_and_replacement_callbacks() {
    check(
        r"
        let t='',n=0,a={get length(){t+='L';return 2;},get 0(){t+='M';return 'a';},get index(){t+='I';return 0;},
            get 1(){t+='C';return {toString(){t+='c';return 'one';}};},get groups(){t+='G';}},
            b={length:1,0:'b',index:1},r={flags:'g',get exec(){t+='e';return function(){t+='x';return n++===0?a:n===2?b:null;};}};
        let v=RegExp.prototype[Symbol.replace].call(r,'ab',function(m,c,p,s){t+='R';if(n!==3)throw 7;
            if(m==='a' && (arguments.length!==4||c!=='one'||p!==0||s!=='ab'))throw 8;
            if(m==='b' && (arguments.length!==3||c!==1||p!=='ab'))throw 9;
            return {toString(){t+='q';return 'X';}};});
        v==='XX' && t==='exMexexLMICcGRqRq'
    ",
    );
    check(
        r"
        let n=0,a={length:2,0:'a',1:'old',index:0},b={length:1,0:'b',index:1},r={flags:'g',exec(){
            if(n++===0)return a;if(n===2){a[0]='x';a[1]='updated';return b;}return null;}};
        RegExp.prototype[Symbol.replace].call(r,'ab',function(m,c){return arguments.length===4?c:m;})==='updatedb'
    ",
    );
}

#[test]
fn numbered_and_named_substitution_resolves_in_order_without_rescanning_expansions() {
    check(
        r"
        let r={flags:'',exec(){return {length:4,0:'bcd',index:1,1:'c',2:undefined,3:'d'};}};
        RegExp.prototype[Symbol.replace].call(r,'abcde','[$1$2$3:$01:$10:$04:$0:$00]')==='a[cd:c:c0:$04:$0:$00]e'
    ",
    );
    check(
        r#"
        let r={flags:'',exec(){return {length:1,0:'b',index:1};}};
        RegExp.prototype[Symbol.replace].call(r,'abcd',"$$|$&|$`|$'")==='a$|b|a|cdcd'
    "#,
    );
    check(
        r"
        let t='',groups={get foo(){t+='g';return {toString(){t+='s';return '$1$<foo>';}};},'':'empty','$1':'weird'},
            r={flags:'',exec(){return {length:2,0:'x',index:0,1:'C',groups};}};
        RegExp.prototype[Symbol.replace].call(r,'x','$<foo>|$<foo>|$1|$<missing>|$<>|$<$1>')==='$1$<foo>|$1$<foo>|C||empty|weird' && t==='gsgs'
    ",
    );
    assert_eq!(Realm::default().eval(r"let r={flags:'',exec(){return {length:2,0:'\ud800',index:0,1:'\udc01',groups:{'\udc00':'\ud801'}};}};RegExp.prototype[Symbol.replace].call(r,'\ud800','$&$1$<\udc00>')"),Ok(Value::String(JsString::from_code_units(vec![0xd800,0xdc01,0xd801]))));
}

#[test]
fn groups_are_boxed_for_text_and_passed_unchanged_to_functions() {
    check(
        r"
        let r={flags:'',exec(){return {length:1,0:'b',index:1,groups:'123'};}};
        RegExp.prototype[Symbol.replace].call(r,'ab','[$<length>]')==='a[3]'
    ",
    );
    check(
        r"
        let r={flags:'',exec(){return {length:1,0:'x',index:0,groups:null};}},ok=false;
        try{RegExp.prototype[Symbol.replace].call(r,'x','literal');}catch(e){ok=e instanceof TypeError;}
        ok && RegExp.prototype[Symbol.replace].call(r,'x',function(){'use strict';return this===undefined && arguments.length===4 && arguments[3]===null?'ok':'bad';})==='ok'
    ",
    );
    check(
        r"
        let group=Symbol(),r={flags:'',exec(){return {length:5,0:'m',index:0,1:undefined,2:7,3:null,4:1n,groups:group};}};
        RegExp.prototype[Symbol.replace].call(r,'m',function(m,a,b,c,d,p,s,g){'use strict';
            return this===undefined && arguments.length===8 && m==='m' && a===undefined && b==='7' && c==='null' && d==='1' && p===0 && s==='m' && g===group?'ok':'bad';})==='ok'
    ",
    );
    check(
        r"
        let r={flags:'',exec(){return [];}};
        RegExp.prototype[Symbol.replace].call(r,'foo',function(m,p,s){return arguments.length===3 && m==='undefined' && p===0 && s==='foo'?'$&':'bad';})==='$&'
    ",
    );
}

#[test]
fn positions_are_clamped_and_oversized_custom_matches_have_safe_suffixes() {
    for (index, expected) in [
        ("undefined", "(0)abc"),
        ("NaN", "(0)abc"),
        ("-0", "(0)abc"),
        ("-Infinity", "(0)abc"),
        ("-7", "(0)abc"),
        ("1.9", "a(1)bc"),
        ("Infinity", "abc(3)"),
        ("7", "abc(3)"),
    ] {
        check(&format!(
            r"
            let r={{flags:'',exec(){{return {{length:1,0:'',index:{index}}};}}}};
            RegExp.prototype[Symbol.replace].call(r,'abc',function(m,p){{return '('+p+')';}})==='{expected}'
        "
        ));
    }
    check(
        r#"
        let r={flags:'',exec(){return {length:1,0:'LONG',index:1};}};
        RegExp.prototype[Symbol.replace].call(r,'ab',"[$']")==='a[]'
    "#,
    );
    check(
        r"
        let t='',r={flags:'',exec(){return {get length(){t+='l';return {valueOf(){t+='n';return 2;}};},get 0(){t+='m';return 'x';},get index(){t+='i';return Symbol();},get 1(){t+='c';return 'x';}};}};
        try{RegExp.prototype[Symbol.replace].call(r,'x','');}catch(e){t+=e instanceof TypeError?'T':'bad';}t==='lnmiT'
    ",
    );
}

#[test]
fn empty_global_matches_advance_by_code_unit_or_unicode_code_point() {
    for (flags, expected, seen) in [
        ("g", vec![46, 0xd83d, 46, 0xde00, 46, 120, 46], "0,1,2,3,4"),
        ("gu", vec![46, 0xd83d, 0xde00, 46, 120, 46], "0,2,3,4"),
        ("gv", vec![46, 0xd83d, 0xde00, 46, 120, 46], "0,2,3,4"),
    ] {
        let mut realm = Realm::default();
        assert_eq!(realm.eval(&format!(r"let seen=[],r={{flags:'{flags}',exec(s){{seen.push(this.lastIndex);return this.lastIndex<=s.length?{{length:1,0:'',index:this.lastIndex}}:null;}}}};RegExp.prototype[Symbol.replace].call(r,'\ud83d\ude00x','.')")),Ok(Value::String(JsString::from_code_units(expected))));
        assert_eq!(
            realm.eval("seen.join(',')"),
            Ok(Value::String(JsString::from(seen)))
        );
    }
    check(
        r"
        let n=0,r={flags:'gv',exec(){if(n++===0){this.lastIndex=Infinity;return {length:1,0:'',index:0};}return null;}};
        RegExp.prototype[Symbol.replace].call(r,'','')==='' && r.lastIndex===9007199254740992
    ",
    );
}

#[test]
fn overlapping_results_still_call_replacers_and_propagate_their_errors() {
    check(
        r"
        let n=0,t='',r={flags:'g',exec(){return n++===0?{length:1,0:'0',index:3}:n===2?{length:1,0:'0',index:1}:null;}};
        RegExp.prototype[Symbol.replace].call(r,'abcde',function(m,p){t+=p;return 'X';})==='abcXe' && t==='31'
    ",
    );
    check(
        r"
        let n=0,t='',r={flags:'g',exec(){return n++===0?{length:1,0:'0',index:3}:n===2?{length:1,0:'0',index:1}:null;}};
        try{RegExp.prototype[Symbol.replace].call(r,'abcde',function(m,p){t+=p;if(p===1)throw 7;return 'X';});}catch(e){t+=e;}t==='317'
    ",
    );
    check(
        r"
        let n=0,r={flags:'g',exec(){return n++<2?{length:1,0:'',index:0}:null;}};
        RegExp.prototype[Symbol.replace].call(r,'x','X')==='XXx'
    ",
    );
}

#[test]
fn string_hooks_and_poisoned_library_properties_do_not_change_native_replacement() {
    check(
        r"
        let f=RegExp.prototype[Symbol.replace],r={flags:'g',[Symbol.match]:true,[Symbol.replace]:f,exec(s){let i=this.lastIndex;if(i>=s.length)return null;this.lastIndex++;return {length:1,0:s.charAt(i),index:i};}};
        'ab'.replace(r,function(m){return m.toUpperCase();})==='AB' && 'ab'.replaceAll(r,function(m){return m.toUpperCase();})==='AB'
    ",
    );
    check(
        r"
        let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:2,0:'x',index:1,1:'y',groups:{foo:'z'}};}};
        delete Array.prototype.concat;delete Array.prototype.push;delete Array.prototype[Symbol.iterator];delete Function.prototype.apply;
        delete String.prototype.charAt;delete String.prototype.charCodeAt;delete String.prototype.indexOf;delete String.prototype.slice;delete String.prototype.substring;
        for(let i=0;i<5;i++)Object.defineProperty(Array.prototype,i,{get(){throw 7;},set(){throw 8;}});
        f.call(r,'axb','$`$1$<foo>')==='aayzb' && f.call(r,'axb',function(){return 'Q';})==='aQb'
    ",
    );
}

#[test]
fn optional_output_argument_and_work_limits_bypass_javascript_cleanup() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(32),
        ..Limits::default()
    });
    realm.eval("let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:1,0:'x',index:10};}},m=0").unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'x'.repeat(20),'z'.repeat(20));}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("f.call(r,'x'.repeat(20),'z'.repeat(13)).length"),
        Ok(Value::Number(32.0))
    );
    assert_eq!(realm.eval("try{f.call({flags:'',exec(){return {length:1,0:'x',index:28,groups:{get foo(){throw 7;}}};}},'x'.repeat(32),'$<foo>');}catch(e){m=e;}m===7"),Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_arguments: Some(6),
        ..Limits::default()
    });
    realm.eval("let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:5,0:'',index:0,1:'',2:'',3:'',4:''};}},m=0").unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'',function(){m=3;});}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let f=RegExp.prototype[Symbol.replace],r={flags:'g',exec(){return {0:'x'};}},m=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'','');}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    check(
        "let r={flags:'',exec(){return {length:1,0:'x',index:0};}};RegExp.prototype[Symbol.replace].call(r,'x','z'.repeat(120000)).length===120000",
    );
}

#[test]
fn recursive_exec_captures_named_getters_and_replacers_use_native_stack_guards() {
    for setup in [
        "let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return f.call(r,'','');}},g='';",
        "let f=RegExp.prototype[Symbol.replace],r={get flags(){return f.call(r,'','');}},g='';",
        "let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:1,0:'x',index:0};}},g=function(){return f.call(r,'',g);};",
        "let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:1,0:'x',index:0,groups:{get foo(){return f.call(r,'','$<foo>');}}};}},g='$<foo>';",
        "let f=RegExp.prototype[Symbol.replace],r={flags:'',exec(){return {length:2,0:'x',index:0,1:{toString(){return f.call(r,'','');}}};}},g='';",
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let m=0").unwrap();
        assert!(
            matches!(
                realm.eval("try{f.call(r,'',g);}catch{m=1;}finally{m=2;}"),
                Err(Error::Limit { .. })
            ),
            "{setup}"
        );
        assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
    }
}
