//! Generic matchAll construction and RegExp String Iterator semantics (22.2.6.9, 22.2.9).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn creation_orders_input_species_flags_construction_and_lastindex_before_lazy_exec() {
    check(
        r"
        let t='',fake={set lastIndex(v){t+='w';if(v!==2)throw 7;},get exec(){throw 8;}},
            r={get constructor(){t+='c';return {get [Symbol.species](){t+='p';return C;}};},get flags(){t+='f';return {toString(){t+='F';return 'gu';}};},
                get lastIndex(){t+='l';return {valueOf(){t+='L';return 2.9;}};},get [Symbol.match](){throw 9;},get source(){throw 10;},get global(){throw 11;},get unicode(){throw 12;}};
        function C(pattern,flags){t+='n';if(new.target!==C||arguments.length!==2||pattern!==r||flags!=='gu'||!(this instanceof C))throw 13;return fake;}
        let i=RegExp.prototype[Symbol.matchAll].call(r,{toString(){t+='s';return 'abc';}});
        typeof i.next==='function'&&t==='scpfFnlLw'&&i[Symbol.iterator]()===i
    ",
    );
    check(
        r"
        let fake={lastIndex:0,exec(){return null;}},r={flags:'',lastIndex:1,constructor:{[Symbol.species]:function(){r.lastIndex=3;return fake;}}};
        RegExp.prototype[Symbol.matchAll].call(r,'abcd');fake.lastIndex===3
    ",
    );
    for receiver in ["undefined", "null", "1", "'x'", "true", "1n", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "RegExp.prototype[Symbol.matchAll].call({receiver},{{toString(){{throw 7;}}}})"
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
fn species_defaults_and_invalid_constructors_preserve_the_early_checks() {
    for constructor in ["null", "1", "false", "''", "1n", "Symbol()"] {
        check(&format!(
            "let ok=false;try{{RegExp.prototype[Symbol.matchAll].call({{constructor:{constructor},get flags(){{throw 7;}}}},'');}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    for species in ["{}", "Date.now", "()=>{}", "({m(){}}).m", "1n", "Symbol()"] {
        check(&format!(
            "let ok=false;try{{RegExp.prototype[Symbol.matchAll].call({{constructor:{{[Symbol.species]:{species}}},get flags(){{throw 7;}}}},'');}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    for constructor in [
        "undefined",
        "{}",
        "{[Symbol.species]:undefined}",
        "{[Symbol.species]:null}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let f=RegExp.prototype[Symbol.matchAll],called=false;RegExp=function(){called=true;throw 7;};").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "let caught=false;try{{f.call({{constructor:{constructor},flags:'',get lastIndex(){{throw 8;}}}},'');}}catch(e){{caught=e===8;}}caught"
            )),
            Ok(Value::Boolean(true)),
            "{constructor}"
        );
        assert_eq!(realm.eval("called"), Ok(Value::Boolean(false)));
    }
}

#[test]
fn lastindex_is_copied_with_tolength_and_strict_write_before_iteration() {
    for (value, expected) in [
        ("undefined", "0"),
        ("NaN", "0"),
        ("-1", "0"),
        ("-Infinity", "0"),
        ("-0", "0"),
        ("2.9", "2"),
        ("Infinity", "9007199254740991"),
    ] {
        check(&format!(
            "let fake={{exec(){{return null;}}}},r={{flags:'',lastIndex:{value},constructor:{{[Symbol.species]:function(){{return fake;}}}}}};RegExp.prototype[Symbol.matchAll].call(r,'');Object.is(fake.lastIndex,{expected})"
        ));
    }
    for value in ["1n", "Symbol()"] {
        check(&format!(
            "let ok=false;try{{RegExp.prototype[Symbol.matchAll].call({{flags:'',lastIndex:{value},constructor:{{[Symbol.species]:function(){{return {{set lastIndex(v){{throw 7;}}}};}}}}}},'');}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    check(
        r"
        let fake={},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}},ok=false;
        Object.defineProperty(fake,'lastIndex',{value:0,writable:false});
        try{RegExp.prototype[Symbol.matchAll].call(r,'');}catch(e){ok=e instanceof TypeError;}ok
    ",
    );
}

#[test]
fn nonglobal_results_preserve_identity_without_reading_match_properties() {
    check(
        r"
        let n=0,z={get 0(){throw 7;},get length(){throw 8;},get index(){throw 9;},get groups(){throw 10;}},fake={get flags(){throw 11;},get global(){throw 12;},get unicode(){throw 13;},exec(s){n++;if(this!==fake||s!=='abc'||arguments.length!==1)throw 14;return z;}},
            r={flags:'',lastIndex:2,constructor:{[Symbol.species]:function(){return fake;}}};
        let i=RegExp.prototype[Symbol.matchAll].call(r,'abc');r.lastIndex=0;r.flags='g';
        let a=i.next();fake.exec=function(){throw 15;};let b=i.next(),c=i.next();
        a.value===z&&a.done===false&&b.value===undefined&&b.done===true&&c.done===true&&b!==c&&n===1&&fake.lastIndex===2
    ",
    );
    check(
        r"
        let n=0,fake={exec(){n++;return null;}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');
        i.next().done&&i.next().done&&n===1
    ",
    );
}

#[test]
fn global_exec_is_live_and_abrupt_completions_leave_the_iterator_retryable() {
    check(
        r"
        let fake={get exec(){throw 7;}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'abc'),ok=false;
        try{i.next();}catch(e){ok=e===7;}
        Object.defineProperty(fake,'exec',{value:function(){return {get 0(){throw 8;}};},writable:true,configurable:true});
        try{i.next();}catch(e){ok=ok&&e===8;}
        fake.exec=function(){return {0:{toString(){throw 9;}}};};try{i.next();}catch(e){ok=ok&&e===9;}
        fake.exec=function(){return 1;};try{i.next();}catch(e){ok=ok&&e instanceof TypeError;}
        let z={0:'a',get length(){throw 10;},get index(){throw 11;},get groups(){throw 12;}};fake.exec=function(){return z;};
        let a=i.next();fake.exec=function(){return null;};ok&&a.value===z&&!a.done&&i.next().done
    ",
    );
}

#[test]
fn matcher_slots_survive_collection_after_public_references_are_cleared() {
    let mut realm = Realm::default();
    let Value::Object(matcher)=realm.eval("let fake={exec(s){return {0:s};}},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'captured');fake").unwrap() else {panic!()};
    realm.eval("fake=null;r=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&matcher).is_ok());
    assert_eq!(
        realm.eval("i.next().value[0]==='captured'"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&matcher).is_ok());
    assert_eq!(realm.eval("i.next().done"), Ok(Value::Boolean(true)));
    realm.eval("i=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&matcher).is_err());
}

#[test]
fn optional_work_and_output_aborts_skip_cleanup_without_completing_the_iterator() {
    let mut realm = Realm::new(Limits {
        max_string_units: Some(32),
        ..Limits::default()
    });
    realm.eval("let fake={exec(){return {0:{toString(){return 'x'.repeat(33);}}};}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),m=0").unwrap();
    assert!(matches!(
        realm.eval("try{i.next();}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("fake.exec=function(){return null;};i.next().done"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm.eval("let fake={exec(){return {0:'x'};}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),m=0").unwrap();
    assert!(matches!(
        realm.eval("try{while(true)i.next();}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("fake.exec=function(){return null;};i.next().done"),
        Ok(Value::Boolean(true))
    );
    check(
        "let fake={exec(s){return {0:s};}},r={flags:'',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'x'.repeat(120000));i.next().value[0].length===120000&&i.next().done",
    );
}

#[test]
fn recursive_creation_and_next_hooks_use_native_stack_guards() {
    for (setup, operation) in [
        (
            "let f=RegExp.prototype[Symbol.matchAll],r={get constructor(){return f.call(r,'');}};",
            "f.call(r,'')",
        ),
        (
            "let f=RegExp.prototype[Symbol.matchAll],r={flags:'',constructor:{[Symbol.species]:function(){return f.call(r,'');}}};",
            "f.call(r,'')",
        ),
        (
            "let f=RegExp.prototype[Symbol.matchAll],r={flags:'',get lastIndex(){return f.call(r,'');},constructor:{[Symbol.species]:function(){return {};}}};",
            "f.call(r,'')",
        ),
        (
            "let fake={exec(){return i.next();}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');",
            "i.next()",
        ),
        (
            "let fake={exec(){return {get 0(){return i.next();}};}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');",
            "i.next()",
        ),
        (
            "let fake={exec(){this.lastIndex={valueOf(){return i.next();}};return {0:''};}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');",
            "i.next()",
        ),
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let m=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!("try{{{operation};}}catch{{m=1;}}finally{{m=2;}}")),
                Err(Error::Limit { .. })
            ),
            "{setup}"
        );
        assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
    }
}

#[test]
fn empty_global_matches_advance_lossless_utf16_with_the_captured_unicode_flags() {
    for (flags, indices, count) in [
        ("g", "0,1,2,3,4,", 4),
        ("gu", "0,2,3,4,", 3),
        ("gv", "0,2,3,4,", 3),
    ] {
        check(&format!(
            r"let seen='',z={{0:{{toString(){{return '';}}}}}},fake={{exec(s){{seen+=this.lastIndex+',';return this.lastIndex>s.length?null:z;}}}},r={{flags:'{flags}',constructor:{{[Symbol.species]:function(){{return fake;}}}}}},i=RegExp.prototype[Symbol.matchAll].call(r,'\ud800\udc00X'),n=0;r.flags='g';for(let a of i){{if(a!==z)throw 7;n++;}}n==={count}&&seen==='{indices}'"
        ));
    }
    check(
        r"
        let n=0,fake={exec(){if(n++>0)return null;this.lastIndex=Infinity;return {0:''};}},r={flags:'gu',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'x');
        !i.next().done&&fake.lastIndex===9007199254740992&&i.next().done
    ",
    );
    check(
        r"
        let fake={exec(){return {0:''};}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),ok=false;
        Object.defineProperty(fake,'lastIndex',{value:0,writable:false});try{i.next();}catch(e){ok=e instanceof TypeError;}
        Object.defineProperty(fake,'lastIndex',{writable:true});ok&&!i.next().done&&fake.lastIndex===1
    ",
    );
}

#[test]
fn reentrant_next_calls_do_not_reset_a_nested_completed_state() {
    check(
        r"
        let n=0,z={0:'x'},fake={exec(){if(n++===0){if(!i.next().done)throw 7;return z;}return null;}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');
        let a=i.next();a.value===z&&!a.done&&i.next().done&&n===2
    ",
    );
    check(
        r"
        let n=0,z={get 0(){if(!i.next().done)throw 7;return '';}},fake={exec(){return n++===0?z:null;}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,'');
        let a=i.next();a.value===z&&!a.done&&i.next().done&&fake.lastIndex===1&&n===2
    ",
    );
    check(
        r"
        let n=0,fake={exec(){if(n++===0){i.next();throw 7;}return null;}},r={flags:'g',constructor:{[Symbol.species]:function(){return fake;}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),ok=false;
        try{i.next();}catch(e){ok=e===7;}ok&&i.next().done&&n===2
    ",
    );
}

#[test]
fn iterator_prototypes_results_and_brands_have_the_standard_descriptors() {
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){return {exec(){return null;}};}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),p=Object.getPrototypeOf(i),n=p.next,
            d=Object.getOwnPropertyDescriptor(p,'next'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),a=Object.getOwnPropertyDescriptor(n,'name'),l=Object.getOwnPropertyDescriptor(n,'length'),z=i.next(),v=Object.getOwnPropertyDescriptor(z,'value'),b=Object.getOwnPropertyDescriptor(z,'done');
        Object.getPrototypeOf(p)===Iterator.prototype&&i instanceof Iterator&&i[Symbol.iterator]()===i&&Object.getOwnPropertyNames(i).length===0&&Object.getOwnPropertyNames(p).join(',')==='next'&&Object.getOwnPropertySymbols(p).length===1&&
        d.value===n&&d.writable&&!d.enumerable&&d.configurable&&t.value==='RegExp String Iterator'&&!t.writable&&!t.enumerable&&t.configurable&&
        a.value==='next'&&!a.writable&&!a.enumerable&&a.configurable&&l.value===0&&!l.writable&&!l.enumerable&&l.configurable&&
        Object.getPrototypeOf(z)===Object.prototype&&Object.getOwnPropertyNames(z).join(',')==='value,done'&&v.value===undefined&&v.writable&&v.enumerable&&v.configurable&&b.value===true&&b.writable&&b.enumerable&&b.configurable&&
        Object.prototype.toString.call(i)==='[object RegExp String Iterator]'
    ",
    );
    for receiver in [
        "undefined",
        "null",
        "1",
        "'x'",
        "true",
        "1n",
        "Symbol()",
        "{}",
        "p",
        "Object.create(i)",
        "[][Symbol.iterator]()",
    ] {
        check(&format!(
            "let r={{flags:'',constructor:{{[Symbol.species]:function(){{return {{exec(){{return null;}}}};}}}}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),p=Object.getPrototypeOf(i),ok=false;try{{p.next.call({receiver});}}catch(e){{ok=e instanceof TypeError;}}ok"
        ));
    }
    check(
        r"
        let r={flags:'',constructor:{[Symbol.species]:function(){return {exec(){return null;}};}}},i=RegExp.prototype[Symbol.matchAll].call(r,''),ok=false;
        try{Reflect.construct(function(){},[],i.next);}catch(e){ok=e instanceof TypeError;}ok
    ",
    );
}

#[test]
fn string_matchall_and_iterator_helpers_bypass_poisoned_public_libraries() {
    check(
        r"
        let t='',f=RegExp.prototype[Symbol.matchAll],fake={exec(s){let n=this.lastIndex;if(n>=s.length)return null;this.lastIndex++;return {0:s[n]};}},
            r={get flags(){t+='f';return 'g';},lastIndex:0,[Symbol.match]:true,get [Symbol.matchAll](){t+='h';return f;},constructor:{[Symbol.species]:function(){t+='c';return fake;}}};
        let i='ab'.matchAll(r);
        delete String.prototype.matchAll;delete String.prototype.charAt;delete RegExp.prototype[Symbol.matchAll];delete Array.prototype.push;delete Array.prototype[Symbol.iterator];delete Function.prototype.apply;
        Array[Symbol.species]=function(){throw 7;};
        for(let n=0;n<3;n++)Object.defineProperty(Array.prototype,n,{get(){throw 8;},set(){throw 9;}});
        Object.defineProperty(Object.prototype,'done',{get(){throw 10;},set(){throw 11;}});
        Object.defineProperty(Object.prototype,'value',{get(){throw 10;},set(){throw 11;}});
        let a=i.map(function(z){return z[0];}).toArray();
        a.length===2&&a[0]==='a'&&a[1]==='b'&&Object.getPrototypeOf(a)===Array.prototype&&t==='fhfc'
    ",
    );
}
