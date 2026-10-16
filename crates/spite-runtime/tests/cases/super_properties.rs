//! Super Reference Records preserve [[ThisValue]] and deferred property keys.

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn reads_use_the_method_home_prototype_and_calls_use_the_actual_receiver() {
    check(
        "let p={x:3,m(v){return this.x+v;}},h={x:10,read(){return super.x;},call(){return (super.m)(2);}};Object.setPrototypeOf(h,p);let other={x:100,call:h.call};Object.setPrototypeOf(other,{x:999});h.read()===3 && h.call()===12 && other.call()===102 && h.read.call(other)===3",
    );
    check(
        "let p={x:3},h={m(){return super.x;}};Object.setPrototypeOf(h,p);let f=h.m.bind({x:999});f()===3 && (Object.setPrototypeOf(h,{x:7}),f()===7)",
    );
    check(
        "let h={m(){return super.toString();}};h.m()==='[object Object]' && Object.setPrototypeOf(h,{toString(){return this;}}).m()===h",
    );
}

#[test]
fn inherited_accessors_receive_objects_and_strict_primitive_this_values() {
    check(
        "let p={get x(){return this.v;},set x(v){this.v=v;}},h={v:2,m(v){super.x=v;return super.x;}};Object.setPrototypeOf(h,p);let r={v:3};h.m.call(r,7)===7 && r.v===7 && h.v===2 && !Object.hasOwn(p,'v')",
    );
    check(
        "let seen, p={get x(){'use strict';return this;},set x(v){'use strict';seen=[this,v];}},h={m(){'use strict';super.x=4;return super.x;}};Object.setPrototypeOf(h,p);h.m.call(7)===7 && seen[0]===7 && seen[1]===4 && h.m.call(undefined)===undefined && seen[0]===undefined",
    );
    check(
        "let p={get x(){return this.v;},set x(v){this.v=v;}},h={v:3,get x(){return super.x+1;},set x(v){super.x=v+1;}};Object.setPrototypeOf(h,p);let r={v:10};Reflect.get(h,'x',r)===11 && Reflect.set(h,'x',5,r) && r.v===6 && h.v===3",
    );
}

#[test]
fn data_writes_update_the_receiver_without_mutating_the_super_base() {
    check(
        "let p={x:1},h={x:2,m(v){super.x=v;return super.x;}};Object.setPrototypeOf(h,p);let r={};h.m(7)===1 && h.x===7 && h.m.call(r,9)===1 && r.x===9 && p.x===1",
    );
    check(
        "let calls=0,p={x:1},h={m(){super.x=7;return 3;},strict(){'use strict';super.x=7;}},r={set x(v){calls++;}};Object.setPrototypeOf(h,p);let caught=false;try{h.strict.call(r);}catch(e){caught=e instanceof TypeError;}caught && h.m.call(r)===3 && calls===0 && p.x===1",
    );
    check(
        "let p={x:1},h={m(){super.x=7;},strict(){'use strict';super.x=7;}};Object.setPrototypeOf(h,p);Object.freeze(h);let caught=false;try{h.strict();}catch(e){caught=e instanceof TypeError;}h.m();caught && p.x===1 && !Object.hasOwn(h,'x')",
    );
}

#[test]
fn non_writable_base_and_primitive_receivers_follow_reference_strictness() {
    check(
        "let p={},h={m(){return super.x=7;},strict(){'use strict';super.x=7;}};Object.defineProperty(p,'x',{value:1});Object.setPrototypeOf(h,p);let caught=false;try{h.strict();}catch(e){caught=e instanceof TypeError;}caught && h.m()===7 && p.x===1 && !Object.hasOwn(h,'x')",
    );
    check(
        "let h={m(){'use strict';return super.x=7;}};Object.setPrototypeOf(h,{});let caught=false;try{h.m.call(1);}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let h={m(){'use strict';return super.x=7;}},p={};Object.defineProperty(p,'x',{get(){return 1;}});Object.setPrototypeOf(h,p);let caught=false;try{h.m();}catch(e){caught=e instanceof TypeError;}caught && !Object.hasOwn(h,'x')",
    );
}

#[test]
fn computed_expression_precedes_base_lookup_and_conversion_follows_it() {
    // Edition 17 13.3.7.1/3 and 6.2.5.5: capture the prototype after the name
    // expression, then convert the name when GetValue executes.
    check(
        "let log='',a={x:1},b={x:2},c={x:3},k={toString(){log+='K';Object.setPrototypeOf(h,c);return 'x';}},h={m(){return super[(log+='E',Object.setPrototypeOf(h,b),k)];}};Object.setPrototypeOf(h,a);h.m()===2 && log==='EK' && Object.getPrototypeOf(h)===c",
    );
    check(
        "let log='',key={[Symbol.toPrimitive](hint){log+=hint;return Symbol.iterator;}},p={[Symbol.iterator]:7},h={m(){return super[key];}};Object.setPrototypeOf(h,p);h.m()===7 && log==='string'",
    );
    check(
        "let log='',h={m(){return super[(log+='E',{toString(){log+='K';return 'x';}})];}};Object.setPrototypeOf(h,null);let caught=false;try{h.m();}catch(e){caught=e instanceof TypeError;}caught && log==='E'",
    );
    check(
        "let p={['\\uD800']:7},h={m(){return super['\\uD800'];}};Object.setPrototypeOf(h,p);h.m()===7",
    );
}

