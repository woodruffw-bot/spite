//! ArraySpeciesCreate with class construction and private elements (7.3.22, 15.7.14).

use spite_runtime::{Realm, Value};

const CALLS: [(&str, usize); 7] = [
    ("map(v=>v)", 3),
    ("filter(()=>true)", 0),
    ("slice()", 3),
    ("concat()", 0),
    ("flat()", 0),
    ("flatMap(v=>[v])", 0),
    ("splice(0,3)", 3),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn inherited_species_constructs_fresh_array_subclasses_with_private_brands() {
    for (call, length) in CALLS {
        check(&format!(
            "let args,target;class C extends Array{{#initial=this.length;#read(){{return this.#initial;}}constructor(...xs){{super(...xs);args=xs;target=new.target;}}read(){{return this.#read();}}}}let a=new C(1,2,3),b=a.{call};b!==a && b instanceof C && Array.isArray(b) && Object.getPrototypeOf(b)===C.prototype && b.join()==='1,2,3' && b.read()==={length} && a.read()===3 && args.length===1 && args[0]==={length} && target===C"
        ));
    }
}

#[test]
fn base_and_derived_elements_initialize_before_the_species_constructor_body() {
    for (call, length) in CALLS {
        check(&format!(
            "let log='';class B extends Array{{#b=this.#m();#m(){{log+='b';return this.length;}}readB(){{return this.#b;}}}}class C extends B{{#c=this.#m();#m(){{log+='c';return this.readB();}}constructor(n){{log+='s';super(n);log+='e';if(this.#c!==n)throw 7;}}readC(){{return this.#c;}}}}let a=new C(3);a[0]=1;a[1]=2;a[2]=3;log='';let b=a.{call};log==='sbce' && b.readB()==={length} && b.readC()==={length} && b.join()==='1,2,3'"
        ));
    }
}

#[test]
fn default_derived_species_forwards_without_observing_the_array_iterator() {
    for (call, length) in CALLS {
        check(&format!(
            "class C extends Array{{#n=this.length;read(){{return this.#n;}}}}let a=new C(1,2,3);Array.prototype[Symbol.iterator]=function(){{throw 7;}};let b=a.{call};Array.isArray(b) && b instanceof C && b.read()==={length} && b.join()==='1,2,3'"
        ));
    }
}

#[test]
fn custom_class_species_can_produce_ordinary_objects_with_private_elements() {
    for (call, length) in CALLS {
        let writes_length =
            call.starts_with("slice") || call.starts_with("concat") || call.starts_with("splice");
        let final_length = if writes_length { 3 } else { 99 };
        check(&format!(
            "let argc,target;class Result{{length=99;#n;#get(){{return this.#n;}}constructor(n){{this.#n=n;argc=arguments.length;target=new.target;}}read(){{return this.#get();}}}}class C extends Array{{static get [Symbol.species](){{return Result;}}}}let a=new C(1,2,3),b=a.{call};b instanceof Result && !Array.isArray(b) && b[0]===1 && b[1]===2 && b[2]===3 && b.length==={final_length} && b.read()==={length} && argc===1 && target===Result"
        ));
    }
}

#[test]
fn null_species_uses_the_intrinsic_array_without_initializing_subclass_elements() {
    for (call, _) in CALLS {
        check(&format!(
            "let fields=0,species=0;class C extends Array{{#x=fields++;static get [Symbol.species](){{species++;return null;}}}}let a=new C(1,2,3),p=Array.prototype;fields=0;Array=function(){{throw 7;}};let b=a.{call};Object.getPrototypeOf(b)===p && !(b instanceof C) && b.join()==='1,2,3' && fields===0 && species===1"
        ));
    }
}

#[test]
fn returned_objects_receive_derived_private_brands_before_result_definitions() {
    for (call, length) in CALLS {
        check(&format!(
            "let escaped={{}},n;class Base{{constructor(x){{n=x;return escaped;}}}}class Result extends Base{{#x=7;#read(){{return this.#x;}}static read(o){{return o.#read();}}}}let a=[1,2,3];a.constructor={{[Symbol.species]:Result}};let b=a.{call};b===escaped && Result.read(b)===7 && b[0]===1 && b[1]===2 && b[2]===3 && n==={length}"
        ));
        check(&format!(
            "let escaped=Object.preventExtensions({{}});class Base{{constructor(){{return escaped;}}}}class Result extends Base{{#x=7;#read(){{return this.#x;}}static read(o){{return o.#read();}}}}let a=[1,2,3];a.constructor={{[Symbol.species]:Result}};let caught=false;try{{a.{call};}}catch(e){{caught=e instanceof TypeError;}}caught && Result.read(escaped)===7 && !Object.hasOwn(escaped,'0')"
        ));
    }
}

#[test]
fn abrupt_species_fields_preserve_prior_elements_and_prevent_source_visits() {
    for (call, _) in CALLS {
        check(&format!(
            "let escaped,reads=0,calls=0;class Result extends Array{{first=(escaped=this,1);#m(){{return 7;}}#x=(()=>{{throw 9;}})();later=++calls;static inspect(o){{return o.first===1 && o.#m()===7 && !(#x in o) && !Object.hasOwn(o,'later');}}}}let a=[1,2,3];Object.defineProperty(a,'0',{{get(){{reads++;return 1;}}}});a.constructor={{[Symbol.species]:Result}};let caught=false;try{{a.{call};}}catch(e){{caught=e===9;}}caught && Result.inspect(escaped) && reads===0 && calls===0"
        ));
    }
}

#[test]
fn species_results_retain_private_fields_and_method_captures_during_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let result;(function(){let capture={n:7};class C extends Array{#value=capture;#read(){return this.#value.n;}read(){return this.#read();}}result=new C(1,2,3).map(v=>v);})();",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("result.read()===7 && result.join()==='1,2,3'"),
        Ok(Value::Boolean(true))
    );
    realm.eval("result=undefined").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 8);
}
