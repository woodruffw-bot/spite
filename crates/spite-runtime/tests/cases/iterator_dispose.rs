//! %IteratorPrototype% [ @@dispose ] (27.1.3.3).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn return_is_read_once_called_without_arguments_and_its_result_is_ignored() {
    check(
        "const dispose=Iterator.prototype[Symbol.dispose];for(const result of [undefined,null,false,7,'text',0n,Symbol(),{get then(){throw 1}},new Proxy({},{get(){throw 2}})]){let gets=0,calls=0;const receiver={get return(){gets++;return function(){'use strict';if(this!==receiver||arguments.length!==0)throw 3;calls++;return result}}};if(dispose.call(receiver,99)!==undefined||gets!==1||calls!==1)throw 4;}true",
    );
    check(
        "const dispose=Iterator.prototype[Symbol.dispose],receiver={return(){calls++}};let calls=0;dispose.call(receiver);receiver.return=function(){calls+=10};dispose.call(receiver);calls===11",
    );
}

#[test]
fn nullish_methods_are_optional_but_noncallable_methods_and_nullish_receivers_throw() {
    check(
        "const dispose=Iterator.prototype[Symbol.dispose];for(const receiver of [{},{return:null},{return:undefined}])if(dispose.call(receiver)!==undefined)throw 1;let caught=0;for(const receiver of [null,undefined,{return:false},{return:1},{return:Symbol()},{return:{}}]){try{dispose.call(receiver)}catch(e){if(e instanceof TypeError)caught++}}let construct=false;try{Reflect.construct(dispose,[])}catch(e){construct=e instanceof TypeError}caught===6&&construct&&!Object.hasOwn(dispose,'prototype')",
    );
    check(
        "const dispose=Iterator.prototype[Symbol.dispose],marker={};let caught=0;try{dispose.call({get return(){throw marker}})}catch(e){if(e===marker)caught++}try{dispose.call({return(){throw marker}})}catch(e){if(e===marker)caught++}caught===2",
    );
}

#[test]
fn inherited_methods_and_accessors_preserve_primitive_receivers() {
    check(
        "const dispose=Iterator.prototype[Symbol.dispose];let gets=0,calls=0;Object.defineProperty(Number.prototype,'return',{get(){'use strict';if(this!==7)throw 1;gets++;return function(){'use strict';if(this!==7||arguments.length!==0)throw 2;calls++;return 8}},configurable:true});dispose.call(7)===undefined&&gets===1&&calls===1",
    );
    check(
        "const dispose=Iterator.prototype[Symbol.dispose];let seen;String.prototype.return=function(){'use strict';seen=this;return 9};dispose.call('text')===undefined&&seen==='text'",
    );
}

#[test]
fn iterator_helpers_close_after_collection_and_remain_completed() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let closed=0,dispose=Iterator.prototype[Symbol.dispose],source={next(){return{done:false,value:1}},return(){closed++;return{done:true,value:99}}},helper=Iterator.from(source).map(x=>x+1);helper.next().value===2"), Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("helper[Symbol.dispose]()===undefined&&closed===1&&helper.next().done&&dispose.call(helper)===undefined&&closed===1"), Ok(Value::Boolean(true)));
}