#[test]
fn simple_assignment_defers_conversion_until_after_rhs_but_retains_base() {
    check(
        "let log='',seen='',a={set x(v){seen='A'+v;}},b={set x(v){seen='B'+v;}},k={toString(){log+='K';return 'x';}},h={m(){return super[(log+='E',k)]=(log+='R',Object.setPrototypeOf(h,b),7);}};Object.setPrototypeOf(h,a);h.m()===7 && log==='ERK' && seen==='A7'",
    );
    check(
        "let log='',h={m(){super[{toString(){log+='K';return 'x';}}]=(log+='R',7);}};Object.setPrototypeOf(h,null);let caught=false;try{h.m();}catch(e){caught=e instanceof TypeError;}caught && log==='R'",
    );
    check(
        "let log='',h={m(){super[{toString(){log+='K';return 'x';}}]=(log+='R',function(){throw 8;}());}};let caught=false;try{h.m();}catch(e){caught=e===8;}caught && log==='R'",
    );
}

#[test]
fn compound_logical_and_update_operations_convert_the_name_once() {
    check(
        "let log='',p={get x(){log+='G';return 2;},set x(v){log+='S'+v;}},k={toString(){log+='K';return 'x';}},h={m(){return super[k]+=(log+='R',3);}};Object.setPrototypeOf(h,p);h.m()===5 && log==='KGRS5'",
    );
    check(
        "let log='',p={get x(){log+='G';return 2n;},set x(v){log+='S'+v;}},k={toString(){log+='K';return 'x';}},h={m(){return super[k]++;}};Object.setPrototypeOf(h,p);h.m()===2n && log==='KGS3'",
    );
    check(
        "let log='',p={get x(){log+='G';return 2;},set x(v){log+='S';}},h={m(){return super[{toString(){log+='K';return 'x';}}]||=(log+='R',3);}};Object.setPrototypeOf(h,p);h.m()===2 && log==='KG'",
    );
}

#[test]
fn optional_calls_and_tagged_templates_keep_the_super_receiver() {
    check(
        "let log='',p={m(v){return this.x+v;}},h={x:4,m(){return super.m?.(3);},missing(){return super.no?.(log+='A');}};Object.setPrototypeOf(h,p);h.m()===7 && h.missing()===undefined && log===''",
    );
    check(
        "let p={tag(s,v){return this.x+s[0]+v;}},h={x:'H',m(){return super.tag`T${7}`;}};Object.setPrototypeOf(h,p);h.m()==='HT7'",
    );
    check(
        "let p={m(){'use strict';return this;}},h={m(){return (0,super.m)();}};Object.setPrototypeOf(h,p);h.m()===undefined",
    );
    check(
        "function F(v){this.x=v;}let p={F},h={m(){return new super.F(7);}};Object.setPrototypeOf(h,p);h.m().x===7",
    );
}

#[test]
fn deletion_throws_before_base_or_name_conversion_without_reading_the_property() {
    // 13.5.1.2: Super Reference Records cannot be deleted, even when null is
    // the base; computed-name evaluation still precedes the error.
    for base in ["{get x(){log+='G';}}", "null"] {
        check(&format!(
            "let log='',h={{m(){{delete (super[(log+='E',{{toString(){{log+='K';return 'x';}}}})]);}}}};Object.setPrototypeOf(h,{base});let caught=false;try{{h.m();}}catch(e){{caught=e instanceof ReferenceError;}}caught && log==='E'"
        ));
    }
    check(
        "let h={m(){'use strict';delete super.x;}},caught=false;try{h.m();}catch(e){caught=e instanceof ReferenceError;}caught",
    );
}

#[test]
fn destructuring_and_loop_assignment_heads_use_super_references() {
    check(
        "let log='',p={set x(v){log+=v;}},h={m(){[super.x,super.x]=[1,2];({x:super.x}={x:3});for(super.x of [4,5]){}for(super.x in {a:1}){}}};Object.setPrototypeOf(h,p);h.m();log==='12345a'",
    );
    check(
        "let p={x:1},h={m(){super.x??=7;return super.x;}};Object.setPrototypeOf(h,p);h.m()===1 && !Object.hasOwn(h,'x')",
    );
}

#[test]
fn arrows_eval_and_default_parameters_inherit_method_home_and_this() {
    check(
        "let p={x:3},h={x:9,m(v=super.x){return ()=>super.x+this.x+v;},e(){return eval('(()=>super.x+this.x)()');}};Object.setPrototypeOf(h,p);let f=h.m();f()===15 && h.e()===12",
    );
    check(
        "let p={x:2},h={m(){return eval('()=>super.x');}};Object.setPrototypeOf(h,p);let f=h.m();Object.setPrototypeOf(h,{x:7});f()===7",
    );
    check(
        "let p={m(){return this.x;}},h={x:7,e(){return eval('super.m()');}};Object.setPrototypeOf(h,p);h.e()===7",
    );
    check(
        "let h={m(){return function(){return eval('super.x');};}},caught=false;try{h.m()();}catch(e){caught=e instanceof SyntaxError;}caught",
    );
    check(
        "var x=1;let h={m(){let x=2;return super.eval('x');}};Object.setPrototypeOf(h,{eval});h.m()===1",
    );
}

#[test]
fn captured_method_homes_survive_collection_and_mutation() {
    let mut realm = Realm::default();
    realm.eval("let arrow=(function(){let p={x:7},h={x:9,m(){return ()=>super.x+this.x;}};Object.setPrototypeOf(h,p);return h.m();})();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("arrow()"), Ok(Value::Number(16.0)));
    assert_eq!(realm.eval("let f=(function(){let h={m(){return eval('()=>super.x');}};Object.setPrototypeOf(h,{x:8});return h.m();})();"), Ok(Value::Undefined));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(8.0)));
}
