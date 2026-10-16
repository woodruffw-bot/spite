//! Object methods, merged accessor descriptors, and ordinary function call state.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn methods_bind_receivers_and_capture_outer_environments_without_a_self_binding() {
    check(
        "let m=7,o={x:2,m(v=3){return this.x+v+m;}};o.m()===12 && o.m.call({x:4},5)===16 && o.m!==({m(){}}).m",
    );
    check(
        "function outer(x){return {m(){return ()=>x+this.y;},y:2};}let o=outer(7),f=o.m();f()===9",
    );
    check(
        "let o={m(){return this;},strict(){'use strict';return this;}};o.m.call(null)===globalThis && typeof o.m.call(7)==='object' && o.strict.call(undefined)===undefined && o.strict.call(7)===7",
    );
}

#[test]
fn methods_and_accessors_have_standard_descriptors_names_and_lengths() {
    check(
        "let o={m(a,b=1){},get x(){return 7;},set x(v){}},m=Object.getOwnPropertyDescriptor(o,'m'),x=Object.getOwnPropertyDescriptor(o,'x');m.writable && m.enumerable && m.configurable && m.value.name==='m' && m.value.length===1 && x.enumerable && x.configurable && x.get.name==='get x' && x.get.length===0 && x.set.name==='set x' && x.set.length===1 && !Object.hasOwn(m.value,'prototype') && !Object.hasOwn(x.get,'prototype') && !Object.hasOwn(x.set,'prototype')",
    );
    check(
        "let o={set x(v=1){},0x10n(){},true(){},['\\uD800'](){}},d=Object.getOwnPropertyDescriptor(o,'x');d.set.length===0 && o[16].name==='16' && o.true.name==='true' && o['\\uD800'].name==='\\uD800' && Object.getOwnPropertyNames(o[16]).join(',')==='length,name'",
    );
    check(
        "let o={m(){}},d=Object.getOwnPropertyDescriptor(o.m,'name'),l=Object.getOwnPropertyDescriptor(o.m,'length');!d.writable && !d.enumerable && d.configurable && !l.writable && !l.enumerable && l.configurable",
    );
}

#[test]
fn symbol_method_names_distinguish_absent_empty_and_raw_descriptions() {
    check(
        "let absent=Symbol(),empty=Symbol(''),raw=Symbol('\\uD800'),o={[absent](){},[empty](){},get [raw](){return 7;},set [raw](v){}};let d=Object.getOwnPropertyDescriptor(o,raw);o[absent].name==='' && o[empty].name==='[]' && d.get.name==='get [\\uD800]' && d.set.name==='set [\\uD800]'",
    );
    check("let s=Symbol(),o={get [s](){}};Object.getOwnPropertyDescriptor(o,s).get.name==='get '");
}

#[test]
fn computed_names_convert_once_before_later_properties_without_running_bodies() {
    check(
        "let log='',key={[Symbol.toPrimitive](hint){log+=hint;return 'x';}},o={[key](){log+='body';},get [(log+='g','value')](){log+='get';return 7;},next:(log+='n',1)};log==='stringgn' && o.x.name==='x' && (o.value===7) && log==='stringgnget'",
    );
    check(
        "let log='',caught=false;try{({[{toString(){log+='k';throw 7;}}](){},after:(log+='a',1)});}catch(e){caught=e===7;}caught && log==='k'",
    );
}

#[test]
fn accessor_pairs_merge_and_data_redefinitions_replace_them_in_source_order() {
    check(
        "let o={get x(){return this.v;},set x(value){this.v=value;},get x(){return this.v+1;}};o.x=7;let d=Object.getOwnPropertyDescriptor(o,'x');o.x===8 && typeof d.set==='function' && typeof d.get==='function' && Object.keys(o).join(',')==='x,v'",
    );
    check("let o={set x(v){this.v=v;},get x(){return this.v;}};o.x=8;o.x===8");
    check(
        "let o={get x(){throw 7;},set x(v){throw 8;},x:3};let d=Object.getOwnPropertyDescriptor(o,'x');o.x===3 && d.value===3 && d.writable && !Object.hasOwn(d,'get')",
    );
    check(
        "let o={x:3,get x(){return 7;}};let d=Object.getOwnPropertyDescriptor(o,'x');o.x===7 && d.set===undefined && !Object.hasOwn(d,'value')",
    );
    check("let o={get x(){throw 7;},x(){return 9;}};o.x()===9");
}

