//! Generic Promise.prototype.catch dispatch (27.2.5.1).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn catch_reads_then_once_and_forwards_exact_arguments_receiver_and_result() {
    check(
        "let calls=0,gets=0,marker={},result={},receiver={get then(){gets++;return function(a,b){'use strict';if(this!==receiver||arguments.length!==2||a!==undefined||b!==marker)throw 1;calls++;return result}}};Promise.prototype.catch.call(receiver,marker,99)===result&&gets===1&&calls===1",
    );
    check(
        "let marker=new Proxy({},{get(){throw 1}}),receiver={then(a,b){if(a!==undefined||b!==marker)throw 2;return 7}};Promise.prototype.catch.call(receiver,marker)===7",
    );
}

#[test]
fn primitive_getters_and_strict_then_methods_receive_the_original_value() {
    check(
        "let gets=0,calls=0;Object.defineProperty(Number.prototype,'then',{get(){'use strict';if(this!==7)throw 1;gets++;return function(a,b){'use strict';if(this!==7||arguments.length!==2||a!==undefined||b!==undefined)throw 2;calls++;return 9}},configurable:true});Promise.prototype.catch.call(7)===9&&gets===1&&calls===1",
    );
    check(
        "let symbol=Symbol(),seen;Symbol.prototype.then=function(){'use strict';seen=this;return 1};Promise.prototype.catch.call(symbol)===1&&seen===symbol",
    );
}

#[test]
fn missing_and_noncallable_then_values_throw_and_abrupt_completions_retain_identity() {
    check(
        "let caught=0;for(const receiver of [undefined,null,{}, {then:null},{then:1},{then:Symbol()},{then:{}}]){try{Promise.prototype.catch.call(receiver)}catch(e){if(e instanceof TypeError)caught++}}caught===7",
    );
    check(
        "let marker={},caught=0;try{Promise.prototype.catch.call({get then(){throw marker}})}catch(e){if(e===marker)caught++}try{Promise.prototype.catch.call({then(){throw marker}})}catch(e){if(e===marker)caught++}caught===2",
    );
}

#[test]
fn proxy_get_and_apply_hooks_follow_generic_invoke_semantics() {
    check(
        "let log='',receiver,method=new Proxy(function(){},{apply(t,self,args){log+='a';if(self!==receiver||args.length!==2||args[0]!==undefined||args[1]!==3)throw 1;return 8}});receiver=new Proxy({},{get(t,p,self){if(p!=='then'||self!==receiver)throw 2;log+='g';return method}});Promise.prototype.catch.call(receiver,3)===8&&log==='ga'",
    );
}
