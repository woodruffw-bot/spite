use super::*;

#[test]
fn intrinsic_has_instance_is_fixed_callable_and_ordinary() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let fp=Object.getPrototypeOf(function(){}),method=fp[hasInstance],d=Object.getOwnPropertyDescriptor(fp,hasInstance);typeof method==='function' && d.value===method && !d.writable && !d.enumerable && !d.configurable",
    );
    check(
        &mut realm,
        "method.name==='[Symbol.hasInstance]' && method.length===1 && !Object.hasOwn(method,'prototype') && !delete fp[hasInstance]",
    );
    check(
        &mut realm,
        "function F(){}let o=new F;method.call(F,o) && !method.call(F,1) && !method.call({},o) && !method.call(null,o) && !method.call(s,o)",
    );
    assert!(matches!(
        realm.eval("new method"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval("Object.defineProperty(fp,hasInstance,{value:()=>true})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        &mut realm,
        "F[hasInstance]=()=>true;F[hasInstance]===method && !(1 instanceof F)",
    );
    check(
        &mut realm,
        "Object.defineProperty(F,hasInstance,{value:()=>true,configurable:true});1 instanceof F && !method.call(F,1)",
    );
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "fp[hasInstance]===method && method.call(F,o)");
}

#[test]
fn custom_hooks_run_before_callability_and_convert_results_without_coercion() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let seen,arg,count=0,target={[hasInstance]:function(value){seen=this;arg=value;count=arguments.length;return {};}};(s instanceof target) && seen===target && arg===s && count===1",
    );
    check(
        &mut realm,
        "target[hasInstance]=()=>({valueOf:()=>{throw 7;},[convert]:()=>{throw 8;}});0 instanceof target",
    );
    for result in ["false", "0", "NaN", "''", "null", "undefined", "0n"] {
        check(
            &mut realm,
            &format!("target[hasInstance]=()=>{result};!(1 instanceof target)"),
        );
    }
    for result in ["true", "1", "'x'", "s", "1n"] {
        check(
            &mut realm,
            &format!("target[hasInstance]=()=>{result};1 instanceof target"),
        );
    }
    check(
        &mut realm,
        "let p={[hasInstance]:function(v){return this===child && v===7;}},child=Object.create(p);7 instanceof child",
    );
}

#[test]
fn hook_lookup_and_abrupt_completions_precede_prototype_access() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',target={};Object.defineProperty(target,hasInstance,{get:function(){log+=this===target?'g':'bad';return function(v){log+='c';return v===3;};}});Object.defineProperty(target,'prototype',{get:()=>{throw 8;}});3 instanceof target && log==='gc'",
    );
    assert_eq!(realm.eval("let abrupt={};Object.defineProperty(abrupt,hasInstance,{get:()=>{throw 7;}});1 instanceof abrupt"),Err(Error::Thrown(Value::Number(7.0))));
    assert_eq!(
        realm.eval("1 instanceof {[hasInstance]:()=>{throw 9;}}"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    for invalid in ["false", "0", "''", "s", "{}"] {
        assert!(
            matches!(
                realm.eval(&format!("1 instanceof {{[hasInstance]:{invalid}}}")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{invalid}"
        );
    }
    for absent in ["null", "undefined"] {
        let mut realm = realm_with_symbols();
        check(
            &mut realm,
            &format!(
                "function F(){{}}Object.defineProperty(F,hasInstance,{{value:{absent}}});new F instanceof F && !(1 instanceof F)"
            ),
        );
        assert!(matches!(
            realm.eval(&format!("1 instanceof {{[hasInstance]:{absent}}}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn bound_functions_delegate_to_live_target_hooks_even_for_primitive_values() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "function F(){}let B=F.bind(null),C=B.bind(null),seen;Object.defineProperty(F,hasInstance,{value:function(v){seen=this;return v===s;},configurable:true});s instanceof C && seen===F && !(1 instanceof C)",
    );
    check(
        &mut realm,
        "Object.defineProperty(F,hasInstance,{value:()=>true});1 instanceof B",
    );
    check(
        &mut realm,
        "Object.defineProperty(B,hasInstance,{value:()=>false});!(1 instanceof C)",
    );
    check(
        &mut realm,
        "let method=Object.getPrototypeOf(F)[hasInstance];method.call(B,s)",
    );
    // Calling the intrinsic on B ignores B's custom hook and delegates to F.
    check(
        &mut realm,
        "Object.defineProperty(C,hasInstance,{value:()=>true});s instanceof C",
    );
}

#[test]
fn recursive_instance_hooks_are_bounded_and_host_aborts_skip_handlers() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        for setup in [
            "let o={[hasInstance]:()=>1 instanceof o}",
            "let o={};Object.defineProperty(o,hasInstance,{get:()=>1 instanceof o})",
            "function F(){}let o=F.bind(null);Object.defineProperty(F,hasInstance,{value:()=>1 instanceof o})",
        ] {
            let mut realm=realm_with_symbols();
            realm.eval(setup).unwrap();
            realm.eval("let flag=0").unwrap();
            assert!(matches!(realm.eval("try{1 instanceof o;}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})),"{setup}");
            check(&mut realm,"flag===0 && 1 instanceof {[hasInstance]:()=>true}");
        }
    }).unwrap().join().unwrap();
}

#[test]
fn implicit_instance_calls_respect_the_argument_limit() {
    for setup in ["function F(){}", "let F={[hasInstance]:()=>true}"] {
        let mut realm = realm_with_symbols();
        realm.eval(setup).unwrap();
        realm.eval("let flag=0").unwrap();
        realm.limits.max_arguments = Some(0);
        assert!(matches!(
            realm.eval("try{1 instanceof F;}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        check(&mut realm, "flag===0");
    }
}
