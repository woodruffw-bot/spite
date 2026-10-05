//! Recursive execution must hit host limits before exhausting a modest native stack.

use spite_runtime::{Error, Realm, Value};

#[test]
fn recursion_limits_work_on_a_two_mebibyte_thread_stack() {
    for (index, (setup, call)) in [
        ("let f=(x=f())=>1;", "f()"),
        ("function f(x=f()){return 1;}", "f()"),
        ("function f(){'use strict';return f();}", "f()"),
        ("function F(){new F;}", "new F"),
        ("class C{constructor(){new C;}}", "new C"),
        ("class C{constructor(x=new C){}}", "new C"),
        ("class B{constructor(){new D;}}class D extends B{}", "new D"),
        ("class B{constructor(){new D;}}class D extends B{constructor(){super();}}", "new D"),
        ("class D extends Object{constructor(x=new D){super();}}", "new D"),
        ("class D extends Object{constructor(){eval('new D');super();}}", "new D"),
        ("class C{m(){return this.m();}}let c=new C;", "c.m()"),
        ("function f(){return class{[f()](){}};}", "f()"),
        ("var code='class C{[eval(code)](){}}';", "eval(code)"),
        ("var code='eval(code)';", "eval(code)"),
        ("var code='(0,eval)(code)';", "(0,eval)(code)"),
        ("var code='globalThis.eval(code)';", "globalThis.eval(code)"),
        ("var code='eval?.(code)';", "eval?.(code)"),
        ("var code='eval.call(null,code)';", "eval.call(null,code)"),
        ("var code='eval.apply(null,[code])';", "eval.apply(null,[code])"),
        ("var code='Reflect.apply(eval,null,[code])';", "Reflect.apply(eval,null,[code])"),
        ("var code='bound(code)',bound=eval.bind(null);", "bound(code)"),
        ("var code='true && eval(code)';", "eval(code)"),
        ("var code='0, eval(code)';", "eval(code)"),
        ("var deep='('.repeat(48)+'1'+')'.repeat(48),n=12,code='n--?eval(code):eval(deep)';", "eval(code)"),
        ("var deep='('.repeat(48)+'1'+')'.repeat(48),n=12,code='n--?eval(code):Function(deep)()';", "eval(code)"),
        ("function f(){return eval('f()');}", "f()"),
        ("function f(){return (0,eval)('f()');}", "f()"),
        ("let p={m(){return h.m();}},h={m(){return super.m();}};Object.setPrototypeOf(h,p);", "h.m()"),
        ("let p={get x(){return h.m();}},h={m(){return super.x;}};Object.setPrototypeOf(h,p);", "h.m()"),
        ("let p={set x(v){h.m();}},h={m(){super.x=1;}};Object.setPrototypeOf(h,p);", "h.m()"),
        ("let k={toString(){return h.m();}},h={m(){return super[k];}};", "h.m()"),
        ("let h={m(){return eval('super.x');}},p={get x(){return h.m();}};Object.setPrototypeOf(h,p);", "h.m()"),
        ("let o={valueOf:()=>+o};", "+o"),
        ("let o={valueOf:()=>Number(o)};", "Number(o)"),
        ("let o={valueOf:()=>isFinite(o)};", "isFinite(o)"),
        ("let o={valueOf:()=>isNaN(o)};", "isNaN(o)"),
        ("let o={toString:()=>Error(o)};", "Error(o)"),
        ("let o={toString:()=>String(o)};", "String(o)"),
        ("let o={valueOf:()=>String.fromCodePoint(o)};", "String.fromCodePoint(o)"),
        ("let o={toString:()=>String.prototype.charAt.call(o)};", "String.prototype.charAt.call(o)"),
        ("let o={toString:()=>String.prototype.toWellFormed.call(o)};", "String.prototype.toWellFormed.call(o)"),
        ("let o={toString:()=>''.concat(o)};", "''.concat(o)"),
        ("let o={valueOf:()=>''.slice(o)};", "''.slice(o)"),
        ("let o={toString:()=>String.prototype.trim.call(o)};", "String.prototype.trim.call(o)"),
        ("let o={valueOf:()=>''.repeat(o)};", "''.repeat(o)"),
        ("let o={toString:()=>''.padStart(1,o)};", "''.padStart(1,o)"),
        ("let o={toString:()=>''.indexOf(o)};", "''.indexOf(o)"),
        ("let o={toString:()=>''.startsWith(o)};", "''.startsWith(o)"),
        ("let o={toString:()=>String.raw({raw:{0:o,length:1}})};", "String.raw({raw:{0:o,length:1}})"),
        (
            "let d={};Object.defineProperty(d,'value',{get:()=>Object.defineProperty({},'x',d)});",
            "Object.defineProperty({},'x',d)",
        ),
        (
            "let d={};Object.defineProperty(d,'x',{enumerable:true,get:()=>Object.defineProperties({},d)});",
            "Object.defineProperties({},d)",
        ),
        (
            "let o={};Object.defineProperty(o,'x',{enumerable:true,get:()=>Object.assign({},o)});",
            "Object.assign({},o)",
        ),
        (
            "let o={name:{toString:()=>Error.prototype.toString.call(o)}};",
            "Error.prototype.toString.call(o)",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        std::thread::Builder::new()
            .name(format!("bounded-recursion-{index}"))
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let mut realm = Realm::default();
                realm.eval(&format!("let flag=0;{setup}")).unwrap();
                assert!(
                    matches!(
                        realm.eval(&format!("try{{{call};}}catch{{flag=1;}}finally{{flag=2;}}")),
                        Err(Error::Limit { .. })
                    ),
                    "{setup}"
                );
                assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
                assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
                assert_eq!(realm.eval("Number('4')"), Ok(Value::Number(4.0)));
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
