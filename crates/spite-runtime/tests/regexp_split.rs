//! Generic RegExp splitting with custom species and exec (22.2.6.14).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn input_species_flags_construction_and_limit_follow_the_specification_order() {
    check(
        r"
        let f=RegExp.prototype[Symbol.split],t='',r={get constructor(){t+='c';return {get [Symbol.species](){t+='p';return C;}};},get flags(){t+='f';return {toString(){t+='F';return 'gy';}};},
            get lastIndex(){throw 7;},get source(){throw 8;},get global(){throw 9;},get sticky(){throw 10;}};
        function C(pattern,flags){t+='n';if(new.target!==C||arguments.length!==2||pattern!==r||flags!=='gy'||!(this instanceof C))throw 11;return {get exec(){throw 12;},set lastIndex(v){throw 13;}};}
        let a=f.call(r,{toString(){t+='s';return 'x';}},{valueOf(){t+='l';return 0;}});
        Array.isArray(a)&&a.length===0&&t==='scpfFnl'
    ",
    );
    check(
        r"
        let t='',r={get constructor(){t+='c';throw 7;},get flags(){throw 8;}},ok=false;
        try{RegExp.prototype[Symbol.split].call(r,{toString(){t+='s';return 'x';}});}catch(e){ok=e===7;}
        ok&&t==='sc'
    ",
    );
    for receiver in ["undefined", "null", "1", "'x'", "true", "1n", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "RegExp.prototype[Symbol.split].call({receiver},{{toString(){{throw 7;}}}})"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
}

#[test]
fn species_checks_precede_flags_and_defaults_use_the_intrinsic_constructor() {
    for constructor in ["null", "false", "1", "''", "1n", "Symbol()"] {
        check(&format!(
            "let ok=false;try{{RegExp.prototype[Symbol.split].call({{constructor:{constructor},get flags(){{throw 7;}}}},'');}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    for species in [
        "{}",
        "0",
        "''",
        "1n",
        "Symbol()",
        "Date.now",
        "()=>{}",
        "({m(){}}).m",
    ] {
        check(&format!(
            "let ok=false;try{{RegExp.prototype[Symbol.split].call({{constructor:{{[Symbol.species]:{species}}},get flags(){{throw 7;}}}},'');}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    for constructor in [
        "undefined",
        "{}",
        "{[Symbol.species]:undefined}",
        "{[Symbol.species]:null}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let f=RegExp.prototype[Symbol.split],called=false;RegExp=function(){called=true;throw 7;};").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "f.call({{constructor:{constructor},flags:''}},'',0)"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{constructor}"
        );
        assert_eq!(realm.eval("called"), Ok(Value::Boolean(false)));
    }
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){throw 7;}}},ok=false;
        try{RegExp.prototype[Symbol.split].call(r,'',{valueOf(){throw 8;}});}catch(e){ok=e===7;}ok
    ",
    );
}

#[test]
fn sticky_flag_addition_preserves_the_original_utf16_flags() {
    for (flags, expected) in [
        ("", "y"),
        ("g", "gy"),
        ("Y", "Yy"),
        ("uy", "uy"),
        ("v", "vy"),
        ("abycd", "abycd"),
        (r"\ud800", r"\ud800y"),
    ] {
        check(&format!(
            "let seen,r={{flags:'{flags}',constructor:{{[Symbol.species]:function(p,f){{seen=f;return {{exec(){{return null;}}}};}}}}}};RegExp.prototype[Symbol.split].call(r,'').length===1&&seen==='{expected}'"
        ));
    }
}

#[test]
fn limits_use_uint32_and_stop_before_later_result_reads() {
    for (limit, length) in [
        ("undefined", 4),
        ("-1", 4),
        ("1.9", 1),
        ("4294967297", 1),
        ("4294967296", 0),
        ("NaN", 0),
        ("Infinity", 0),
        ("-Infinity", 0),
        ("null", 0),
        ("true", 1),
        ("-4294967295", 1),
    ] {
        check(&format!(
            "let r={{flags:'',constructor:{{[Symbol.species]:function(){{return {{exec(){{this.lastIndex=1;return {{length:3,1:'A',2:'B'}};}}}};}}}}}};RegExp.prototype[Symbol.split].call(r,'a',{limit}).length==={length}"
        ));
    }
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){return {exec(){this.lastIndex=1;return {get length(){throw 7;}};}};}}};
        RegExp.prototype[Symbol.split].call(r,'a',1).length===1
    ",
    );
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){return {exec(){this.lastIndex=1;return {length:Infinity,1:9,get 2(){throw 7;}};}};}}};
        let a=RegExp.prototype[Symbol.split].call(r,'a',2);a.length===2&&a[0]===''&&a[1]===9
    ",
    );
    for limit in ["1n", "Symbol()"] {
        assert!(matches!(Realm::default().eval(&format!("RegExp.prototype[Symbol.split].call({{flags:'',constructor:{{[Symbol.species]:function(){{return {{exec(){{throw 7;}}}};}}}}}},'',{limit})")),Err(Error::Exception{kind:ExceptionKind::TypeError,..})),"{limit}");
    }
}

