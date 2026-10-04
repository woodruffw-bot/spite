//! Species-dependent map/filter preserve live visits and ordered partial effects.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const METHODS: [&str; 2] = ["map", "filter"];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn map_preserves_holes_and_filter_packs_selected_original_values() {
    check(
        "let a=[1,,3],b=a.map((v,k,o)=>v+k);b!==a && b.length===3 && b[0]===1 && !(1 in b) && b[2]===5",
    );
    check(
        "let o={},a=[undefined,,o,0,4],b=a.filter(v=>v!==0);b.length===3 && Object.hasOwn(b,'0') && b[0]===undefined && b[1]===o && b[2]===4",
    );
    check(
        "let a=[1,2],b=a.filter((v,k)=>{a[k]=9;return true;});b.join()==='1,2' && a.join()==='9,9'",
    );
    check("let a=[1],b=a.filter(()=>({valueOf:()=>{throw 7;}}));b[0]===1");
    check(
        "let a=[,,],b=a.map(()=>{throw 7;}),c=a.filter(()=>{throw 8;});b.length===2 && !(0 in b) && !(1 in b) && c.length===0",
    );
}

#[test]
fn generic_receivers_are_boxed_and_never_consult_constructor() {
    for method in METHODS {
        check(&format!(
            "let o={{0:7,length:'1.9',get constructor(){{throw 7;}}}},b=Array.prototype.{method}.call(o,v=>v);Array.isArray(b) && b.length===1 && b[0]===7"
        ));
        check(&format!(
            "let receiver;let b=Array.prototype.{method}.call('💩',(v,k,o)=>{{receiver=o;return v;}});b.length===2 && b[0]==='\\uD83D' && b[1]==='\\uDCA9' && receiver.valueOf()==='💩'"
        ));
        check(&format!(
            "Array.prototype.{method}.call(7,()=>{{throw 7;}}).length===0"
        ));
        check(&format!(
            "let a=Object.create(Array.prototype);a[0]=7;Object.defineProperty(a,'length',{{value:1}});Object.defineProperty(a,'constructor',{{get:()=>{{throw 7;}}}});Array.prototype.{method}.call(a,v=>v)[0]===7"
        ));
    }
}

#[test]
fn visits_are_live_inherited_and_use_snapshotted_length() {
    for method in METHODS {
        check(&format!(
            "let a=[1,,3],log='';Array.prototype[1]=2;let b=a.{method}((v,k,o)=>{{if(o!==a)throw 7;log+=k+':'+v+';';if(k===0){{a[2]=9;a[3]=4;}}return v;}});log==='0:1;1:2;2:9;' && b.join()==='1,2,9'"
        ));
        check(&format!(
            "let a=[1,2,3],log='';a.{method}((v,k)=>{{log+=v;if(k===0){{delete a[1];a.length=0;}}return true;}});log==='1'"
        ));
        check(&format!(
            "let a=[1,,3],log='';a.{method}((v,k)=>{{log+=v;if(k===0)a[1]=2;return true;}});log==='123'"
        ));
    }
}

#[test]
fn callbacks_receive_three_arguments_and_the_specified_this_arg() {
    for method in METHODS {
        check(&format!(
            "let a=[7],receiver={{}},valid=false;a.{method}(function(v,k,o){{'use strict';valid=arguments.length===3 && v===7 && k===0 && o===a && this===receiver;return true;}},receiver);valid"
        ));
        check(&format!(
            "let valid=false;[7].{method}(function(){{'use strict';valid=this===undefined;return true;}});valid"
        ));
        check(&format!(
            "let valid=false;[7].{method}(function(){{valid=this===globalThis;return true;}});valid"
        ));
        check(&format!(
            "let valid=false;[7].{method}(function(){{'use strict';valid=this===false;return true;}},false);valid"
        ));
    }
}

