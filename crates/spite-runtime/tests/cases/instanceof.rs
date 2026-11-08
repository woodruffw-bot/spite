//! Standard instanceof behavior for the exposed function and object kinds.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn truth(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prototype_identity_and_ancestry_determine_ordinary_instances() {
    for source in [
        "function F(){}let o=new F;o instanceof F",
        "function F(){}let o={__proto__:F.prototype};o instanceof F",
        "function F(){}let o={__proto__:{__proto__:F.prototype}};o instanceof F",
        "function F(){}!(F.prototype instanceof F)",
        "function F(){}!({} instanceof F)",
        "function F(){}let o=new F;F.prototype={};!(o instanceof F)",
        "function F(){}function G(){}let o=new F;G.prototype=F.prototype;o instanceof G",
        "function F(){}!((new F instanceof F) instanceof F)",
        "function F(){}let o={valueOf:()=>{throw 7;}};!(o instanceof F)",
    ] {
        truth(source);
    }
    for primitive in ["undefined", "null", "false", "1", "1n", "'x'"] {
        truth(&format!(
            "function F(){{}}F.prototype=null;!({primitive} instanceof F)"
        ));
    }
    assert!(matches!(
        Realm::default().eval("function F(){}F.prototype=null;({}) instanceof F"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn arrow_and_bound_checks_use_callability_not_constructibility() {
    truth("let F=()=>1,p={};F.prototype=p;({__proto__:p}) instanceof F");
    truth("function F(){}let B=F.bind(null).bind({});new F instanceof B && new B instanceof F");
    truth("function F(){}let B=F.bind(null);B.prototype={};new F instanceof B");
    truth("let F=()=>1,p={};F.prototype=p;let B=F.bind(null);({__proto__:p}) instanceof B");
    truth("!(1 instanceof (()=>1)) && !(1 instanceof (()=>1).bind(null))");
    for source in [
        "({}) instanceof (()=>1)",
        "({}) instanceof (()=>1).bind(null)",
        "({}) instanceof ({}).toString",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    // 7.3.21 and 13.10.2: the intrinsic skips prototype access for a
    // primitive, while bound delegation still performs the live target lookup.
    truth(
        "let pair=Proxy.revocable(function(){},{}),h=Function.prototype[Symbol.hasInstance];pair.revoke();let caught=false;try{0 instanceof pair.proxy}catch(e){caught=e instanceof TypeError}caught&&h.call(pair.proxy,0)===false",
    );
    truth(
        "let pair=Proxy.revocable(function(){},{}),bound=pair.proxy.bind(null),h=Function.prototype[Symbol.hasInstance];pair.revoke();let caught=false;try{h.call(bound,0)}catch(e){caught=e instanceof TypeError}caught",
    );
    truth(
        "let log='',pair=Proxy.revocable(function(){},{get(t,k,r){if(k===Symbol.hasInstance){log+='h';pair.revoke();return null}if(k==='prototype')throw 7;return Reflect.get(t,k,r)}});(0 instanceof pair.proxy)===false&&log==='h'",
    );
    truth(
        "let log='',pair=Proxy.revocable(function(){},{get(t,k,r){if(k===Symbol.hasInstance){log+='h';pair.revoke();return null}if(k==='prototype')throw 7;return Reflect.get(t,k,r)}}),bound=pair.proxy.bind(null),h=Function.prototype[Symbol.hasInstance];h.call(bound,0)===false&&log==='h'",
    );
}

#[test]
fn inherited_intrinsic_has_instance_handles_non_callable_receivers() {
    // The fixed Function.prototype hook returns false for non-callable this.
    truth("let target={__proto__:({}).toString};!({} instanceof target) && !(1 instanceof target)");
    for source in [
        "1 instanceof 2",
        "1 instanceof null",
        "1 instanceof {}",
        "({}) instanceof {prototype:{}}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
}

#[test]
fn both_operands_are_evaluated_before_the_right_operand_is_checked() {
    truth(
        "let order='';let caught=false;try{(order+='l',1) instanceof (order+='r',2);}catch{caught=true;}caught && order==='lr'",
    );
    truth(
        "let effect=0;function left(){throw 7;}try{left() instanceof (effect=1);}catch{}effect===0",
    );
    truth(
        "let effect=0;function F(){}let p=F.prototype;let o={__proto__:p};o instanceof (effect=7,F) && effect===7",
    );
}
