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
fn truth_table_string_classes_preserve_native_captures_and_string_consumers() {
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\b))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>\\b^)|(?<n7>$))",input="",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\b))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>\\b^)|(?<n9>$))",input="ab",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\",[2,2]],2,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\b))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>\\b^)|(?<n9>$))",input="a\n!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\"],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\b))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>\\b^)|(?<n7>$))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"\",\"a\",\"\",\"<undefined>\",\"\"],[[2,3],[2,2],\"<undefined>\",[2,2],[2,3],[3,3],\"<undefined>\",[3,3]],2,3]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>\\B^)|(?<n7>\\b$))",input="a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>\\B^)|(?<n9>\\b$))",input="!ab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>\\B^)|(?<n9>\\b$))",input="\u017fa",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],[2,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",[2,2]],2,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\b)){2,3}?(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>\\B^)|(?<n7>\\b$))",input="ab\r\n",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>^)|(?<n7>\\B))",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[1,1],\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>^)|(?<n9>\\B))",input="a\n!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[2,2],[2,2],[2,2],\"<undefined>\",[2,2],\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\"],2,2]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>^)|(?<n9>\\B))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\"],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>$)){3}(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>^)|(?<n7>\\B))",input="",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>(?<n7>^)|(?<n8>\\b))\\B)",input="!ab!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",input="\u017fa",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>(?<n9>^)|(?<n10>\\b))\\B)",input="ab\r\n",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>$))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>(?<n7>^)|(?<n8>\\b))\\B)",input="a",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>$)|(?<n7>\\B))",input="a\n!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>$)|(?<n9>\\B))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>$)|(?<n9>\\B))",input="",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\"],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\B^)|(?<n3>\\b$))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>$)|(?<n7>\\B))",input="ab",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>\\b^)|(?<n7>\\B$))",input="\u017fa",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",input="ab\r\n",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\"],0,0]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>\\b^)|(?<n9>\\B$))",input="a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>^)|(?<n3>\\B))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>\\b^)|(?<n7>\\B$))",input="!ab!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>[\\q{a|ab}])+(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>[\\q{a|ab}])|(?<n7>))+?(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",input="",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>(?<n6>)|(?<n7>[\\q{|a|ab}])){2,3}(?<n8>(?<n9>\\b\\B)|(?<n10>\\b\\B))",input="ab",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>(?<n3>^)|(?<n4>\\b))\\B)(?<n5>[a-z\\q{ab}]){1,3}(?<n6>(?<n7>\\b\\B)|(?<n8>\\b\\B))",input="a\n!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>^)|(?<n7>\\b))",input="ab\r\n",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>^)|(?<n9>\\b))",input="a",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>^)|(?<n9>\\b))",input="!ab!",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>$)|(?<n3>\\B))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>^)|(?<n7>\\b))",input="\u017fa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>$)|(?<n7>\\b)){2,3}?",input="",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>$)|(?<n9>\\b)){2,3}?",input="ab",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>$)|(?<n9>\\b)){2,3}?",input="a\n!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"<undefined>\",\"a\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],\"<undefined>\",[0,1],[1,1],\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b^)|(?<n3>\\B$))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>$)|(?<n7>\\b)){2,3}?",input="\ud83d\ude00a",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[\\q{a|ab}])+(?<n5>(?<n6>^)|(?<n7>$)){3}",input="a",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>[\\q{a|ab}])|(?<n6>))+?(?<n7>(?<n8>^)|(?<n9>$)){3}",input="!ab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>(?<n5>)|(?<n6>[\\q{|a|ab}])){2,3}(?<n7>(?<n8>^)|(?<n9>$)){3}",input="\u017fa",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<n1>(?<n2>\\b\\B)|(?<n3>\\b\\B))(?<n4>[a-z\\q{ab}]){1,3}(?<n5>(?<n6>^)|(?<n7>$)){3}",input="ab\r\n",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvg"),a=Array.from("ab!aa".matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1],\"<undefined>\",[1,1]],0,\"ab!aa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1],\"<undefined>\",[1,1]],true,true],[[\"aa\",\"\",\"<undefined>\",\"\",\"aa\",\"a\",\"a\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[3,5],[3,3],\"<undefined>\",[3,3],[3,5],[4,5],[4,5],\"<undefined>\",[5,5],[5,5],\"<undefined>\"],3,\"ab!aa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"<undefined>\",\"\",\"aa\",\"a\",\"a\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[3,3],\"<undefined>\",[3,3],[3,5],[4,5],[4,5],\"<undefined>\",[5,5],[5,5],\"<undefined>\"],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[5,5],[5,5],\"<undefined>\",[5,5],[5,5],[5,5],[5,5],\"<undefined>\",[5,5],[5,5],\"<undefined>\"],5,\"ab!aa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[5,5],\"<undefined>\",[5,5],[5,5],[5,5],[5,5],\"<undefined>\",[5,5],[5,5],\"<undefined>\"],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var calls=[],r=new RegExp(P,"vg"),out="ab!aa".replace(r,function(){var a=Array.from(arguments);calls.push(a.map(v=>v===undefined?"<undefined>":typeof v==="object"?Object.keys(v).map(k=>[k,v[k]===undefined?"<undefined>":v[k]]):v));return "["+a[0]+"]"});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"[a]b![aa][]\",[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\",0,\"ab!aa\",[[\"b\",\"\"],[\"s\",\"\"],[\"w\",\"<undefined>\"],[\"whole\",\"a\"],[\"it\",\"a\"],[\"c\",\"a\"],[\"e\",\"<undefined>\"],[\"a\",\"\"],[\"end\",\"<undefined>\"],[\"nw\",\"\"]]],[\"aa\",\"\",\"<undefined>\",\"\",\"aa\",\"a\",\"a\",\"<undefined>\",\"\",\"\",\"<undefined>\",3,\"ab!aa\",[[\"b\",\"\"],[\"s\",\"<undefined>\"],[\"w\",\"\"],[\"whole\",\"aa\"],[\"it\",\"a\"],[\"c\",\"a\"],[\"e\",\"<undefined>\"],[\"a\",\"\"],[\"end\",\"\"],[\"nw\",\"<undefined>\"]]],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\",5,\"ab!aa\",[[\"b\",\"\"],[\"s\",\"<undefined>\"],[\"w\",\"\"],[\"whole\",\"\"],[\"it\",\"\"],[\"c\",\"\"],[\"e\",\"<undefined>\"],[\"a\",\"\"],[\"end\",\"\"],[\"nw\",\"<undefined>\"]]]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvg");r.lastIndex=9;var a="!abaa!".split(r),b="!abaa!".search(r);return JSON.stringify([a.map(v=>v===undefined?"<undefined>":v),b,r.lastIndex]);})()==="[[\"!\",\"\",\"<undefined>\",\"\",\"aba\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\",\"a!\"],0,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"vg");Object.defineProperty(r,"lastIndex",{writable:false});var caught=false;try{r.exec("ab")}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvg"),a=r.exec(""),b=r.exec("ab");return JSON.stringify([row(a),row(b),r.lastIndex]);})()==="[[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\"],0,\"\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0],\"<undefined>\"],true,true],[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],0,\"ab\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],true,true],2]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[a-z\\q{|S|sa|aX}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvgi"),a=r.exec("ſaX!");return JSON.stringify([row(a),r.lastIndex]);})()==="[[[\"\u017fa\",\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\u017fa\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],\"<undefined>\",[2,2]],0,\"\u017faX!\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"\u017fa\",\"\u017fa\",\"\u017fa\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],\"<undefined>\",[2,2]],true,true],2]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<b>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvgm"),a=Array.from("ab\n\naa".matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"ab\",\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,2],[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],0,\"ab\\n\\naa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\",[0,2],[0,2],[0,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[2,2],[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],2,\"ab\\n\\naa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[2,2],\"<undefined>\",[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2],[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[3,3],[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],\"<undefined>\",[3,3],[3,3],\"<undefined>\"],3,\"ab\\n\\naa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[3,3],[3,3],\"<undefined>\",[3,3],[3,3],[3,3],\"<undefined>\",[3,3],[3,3],\"<undefined>\"],true,true],[[\"aa\",\"\",\"\",\"<undefined>\",\"aa\",\"a\",\"a\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[4,6],[4,4],[4,4],\"<undefined>\",[4,6],[5,6],[5,6],\"<undefined>\",[6,6],[6,6],\"<undefined>\"],4,\"ab\\n\\naa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"aa\",\"a\",\"a\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[4,4],[4,4],\"<undefined>\",[4,6],[5,6],[5,6],\"<undefined>\",[6,6],[6,6],\"<undefined>\"],true,true],[[\"\",\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[6,6],[6,6],\"<undefined>\",[6,6],[6,6],[6,6],[6,6],\"<undefined>\",[6,6],[6,6],\"<undefined>\"],6,\"ab\\n\\naa\",[\"b\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"<undefined>\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\",\"<undefined>\"],[[6,6],\"<undefined>\",[6,6],[6,6],[6,6],[6,6],\"<undefined>\",[6,6],[6,6],\"<undefined>\"],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var P="(?<\\u{1D49C}>(?<s>^)|(?<w>\\b))(?<whole>(?<it>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<a>(?<end>$)|(?<nw>\\B))";var r=new RegExp(P,"dvg"),s=new RegExp(r);r.lastIndex=3;var a=s.exec("ab!");return JSON.stringify([row(a),s.source===r.source,s.flags===r.flags,s.lastIndex,r.lastIndex]);})()==="[[[\"a\",\"\",\"\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,1],[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1],\"<undefined>\",[1,1]],0,\"ab!\",[\"\ud835\udc9c\",\"s\",\"w\",\"whole\",\"it\",\"c\",\"e\",\"a\",\"end\",\"nw\"],[\"\",\"\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1],\"<undefined>\",[1,1]],true,true],true,true,1,3]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<b>(?<s>^)|(?<w>\b))(?<c>[\q{a|ab}])+(?<a>(?<end>$)|(?<nw>\B))/dvg,clone=new RegExp(r),m=r.exec('ab!aa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('ab');m[0]==='a'&&m.groups.s===''&&m.groups.w===undefined&&m.groups.c==='a'&&m.groups.end===undefined&&m.groups.nw===''&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===0&&m.indices[4][1]===1&&n.groups.c==='ab'&&n.groups.end===''&&n.groups.nw===undefined&&clone.source===r.source&&clone.lastIndex===2&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}
#[test]
fn truth_table_string_classes_preserve_deep_points_and_disjunctive_endpoint_retries() {
    let pattern = JsString::from(
        format!(
            r"{}((^)|(\b)){}([\q{{a|ab}}])+",
            "(".repeat(1000),
            ")".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('ab');m[0]==='ab'&&m.length===1005&&m[1]===''&&m[1002]===''&&m[1003]===undefined&&m[1004]==='ab'&&m.indices[1004][0]===0&&m.indices[1004][1]===2"
    ));
    check(
        r"var r=/^((^)|(\b))(?<c>[\q{ab!|a}])+((^)|(\B))/dv,m=r.exec('ab!Z');m[0]==='a'&&m.groups.c==='a'&&m[2]===''&&m[3]===undefined&&m[6]===undefined&&m[7]===''&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===0&&m.indices[4][1]===1",
    );
    check(
        r"var r=/^((^)|(\b))(?<c>[\q{a|aa}])*(($)|(\B))/dv,m=r.exec('aaaa!');m[0]==='aaa'&&m.groups.c==='a'&&m[2]===''&&m[3]===undefined&&m[6]===undefined&&m[7]===''&&m.indices.groups.c[0]===2&&m.indices.groups.c[1]===3",
    );
    check(
        r"var r=/((^)|(\b))(?<whole>(?<it>(?<c>[\q{|a|ab}])|(?<e>)){2})(($)|(\B))/dv,m=r.exec('ab'),n=r.exec(''),p=r.exec('!');m[0]==='ab'&&m.groups.c===''&&m.groups.e===undefined&&m.groups.it===''&&m[2]===''&&m[3]===undefined&&m[9]===''&&m[10]===undefined&&m.indices.groups.c===m.indices[6]&&m.indices[6][0]===2&&n[0]===''&&n.groups.c===''&&n.groups.e===undefined&&n[9]===''&&n[10]===undefined&&p[0]===''&&p.groups.c===''&&p.groups.e===undefined&&p[9]===undefined&&p[10]===''",
    );
    let word = "x".repeat(20_000);
    let pattern = JsString::from(
        format!(r"((^)|(\b))(?<c>[\p{{Any}}\q{{{word}|{word}x}}])+(($)|(\B))").as_str(),
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
fn truth_table_string_classes_keep_unproved_class_spines_and_unknown_proofs_distinct() {
    assert_eq!(Realm::default().eval(r####"var pattern="(^|\\b)(([\\q{|a|ab}])|()){3}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"\",\"\",\"<undefined>\"],[[0,3],[0,0],[3,3],[3,3],\"<undefined>\"],0,0]""####),Ok(Value::Boolean(true)));
    assert_eq!(Realm::default().eval(r####"var pattern="(^|\\b)([\\p{Any}\\q{|ab}]){3}",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&(m===null||(m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")))&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"\"],[[0,3],[0,0],[3,3]],0,0]""####),Ok(Value::Boolean(true)));
    for pattern in [
        r"(^|\b)([\q{a|ab}])",
        r"((\b)|z|())([\q{a|ab}])+",
        r"((\b)|\1|())([\q{a|ab}])+",
        r"((\b\B)|(\b\B))(()|([\q{ab}&&\p{RGI_Emoji}]))+",
        r"(^|\b)([\q{a|ab}])+[a]",
        r"(^|\b)([\q{a|ab}])+|()",
    ] {
        let pattern = JsString::from(pattern);
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}}")),Err(Error::Unsupported{..})),"{pattern:?}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