#[test]
fn callback_validation_precedes_species_and_follows_length_conversion() {
    for method in METHODS {
        for callback in ["undefined", "null", "7", "{}"] {
            check(&format!(
                "let hits=0,a=[];Object.defineProperty(a,'constructor',{{get:()=>{{hits++;throw 7;}}}});let caught=false;try{{a.{method}({callback});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
        assert_eq!(
            Realm::default().eval(&format!(
                "Array.prototype.{method}.call({{get length(){{throw 7;}}}},null)"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        for receiver in ["null", "undefined", "{length:1n}"] {
            assert!(matches!(
                Realm::default().eval(&format!(
                    "Array.prototype.{method}.call({receiver},()=>true)"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        }
    }
}

#[test]
fn species_lookup_and_construction_precede_even_empty_traversals() {
    for method in METHODS {
        let length = if method == "map" { 2 } else { 0 };
        check(&format!(
            "let log='',a=[7,8],receiver,n,argc,target;function C(x){{log+='C';n=x;argc=arguments.length;target=new.target;}}let holder={{get [Symbol.species](){{receiver=this;log+='s';return C;}}}};Object.defineProperty(a,'constructor',{{get:()=>{{log+='c';return holder;}}}});Object.defineProperty(a,'0',{{get:()=>{{log+='g';return 7;}}}});let b=a.{method}(v=>{{log+='f';return v;}});log==='csCgff' && receiver===holder && n==={length} && argc===1 && target===C && b instanceof C && b[0]===7 && b[1]===8 && !Object.hasOwn(b,'length')"
        ));
        check(&format!(
            "let n;function C(x){{n=x;}}let a=[];a.constructor={{[Symbol.species]:C}};a.{method}(()=>{{throw 7;}}) instanceof C && n===0"
        ));
        check(&format!(
            "let a=[7],target;function C(prefix,n){{target=new.target;this.n=n;this.prefix=prefix;}}let B=C.bind(null,9);a.constructor={{[Symbol.species]:B}};let b=a.{method}(v=>v);b instanceof C && target===C && b.prefix===9 && b.n==={single}",
            single = if method == "map" { 1 } else { 0 }
        ));
    }
}

#[test]
fn undefined_and_null_species_use_the_intrinsic_array() {
    for method in METHODS {
        for constructor in [
            "undefined",
            "{}",
            "{[Symbol.species]:undefined}",
            "{[Symbol.species]:null}",
            "()=>{throw 7;}",
        ] {
            check(&format!(
                "let a=[7],p=Array.prototype;a.constructor={constructor};Array=function(){{throw 8;}};let b=a.{method}(v=>v);b[0]===7 && b.length===1 && Object.getPrototypeOf(b)===p"
            ));
        }
        check(&format!(
            "let a=[7];delete Array[Symbol.species];a.{method}(v=>v)[0]===7"
        ));
    }
}

#[test]
fn invalid_constructors_are_not_coerced_and_abrupt_lookup_stops_visits() {
    for method in METHODS {
        for constructor in [
            "null",
            "false",
            "7",
            "'x'",
            "Symbol()",
            "{[Symbol.species]:false}",
            "{[Symbol.species]:()=>1}",
            "{[Symbol.species]:Array.of}",
        ] {
            check(&format!(
                "let a=[7],n=0;a.constructor={constructor};let caught=false;try{{a.{method}(()=>{{n++;return true;}});}}catch(e){{caught=e instanceof TypeError;}}caught && n===0"
            ));
        }
        for setup in [
            "Object.defineProperty(a,'constructor',{get:()=>{throw 7;}})",
            "a.constructor={get [Symbol.species](){throw 7;}}",
            "a.constructor={[Symbol.species]:function(){throw 7;}}",
        ] {
            check(&format!(
                "let a=[1],n=0;{setup};let caught=false;try{{a.{method}(()=>{{n++;}});}}catch(e){{caught=e===7;}}caught && n===0"
            ));
        }
    }
}

#[test]
fn result_definitions_bypass_setters_and_keep_prior_effects_on_rejection() {
    for method in METHODS {
        check(&format!(
            "let a=[7],o={{}};Object.defineProperty(o,'0',{{get:()=>{{throw 8;}},configurable:true}});function C(){{return o;}}a.constructor={{[Symbol.species]:C}};let b=a.{method}(v=>v),d=Object.getOwnPropertyDescriptor(b,'0');b===o && d.value===7 && d.writable && d.enumerable && d.configurable"
        ));
        check(&format!(
            "let a=[7,8,9],o={{}},n=0;Object.defineProperty(o,'1',{{value:0,configurable:false}});function C(){{return o;}}a.constructor={{[Symbol.species]:C}};let caught=false;try{{a.{method}(v=>{{n++;return v;}});}}catch(e){{caught=e instanceof TypeError;}}caught && n===2 && o[0]===7 && o[1]===0 && !Object.hasOwn(o,'2')"
        ));
        check(&format!(
            "let a=[7],n=0;Object.defineProperty(Array.prototype,'0',{{set:()=>{{n++;throw 7;}},configurable:true}});let b=a.{method}(v=>v);b[0]===7 && n===0"
        ));
    }
}

#[test]
fn species_mutations_and_self_results_are_observed_live() {
    for method in METHODS {
        check(&format!(
            "let a=[1,2,3];function C(){{a.length=1;return {{}};}}a.constructor={{[Symbol.species]:C}};let b=a.{method}(v=>v);b[0]===1 && !Object.hasOwn(b,'1') && !Object.hasOwn(b,'2')"
        ));
        check(&format!(
            "let a=[1,2],log='';function C(){{return a;}}a.constructor={{[Symbol.species]:C}};a.{method}(v=>{{log+=v;return v;}})===a && log==='12'"
        ));
    }
}

#[test]
fn abrupt_callbacks_preserve_escaped_results_and_stop_later_reads() {
    for method in METHODS {
        check(&format!(
            "let a=[7,8,9],o,n=0;function C(){{o=this;}}a.constructor={{[Symbol.species]:C}};let caught=false;try{{a.{method}(v=>{{n++;if(v===8)throw 1;return v;}});}}catch(e){{caught=e===1;}}caught && n===2 && o[0]===7 && !Object.hasOwn(o,'1')"
        ));
    }
}

#[test]
fn huge_inputs_fail_with_the_right_kind_and_custom_species_keep_full_lengths() {
    assert!(matches!(
        Realm::default().eval("Array.prototype.map.call({length:4294967296},()=>1)"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    for method in METHODS {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5_000),
            ..Limits::default()
        });
        let length = if method == "map" {
            "4294967295"
        } else {
            "Infinity"
        };
        assert!(matches!(realm.eval(&format!("try{{Array.prototype.{method}.call({{length:{length}}},()=>true);}}catch{{throw 7;}}")),Err(Error::Limit{..})));
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    assert!(matches!(realm.eval("let n,a=Array(4294967295);function C(x){n=x;}a.constructor={[Symbol.species]:C};a.map(()=>true)"),Err(Error::Limit{..})));
    assert_eq!(realm.eval("n"), Ok(Value::Number(4294967295.0)));
}

#[test]
fn standard_function_shape_and_escaped_results_survive_collection() {
    for method in METHODS {
        check(&format!(
            "let f=Array.prototype.{method},d=Object.getOwnPropertyDescriptor(Array.prototype,'{method}');f.name==='{method}' && f.length===1 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Array.prototype.{method}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let m=Array.prototype.map,f=Array.prototype.filter,o={},a=m.call([o],v=>v),b=f.call([o],()=>true);delete Array.prototype.map;delete Array.prototype.filter").unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm
            .eval("a[0]===o && b[0]===o && m.call([o],v=>v)[0]===o && f.call([o],()=>true)[0]===o"),
        Ok(Value::Boolean(true))
    );
}
