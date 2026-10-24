//! Required predicate choices preserve conditional lexical captures.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}
#[test]
fn conjunctive_predicate_choice_single_string_classes_preserve_native_captures_and_string_consumers()
 {
    check(
        r####"var pattern="(?<n1>(?<n2>\\b)|(?<n3>\\b))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\B^)|(?<n7>^))",input="",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b)|(?<n3>\\b))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\B^)|(?<n8>^))",input="ab",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b)|(?<n3>\\b))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\B^)|(?<n7>^))",input="a\n!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b)|(?<n3>\\b))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\B^)|(?<n8>^))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B)|(?<n3>\\b\\B)){2,3}?(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b$)|(?<n7>$))",input="a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B)|(?<n3>\\b\\B)){2,3}?(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b$)|(?<n8>$))",input="!ab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[4,4],[4,4],[4,4],\"<undefined>\",[4,4],[4,4],[4,4],\"<undefined>\",[4,4]],4,4]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B)|(?<n3>\\b\\B)){2,3}?(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b$)|(?<n7>$))",input="\u017fa",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"\",\"\",\"<undefined>\"],[[1,2],[1,1],[1,1],\"<undefined>\",[1,2],[2,2],[2,2],\"<undefined>\"],1,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B)|(?<n3>\\b\\B)){2,3}?(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b$)|(?<n8>$))",input="ab\r\n",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>^)){3}(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\B$)|(?<n7>$))",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[2,2],\"<undefined>\",[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>^)){3}(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\B$)|(?<n8>$))",input="a\n!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>^)){3}(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\B$)|(?<n7>$))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>^)){3}(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\B$)|(?<n8>$))",input="",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>^))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b^$)|(?<n7>^$))",input="!ab!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>^))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b^$)|(?<n8>^$))",input="\u017fa",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>^))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b^$)|(?<n7>^$))",input="ab\r\n",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>^))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b^$)|(?<n8>^$))",input="a",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b$)|(?<n3>$))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b\\B)|(?<n7>\\b))",input="a\n!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b$)|(?<n3>$))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b\\B)|(?<n8>\\b))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],3,3]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b$)|(?<n3>$))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b\\B)|(?<n7>\\b))",input="",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b$)|(?<n3>$))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b\\B)|(?<n8>\\b))",input="ab",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B$)|(?<n3>$))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b\\B)|(?<n7>\\B))",input="\u017fa",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B$)|(?<n3>$))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b\\B)|(?<n8>\\B))",input="ab\r\n",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],3,3]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B$)|(?<n3>$))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b\\B)|(?<n7>\\B))",input="a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B$)|(?<n3>$))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b\\B)|(?<n8>\\B))",input="!ab!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^$)|(?<n3>^$))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b\\B)|(?<n7>\\b\\B))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^$)|(?<n3>^$))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",input="",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^$)|(?<n3>^$))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b\\B)|(?<n7>\\b\\B))",input="ab",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^$)|(?<n3>^$))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",input="a\n!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b)|(?<n7>\\b))",input="ab\r\n",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b)|(?<n8>\\b))",input="a",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"\",\"<undefined>\"],[[0,1],[0,0],\"<undefined>\",[0,0],[0,1],[0,1],[1,1],[1,1],\"<undefined>\"],0,1]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b)|(?<n7>\\b))",input="!ab!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"<undefined>\",\"\",\"ab\",\"\",\"\",\"<undefined>\"],[[1,3],[1,1],\"<undefined>\",[1,1],[1,3],[3,3],[3,3],\"<undefined>\"],1,3]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b)|(?<n8>\\b))",input="\u017fa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"a\",\"\",\"\",\"<undefined>\"],[[1,2],[1,1],\"<undefined>\",[1,1],[1,2],[1,2],[2,2],[2,2],\"<undefined>\"],1,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\B))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\B)|(?<n7>\\b\\B)){2,3}?",input="",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\B))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\B)|(?<n8>\\b\\B)){2,3}?",input="ab",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[1,1],[1,1],\"<undefined>\",[1,1],[1,1],[1,1],[1,1],[1,1],\"<undefined>\"],1,1]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\B))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\B)|(?<n7>\\b\\B)){2,3}?",input="a\n!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"!\",\"\",\"<undefined>\",\"\",\"!\",\"\",\"\",\"<undefined>\"],[[2,3],[2,2],\"<undefined>\",[2,2],[2,3],[3,3],[3,3],\"<undefined>\"],2,3]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\B))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\B)|(?<n8>\\b\\B)){2,3}?",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[\\q{a|ab}])(?<n5>(?<n6>\\b^)|(?<n7>^)){3}",input="a",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[\\q{|a|ab}])){1}(?<n6>(?<n7>\\b^)|(?<n8>^)){3}",input="!ab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[\\p{Any}\\q{|ab}]){0001,0001}?(?<n5>(?<n6>\\b^)|(?<n7>^)){3}",input="\u017fa",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[a-z\\q{ab}]){1}){1,1}?(?<n6>(?<n7>\\b^)|(?<n8>^)){3}",input="ab\r\n",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvg"),a=Array.from("ab!aa".matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2]],0,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2]],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],2,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],3,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],\"<undefined>\",[3,3],[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[5,5],[5,5],\"<undefined>\",[5,5],[5,5],[5,5],[5,5],[5,5],\"<undefined>\"],5,\"ab!aa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[5,5],\"<undefined>\",[5,5],[5,5],[5,5],[5,5],[5,5],\"<undefined>\"],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var calls=[],r=new RegExp(P,"vg"),out="ab!aa".replace(r,function(){var a=Array.from(arguments);calls.push(a.map(v=>v===undefined?"<undefined>":typeof v==="object"?Object.keys(v).map(k=>[k,v[k]===undefined?"<undefined>":v[k]]):v));return "["+a[0]+"]"});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"[ab][]![]aa[]\",[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\",0,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"\"],[\"nw\",\"<undefined>\"],[\"whole\",\"ab\"],[\"c\",\"ab\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"wb\",\"\"]]],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",2,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"<undefined>\"],[\"nw\",\"\"],[\"whole\",\"\"],[\"c\",\"\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"wb\",\"\"]]],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",3,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"<undefined>\"],[\"nw\",\"\"],[\"whole\",\"\"],[\"c\",\"\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"wb\",\"\"]]],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\",5,\"ab!aa\",[[\"b\",\"\"],[\"w\",\"<undefined>\"],[\"nw\",\"\"],[\"whole\",\"\"],[\"c\",\"\"],[\"a\",\"\"],[\"end\",\"\"],[\"wb\",\"<undefined>\"]]]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvg");r.lastIndex=9;var a="!abaa!".split(r),b="!abaa!".search(r);return JSON.stringify([a.map(v=>v===undefined?"<undefined>":v),b,r.lastIndex]);})()==="[[\"!\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"abaa\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"!\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"vg");Object.defineProperty(r,"lastIndex",{writable:false});var caught=false;try{r.exec("ab")}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvg"),a=r.exec(""),b=r.exec("ab");return JSON.stringify([row(a),row(b),r.lastIndex]);})()==="[null,[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\"],0,\"ab\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\"],true,true],2]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[a-z\\q{|S|sa|aX}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvgi"),a=r.exec("ſaX!");return JSON.stringify([row(a),r.lastIndex]);})()==="[[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,\"\u017faX!\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],true,true],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvgm"),a=Array.from("ab\n\naa".matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\"],0,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],[2,2],\"<undefined>\"],2,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[4,4],[4,4],[4,4],\"<undefined>\",[4,4],[4,4],[4,4],\"<undefined>\",[4,4]],4,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[4,4],[4,4],\"<undefined>\",[4,4],[4,4],[4,4],\"<undefined>\",[4,4]],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[6,6],[6,6],\"<undefined>\",[6,6],[6,6],[6,6],[6,6],[6,6],\"<undefined>\"],6,\"ab\\n\\naa\",[\"b\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"\",\"<undefined>\"],[[6,6],\"<undefined>\",[6,6],[6,6],[6,6],[6,6],[6,6],\"<undefined>\"],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<\\u{1D49C}>(?<w>^\\b)|(?<nw>\\b))(?<whole>(?<c>[\\q{|a|ab}]))(?<a>(?<end>\\b$)|(?<wb>\\b))";var r=new RegExp(P,"dvg"),s=new RegExp(r);r.lastIndex=3;var a=s.exec("ab!");return JSON.stringify([row(a),s.source===r.source,s.flags===r.flags,s.lastIndex,r.lastIndex]);})()==="[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2]],0,\"ab!\",[\"\ud835\udc9c\",\"w\",\"nw\",\"whole\",\"c\",\"a\",\"end\",\"wb\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[2,2],\"<undefined>\",[2,2]],true,true],true,true,2,3]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<b>(?<w>^\b)|(?<nw>\b))(?<c>[\q{a|ab}])(?<a>(?<end>\b$)|(?<wb>\b))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('a');m[0]==='ab'&&m.groups.w===''&&m.groups.nw===undefined&&m.groups.c==='ab'&&m.groups.end===undefined&&m.groups.wb===''&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===0&&m.indices[4][1]===2&&n.groups.c==='a'&&n.groups.end===''&&n.groups.wb===undefined&&clone.source===r.source&&clone.lastIndex===1&&r.lastIndex===2"),Ok(Value::Boolean(true)));
}

#[test]
fn conjunctive_predicate_choice_single_string_classes_preserve_deep_points_and_live_endpoint_retries()
 {
    let pattern = JsString::from(
        format!(
            r"{}((\b^)|(^)){}([\q{{a|ab}}])",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('ab');m[0]==='ab'&&m.length===1005&&m[1]===''&&m[1002]===''&&m[1003]===undefined&&m[1004]==='ab'&&m.indices[1004][0]===0&&m.indices[1004][1]===2"
    ));
    check(
        r"var r=/^((\b)|(\b))(?<c>[\q{ab!|a|X|!}])((\B)|(\b\B))/dv,m=r.exec('ab!Xc');m[0]==='a'&&m.groups.c==='a'&&m[2]===''&&m[3]===undefined&&m[6]===''&&m[7]===undefined&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===0&&m.indices[4][1]===1",
    );
    check(
        r"var r=/((\b^)|(^))(?<whole>(?<c>[\q{|a|ab}])){1}((\b$)|($))/dv,m=r.exec('ab'),n=r.exec('');m[0]==='ab'&&m.groups.c==='ab'&&m[2]===''&&m[3]===undefined&&m[7]===''&&m[8]===undefined&&m.indices.groups.c===m.indices[5]&&m.indices[5][0]===0&&m.indices[5][1]===2&&n[0]===''&&n.groups.c===''&&n[2]===undefined&&n[3]===''&&n[7]===undefined&&n[8]===''",
    );
    let word = "x".repeat(20_000);
    let pattern = JsString::from(
        format!(r"((\b^)|(^))(?<c>[\p{{Any}}\q{{{word}|{word}x}}])((\b$)|($))").as_str(),
    );
    let input = JsString::from(word.as_str());
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec({input:?}),n=r.exec('x');m[0].length===20000&&m.groups.c.length===20000&&m[2]===''&&m[3]===undefined&&m[6]===''&&m[7]===undefined&&m.indices.groups.c[0]===0&&m.indices.groups.c[1]===20000&&n.groups.c==='x'&&n[6]===''&&n[7]===undefined"
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm.eval(&format!(
            "try{{r=new RegExp({pattern:?},'v')}}catch{{marker=1}}finally{{marker=2}}"
        )),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("r===7&&marker===0"), Ok(Value::Boolean(true)));
}
#[test]
fn conjunctive_predicate_choice_single_string_classes_prove_unknown_leaves_even_behind_never_gates()
{
    for pattern in [
        r"((\b\B)|(\b\B))([\q{ab}&&\p{RGI_Emoji}])",
        r"(^|\b)([\q{a|ab}])",
        r"($|\b)([\q{a|ab}])",
        r"(^|$)([\q{a|ab}])",
        r"((\b)|z|())([\q{a|ab}])",
        r"((\b)|\1|())([\q{a|ab}])",
        r"((\b\B)|(\b\B))(()|([\q{ab}&&\p{RGI_Emoji}]))+",
        r"((\b)|(\b))([\q{a|ab}])[a]",
        r"((\b)|(\b))((([\q{a|ab}])){2}){2}",
        r"((\b)|(\b))([\q{a|ab}])|()",
    ] {
        let pattern = JsString::from(pattern);
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}}")),Err(Error::Unsupported{..})),"{pattern:?}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
