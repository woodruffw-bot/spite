//! Generic RegExp match/search via custom exec (22.2.6.8/12, 22.2.7.1/3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn nonglobal_match_orders_conversion_and_returns_the_exact_exec_result() {
    check(
        r"
        let f=RegExp.prototype[Symbol.match],t='',result={},r={
            get flags(){t+='f';return {toString(){t+='c';return '';}};},
            get lastIndex(){throw 9;},
            get exec(){t+='e';return function(s){t+='x';return this===r && s==='s' && arguments.length===1?result:null;};}
        };
        f.call(r,{toString(){t+='s';return 's';}})===result && t==='sfcex'
    ",
    );
    check("RegExp.prototype[Symbol.match].call({flags:'u',exec(){return null;}},'')===null");
    check(
        r"
        let t='';try{RegExp.prototype[Symbol.match].call({get flags(){t+='f';throw 8;}},{toString(){t+='s';throw 7;}});}
        catch(e){t+=e;}t==='s7'
    ",
    );
    check(
        r"
        let t='';try{RegExp.prototype[Symbol.match].call({flags:Symbol(),get exec(){t+='e';}},'');}
        catch(e){t=e instanceof TypeError?'type':t;}t==='type'
    ",
    );
}

#[test]
fn global_match_reads_live_exec_and_creates_intrinsic_own_data_elements() {
    check(
        r"
        let f=RegExp.prototype[Symbol.match],t='',n=0,index=7,r={
            get flags(){t+='f';return 'g';},get global(){throw 9;},
            get lastIndex(){t+='l';return index;},set lastIndex(v){t+='s'+v;index=v;},
            get exec(){t+='e';return function(s){t+='x';if(this!==r||s!=='a'||arguments.length!==1)throw 8;
                n++;if(n===1)return {get 0(){t+='z';return {toString(){t+='c';return 'one';}};}};
                if(n===2){Object.defineProperty(r,'exec',{value(){t+='q';return null;}});return {0:2};}
                throw 7;};}
        };
        Object.defineProperty(Array.prototype,'0',{set(){throw 6;},configurable:true});
        let a=f.call(r,'a');
        Array.isArray(a) && Object.getPrototypeOf(a)===Array.prototype && a.length===2 && a[0]==='one' && a[1]==='2' &&
        Object.getOwnPropertyDescriptor(a,'0').writable && Object.getOwnPropertyDescriptor(a,'0').enumerable &&
        Object.getOwnPropertyDescriptor(a,'0').configurable && t==='fs0exzcexq' && index===0
    ",
    );
    check("RegExp.prototype[Symbol.match].call({flags:'g',exec(){return null;}},'')===null");
    check(
        r"
        let t='',r={flags:'g',exec(){t+='e';return {get 0(){t+='z';return {toString(){t+='c';throw 7;}};}};},
            get lastIndex(){t+='l';return 0;},set lastIndex(v){t+='s';}};
        try{RegExp.prototype[Symbol.match].call(r,'');}catch(e){t+=e;}t==='sezc7'
    ",
    );
    check(
        r"
        let f=RegExp.prototype[Symbol.match],r={flags:'g',lastIndex:0,exec(){return null;}};
        Object.defineProperty(r,'lastIndex',{writable:false});let ok=false;
        try{f.call(r,'');}catch(e){ok=e instanceof TypeError;}ok
    ",
    );
}

#[test]
fn empty_match_advancement_respects_both_unicode_flags_and_utf16_boundaries() {
    for (flags, expected) in [
        ("g", "0,1,2,3,4,5"),
        ("gu", "0,2,3,4,5"),
        ("gv", "0,2,3,4,5"),
        ("guv", "0,2,3,4,5"),
    ] {
        check(&format!(
            r"
            let f=RegExp.prototype[Symbol.match],seen=[],r={{flags:'{flags}',exec(s){{seen.push(this.lastIndex);return this.lastIndex<=s.length?{{0:''}}:null;}}}};
            let a=f.call(r,'\ud83d\ude00\ud800x');a.length===seen.length-1 && seen.join(',')==='{expected}'
        "
        ));
    }
    for (index, expected) in [
        ("-7", "1"),
        ("NaN", "1"),
        ("1.9", "2"),
        ("Infinity", "9007199254740992"),
        ("9007199254740991", "9007199254740992"),
    ] {
        check(&format!(
            r"
            let n=0,r={{flags:'gu',exec(){{if(n++===0){{this.lastIndex={index};return {{0:''}};}}return null;}}}};
            RegExp.prototype[Symbol.match].call(r,'x').length===1 && r.lastIndex==={expected}
        "
        ));
    }
    check(
        r"
        let n=0,t='',r={flags:'g',exec(){if(n++===0){this.lastIndex={valueOf(){t+='c';return 0;}};return {0:''};}return null;}};
        RegExp.prototype[Symbol.match].call(r,'').length===1 && t==='c' && r.lastIndex===1
    ",
    );
    check(
        r"
        let r={flags:'g',exec(){this.lastIndex=1n;return {0:''};}},ok=false;
        try{RegExp.prototype[Symbol.match].call(r,'');}catch(e){ok=e instanceof TypeError;}ok && r.lastIndex===1n
    ",
    );
}

