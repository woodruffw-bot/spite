use super::*;

#[test]
fn conversion_hooks_receive_exact_hints_original_receivers_and_one_argument() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',o={[convert]:function(hint){log+=hint+',';return this===o && arguments.length===1 ? 7 : 0;}};Number(o)===7 && String(o)==='7' && o+1===8 && o==7 && o<8 && log==='number,string,default,default,number,'",
    );
    check(
        &mut realm,
        "let p={[convert]:function(hint){return this.x;}};let child={__proto__:p,x:9};Number(child)===9",
    );
    check(
        &mut realm,
        "let symbolResult={[convert]:()=>s},target={[s]:3};target[symbolResult]===3 && symbolResult==s && s==symbolResult",
    );
    assert!(matches!(
        realm.eval("String(symbolResult)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn absent_hooks_fall_back_but_noncallable_and_object_results_throw() {
    for absent in ["null", "undefined"] {
        let mut realm = realm_with_symbols();
        check(
            &mut realm,
            &format!(
                "let log='',o={{[convert]:{absent},valueOf:()=>{{log+='v';return 3;}},toString:()=>{{log+='s';return '4';}}}};Number(o)===3 && String(o)==='4' && log==='vs'"
            ),
        );
    }
    for invalid in ["false", "0", "''", "s", "{}"] {
        let mut realm = realm_with_symbols();
        realm
            .eval(&format!(
                "let called=false,o={{[convert]:{invalid},valueOf:()=>{{called=true;return 1;}}}};"
            ))
            .unwrap();
        assert!(
            matches!(
                realm.eval("Number(o)"),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{invalid}"
        );
        check(&mut realm, "!called");
    }
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let fallback=false,o={[convert]:()=>({}),valueOf:()=>{fallback=true;return 1;}};let caught=false;try{Number(o);}catch(e){caught=e instanceof TypeError;}caught && !fallback",
    );
}

#[test]
fn hook_getters_and_calls_preserve_abrupt_completions_and_lookup_order() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',p={},o=Object.create(p);Object.defineProperty(p,convert,{get:function(){log+=this===o?'g':'wrong';return function(hint){log+=hint;return this===o?2:0;};}});Object.defineProperty(o,'valueOf',{get:()=>{throw 8;}});+o===2 && log==='gnumber'",
    );
    assert_eq!(realm.eval("let abrupt={};Object.defineProperty(abrupt,convert,{get:()=>{throw s;}});Number(abrupt)"), Err(Error::Thrown(realm.eval("s").unwrap())));
    assert_eq!(
        realm.eval("Number({[convert]:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check(
        &mut realm,
        "let fresh={[convert]:()=>1};Number(fresh)===1 && (fresh[convert]=()=>2,Number(fresh)===2)",
    );
}

#[test]
fn recursive_conversion_hooks_abort_within_the_supported_stack_and_skip_handlers() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for setup in [
                "let o={[convert]:()=>Number(o)}",
                "let o={};Object.defineProperty(o,convert,{get:()=>Number(o)})",
                "let o={[convert]:()=>({})[o]}",
            ] {
                let mut realm = realm_with_symbols();
                realm.eval(setup).unwrap();
                realm.eval("let flag=0").unwrap();
                assert!(
                    matches!(
                        realm.eval("try{Number(o);}catch{flag=1;}finally{flag=2;}"),
                        Err(Error::Limit { .. })
                    ),
                    "{setup}"
                );
                check(&mut realm, "flag===0 && Number({[convert]:()=>3})===3");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
