//! ArrayBuffer construction, brands, resizing, species, and detachment (25.1).

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn exception(source: &str, expected: ExceptionKind) {
    assert!(
        matches!(
            Realm::default().eval(source),
            Err(Error::Exception { kind, .. }) if kind == expected
        ),
        "{source}"
    );
}

#[test]
fn construction_requires_new_and_uses_to_index_without_wrapping() {
    check(
        "let calls=0,caught=false;try{ArrayBuffer({valueOf(){calls++;return 3;}},{get maxByteLength(){calls++;return 4;}});}catch(e){caught=e instanceof TypeError;}caught && calls===0",
    );
    for (input, length) in [
        ("undefined", 0),
        ("null", 0),
        ("false", 0),
        ("true", 1),
        ("NaN", 0),
        ("-0", 0),
        ("-0.9", 0),
        ("3.9", 3),
        ("'4'", 4),
    ] {
        check(&format!(
            "let b=new ArrayBuffer({input});b.byteLength==={length} && b.maxByteLength==={length} && !b.resizable && !b.detached && Object.is(b.byteLength,{length})"
        ));
    }
    check("new ArrayBuffer().byteLength===0 && new (ArrayBuffer.bind(null,3))().byteLength===3");
    check(
        "let calls=0,hint;let b=new ArrayBuffer({[Symbol.toPrimitive](h){calls++;hint=h;return 3;}});calls===1 && hint==='number' && b.byteLength===3",
    );
    for input in ["-1", "-Infinity", "Infinity", "9007199254740992"] {
        exception(
            &format!("new ArrayBuffer({input})"),
            ExceptionKind::RangeError,
        );
    }
    for input in ["1n", "Symbol()"] {
        exception(
            &format!("new ArrayBuffer({input})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn maximum_option_ignores_primitives_and_observes_only_the_named_property() {
    for options in ["undefined", "null", "false", "3", "'x'", "1n", "Symbol()"] {
        check(&format!(
            "Object.defineProperty(Object.prototype,'maxByteLength',{{get(){{throw 7;}},configurable:true}});let b=new ArrayBuffer(2,{options});b.byteLength===2 && b.maxByteLength===2 && !b.resizable"
        ));
    }
    check(
        "let calls=0;let options=Object.create({get maxByteLength(){calls++;return {valueOf(){calls++;return 5.9;}};}});options.valueOf=()=>{throw 7;};let b=new ArrayBuffer(2,options);calls===2 && b.byteLength===2 && b.maxByteLength===5 && b.resizable",
    );
    check("let b=new ArrayBuffer(2,{maxByteLength:undefined});!b.resizable && b.maxByteLength===2");
    check("let b=new ArrayBuffer(0,{maxByteLength:null});b.resizable && b.maxByteLength===0");
    for maximum in ["-1", "Infinity", "9007199254740992"] {
        exception(
            &format!("new ArrayBuffer(0,{{maxByteLength:{maximum}}})"),
            ExceptionKind::RangeError,
        );
    }
    for maximum in ["1n", "Symbol()"] {
        exception(
            &format!("new ArrayBuffer(0,{{maxByteLength:{maximum}}})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn constructor_orders_conversions_range_checks_and_newtarget_prototype_lookup() {
    check(
        "let log='',p={};function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return p;}});let b=Reflect.construct(ArrayBuffer,[{valueOf(){log+='l';return 2;}},{get maxByteLength(){log+='o';return {valueOf(){log+='m';return 5;}};}}],N);log==='lomp' && Object.getPrototypeOf(b)===p && Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,'byteLength').get.call(b)===2",
    );
    check(
        "let log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 7;}});try{Reflect.construct(ArrayBuffer,[{valueOf(){log+='l';return 3;}},{get maxByteLength(){log+='o';return {valueOf(){log+='m';return 2;}};}}],N);}catch(e){caught=e instanceof RangeError;}caught && log==='lom'",
    );
    check(
        "let log='',p={};function C(){}let b=Reflect.construct(ArrayBuffer,[{valueOf(){log+='l';return 2;}},{get maxByteLength(){log+='o';C.prototype=p;return 5;}}],C);log==='lo' && Object.getPrototypeOf(b)===p",
    );
    check(
        "function C(){}C.prototype=7;let b=Reflect.construct(ArrayBuffer,[2],C);Object.getPrototypeOf(b)===ArrayBuffer.prototype && b.byteLength===2",
    );
    check(
        "let calls=0,caught=false;try{new ArrayBuffer({valueOf(){throw 7;}},{get maxByteLength(){calls++;return 2;}});}catch(e){caught=e===7;}caught && calls===0",
    );
    check(
        "let log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 9;}});try{Reflect.construct(ArrayBuffer,[1,{get maxByteLength(){log+='m';throw 7;}}],N);}catch(e){caught=e===7;}caught && log==='m'",
    );
    check(
        "let log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 9;}});try{Reflect.construct(ArrayBuffer,[1,{maxByteLength:{valueOf(){log+='m';throw 7;}}}],N);}catch(e){caught=e===7;}caught && log==='m'",
    );
    check(
        "class B extends ArrayBuffer{#n=this.byteLength;read(){return this.#n;}}let b=new B(3,{maxByteLength:7});b instanceof B && b instanceof ArrayBuffer && b.read()===3 && b.resizable && b.maxByteLength===7",
    );
}

#[test]
fn descriptor_metadata_and_state_matrix() {
    let mut realm = Realm::default();
    let Value::String(output) = realm
        .eval(
            "let rows=[];for(let name of ['byteLength','detached','maxByteLength','resizable']){let d=Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,name);rows.push(name+' '+[d.get.name,d.get.length,d.set===undefined,d.enumerable,d.configurable,Object.hasOwn(d.get,'prototype')].join(','));}for(let name of ['resize','slice','transfer','transferToFixedLength']){let d=Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,name);rows.push(name+' '+[d.value.name,d.value.length,d.writable,d.enumerable,d.configurable,Object.hasOwn(d.value,'prototype')].join(','));}function state(name,b){rows.push(name+' '+[b.byteLength,b.maxByteLength,b.resizable,b.detached].join(','));}let fixed=new ArrayBuffer(3),resizable=new ArrayBuffer(3,{maxByteLength:7});state('fixed',fixed);state('resizable',resizable);resizable.resize(0);state('resized',resizable);let moved=resizable.transfer(2);state('detached-resizable',resizable);state('moved',moved);let frozen=moved.transferToFixedLength(4);state('detached-moved',moved);state('fixed-result',frozen);fixed.transfer();state('detached-fixed',fixed);rows.join('\\n')",
        )
        .unwrap()
    else {
        panic!("descriptor and state matrix must be a string")
    };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @"
    byteLength get byteLength,0,true,false,true,false
    detached get detached,0,true,false,true,false
    maxByteLength get maxByteLength,0,true,false,true,false
    resizable get resizable,0,true,false,true,false
    resize resize,1,true,false,true,false
    slice slice,2,true,false,true,false
    transfer transfer,0,true,false,true,false
    transferToFixedLength transferToFixedLength,0,true,false,true,false
    fixed 3,3,false,false
    resizable 3,7,true,false
    resized 0,7,true,false
    detached-resizable 0,0,true,true
    moved 2,7,true,false
    detached-moved 0,0,true,true
    fixed-result 4,4,false,false
    detached-fixed 0,0,false,true
    ");
}

#[test]
fn intrinsic_edges_function_attributes_and_species_are_standard() {
    check(
        "let A=ArrayBuffer,p=A.prototype,g=Object.getOwnPropertyDescriptor(globalThis,'ArrayBuffer'),d=Object.getOwnPropertyDescriptor(A,'prototype'),n=Object.getOwnPropertyDescriptor(A,'name'),l=Object.getOwnPropertyDescriptor(A,'length');g.value===A && g.writable && !g.enumerable && g.configurable && d.value===p && !d.writable && !d.enumerable && !d.configurable && A.name==='ArrayBuffer' && A.length===1 && !n.writable && !n.enumerable && n.configurable && !l.writable && !l.enumerable && l.configurable && Object.getPrototypeOf(A)===Function.prototype && Object.getPrototypeOf(p)===Object.prototype && p.constructor===A && Reflect.ownKeys(new A(3)).length===0",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(ArrayBuffer,Symbol.species),o={};d.get.name==='get [Symbol.species]' && d.get.length===0 && d.set===undefined && !d.enumerable && d.configurable && d.get.call(o)===o && d.get.call(null)===null && ArrayBuffer[Symbol.species]===ArrayBuffer && !Object.hasOwn(d.get,'prototype')",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,Symbol.toStringTag);d.value==='ArrayBuffer' && !d.writable && !d.enumerable && d.configurable && Object.prototype.toString.call(new ArrayBuffer(0))==='[object ArrayBuffer]' && Object.prototype.toString.call(ArrayBuffer.prototype)==='[object ArrayBuffer]'",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,'constructor');d.value===ArrayBuffer && d.writable && !d.enumerable && d.configurable",
    );
    for function in [
        "ArrayBuffer.isView",
        "ArrayBuffer.prototype.resize",
        "ArrayBuffer.prototype.slice",
        "ArrayBuffer.prototype.transfer",
        "ArrayBuffer.prototype.transferToFixedLength",
        "Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,'byteLength').get",
        "Object.getOwnPropertyDescriptor(ArrayBuffer,Symbol.species).get",
    ] {
        exception(&format!("new ({function})"), ExceptionKind::TypeError);
    }
}

#[test]
fn getters_and_methods_require_own_internal_slots_before_converting_arguments() {
    for receiver in [
        "undefined",
        "null",
        "0",
        "'x'",
        "1n",
        "Symbol()",
        "{}",
        "ArrayBuffer.prototype",
        "Object.create(new ArrayBuffer(2))",
        "Object.create(ArrayBuffer.prototype)",
    ] {
        for name in ["byteLength", "detached", "maxByteLength", "resizable"] {
            exception(
                &format!(
                    "Object.getOwnPropertyDescriptor(ArrayBuffer.prototype,'{name}').get.call({receiver})"
                ),
                ExceptionKind::TypeError,
            );
        }
        for name in ["resize", "slice", "transfer", "transferToFixedLength"] {
            check(&format!(
                "let hits=0,caught=false;try{{ArrayBuffer.prototype.{name}.call({receiver},{{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
}

#[test]
fn is_view_uses_slots_without_observing_arguments_or_its_receiver() {
    check(
        "let d=Object.getOwnPropertyDescriptor(ArrayBuffer,'isView');d.value.name==='isView' && d.value.length===1 && d.writable && !d.enumerable && d.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    for value in [
        "undefined",
        "null",
        "false",
        "3",
        "1n",
        "Symbol()",
        "[]",
        "{}",
        "ArrayBuffer.prototype",
        "new ArrayBuffer(3)",
        "Object.create(new ArrayBuffer(3))",
        "{get buffer(){throw 7;},get byteLength(){throw 8;},[Symbol.toPrimitive](){throw 9;}}",
    ] {
        check(&format!("ArrayBuffer.isView.call(null,{value})===false"));
    }
    check("let b=new ArrayBuffer(2);b.transfer();!ArrayBuffer.isView(b)");
}

#[test]
fn resize_updates_length_preserves_maximum_and_returns_undefined() {
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:7});b.resize(5)===undefined && b.byteLength===5 && b.maxByteLength===7 && b.resizable && !b.detached && b.resize(1.9)===undefined && b.byteLength===1 && b.resize()===undefined && Object.is(b.byteLength,0) && b.maxByteLength===7 && b.resize(7)===undefined && b.byteLength===7",
    );
    check(
        "let hits=0,b=new ArrayBuffer(1,{maxByteLength:4});b.resize({valueOf(){hits++;return 3;}});hits===1 && b.byteLength===3",
    );
    for value in ["-1", "Infinity", "9007199254740992", "8"] {
        check(&format!(
            "let b=new ArrayBuffer(3,{{maxByteLength:7}}),caught=false;try{{b.resize({value});}}catch(e){{caught=e instanceof RangeError;}}caught && b.byteLength===3 && b.maxByteLength===7 && !b.detached"
        ));
    }
    for value in ["1n", "Symbol()"] {
        exception(
            &format!("new ArrayBuffer(0,{{maxByteLength:7}}).resize({value})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn resize_brands_before_conversion_but_checks_detachment_after_conversion() {
    check(
        "let b=new ArrayBuffer(2),hits=0,caught=false;try{b.resize({valueOf(){hits++;throw 7;}});}catch(e){caught=e instanceof TypeError;}caught && hits===0 && b.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:4});b.transfer();let hits=0,caught=false;try{b.resize({valueOf(){hits++;throw 7;}});}catch(e){caught=e===7;}caught && hits===1 && b.detached && b.resizable",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:4});b.transfer();let caught=false;try{b.resize(-1);}catch(e){caught=e instanceof RangeError;}caught && b.detached",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:4});b.transfer();let caught=false;try{b.resize(5);}catch(e){caught=e instanceof TypeError;}caught && b.detached",
    );
}

#[test]
fn resize_reentrancy_uses_live_state_and_preserves_conversion_effects_on_failure() {
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:7});b.resize({valueOf(){b.resize(1);return 5;}});b.byteLength===5 && b.maxByteLength===7",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:7}),moved,caught=false;try{b.resize({valueOf(){moved=b.transfer(2);return 5;}});}catch(e){caught=e instanceof TypeError;}caught && b.detached && moved.byteLength===2 && moved.resizable && moved.maxByteLength===7",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:7}),caught=false;try{b.resize({valueOf(){b.resize(5);return -1;}});}catch(e){caught=e instanceof RangeError;}caught && b.byteLength===5 && !b.detached",
    );
}

#[test]
fn slice_ranges_use_the_original_length_and_create_fixed_fresh_buffers() {
    for (args, length) in [
        ("", 5),
        ("undefined,undefined", 5),
        ("1", 4),
        ("1,null", 0),
        ("-3,-1", 2),
        ("-99,99", 5),
        ("NaN,2.9", 2),
        ("1.9,-1.9", 3),
        ("-0,2", 2),
        ("-Infinity,Infinity", 5),
        ("Infinity", 0),
        ("0,-Infinity", 0),
        ("4,1", 0),
        ("2,NaN", 0),
    ] {
        check(&format!(
            "let a=new ArrayBuffer(5,{{maxByteLength:9}}),b=a.slice({args});b!==a && b.byteLength==={length} && b.maxByteLength==={length} && !b.resizable && !b.detached && a.byteLength===5"
        ));
    }
    check(
        "let a=new ArrayBuffer(5,{maxByteLength:9});let b=a.slice({valueOf(){a.resize(1);return 2;}},{valueOf(){a.resize(7);return 99;}});b.byteLength===3 && a.byteLength===7",
    );
    for args in ["1n", "Symbol()", "0,1n", "0,Symbol()"] {
        exception(
            &format!("new ArrayBuffer(3).slice({args})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn slice_orders_range_conversion_species_lookup_and_construction() {
    check(
        "let log='',n,argc,target,a=new ArrayBuffer(5);function C(x){log+='C';n=x;argc=arguments.length;target=new.target;return new ArrayBuffer(x+1,{maxByteLength:x+3});}Object.defineProperty(a,'constructor',{get(){log+='c';return {get [Symbol.species](){log+='p';return C;}};}});let b=a.slice({valueOf(){log+='s';return 1;}},{valueOf(){log+='e';return 4;}});log==='secpC' && n===3 && argc===1 && target===C && b.byteLength===4 && b.maxByteLength===6 && b.resizable",
    );
    check(
        "let a=new ArrayBuffer(0),hits=0,n;function C(x){hits++;n=x;return new ArrayBuffer(0);}a.constructor={[Symbol.species]:C};let b=a.slice();hits===1 && Object.is(n,0) && b!==a && b.byteLength===0",
    );
    check(
        "let a=new ArrayBuffer(3),hits=0,caught=false;Object.defineProperty(a,'constructor',{get(){hits++;throw 7;}});try{a.slice({valueOf(){throw 8;}});}catch(e){caught=e===8;}caught && hits===0",
    );
    check(
        "let a=new ArrayBuffer(3),hits=0,caught=false;Object.defineProperty(a,'constructor',{get(){hits++;throw 7;}});try{a.slice(0,{valueOf(){throw 8;}});}catch(e){caught=e===8;}caught && hits===0",
    );
}

#[test]
fn slice_species_fallback_uses_intrinsics_and_subclasses_receive_the_count() {
    for constructor in [
        "undefined",
        "{}",
        "{[Symbol.species]:undefined}",
        "{[Symbol.species]:null}",
    ] {
        check(&format!(
            "let A=ArrayBuffer,a=new A(3),p=A.prototype;a.constructor={constructor};ArrayBuffer=function(){{throw 7;}};let b=a.slice(1);Object.getPrototypeOf(b)===p && b.byteLength===2 && !b.resizable"
        ));
    }
    check(
        "let argc,target;class B extends ArrayBuffer{#n=this.byteLength;constructor(n){super(n);argc=arguments.length;target=new.target;}read(){return this.#n;}}let a=new B(5),b=a.slice(1,-1);b!==a && b instanceof B && b.read()===3 && b.byteLength===3 && argc===1 && target===B",
    );
    check(
        "class B extends ArrayBuffer{static get [Symbol.species](){return null;}}let a=new B(3),b=a.slice();!(b instanceof B) && Object.getPrototypeOf(b)===ArrayBuffer.prototype && b.byteLength===3",
    );
    for constructor in [
        "null",
        "7",
        "{[Symbol.species]:7}",
        "{[Symbol.species]:()=>new ArrayBuffer(3)}",
    ] {
        check(&format!(
            "let a=new ArrayBuffer(3),caught=false;a.constructor={constructor};try{{a.slice();}}catch(e){{caught=e instanceof TypeError;}}caught && !a.detached && a.byteLength===3"
        ));
    }
}

#[test]
fn slice_rejects_unbranded_detached_original_and_short_species_results() {
    for result in [
        "{}",
        "ArrayBuffer.prototype",
        "Object.create(new ArrayBuffer(5))",
        "a",
        "new ArrayBuffer(2)",
        "(()=>{let b=new ArrayBuffer(5);b.transfer();return b;})()",
    ] {
        check(&format!(
            "let a=new ArrayBuffer(3),calls=0,caught=false;function C(){{calls++;return {result};}}a.constructor={{[Symbol.species]:C}};try{{a.slice();}}catch(e){{caught=e instanceof TypeError;}}caught && calls===1 && a.byteLength===3 && !a.detached"
        ));
    }
    check(
        "let a=new ArrayBuffer(0),caught=false;function C(){return a;}a.constructor={[Symbol.species]:C};try{a.slice();}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn slice_detachment_checks_preserve_conversion_and_species_effect_order() {
    check(
        "let a=new ArrayBuffer(3),hits=0,caught=false;a.transfer();Object.defineProperty(a,'constructor',{get(){hits++;throw 7;}});try{a.slice({valueOf(){hits++;throw 8;}});}catch(e){caught=e instanceof TypeError;}caught && hits===0",
    );
    check(
        "let a=new ArrayBuffer(3),log='',caught=false;function C(n){log+='C'+n;return new ArrayBuffer(n);}Object.defineProperty(a,'constructor',{get(){log+='c';return {get [Symbol.species](){log+='p';return C;}};}});try{a.slice({valueOf(){log+='s';a.transfer();return 1;}},{valueOf(){log+='e';return 3;}});}catch(e){caught=e instanceof TypeError;}caught && a.detached && log==='secpC2'",
    );
    check(
        "let a=new ArrayBuffer(3),caught=false;function C(){throw 9;}a.constructor={[Symbol.species]:C};try{a.slice({valueOf(){a.transfer();return 0;}});}catch(e){caught=e===9;}caught && a.detached",
    );
    check(
        "let a=new ArrayBuffer(3),result,caught=false;function C(n){a.transfer();result=new ArrayBuffer(n);return result;}a.constructor={[Symbol.species]:C};try{a.slice();}catch(e){caught=e instanceof TypeError;}caught && a.detached && result.byteLength===3 && !result.detached",
    );
    check(
        "let a=new ArrayBuffer(5,{maxByteLength:9}),n;function C(x){n=x;a.resize(1);return new ArrayBuffer(x);}a.constructor={[Symbol.species]:C};let b=a.slice(2,4);n===2 && b.byteLength===2 && a.byteLength===1",
    );
}

#[test]
fn transfer_preserves_resizability_while_fixed_transfer_discards_the_maximum() {
    for method in ["transfer", "transferToFixedLength"] {
        for args in ["", "undefined", "2", "0", "null", "NaN", "1.9"] {
            let length = match args {
                "" | "undefined" => 3,
                "2" => 2,
                "1.9" => 1,
                _ => 0,
            };
            check(&format!(
                "let a=new ArrayBuffer(3),b=a.{method}({args});b!==a && b.byteLength==={length} && b.maxByteLength==={length} && !b.resizable && !b.detached && Object.is(a.byteLength,0) && Object.is(a.maxByteLength,0) && a.detached && !a.resizable"
            ));
            let resizable = method == "transfer";
            let maximum = if resizable { 7 } else { length };
            check(&format!(
                "let a=new ArrayBuffer(3,{{maxByteLength:7}}),b=a.{method}({args});b.byteLength==={length} && b.maxByteLength==={maximum} && b.resizable==={resizable} && !b.detached && a.byteLength===0 && a.maxByteLength===0 && a.resizable && a.detached"
            ));
        }
    }
    check(
        "let a=new ArrayBuffer(2,{maxByteLength:3}),b=a.transferToFixedLength(5);a.detached && b.byteLength===5 && b.maxByteLength===5 && !b.resizable",
    );
    check(
        "let a=new ArrayBuffer(2),b=a.transfer(5);a.detached && b.byteLength===5 && !b.resizable",
    );
}

#[test]
fn transfer_uses_intrinsics_without_constructor_or_species_lookup() {
    for method in ["transfer", "transferToFixedLength"] {
        check(&format!(
            "let A=ArrayBuffer,p=A.prototype;class B extends A{{}}let a=new B(3),hits=0;Object.defineProperty(a,'constructor',{{get(){{hits++;throw 7;}}}});Object.defineProperty(A,Symbol.species,{{get(){{hits++;throw 8;}}}});ArrayBuffer=function(){{throw 9;}};let b=a.{method}(2);hits===0 && a.detached && Object.getPrototypeOf(b)===p && !(b instanceof B) && b.byteLength===2"
        ));
    }
}

#[test]
fn transfer_converts_once_and_keeps_sources_attached_when_conversion_or_range_fails() {
    for method in ["transfer", "transferToFixedLength"] {
        check(&format!(
            "let a=new ArrayBuffer(3),calls=0,hint;let b=a.{method}({{[Symbol.toPrimitive](h){{calls++;hint=h;return 2;}}}});calls===1 && hint==='number' && a.detached && b.byteLength===2"
        ));
        for value in ["-1", "Infinity", "9007199254740992"] {
            check(&format!(
                "let a=new ArrayBuffer(3),caught=false;try{{a.{method}({value});}}catch(e){{caught=e instanceof RangeError;}}caught && !a.detached && a.byteLength===3"
            ));
        }
        check(&format!(
            "let a=new ArrayBuffer(3),caught=false;try{{a.{method}({{valueOf(){{throw 7;}}}});}}catch(e){{caught=e===7;}}caught && !a.detached && a.byteLength===3"
        ));
        for value in ["1n", "Symbol()"] {
            check(&format!(
                "let a=new ArrayBuffer(3),caught=false;try{{a.{method}({value});}}catch(e){{caught=e instanceof TypeError;}}caught && !a.detached && a.byteLength===3"
            ));
        }
    }
    check(
        "let a=new ArrayBuffer(3,{maxByteLength:4}),caught=false;try{a.transfer(5);}catch(e){caught=e instanceof RangeError;}caught && !a.detached && a.byteLength===3 && a.maxByteLength===4",
    );
}

#[test]
fn transfer_checks_detachment_after_supplied_length_conversion() {
    for method in ["transfer", "transferToFixedLength"] {
        check(&format!(
            "let a=new ArrayBuffer(3);a.transfer();let calls=0,caught=false;try{{a.{method}({{valueOf(){{calls++;throw 7;}}}});}}catch(e){{caught=e===7;}}caught && calls===1 && a.detached"
        ));
        check(&format!(
            "let a=new ArrayBuffer(3);a.transfer();let caught=false;try{{a.{method}(-1);}}catch(e){{caught=e instanceof RangeError;}}caught"
        ));
        check(&format!(
            "let a=new ArrayBuffer(3);a.transfer();let calls=0,caught=false;try{{a.{method}({{valueOf(){{calls++;return 2;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && calls===1"
        ));
        check(&format!(
            "let a=new ArrayBuffer(3,{{maxByteLength:7}}),moved,caught=false;try{{a.{method}({{valueOf(){{moved=a.transfer(1);return 2;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && a.detached && moved.byteLength===1 && !moved.detached"
        ));
        check(&format!(
            "let a=new ArrayBuffer(3,{{maxByteLength:7}}),b=a.{method}({{valueOf(){{a.resize(5);return 4;}}}});a.detached && b.byteLength===4"
        ));
    }
}

#[test]
fn branded_instances_methods_and_subclass_edges_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let b,read,resize,transfer,p;(()=>{let captured={value:7};class B extends ArrayBuffer{#x=captured;read(){return this.#x.value;}}b=new B(3,{maxByteLength:9});p=B.prototype;read=b.read;resize=b.resize;transfer=b.transfer;})();",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "read.call(b)===7 && Object.getPrototypeOf(b)===p && resize.call(b,5)===undefined && b.byteLength===5 && b.maxByteLength===9 && !b.detached",
        ),
        Ok(Value::Boolean(true))
    );
    realm.eval("let moved=transfer.call(b,2)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "b.detached && b.resizable && read.call(b)===7 && moved.byteLength===2 && moved.maxByteLength===9 && moved.resizable && !moved.detached",
        ),
        Ok(Value::Boolean(true))
    );
}
