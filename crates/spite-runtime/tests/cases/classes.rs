//! Base ClassDefinitionEvaluation, construction, and method closures (15.7).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn default_and_explicit_constructors_use_base_return_and_prototype_rules() {
    check(
        "class C{} let x=new C; x instanceof C && Object.getPrototypeOf(x)===C.prototype && x.constructor===C && C.length===0 && C.name==='C'",
    );
    check("class C{constructor(x,y=2){this.x=x+y;}} let x=new C(3);x.x===5 && C.length===1");
    for value in ["undefined", "null", "3", "'x'", "true", "Symbol()", "3n"] {
        check(&format!(
            "class C{{constructor(){{this.x=7;return {value};}}}} new C().x===7"
        ));
    }
    check("let o={};class C{constructor(){return o;}}new C()===o");
    check(
        "class C{constructor(x=new.target){this.target=x;}} function N(){}let p={};N.prototype=p;let x=Reflect.construct(C,[],N);Object.getPrototypeOf(x)===p && x.target===N",
    );
    check(
        "class C{}function N(){}N.prototype=7;Object.getPrototypeOf(Reflect.construct(C,[],N))===Object.prototype",
    );
}

#[test]
fn class_calls_evaluate_arguments_and_apply_lists_before_throwing() {
    check(
        "let log='',caught=false;class C{constructor(){log+='body';}}try{C(log+='arg');}catch(e){caught=e instanceof TypeError;}caught && log==='arg'",
    );
    check(
        "let log='',caught=false;class C{}let a={get length(){log+='L';return 2;},get 0(){log+='0';return 1;},get 1(){log+='1';return 2;}};try{Reflect.apply(C,null,a);}catch(e){caught=e instanceof TypeError;}caught && log==='L01'",
    );
    check(
        "class C{constructor(x){this.x=x;}}let B=C.bind(null,7),caught=false;try{B();}catch(e){caught=e instanceof TypeError;}caught && new B().x===7 && new B instanceof C",
    );
    check(
        "let caught=false;class C{}try{C((function(){throw 7;})());}catch(e){caught=e===7;}caught",
    );
}

#[test]
fn constructor_and_method_descriptors_are_non_enumerable_and_exact() {
    check(
        "class C{m(a,b=1){} static s(){} get x(){return 1;}set x(v){}}let p=Object.getOwnPropertyDescriptor(C,'prototype'),c=Object.getOwnPropertyDescriptor(C.prototype,'constructor'),m=Object.getOwnPropertyDescriptor(C.prototype,'m'),x=Object.getOwnPropertyDescriptor(C.prototype,'x');!p.writable && !p.enumerable && !p.configurable && c.writable && !c.enumerable && c.configurable && c.value===C && m.writable && !m.enumerable && m.configurable && m.value.length===1 && !Object.hasOwn(m.value,'prototype') && !x.enumerable && x.configurable && x.get.name==='get x' && x.set.name==='set x' && Object.keys(C).length===0 && Object.keys(C.prototype).length===0 && Object.getPrototypeOf(C)===Function.prototype && Object.getPrototypeOf(C.prototype)===Object.prototype",
    );
    check(
        "class C{m(){}static s(){}}let caught=0;try{new C.prototype.m;}catch(e){caught+=e instanceof TypeError;}try{new C.s;}catch(e){caught+=e instanceof TypeError;}caught===2 && typeof C==='function' && Object.prototype.toString.call(C)==='[object Function]'",
    );
    check(
        "class C{['constructor'](){return 7;}static constructor(){return 8;}}new C().constructor()===7 && C.constructor()===8",
    );
}

#[test]
fn all_class_code_is_strict_without_requiring_a_use_strict_directive() {
    check(
        "class C{m(){return this;}static s(){return this;}}C.prototype.m.call(undefined)===undefined && C.prototype.m.call(7)===7 && C.s.call(null)===null",
    );
    check(
        "class C{constructor(x){arguments[0]=8;this.x=x;}m(x){x=9;return arguments[0];}n(x=1){return x;}}let c=new C(7);c.x===7 && c.m(2)===2 && c.n()===1",
    );
    check(
        "let caught=false;class C{m(){undeclared=1;}}try{new C().m();}catch(e){caught=e instanceof ReferenceError;}caught && typeof undeclared==='undefined'",
    );
    check(
        "let caught=false;class C{m(){return function(){return this;};}}new C().m()()===undefined && (tryStrict(),caught);function tryStrict(){try{C.prototype.m.caller;}catch(e){caught=e instanceof TypeError;}}",
    );
}

#[test]
fn declaration_bindings_are_mutable_but_captured_class_names_are_immutable() {
    check(
        "class C{m(){return C;}static self(){return C;}change(){C=7;}}let original=C,x=new C,caught=false;C=8;try{x.change();}catch(e){caught=e instanceof TypeError;}caught && C===8 && x.m()===original && original.self()===original && !Object.hasOwn(globalThis,'C')",
    );
    check(
        "let C=class Internal{m(){return Internal;}};let x=new C;C=7;x.m()!==7 && typeof Internal==='undefined' && x.m().name==='Internal'",
    );
    check("let C=class{m(){return C;}};let x=new C;C=7;x.m()===7");
    check("let C=1;{class C{}C=2;}C===1");
    assert_eq!(Realm::default().eval("7;class C{}"), Ok(Value::Number(7.0)));
}