#[test]
fn empty_input_executes_once_without_lastindex_or_capture_accesses() {
    for (result, length) in [
        ("null", 1),
        ("{get length(){throw 7;},get 0(){throw 8;}}", 0),
    ] {
        check(&format!(
            "let n=0,fake={{get lastIndex(){{throw 9;}},set lastIndex(v){{throw 10;}},exec(s){{if(this!==fake||s!==''||arguments.length!==1)throw 11;n++;return {result};}}}},r={{flags:'',constructor:{{[Symbol.species]:function(){{return fake;}}}}}};let a=RegExp.prototype[Symbol.split].call(r,'');a.length==={length}&&n===1"
        ));
    }
    for result in ["undefined", "1", "true", "'x'", "1n", "Symbol()"] {
        assert!(matches!(Realm::default().eval(&format!("RegExp.prototype[Symbol.split].call({{flags:'',constructor:{{[Symbol.species]:function(){{return {{exec(){{return {result};}}}};}}}}}},'')")),Err(Error::Exception{kind:ExceptionKind::TypeError,..})),"{result}");
    }
}

#[test]
fn captures_stay_uncoerced_and_reads_remain_live_and_ordered() {
    check(
        r"
        let t='',end=0,c={toString(){throw 7;}},s=Symbol(),z={get 0(){throw 8;},get index(){throw 9;},get groups(){throw 10;},get length(){t+='L';return {valueOf(){t+='l';return 5.9;}};},
            get 1(){t+='A';this[2]=s;return c;},2:'old',get 3(){t+='C';return undefined;},4:1n},
            fake={set lastIndex(v){t+='S';end=v;},get lastIndex(){t+='G';return {valueOf(){t+='g';return end;}};},get exec(){t+='E';return function(input){t+='x';if(this!==fake||input!=='a'||arguments.length!==1)throw 11;end=1;return z;};}},
            r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}};
        let a=RegExp.prototype[Symbol.split].call(r,'a');
        a.length===6&&a[0]===''&&a[1]===c&&a[2]===s&&Object.hasOwn(a,3)&&a[3]===undefined&&a[4]===1n&&a[5]===''&&t==='SExGgLlAC'
    ",
    );
    check(
        r"
        let n=0,fake={exec(){if(this.lastIndex===1){this.lastIndex=2;return {length:1};}this.exec=function(){return null;};n++;return null;}},
            r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}};
        let a=RegExp.prototype[Symbol.split].call(r,'abc');a.length===1&&a[0]==='abc'&&n===1
    ",
    );
}

#[test]
fn failure_and_empty_matches_advance_by_code_point_only_with_unicode_flags() {
    for (flags, indices, length) in [
        ("", "0,1,2,3,4,", 5),
        ("u", "0,2,3,4,", 4),
        ("v", "0,2,3,4,", 4),
    ] {
        for matches in [false, true] {
            let result = if matches {
                "{length:0,get 0(){throw 7;}}"
            } else {
                "null"
            };
            let indices = if matches {
                if flags.is_empty() {
                    "0,1,1,2,2,3,3,4,4,"
                } else {
                    "0,2,2,3,3,4,4,"
                }
            } else {
                indices
            };
            let expected_length = if matches { length } else { 1 };
            check(&format!(
                r"let seen='',end=0,fake={{set lastIndex(v){{seen+=v+',';end=v;}},get lastIndex(){{return end;}},exec(){{return {result};}}}},r={{flags:'{flags}',constructor:{{[Symbol.species]:function(){{return fake;}}}}}};let a=RegExp.prototype[Symbol.split].call(r,'\ud800\udc00\ud800X\udc00');a.length==={expected_length}&&seen==='{indices}'"
            ));
        }
    }
    check(
        r"
        let fake={set lastIndex(v){},get lastIndex(){throw 7;},exec(){return null;}},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}};
        RegExp.prototype[Symbol.split].call(r,'abc')[0]==='abc'
    ",
    );
    check(
        r"
        let fake={exec(){return {length:0};}},r={flags:'u',constructor:{[Symbol.species]:function(){return fake;}}};
        let a=RegExp.prototype[Symbol.split].call(r,'\ud800\udc00');a.length===1&&a[0]==='\ud800\udc00'
    ",
    );
}