#[test]
fn accessor_calls_use_the_original_receiver_and_propagate_abrupt_completion() {
    check(
        "let parent={get x(){return this.v;},set x(value){this.v=value;return 99;}},child=Object.create(parent);child.x=7;child.x===7 && parent.v===undefined && !Object.hasOwn(child,'x')",
    );
    check(
        "let o={get x(){throw 7;},set x(v){throw v;}},a=false,b=false;try{o.x;}catch(e){a=e===7;}try{o.x=8;}catch(e){b=e===8;}a && b",
    );
    check("let o={get x(){return 7;}};o.x=8;o.x===7");
    assert!(matches!(
        Realm::default().eval("'use strict';let o={get x(){return 7;}};o.x=8"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check("let o={set x(v){this.v=v;}};o.x===undefined && (o.x=7,o.v===7)");
}

#[test]
fn proto_methods_and_accessors_are_ordinary_properties_and_definitions_bypass_setters() {
    check(
        "let proto={},o={__proto__:proto,__proto__(){return 7;}};Object.getPrototypeOf(o)===proto && o.__proto__()===7",
    );
    check(
        "let proto={},o={__proto__:proto,get __proto__(){return 7;},set __proto__(v){this.v=v;}};o.__proto__=8;Object.getPrototypeOf(o)===proto && o.__proto__===7 && o.v===8",
    );
    check("let parent={set x(v){throw 7;}},o={__proto__:parent,x(){return 8;}};o.x()===8");
}

#[test]
fn methods_and_accessors_remain_nonconstructible_even_after_public_prototype_changes() {
    for expression in [
        "({m(){}}).m",
        "Object.getOwnPropertyDescriptor({get x(){}},'x').get",
        "Object.getOwnPropertyDescriptor({set x(v){}},'x').set",
    ] {
        check(&format!(
            "let fn={expression},count=0,caught=false;Object.defineProperty(fn,'prototype',{{get(){{throw 7;}}}});try{{new fn(count++);}}catch(e){{caught=e instanceof TypeError;}}caught && count===1"
        ));
        assert!(matches!(
            Realm::default().eval(&format!(
                "let fn={expression},bound=fn.bind(null);new bound"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn method_arguments_and_new_target_have_ordinary_non_arrow_scope() {
    check(
        "let o={m(a){a=7;return arguments[0]===7 && arguments.callee===this.m && new.target===undefined;},strict(a){'use strict';a=7;return arguments[0]===1;}};o.m(1) && o.strict(1)",
    );
    check(
        "let o={m(a=()=>arguments[0],b=()=>new.target){return a()===undefined && b()===undefined;},set x(v){v=7;this.observed=arguments[0];}};o.x=1;o.observed===7 && o.m()",
    );
    check(
        "function Outer(){return {get x(){return ()=>new.target;}};}let o=new Outer;o.x()===undefined",
    );
}

#[test]
fn function_to_string_preserves_exact_method_definition_source() {
    check(
        "let o={ m /*name*/ (x) { return x; },get ['x'] /*body*/ () {return 1;},set x(v){}};let d=Object.getOwnPropertyDescriptor(o,'x');o.m.toString()==='m /*name*/ (x) { return x; }' && d.get.toString()===\"get ['x'] /*body*/ () {return 1;}\" && d.set.toString()==='set x(v){}'",
    );
}

#[test]
fn enumerable_getters_observe_live_keys_after_snapshot() {
    check(
        "let o={a:'A',get b(){delete this.c;this.d='D';return 'B';},c:'C'},r=Object.entries(o);r.length===2 && r[0][0]==='a' && r[1][0]==='b' && r[1][1]==='B' && o.d==='D'",
    );
    check(
        "let o={get a(){Object.defineProperty(this,'b',{enumerable:false});return 1;},b:2};Object.values(o).join(',')==='1'",
    );
}

#[test]
fn recursive_accessors_obey_the_host_limit_and_restore_call_state() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            realm
                .eval("let flag=0,o={get x(){return this.x;}}")
                .unwrap();
            assert!(matches!(
                realm.eval("try{o.x;}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            assert_eq!(
                realm.eval("flag===0 && ({m(){return 7;}}).m()===7"),
                Ok(Value::Boolean(true))
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
