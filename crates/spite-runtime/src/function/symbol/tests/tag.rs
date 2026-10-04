use super::*;

#[test]
fn string_tags_override_internal_brands_and_preserve_utf16_units() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let show=Object.prototype.toString;show.call({[tag]:'Custom'})==='[object Custom]' && show.call({[tag]:''})==='[object ]' && show.call({[tag]:'\\uD800\\u0000\\uDC00'})==='[object \\uD800\\u0000\\uDC00]'",
    );
    for (index, value) in [
        "{}",
        "[]",
        "function(){}",
        "new Error",
        "Object(true)",
        "Object(1)",
        "Object('x')",
        "(function(){return arguments;})()",
    ]
    .into_iter()
    .enumerate()
    {
        check(
            &mut realm,
            &format!(
                "let item{index}={value};item{index}[tag]='Other';show.call(item{index})==='[object Other]'"
            ),
        );
    }
    check(
        &mut realm,
        "let p={[tag]:'Inherited'},o=Object.create(p);show.call(o)==='[object Inherited]' && (p[tag]='Changed',show.call(o)==='[object Changed]')",
    );
}

#[test]
fn nonstring_tags_use_builtin_brands_without_coercion() {
    for (value, brand) in [
        ("{}", "Object"),
        ("[]", "Array"),
        ("function(){}", "Function"),
        ("new Error", "Error"),
        ("Object(true)", "Boolean"),
        ("Object(1)", "Number"),
        ("Object('x')", "String"),
        ("(function(){return arguments;})()", "Arguments"),
    ] {
        for tag_value in [
            "undefined",
            "null",
            "true",
            "1",
            "1n",
            "s",
            "{[convert]:()=>{throw 7;}}",
            "()=>{throw 8;}",
        ] {
            check(
                &mut realm_with_symbols(),
                &format!(
                    "let o={value};o[tag]={tag_value};Object.prototype.toString.call(o)==='[object {brand}]'"
                ),
            );
        }
    }
}

#[test]
fn tag_getters_receive_objects_including_fresh_primitive_wrappers() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let show=Object.prototype.toString,p={},o=Object.create(p),seen;Object.defineProperty(p,tag,{get:function(){'use strict';seen=this;return 'Custom';}});show.call(o)==='[object Custom]' && seen===o",
    );
    check(
        &mut realm,
        "let first,second;Object.defineProperty(Number.prototype,tag,{get:function(){'use strict';if(first===undefined){first=this;}else{second=this;}return this.valueOf()===2?'Two':'Other';}});show.call(2)==='[object Two]' && show.call(2)==='[object Two]' && first!==second && typeof first==='object' && first.valueOf()===2 && second.valueOf()===2",
    );
    check(
        &mut realm,
        "Object.defineProperty(String.prototype,tag,{get:function(){'use strict';return typeof this==='object' && this[0]==='x' && this.length===1?'Text':'Wrong';}});show.call('x')==='[object Text]'",
    );
    assert_eq!(
        realm.eval(
            "let abrupt={};Object.defineProperty(abrupt,tag,{get:()=>{throw s;}});show.call(abrupt)"
        ),
        Err(Error::Thrown(realm.eval("s").unwrap()))
    );
}

#[test]
fn nullish_tags_skip_lookup_and_array_fallback_observes_custom_tags() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let show=Object.prototype.toString;let a=[];a.join=null;a[tag]='Special';a.toString()==='[object Special]'",
    );
    check(
        &mut realm,
        "Object.defineProperty(Object.prototype,tag,{get:()=>{throw 7;}});show.call(null)==='[object Null]' && show.call(undefined)==='[object Undefined]' && show.call(1n)==='[object BigInt]'",
    );
}

#[test]
fn tag_recursion_and_output_are_bounded_before_allocation() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let mut realm=realm_with_symbols();
        realm.eval("let o={},flag=0;Object.defineProperty(o,tag,{get:()=>Object.prototype.toString.call(o)})").unwrap();
        assert!(matches!(realm.eval("try{Object.prototype.toString.call(o);}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
        check(&mut realm,"flag===0");
    }).unwrap().join().unwrap();
    let mut realm = realm_with_symbols();
    realm.eval("let o={[tag]:'xxxx'},flag=0").unwrap();
    realm.limits.max_string_units = Some(12);
    assert!(matches!(
        realm.eval("try{Object.prototype.toString.call(o);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "flag===0");
    realm.limits.max_string_units = Some(13);
    assert_eq!(
        realm.eval("Object.prototype.toString.call(o)"),
        Ok(Value::String(JsString::from("[object xxxx]")))
    );
}
