//! Function intrinsic graph, metadata, and unsupported compilation failures.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn constructor_and_prototype_graph_has_standard_identity_and_descriptors() {
    check(
        "typeof Function==='function' && Function.name==='Function' && Function.length===1 && Object.getPrototypeOf(Function)===Function.prototype && Object.getPrototypeOf(Function.prototype)===Object.prototype && Function.prototype.constructor===Function && Function.constructor===Function && Function.prototype()===undefined",
    );
    check(
        "let p=Object.getOwnPropertyDescriptor(Function,'prototype'),c=Object.getOwnPropertyDescriptor(Function.prototype,'constructor'),g=Object.getOwnPropertyDescriptor(globalThis,'Function');p.value===Function.prototype && !p.writable && !p.enumerable && !p.configurable && c.value===Function && c.writable && !c.enumerable && c.configurable && g.value===Function && g.writable && !g.enumerable && g.configurable",
    );
    for (key, value) in [("name", "'Function'"), ("length", "1")] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Function,'{key}');d.value==={value} && !d.writable && !d.enumerable && d.configurable"
        ));
    }
    check(
        "Function.toString()==='function Function() { [native code] }' && Object.prototype.toString.call(Function)==='[object Function]' && Function.prototype.name==='' && Function.prototype.length===0",
    );
    check(
        "let caught=false;try{new Function.prototype();}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn branding_and_newtarget_validation_use_real_constructor_metadata() {
    check(
        "function f(){}let arrow=()=>0,bound=f.bind(null);f instanceof Function && arrow instanceof Function && bound instanceof Function && Function instanceof Function && !(Function.prototype instanceof Function) && !(Math instanceof Function)",
    );
    check(
        "let o=Reflect.construct(Object,[],Function);typeof o==='object' && Object.getPrototypeOf(o)===Function.prototype && o instanceof Function",
    );
    check(
        "let caught=false;try{Reflect.construct(Function,[],()=>0);}catch(e){caught=e instanceof TypeError;}caught",
    );
    let mut realm = Realm::default();
    let Value::Object(constructor) = realm.eval("Function").unwrap() else {
        panic!("constructor")
    };
    let constructor = realm.inspect_object(&constructor).unwrap();
    assert!(constructor.is_callable() && constructor.is_constructor());
}

#[test]
fn complete_own_reflection_and_integrity_operations_do_not_invoke_accessors() {
    check(
        "let keys=Reflect.ownKeys(Function.prototype);Object.getOwnPropertyNames(Function).join(',')==='length,name,prototype' && keys.length===10 && keys[9]===Symbol.hasInstance && Object.keys(Function.prototype).length===0 && Object.keys(Object.assign({},Function.prototype)).length===0",
    );
    check(
        "let count=0;for(let k in Function.prototype)count++;count===0 && Object.getOwnPropertyDescriptor(Function.prototype,'caller').get===Object.getOwnPropertyDescriptor(Function.prototype,'arguments').get",
    );
    check(
        "Object.freeze(Function);Object.freeze(Function.prototype);Object.isFrozen(Function) && Object.isFrozen(Function.prototype) && Function.prototype.constructor===Function && Function.prototype()===undefined",
    );
    check(
        "let caught=false;Object.freeze(Function.prototype);try{Function.caller;}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let p=Function.prototype;delete p.constructor;!Object.hasOwn(p,'constructor') && p.constructor===Object && Reflect.set(p,'constructor',Function) && p.constructor===Function",
    );
}

#[test]
fn unsupported_dynamic_syntax_skips_handlers_after_argument_effects() {
    for expression in [
        "Function((flag=3,'function* gap(){}'))",
        "new Function((flag=3,'function* gap(){}'))",
        "Function.call(null,(flag=3,'function* gap(){}'))",
        "new (Function.bind(null))((flag=3,'function* gap(){}'))",
        "Reflect.construct(Function,[(flag=3,'function* gap(){}')])",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=5;}}finally{{flag=7;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(3.0)));
    }
}

#[test]
fn intrinsic_constructor_and_prototype_survive_deleted_public_links() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let F=Function,p=Function.prototype;delete p.constructor;delete globalThis.Function;",
        )
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("F.prototype===p && Object.getPrototypeOf(F)===p && F instanceof F && p()===undefined && typeof Function==='undefined'"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("F('return 7')()"), Ok(Value::Number(7.0)));
}