#[test]
fn computed_names_see_tdz_until_all_methods_have_been_defined() {
    check(
        "let caught=false;try{class C{[C](){} }}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    check(
        "let captured;class C{[(captured=()=>C,'x')](){}static y(){return C;}}captured()===C && C.y()===C",
    );
    check(
        "let captured,caught=false;try{class C{[(captured=()=>C,'x')](){}[captured()](){} }}catch(e){caught=e instanceof ReferenceError;}caught && (function(){try{captured();}catch(e){return e instanceof ReferenceError;}})()",
    );
    let mut realm = Realm::default();
    realm
        .eval("let x=1;try{class Failed{[(function(){throw 7;})()](){}}}catch(e){x=e;}")
        .unwrap();
    assert_eq!(realm.eval("x"), Ok(Value::Number(7.0)));
    assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    realm.eval("class Pending{[Pending](){}}").unwrap_err();
    assert_eq!(
        realm.eval("try{Pending;}catch(e){e instanceof ReferenceError;}"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn inferred_names_are_set_before_static_methods_can_overwrite_them() {
    check(
        "let C=class{};let o={key:class{}};let A;A=class{};C.name==='C' && o.key.name==='key' && A.name==='A' && (class{}).name===''",
    );
    check(
        "let C=class{static name(){return 7;}};let o={key:class{static name(){return 8;}}};C.name()===7 && o.key.name()===8",
    );
    check(
        "let [C=class{}]=[];let {x:D=class{}}={};function f(A=class{}){return A.name;}C.name==='C' && D.name==='D' && f()==='A'",
    );
    check("let s=Symbol('S'),o={[s]:class{}};o[s].name==='[S]'");
    check("let C=class Explicit{};C.name==='Explicit'");
}

#[test]
fn computed_names_are_ordered_and_do_not_create_a_class_this_binding() {
    check(
        "let log='',key={toString(){log+='K';return 'x';}};class C{[(log+='A',key)](){}static [(log+='B','y')](){}[(log+='C','z')](){}}log==='AKBC' && C.prototype.x.name==='x' && C.y.name==='y'",
    );
    check(
        "function f(){return class{[this.x](){}[new.target?'new':'call'](){}};}let C=f.call({x:'m'}),D=new f;Object.hasOwn(C.prototype,'m') && Object.hasOwn(C.prototype,'call') && Object.hasOwn(D.prototype,'new')",
    );
    check(
        "let h={m(){return class{[super.x](){} };}};Object.setPrototypeOf(h,{x:'m'});Object.hasOwn(h.m().prototype,'m')",
    );
    check(
        "let caught=false;try{class C{static ['prototype'](){}}}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn class_super_uses_constructor_and_method_home_objects() {
    check(
        "class C{constructor(){this.x=super.x;}m(){return super.x+this.x; }static m(){return super.x+this.x;}}Object.setPrototypeOf(C.prototype,{x:2});Object.setPrototypeOf(C,{x:3});C.x=10;let c=new C; c.x===2 && c.m()===4 && C.m()===13 && C.prototype.m.call({x:20})===22",
    );
    check(
        "class C{constructor(x=super.x){this.x=x;}m(){return eval('()=>super.x+this.x');}}Object.setPrototypeOf(C.prototype,{x:7});new C().m()()===14",
    );
    check(
        "class C{m(){return new.target; }static s(){return new.target;}}new C().m()===undefined && C.s()===undefined",
    );
}

#[test]
fn accessors_merge_and_method_keys_preserve_symbols_and_utf16() {
    check(
        "class C{get x(){return this.v;}set x(v){this.v=v;}get x(){return this.v+1;}}let c=new C;c.x=7;c.x===8 && Object.keys(c).join(',')==='v'",
    );
    check(
        "let s=Symbol('S');class C{[s](){return 7;}['\\uD800'](){return 8;}0x10n(){return 9;}}let c=new C;c[s]()===7 && c[s].name==='[S]' && c['\\uD800']()===8 && c['\\uD800'].name==='\\uD800' && c[16]()===9 && c[16].name==='16'",
    );
    check("class C{get x(){throw 7;}x(){return 8;}}new C().x()===8");
}

#[test]
fn source_stringification_retains_complete_class_and_method_source() {
    check(
        "let source='class C /*comment*/ { constructor(x=1) { this.x=x; } m() { return 7; } }',C=eval('('+source+')');C.toString()===source && C.prototype.m.toString()==='m() { return 7; }'",
    );
    check("let source='class /*x*/ {}',C=eval('('+source+')');C.toString()===source");
    check(
        "class C{static /*a*/ m /*b*/ () {return 7;} static get x(){return 8;}}C.m.toString()==='m /*b*/ () {return 7;}' && Object.getOwnPropertyDescriptor(C,'x').get.toString()==='get x(){return 8;}'",
    );
    check(
        "let source='class C { /*'+String.fromCharCode(55296)+'*/ }',C=eval('('+source+')');C.toString()===source",
    );
}

#[test]
fn eval_class_bindings_remain_lexical_and_strict_keys_do_not_leak_vars() {
    check("let x=7;class C{[eval('var x=9; x')](){}}x===7 && Object.hasOwn(C.prototype,'9')");
    check(
        "let C=eval('class Internal{};Internal');C.name==='Internal' && typeof Internal==='undefined'",
    );
    check("let C=Function('return class C{};')();C.name==='C' && new C instanceof C");
    check("let C=class{constructor(){this.v=eval('new.target');}};new C().v===C");
}

#[test]
fn class_environments_and_home_objects_survive_collection_without_leaking_cycles() {
    let mut realm = Realm::default();
    realm
        .eval("let f=(function(){class C{m(){return C;}}return C.prototype.m;})();")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f().prototype.m===f"), Ok(Value::Boolean(true)));
    realm.eval("f=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    let before = realm.collect(usize::MAX).unwrap().live;
    realm
        .eval("(function(){class C{m(){return C;}}new C;})();")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, before);
}