#[test]
fn lastindex_is_clamped_and_backward_results_follow_the_algorithm() {
    for (end, expected) in [
        ("Infinity", "a.length===2&&a[0]===''&&a[1]===''"),
        ("100", "a.length===2&&a[0]===''&&a[1]===''"),
        ("2.9", "a.length===2&&a[0]===''&&a[1]==='cd'"),
        ("NaN", "a.length===1&&a[0]==='abcd'"),
        ("-1", "a.length===1&&a[0]==='abcd'"),
    ] {
        check(&format!(
            "let fake={{set lastIndex(v){{}},get lastIndex(){{return {end};}},exec(){{return {{length:0}};}}}},r={{flags:'',constructor:{{[Symbol.species]:function(){{return fake;}}}}}};let a=RegExp.prototype[Symbol.split].call(r,'abcd');{expected}"
        ));
    }
    check(
        r"
        let n=0,seen='',fake={exec(){seen+=this.lastIndex+',';if(n++===0){this.lastIndex=2;return {length:0};}if(n===2){this.lastIndex=1;return {length:0};}return null;}},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}};
        let a=RegExp.prototype[Symbol.split].call(r,'abcd');a.length===3&&a[0]===''&&a[1]===''&&a[2]==='bcd'&&seen==='0,2,1,2,3,'
    ",
    );
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){return {set lastIndex(v){},get lastIndex(){return Symbol();},exec(){return {};}};}}},ok=false;
        try{RegExp.prototype[Symbol.split].call(r,'a');}catch(e){ok=e instanceof TypeError;}ok
    ",
    );
}

#[test]
fn strict_writes_hooks_and_poisoned_libraries_preserve_native_semantics() {
    check(
        r"
        let n=0,fake={exec(){n++;return null;}},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}},ok=false;
        Object.defineProperty(fake,'lastIndex',{value:0,writable:false});
        try{RegExp.prototype[Symbol.split].call(r,'a');}catch(e){ok=e instanceof TypeError;}ok&&n===0
    ",
    );
    check(
        r"
        let f=RegExp.prototype[Symbol.split],fake={exec(){this.lastIndex++;return {length:2,1:'C'};}},r={flags:'',[Symbol.split]:f,constructor:{[Symbol.species]:function(){return fake;}}};
        delete Array.prototype.push;delete Array.prototype[Symbol.iterator];delete String.prototype.slice;delete String.prototype.substring;delete Function.prototype.apply;
        Array[Symbol.species]=function(){throw 7;};for(let i=0;i<5;i++)Object.defineProperty(Array.prototype,i,{get(){throw 8;},set(){throw 9;}});
        let a='ab'.split(r,3);a.length===3&&Object.getPrototypeOf(a)===Array.prototype&&a[0]===''&&a[1]==='C'&&a[2]===''
    ",
    );
}

#[test]
fn optional_limits_abort_without_cleanup_and_default_flags_have_no_size_cap() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(32),
        ..Limits::default()
    });
    realm.eval("let f=RegExp.prototype[Symbol.split],r={flags:'x'.repeat(32),constructor:{[Symbol.species]:function(){throw 7;}}},m=0").unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'');}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("r.flags='y'.repeat(32);try{f.call(r,'');}catch(e){m=e;}m===7"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let f=RegExp.prototype[Symbol.split],r={flags:'',constructor:{[Symbol.species]:function(){let n=0;return {exec(){this.lastIndex=n++%2===0?1:0;return {length:0};}};}}},m=0").unwrap();
    assert!(matches!(
        realm.eval("try{f.call(r,'ab');}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    check(
        "let seen,r={flags:'x'.repeat(120000),constructor:{[Symbol.species]:function(p,f){seen=f;return {exec(){return null;}};}}};RegExp.prototype[Symbol.split].call(r,'').length===1&&seen.length===120001&&seen[120000]==='y'",
    );
}

#[test]
fn recursive_species_exec_and_capture_getters_use_native_stack_guards() {
    for body in [
        "({get constructor(){return f.call(r,'a');}})",
        "({constructor:{get [Symbol.species](){return f.call(r,'a');}}})",
        "({constructor:{[Symbol.species]:function(){}},get flags(){return f.call(r,'a');}})",
        "({constructor:{[Symbol.species]:function(){return f.call(r,'a');}},flags:''})",
        "({constructor:{[Symbol.species]:function(){return {exec(){return f.call(r,'a');}};}},flags:''})",
        "({constructor:{[Symbol.species]:function(){return {exec(){this.lastIndex=1;return {length:2,get 1(){return f.call(r,'a');}};}};}},flags:''})",
        "({constructor:{[Symbol.species]:function(){return {set lastIndex(v){},get lastIndex(){return {valueOf(){return f.call(r,'a');}};},exec(){return {};}};}},flags:''})",
        "({constructor:{[Symbol.species]:function(){return {exec(){this.lastIndex=1;return {length:{valueOf(){return f.call(r,'a');}}};}};}},flags:''})",
    ] {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let f=RegExp.prototype[Symbol.split],r={body},m=0"
            ))
            .unwrap();
        assert!(
            matches!(
                realm.eval("try{f.call(r,'a');}catch{m=1;}finally{m=2;}"),
                Err(Error::Limit { .. })
            ),
            "{body}"
        );
        assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
    }
}