#[test]
fn search_restores_exact_lastindex_before_reading_the_uncoerced_result_index() {
    check(
        r"
        let t='',last=-0,answer={valueOf(){throw 8;}},r={
            get flags(){throw 9;},get lastIndex(){t+='l';return last;},set lastIndex(v){t+=Object.is(v,-0)?'m':'z';last=v;},
            get exec(){t+='e';return function(s){t+='x';if(this!==r||s!=='a'||arguments.length!==1||!Object.is(last,0))throw 7;
                return {get index(){t+='i';if(!Object.is(last,-0))throw 6;return answer;}};};}
        };
        RegExp.prototype[Symbol.search].call(r,{toString(){t+='s';return 'a';}})===answer && t==='slzexlmi'
    ",
    );
    check(
        r"
        let writes=0,r={get lastIndex(){return 0;},set lastIndex(v){writes++;},exec(){return null;}};
        RegExp.prototype[Symbol.search].call(r,'')===-1 && writes===0
    ",
    );
    for original in [
        "NaN",
        "Symbol()",
        "{valueOf(){throw 7;}}",
        "1n",
        "undefined",
    ] {
        check(&format!(
            r"
            let old={original},r={{lastIndex:old,exec(){{this.lastIndex=4;return {{index:'7'}};}}}};
            RegExp.prototype[Symbol.search].call(r,'')==='7' && Object.is(r.lastIndex,old)
        "
        ));
    }
    check(
        r"
        let r={lastIndex:4,exec(){this.lastIndex=8;return {};}},v=RegExp.prototype[Symbol.search].call(r,'');
        v===undefined && r.lastIndex===4
    ",
    );
}

#[test]
fn search_abrupt_completions_keep_the_specified_partial_effects() {
    check(
        r"
        let r={lastIndex:3,exec(){this.lastIndex=7;throw 8;}},v;
        try{RegExp.prototype[Symbol.search].call(r,'');}catch(e){v=e;}v===8 && r.lastIndex===7
    ",
    );
    check(
        r"
        let t='',last=3,r={get lastIndex(){return last;},set lastIndex(v){t+='s';last=v;if(v===3)throw 7;},
            exec(){return {get index(){t+='i';throw 8;}};}};
        try{RegExp.prototype[Symbol.search].call(r,'');}catch(e){t+=e;}t==='ss7'
    ",
    );
    check(
        r"
        let t='',r={get lastIndex(){t+='l';throw 8;}};
        try{RegExp.prototype[Symbol.search].call(r,{toString(){t+='s';throw 7;}});}catch(e){t+=e;}t==='s7'
    ",
    );
    check(
        r"
        let r={lastIndex:2,exec(){return 7;}},ok=false;
        try{RegExp.prototype[Symbol.search].call(r,'');}catch(e){ok=e instanceof TypeError;}ok && r.lastIndex===0
    ",
    );
    check(
        r"
        let r={lastIndex:2,exec(){throw 7;}},ok=false;Object.freeze(r);
        try{RegExp.prototype[Symbol.search].call(r,'');}catch(e){ok=e instanceof TypeError;}ok
    ",
    );
    for member in ["match", "search"] {
        assert!(matches!(
            Realm::default().eval(&format!(
                "RegExp.prototype[Symbol.{member}].call({{flags:'',exec:0}},'')"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        for receiver in ["undefined", "null", "1", "'x'", "true", "1n", "Symbol()"] {
            assert!(matches!(Realm::default().eval(&format!("RegExp.prototype[Symbol.{member}].call({receiver},{{toString(){{throw 7;}}}})")),Err(Error::Exception {kind:ExceptionKind::TypeError,..})),"{member} {receiver}");
        }
    }
}

#[test]
fn string_hooks_delegate_and_opted_in_work_limits_bypass_javascript_cleanup() {
    check(
        r"
        let m={flags:'',exec(s){return s==='abc'?{0:'b'}:null;},[Symbol.match]:RegExp.prototype[Symbol.match]},
            s={lastIndex:5,exec(v){return {index:v.length};},[Symbol.search]:RegExp.prototype[Symbol.search]};
        'abc'.match(m)[0]==='b' && 'abc'.search(s)===3 && s.lastIndex===5
    ",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let f=RegExp.prototype[Symbol.match],r={flags:'g',exec(){return {0:'x'};}},m=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'');}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
}

#[test]
fn recursive_match_and_search_callbacks_use_default_native_stack_guards() {
    for setup in [
        "let f=RegExp.prototype[Symbol.match],r={flags:'g',exec(){return f.call(r,'');}};",
        "let f=RegExp.prototype[Symbol.search],r={lastIndex:0,exec(){return f.call(r,'');}};",
        "let f=RegExp.prototype[Symbol.match],r={get flags(){return f.call(r,'');}};",
        "let f=RegExp.prototype[Symbol.search],r={get lastIndex(){return f.call(r,'');}};",
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let m=0").unwrap();
        assert!(
            matches!(
                realm.eval("try{f.call(r,'');}catch{m=1;}finally{m=2;}"),
                Err(Error::Limit { .. })
            ),
            "{setup}"
        );
        assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
    }
}
