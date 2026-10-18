//! TypedArray observers, callbacks, reductions, and species results (23.2.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const KINDS: [(&str, bool); 12] = [
    ("Int8Array", false),
    ("Uint8Array", false),
    ("Uint8ClampedArray", false),
    ("Int16Array", false),
    ("Uint16Array", false),
    ("Int32Array", false),
    ("Uint32Array", false),
    ("BigInt64Array", true),
    ("BigUint64Array", true),
    ("Float16Array", false),
    ("Float32Array", false),
    ("Float64Array", false),
];

const METHODS: [&str; 17] = [
    "at",
    "every",
    "some",
    "forEach",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "includes",
    "indexOf",
    "lastIndexOf",
    "join",
    "toLocaleString",
    "reduce",
    "reduceRight",
    "map",
    "filter",
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn exception(source: &str, expected: ExceptionKind) {
    let result = Realm::default().eval(source);
    assert!(
        matches!(&result, Err(Error::Exception { kind, .. }) if *kind == expected),
        "{source}: {result:?}"
    );
}

#[test]
fn all_twelve_kinds_support_the_complete_observer_and_callback_family() {
    for (name, bigint) in KINDS {
        let (one, two, three, six, factor) = if bigint {
            ("1n", "2n", "3n", "6n", "2n")
        } else {
            ("1", "2", "3", "6", "2")
        };
        check(&format!(
            "let a={name}.of({one},{two},{three}),count=0,mapped=a.map(v=>v*{factor}),kept=a.filter((v,i)=>i!==1);a.at(-1)==={three} && a.every(v=>v>0) && a.some(v=>v==={two}) && a.forEach(()=>{{count++;}})===undefined && count===3 && a.find(v=>v>={two})==={two} && a.findIndex(v=>v>={two})===1 && a.findLast(v=>v<={two})==={two} && a.findLastIndex(v=>v<={two})===1 && a.includes({two}) && a.indexOf({two})===1 && a.lastIndexOf({two})===1 && a.join('|')==='1|2|3' && a.toLocaleString()==='1,2,3' && a.reduce((p,v)=>p+v)==={six} && a.reduceRight((p,v)=>p+v)==={six} && mapped instanceof {name} && mapped.length===3 && mapped[0]==={two} && mapped[2]==={six} && kept instanceof {name} && kept.length===2 && kept[0]==={one} && kept[1]==={three}"
        ));
    }
}

#[test]
fn entry_validation_preempts_argument_hooks_for_unbranded_detached_and_outside_views() {
    for method in METHODS {
        for receiver in [
            "null",
            "{}",
            "new DataView(new ArrayBuffer(8))",
            "Object.create(new Uint8Array(3))",
        ] {
            exception(
                &format!(
                    "Uint8Array.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}},toString(){{throw 8;}}}})"
                ),
                ExceptionKind::TypeError,
            );
        }
        check(&format!(
            "let a=new Uint8Array(3),hits=0,caught=false;a.buffer.transfer();try{{a.{method}({{valueOf(){{hits++;throw 7;}},toString(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
        check(&format!(
            "let b=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(b,1,3),hits=0,caught=false;b.resize(2);try{{a.{method}({{valueOf(){{hits++;throw 7;}},toString(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
}

#[test]
fn internal_length_and_kind_ignore_throwing_public_state_overrides() {
    check(
        "let a=Uint16Array.of(1,2,3);for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT'])Object.defineProperty(a,key,{get(){throw 7;}});let count=0,mapped=a.map(v=>v+1),kept=a.filter(v=>v!==2);a.at(1)===2 && a.every(v=>v>0) && a.some(v=>v===2) && a.forEach(()=>{count++;})===undefined && count===3 && a.find(v=>v===2)===2 && a.findIndex(v=>v===2)===1 && a.findLast(v=>v!==2)===3 && a.findLastIndex(v=>v!==2)===2 && a.includes(2) && a.indexOf(2)===1 && a.lastIndexOf(2)===1 && a.join('|')==='1|2|3' && a.toLocaleString()==='1,2,3' && a.reduce((p,v)=>p+v)===6 && a.reduceRight((p,v)=>p+v)===6 && mapped instanceof Uint16Array && mapped[2]===4 && kept[0]===1 && kept[1]===3",
    );
}

#[test]
fn at_converts_indices_after_capturing_length_without_revalidating_live_bounds() {
    for (index, expected) in [
        ("undefined", "1"),
        ("NaN", "1"),
        ("-0", "1"),
        ("1.9", "2"),
        ("-1.9", "3"),
        ("3", "undefined"),
        ("-4", "undefined"),
        ("Infinity", "undefined"),
        ("-Infinity", "undefined"),
    ] {
        check(&format!("Uint8Array.of(1,2,3).at({index})==={expected}"));
    }
    exception("new Uint8Array(0).at(Symbol())", ExceptionKind::TypeError);
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b);a[2]=7;let value=a.at({valueOf(){b.resize(6);a[5]=9;return -1;}});value===7 && a.length===6",
    );
    check(
        "let a=Uint8Array.of(1,2,3),hits=0;let value=a.at({valueOf(){hits++;a.buffer.transfer();return 0;}});hits===1 && value===undefined",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b,0,3);let value=a.at({valueOf(){b.resize(1);return 0;}});value===undefined && a.length===0",
    );
}

#[test]
fn searches_distinguish_same_value_zero_strict_equality_and_omitted_last_index() {
    check(
        "let a=Float64Array.of(NaN,-0,0,NaN);a.includes(NaN) && a.indexOf(NaN)===-1 && a.lastIndexOf(NaN)===-1 && a.indexOf(0)===1 && a.lastIndexOf(-0)===2 && !a.includes(undefined) && a.lastIndexOf(0,undefined)===-1 && a.lastIndexOf(0)===2",
    );
    check(
        "let a=Uint8Array.of(1,2,1);a.lastIndexOf(1)===2 && a.lastIndexOf(1,undefined)===0 && a.lastIndexOf(1,NaN)===0 && a.lastIndexOf(1,Infinity)===2 && a.lastIndexOf(1,-Infinity)===-1 && a.lastIndexOf(1,-2)===0 && a.indexOf(1,-2)===2 && a.indexOf(1,Infinity)===-1 && a.indexOf(1,-Infinity)===0 && !a.includes(1,Infinity) && a.includes(1,-Infinity)",
    );
    check(
        "let a=BigInt64Array.of(1n,2n);a.includes(1n) && !a.includes(1) && a.indexOf(1)===-1 && a.lastIndexOf(1)===-1",
    );
    for method in ["includes", "indexOf", "lastIndexOf"] {
        check(&format!(
            "let hits=0,search={{valueOf(){{hits++;throw 7;}}}};Uint8Array.of(1).{method}(search);hits===0"
        ));
        check(&format!(
            "let hits=0,from={{valueOf(){{hits++;throw 7;}}}};new Uint8Array(0).{method}(1,from);hits===0"
        ));
        exception(
            &format!("Uint8Array.of(1).{method}(1,1n)"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn includes_gets_invalidated_indices_while_index_searches_use_presence() {
    for (method, expected) in [
        ("includes", "true"),
        ("indexOf", "-1"),
        ("lastIndexOf", "-1"),
    ] {
        check(&format!(
            "let b=new ArrayBuffer(3,{{maxByteLength:8}}),a=new Uint8Array(b),p=Object.create(Object.getPrototypeOf(a));Object.defineProperty(p,'0',{{get(){{throw 7;}}}});Object.setPrototypeOf(a,p);let from={{valueOf(){{b.resize(0);return {};}}}};a.{method}(undefined,from)==={expected}",
            if method == "lastIndexOf" { 2 } else { 0 }
        ));
        check(&format!(
            "let a=Uint8Array.of(1,2,3),from={{valueOf(){{a.buffer.transfer();return {};}}}};a.{method}(undefined,from)==={expected}",
            if method == "lastIndexOf" { 2 } else { 0 }
        ));
    }
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:8}),a=new Uint8Array(b);a[0]=1;a[1]=2;!a.includes(9,{valueOf(){b.resize(4);a[3]=9;return 0;}})",
    );
}

#[test]
fn callbacks_receive_exact_three_arguments_this_arg_and_direction_for_both_contents() {
    for (name, bigint) in [("Uint8Array", false), ("BigInt64Array", true)] {
        let values = if bigint { "1n,2n,3n" } else { "1,2,3" };
        for method in [
            "every",
            "some",
            "forEach",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "map",
            "filter",
        ] {
            let result = match method {
                "every" | "filter" => "true",
                "map" => "value",
                _ => "false",
            };
            let order = if method.starts_with("findLast") {
                "210"
            } else {
                "012"
            };
            check(&format!(
                "let a={name}.of({values}),log='',valid=true;a.{method}(function(value,index,source){{'use strict';valid=valid && this===false && arguments.length===3 && source===a && value===a[index];log+=index;return {result};}},false);valid && log==='{order}'"
            ));
        }
    }
}

#[test]
fn callback_validation_happens_on_empty_views_before_any_species_lookup() {
    for method in [
        "every",
        "some",
        "forEach",
        "find",
        "findIndex",
        "findLast",
        "findLastIndex",
        "map",
        "filter",
        "reduce",
        "reduceRight",
    ] {
        check(&format!(
            "let a=new Uint8Array(0),hits=0,caught=false;Object.defineProperty(a,'constructor',{{get(){{hits++;throw 7;}}}});try{{a.{method}(null);}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
}

#[test]
fn predicates_short_circuit_and_find_returns_the_value_read_before_mutation() {
    check("let a=Uint8Array.of(1,2,3),count=0;!a.every(v=>{count++;return v<2;}) && count===2");
    check("let a=Uint8Array.of(1,2,3),count=0;a.some(v=>{count++;return v===2;}) && count===2");
    for (method, expected, index) in [
        ("find", "1", 0),
        ("findIndex", "0", 0),
        ("findLast", "3", 2),
        ("findLastIndex", "2", 2),
    ] {
        check(&format!(
            "let a=Uint8Array.of(1,2,3),count=0,o={{valueOf(){{throw 7;}}}};let result=a.{method}((v,i)=>{{count++;if(i!=={index})throw 8;a[i]=9;return o;}});result==={expected} && count===1 && a[{index}]===9"
        ));
    }
}

#[test]
fn callback_traversals_keep_the_captured_range_and_get_each_current_value_after_detachment() {
    for method in [
        "every",
        "some",
        "forEach",
        "find",
        "findIndex",
        "findLast",
        "findLastIndex",
    ] {
        let descending = method.starts_with("findLast");
        let result = if method == "every" { "true" } else { "false" };
        let expected = if descending {
            "2:3;1:undefined;0:undefined;"
        } else {
            "0:1;1:undefined;2:undefined;"
        };
        check(&format!(
            "let a=Uint8Array.of(1,2,3),log='',first=true;a.{method}((v,i)=>{{log+=i+':'+v+';';if(first){{first=false;a.buffer.transfer();}}return {result};}});log==='{expected}'"
        ));
    }
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b,0,3),log='';a[0]=1;a[1]=2;a[2]=3;a.forEach((v,i)=>{log+=i+':'+v+';';if(i===0)b.resize(1);if(i===1)b.resize(3);});log==='0:1;1:undefined;2:0;'",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b),log='';a[0]=1;a[1]=2;a[2]=3;a.forEach((v,i)=>{log+=i+':'+v+';';if(i===0){b.resize(6);a[1]=9;a[3]=7;}});log==='0:1;1:9;2:3;' && a.length===6",
    );
}

#[test]
fn reductions_distinguish_omitted_initial_values_and_preserve_callback_signatures() {
    for method in ["reduce", "reduceRight"] {
        exception(
            &format!("new Uint8Array(0).{method}(()=>1)"),
            ExceptionKind::TypeError,
        );
        check(&format!(
            "let hits=0;new Uint8Array(0).{method}(()=>{{hits++;throw 7;}},undefined)===undefined && Uint8Array.of(7).{method}(()=>{{hits++;throw 8;}})===7 && hits===0"
        ));
        let order = if method == "reduce" { "012" } else { "210" };
        check(&format!(
            "let a=BigInt64Array.of(1n,2n,3n),log='',valid=true;let result=a.{method}(function(acc,value,index,source){{'use strict';valid=valid && this===undefined && arguments.length===4 && source===a;log+=index;return acc+value;}},0n);valid && log==='{order}' && result===6n"
        ));
        check(&format!(
            "let calls=0,o={{valueOf(){{throw 7;}}}};let result=Uint8Array.of(7).{method}((acc,v)=>{{calls++;return acc===undefined && v===7;}},undefined);result && calls===1 && new Uint8Array(0).{method}(()=>0,o)===o"
        ));
    }
}

#[test]
fn reductions_get_invalidated_indices_without_presence_skips_or_length_growth() {
    for method in ["reduce", "reduceRight"] {
        let expected = if method == "reduce" {
            "0:1;1:undefined;2:undefined;"
        } else {
            "2:3;1:undefined;0:undefined;"
        };
        check(&format!(
            "let a=BigInt64Array.of(1n,2n,3n),first=true,log='';let result=a.{method}((acc,v,i)=>{{log+=i+':'+v+';';if(first){{first=false;a.buffer.transfer();}}return acc;}},7n);result===7n && log==='{expected}'"
        ));
    }
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b),log='';a[0]=1;a[1]=2;a[2]=3;let result=a.reduce((acc,v,i)=>{log+=i+':'+v+';';if(i===1)b.resize(0);return acc;});result===1 && log==='1:2;2:undefined;'",
    );
}

#[test]
fn join_converts_separators_even_when_empty_and_uses_live_gets_over_captured_length() {
    // An ordinary abrupt separator conversion remains the original thrown value.
    assert_eq!(
        Realm::default().eval("new Uint8Array(0).join({toString(){throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    exception("new Uint8Array(0).join(Symbol())", ExceptionKind::TypeError);
    check(
        "let a=Uint8Array.of(1,2,3),hits=0;let result=a.join({toString(){hits++;a.buffer.transfer();return ':';}});result==='::' && hits===1",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b),hits=0;a[0]=1;a[1]=2;a[2]=3;let result=a.join({toString(){hits++;b.resize(6);a[3]=9;return '|';}});result==='1|2|3' && hits===1",
    );
    check(
        "let b=new ArrayBuffer(3,{maxByteLength:8}),a=new Uint8Array(b,0,3);a[0]=1;a[1]=2;a[2]=3;a.join({toString(){b.resize(1);return ':';}})==='::'",
    );
    check(
        "Number.prototype.toString=function(){throw 7;};BigInt.prototype.toString=function(){throw 8;};Uint8Array.of(1,2).join()==='1,2' && BigInt64Array.of(1n,2n).join('|')==='1|2'",
    );
}

#[test]
fn callback_and_species_abrupt_completions_preserve_the_original_throw_and_order() {
    for method in [
        "every",
        "some",
        "forEach",
        "find",
        "findIndex",
        "findLast",
        "findLastIndex",
        "map",
        "filter",
        "reduce",
        "reduceRight",
    ] {
        assert_eq!(
            Realm::default().eval(&format!("Uint8Array.of(1,2).{method}(()=>{{throw 7;}})")),
            Err(Error::Thrown(Value::Number(7.0))),
            "{method}"
        );
    }
    for (method, callbacks) in [("map", 0), ("filter", 3)] {
        check(&format!(
            "let a=Uint8Array.of(1,2,3),hits=0,log='',caught=false;Object.defineProperty(a,'constructor',{{get(){{log+='c';return {{get [Symbol.species](){{log+='s';throw 7;}}}};}}}});try{{a.{method}(v=>{{hits++;return v;}});}}catch(e){{caught=e===7;}}caught && hits==={callbacks} && log==='cs'"
        ));
    }
}

#[test]
fn locale_calls_receive_primitive_this_no_reserved_arguments_and_stringify_results() {
    for (name, prototype, values, scalar_type) in [
        ("Uint8Array", "Number", "1,2", "number"),
        ("BigInt64Array", "BigInt", "1n,2n", "bigint"),
    ] {
        check(&format!(
            "let a={name}.of({values}),log='',calls=0;{prototype}.prototype.toLocaleString=function(){{'use strict';if(typeof this!=='{scalar_type}' || arguments.length!==0)throw 7;calls++;let value=this;log+='c'+value;return {{toString(){{log+='s'+value;return 'v'+value;}}}};}};let ignored={{toString(){{throw 8;}},valueOf(){{throw 9;}}}};a.toLocaleString(ignored,ignored)==='v1,v2' && calls===2 && log==='c1s1c2s2'"
        ));
    }
    check(
        "let a=Uint8Array.of(1,2,3),calls=0;Number.prototype.toLocaleString=function(){'use strict';calls++;return {toString(){a.buffer.transfer();return 'first';}};};a.toLocaleString()==='first,,' && calls===1",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer),log='';a[0]=1;a[1]=2;a[2]=3;Object.defineProperty(Number.prototype,'toLocaleString',{get(){log+='g';buffer.resize(0);return function(){'use strict';log+='m'+this;return 'first';};}});a.toLocaleString()==='first,,' && log==='gm1'",
    );
    check(
        "let a=Uint8Array.of(1,2,3);Number.prototype.toLocaleString=7;let caught=false;try{a.toLocaleString();}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn map_constructs_species_before_callbacks_and_sets_results_after_each_call() {
    check(
        "let a=Uint8Array.of(1,2,3),log='',count,argc,target;function C(n){log+='C'+n;count=n;argc=arguments.length;target=new.target;return new Uint16Array(n+1);}Object.defineProperty(a,'constructor',{get(){log+='c';return {get [Symbol.species](){log+='s';return C;}};}});let b=a.map((v,i)=>{log+='m'+i;return {valueOf(){log+='v'+i;return v+256;}};});log==='csC3m0v0m1v1m2v2' && count===3 && argc===1 && target===C && b instanceof Uint16Array && b.length===4 && b[0]===257 && b[2]===259 && b[3]===0",
    );
}

#[test]
fn filter_retains_pre_callback_values_and_constructs_only_after_all_predicates() {
    check(
        "let a=Uint8Array.of(1,2,3),log='',count,argc;function C(n){log+='C'+n;count=n;argc=arguments.length;return new Uint16Array(n+1);}Object.defineProperty(a,'constructor',{get(){log+='c';return {get [Symbol.species](){log+='s';return C;}};}});let b=a.filter((v,i)=>{log+='f'+i;a[i]=v+10;return i!==1;});log==='f0f1f2csC2' && count===2 && argc===1 && b instanceof Uint16Array && b.length===3 && b[0]===1 && b[1]===3 && b[2]===0 && a[0]===11 && a[2]===13",
    );
    check(
        "let a=Uint8Array.of(1),hits=0,count;function C(n){hits++;count=n;return new Uint8Array(0);}a.constructor={[Symbol.species]:C};let b=a.filter(()=>false);hits===1 && Object.is(count,0) && b.length===0",
    );
}

#[test]
fn species_defaults_use_the_intrinsic_element_kind_despite_public_binding_changes() {
    for method in ["map", "filter"] {
        for constructor in [
            "undefined",
            "{}",
            "{[Symbol.species]:undefined}",
            "{[Symbol.species]:null}",
        ] {
            check(&format!(
                "let A=Uint16Array,a=A.of(1,2),p=A.prototype;a.constructor={constructor};Uint16Array=function(){{throw 7;}};let b=a.{method}(v=>v);Object.getPrototypeOf(b)===p && b[0]===1 && b[1]===2 && b.BYTES_PER_ELEMENT===2"
            ));
        }
    }
}

#[test]
fn species_validation_rejects_wrong_brands_short_or_outside_results_without_later_callbacks() {
    for method in ["map", "filter"] {
        for result in [
            "{}",
            "[]",
            "new DataView(new ArrayBuffer(3))",
            "Uint8Array.prototype",
            "Object.create(new Uint8Array(3))",
            "new Uint8Array(2)",
            "(()=>{let b=new Uint8Array(3);b.buffer.transfer();return b;})()",
            "(()=>{let buffer=new ArrayBuffer(3,{maxByteLength:6}),b=new Uint8Array(buffer,0,3);buffer.resize(1);return b;})()",
            "new BigInt64Array(3)",
        ] {
            check(&format!(
                "let a=Uint8Array.of(1,2,3),hits=0,caught=false;function C(){{return {result};}}a.constructor={{[Symbol.species]:C}};try{{a.{method}(v=>{{hits++;return v;}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits==={}",
                if method == "map" { 0 } else { 3 }
            ));
        }
        for constructor in [
            "null",
            "7",
            "{[Symbol.species]:7}",
            "{[Symbol.species]:()=>new Uint8Array(3)}",
        ] {
            check(&format!(
                "let a=Uint8Array.of(1,2,3),hits=0,caught=false;a.constructor={constructor};try{{a.{method}(v=>{{hits++;return v;}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits==={}",
                if method == "map" { 0 } else { 3 }
            ));
        }
        check(&format!(
            "let a=new Uint8Array(0),caught=false;a.constructor={{[Symbol.species]:BigInt64Array}};try{{a.{method}(()=>true);}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
        check(&format!(
            "let a=new BigInt64Array(0),caught=false;a.constructor={{[Symbol.species]:Uint8Array}};try{{a.{method}(()=>true);}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn species_can_alias_source_and_map_writes_are_observed_by_later_callbacks() {
    check(
        "let a=Uint8Array.of(1,2,3);function C(){return a;}a.constructor={[Symbol.species]:C};let b=a.map((v,i)=>i===0?9:a[i-1]+v);b===a && a[0]===9 && a[1]===11 && a[2]===14",
    );
    check(
        "let a=Uint8Array.of(1,2,3),count;function C(n){count=n;a[0]=9;return a;}a.constructor={[Symbol.species]:C};let b=a.filter(v=>v>1);b===a && count===2 && a.length===3 && a[0]===2 && a[1]===3 && a[2]===3",
    );
}

#[test]
fn source_invalidation_during_species_does_not_trigger_another_source_validation() {
    check(
        "let a=Uint8Array.of(1,2,3),log='';function C(n){a.buffer.transfer();return new Uint8Array(n);}a.constructor={[Symbol.species]:C};let b=a.map((v,i)=>{log+=String(v)+';';return i+1;});a.length===0 && b[0]===1 && b[2]===3 && log==='undefined;undefined;undefined;'",
    );
    check(
        "let a=BigInt64Array.of(1n,2n,3n);function C(n){a.buffer.transfer();return new BigInt64Array(n);}a.constructor={[Symbol.species]:C};let b=a.filter(()=>true);a.length===0 && b.length===3 && b[0]===1n && b[2]===3n",
    );
    check(
        "let a=BigInt64Array.of(1n,2n),hits=0,caught=false;function C(n){a.buffer.transfer();return new BigInt64Array(n);}a.constructor={[Symbol.species]:C};try{a.map(v=>{hits++;return v;});}catch(e){caught=e instanceof TypeError;}caught && hits===1 && a.length===0",
    );
}

#[test]
fn map_sets_convert_values_even_after_the_custom_target_loses_bounds() {
    check(
        "let a=Uint8Array.of(1,2,3),target,buffer,log='';function C(n){buffer=new ArrayBuffer(n,{maxByteLength:6});target=new Uint8Array(buffer);return target;}a.constructor={[Symbol.species]:C};let b=a.map((v,i)=>{if(i===0)buffer.resize(0);if(i===1)buffer.resize(3);return {valueOf(){log+=i;return v;}};});b===target && log==='012' && b.length===3 && b[0]===0 && b[1]===2 && b[2]===3",
    );
    check(
        "let a=BigInt64Array.of(1n,2n),target,hits=0,caught=false;function C(n){target=new BigInt64Array(n);return target;}a.constructor={[Symbol.species]:C};try{a.map((v,i)=>{hits++;target.buffer.transfer();return 1;});}catch(e){caught=e instanceof TypeError;}caught && hits===1 && target.length===0",
    );
}

#[test]
fn filter_copies_invalidated_values_as_number_zero_or_propagates_actual_bigint_failure() {
    check(
        "let a=Uint8Array.of(1,2,3),hits=0;let b=a.filter((v,i)=>{hits++;if(i===0)a.buffer.transfer();return true;});hits===3 && b.length===3 && b[0]===1 && b[1]===0 && b[2]===0",
    );
    check(
        "let a=BigInt64Array.of(1n,2n,3n),hits=0,target,caught=false;function C(n){target=new BigInt64Array(n);return target;}a.constructor={[Symbol.species]:C};try{a.filter((v,i)=>{hits++;if(i===0)a.buffer.transfer();return true;});}catch(e){caught=e instanceof TypeError;}caught && hits===3 && target.length===3 && target[0]===1n && target[1]===0n",
    );
}

#[test]
fn filter_lists_are_not_call_argument_lists_and_opted_in_limits_remain_host_failures() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(3),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval(
            "let a=new Uint8Array([1,2,3,4,5]),b=a.filter(()=>true),c=a.map(v=>v);b.length===5 && b[4]===5 && c.length===5 && c[4]===5",
        ),
        Ok(Value::Boolean(true))
    );
    for (method, expected_species) in [("map", 1), ("filter", 0)] {
        let mut realm = Realm::new(Limits {
            max_bigint_bits: Some(8),
            ..Limits::default()
        });
        realm
            .eval(
                "let bytes=new Uint8Array(8);for(let i=0;i<8;i++)bytes[i]=255;let a=new BigUint64Array(bytes.buffer),flag=0,species=0,calls=0;function C(n){species++;return new BigUint64Array(n);}a.constructor={[Symbol.species]:C};",
            )
            .unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{a.{method}(v=>{{calls++;return v;}});}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("calls"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("species"),
            Ok(Value::Number(f64::from(expected_species)))
        );
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    realm.eval("let flag=0,a=new Uint8Array(100000)").unwrap();
    assert!(matches!(
        realm.eval("try{a.map(v=>v);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_string_units: Some(8),
        ..Limits::default()
    });
    realm.eval("let flag=0,a=new Uint8Array(10)").unwrap();
    assert!(matches!(
        realm.eval("try{a.join(',');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
