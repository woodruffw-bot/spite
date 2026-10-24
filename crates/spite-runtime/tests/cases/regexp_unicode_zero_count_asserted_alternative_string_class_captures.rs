//! Required empty iterations preserve class-versus-empty capture identity.
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
fn zero_count_asserted_alternative_string_class_native_tables_consumers_and_required_empty_identity()
 {
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)))(?<after>$)",input="",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1})(?<after>$)",input="aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1}?)(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1,1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvyi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="\u017faX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "z", "z1", "z2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="\n\nab\n",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "z", "z1", "z2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[2,2],[2,2],[2,2],[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2]],2,2]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1})(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b)",input="a",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b)",input="!ab!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1,1})(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\B)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){0001,0001}?)(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\B)",input="aba",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)))(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\B)",input="aaaa",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "z3", "z4", "whole", "iteration", "c", "e", "s", "s1", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,2],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,2],[1,2],[1,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2]],1,2]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1})(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\B)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "z3", "z4", "whole", "iteration", "e", "c", "s", "s1", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1}?)(?<after>\\b)",input="\u017faX",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1,1})(?<after>\\b)",input="\n\nab\n",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){0001,0001}?)(?<after>\\b)",input="a",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])))(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="!ab!",r=new RegExp(pattern,"dgvyim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "z", "z1", "z2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>$)",input="aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1,1})(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){0001,0001}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>$)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)))(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b)",input="\u017faX",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1})(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\b)",input="\n\nab\n",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1}?)(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\b)",input="a",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "z3", "z4", "whole", "iteration", "c", "e", "s", "s1", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1,1})(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>\\b)",input="!ab!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "z3", "z4", "whole", "iteration", "e", "c", "s", "s1", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1],[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],1,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){0001,0001}?)(?<after>\\B)",input="",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])))(?<after>\\B)",input="aba",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[1,1],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1],[1,1],[1,1],\"<undefined>\",[1,1]],1,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1})(?<after>\\B)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[1,2],[1,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,2],[1,2],[1,2],\"<undefined>\",[2,2]],1,2]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\B)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvyi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "z", "z1", "z2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1,1})(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="\u017faX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="\n\nab\n",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)))(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b)",input="a",r=new RegExp(pattern,"dgvym"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){1})(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>\\b)",input="!ab!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>\\b){1,}|(?<z2>\\B)?){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1}?)(?<s>(?<s1>$)|(?<s2>^)){000,000}?(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "s", "s1", "s2", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1,1})(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>$)",input="aba",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){0001,0001}?)(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<z>(?<z1>(?<z2>\\b){0})|(?<z3>(?<z4>\\B))){0}(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])))(?<s>(?<s1>^){99999999999999999999999}){0}(?<after>$)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvim"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){1})(?<after>\\b)",input="\u017faX",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){1}?)(?<after>\\b)",input="\n\nab\n",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],\"<undefined>\",\"<undefined>\",\"<undefined>\",[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],2,2]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<z>(?<z1>^)|(?<z2>\\b\\B)){0}(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{|ab}])|(?<e>)){1,1})(?<after>\\b)",input="a",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "z", "z1", "z2", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"a\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"a\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,1],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,1],[0,1],[0,1],\"<undefined>\",[1,1]],0,1]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<e>)|(?<c>[a-z\\q{|S|sa|aX}])){0001,0001}?)(?<z>(?<z1>\\B)*|(?<z2>\\b)+){0}?(?<after>\\b)",input="!ab!",r=new RegExp(pattern,"dgvyi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?:(?:\b)|(?:\B)){0}(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{a|ab}])){1})(?<after>\b)/dvg,a=Array.from('ab!aa!'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,\"ab!aa!\",[\"before\",\"whole\",\"iteration\",\"e\",\"c\",\"after\"],[\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],true,true],[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],2,\"ab!aa!\",[\"before\",\"whole\",\"iteration\",\"e\",\"c\",\"after\"],[\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[2,2],[2,2],[2,2],[2,2],\"<undefined>\",[2,2]],true,true],[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],[3,3],[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],3,\"ab!aa!\",[\"before\",\"whole\",\"iteration\",\"e\",\"c\",\"after\"],[\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[3,3],[3,3],[3,3],[3,3],\"<undefined>\",[3,3]],true,true],[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[5,5],[5,5],[5,5],[5,5],[5,5],\"<undefined>\",[5,5]],5,\"ab!aa!\",[\"before\",\"whole\",\"iteration\",\"e\",\"c\",\"after\"],[\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[5,5],[5,5],[5,5],[5,5],\"<undefined>\",[5,5]],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/(?:(?:\b)|(?:\B)){0}(?<before>\b)(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])|(?<e>)){1})(?<after>\b)/vg,out='ab!X!'.replace(r,function(all,b,w,it,c,e,a,offset,input,g){calls.push([all,b,w,it,c===undefined?'<undefined>':c,e===undefined?'<undefined>':e,a,offset,input,arguments.length,g.c===c,g.e===e,Object.getPrototypeOf(g)===null]);return c===undefined?'<E>':'<C:'+c+'>'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"<C:ab><C:!><C:X><C:>!\",[[\"ab\",\"\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\",0,\"ab!X!\",10,true,true,true],[\"!\",\"\",\"!\",\"!\",\"!\",\"<undefined>\",\"\",2,\"ab!X!\",10,true,true,true],[\"X\",\"\",\"X\",\"X\",\"X\",\"<undefined>\",\"\",3,\"ab!X!\",10,true,true,true],[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",4,\"ab!X!\",10,true,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?:(?:\b)|(?:\B)){0}(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{a|ab}])){1})(?<after>\b)/dvg;r.lastIndex=9;var a='!ab!X!'.split(r),at='!ab!X!'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"!\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"ab\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"!\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"X\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"!\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?:(?:\b)|(?:\B)){0}^(()|([\p{Any}\q{a|ab}])){1}\b/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('x')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?:(?:\b)|(?:\B)){0}^(?<\u{1D49C}>)(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{|😀|😀a|aX}])){1})(?<after>)$/dvgy;r.lastIndex=1;var m=r.exec('😀aX');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[null,0,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?:(?:\b)|(?:\B)){0}(?<before>\b)(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{|S|sa|aX}])){1})(?<after>\b)/dvi,m=r.exec('ſaX!');return JSON.stringify([row(m)]);})()==="[[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,\"\u017faX!\",[\"before\",\"whole\",\"iteration\",\"e\",\"c\",\"after\"],[\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var z=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])|(?<e>)){1})\b/dv.exec('x'),e=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<iteration>(?<e>)|(?<c>[\p{Any}\q{|a|ab}])){1})\b/dv.exec('x'),c=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<iteration>(?<c>[\p{Any}\q{|a|ab}])|(?<e>)){1})\b/dv.exec('x');return JSON.stringify([row(z),row(e),row(c)]);})()==="[[[\"x\",\"x\",\"x\",\"x\",\"<undefined>\"],[[0,1],[0,1],[0,1],[0,1],\"<undefined>\"],0,\"x\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"x\",\"x\",\"x\",\"<undefined>\"],[[0,1],[0,1],[0,1],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\"],0,\"x\",[\"whole\",\"iteration\",\"e\",\"c\"],[\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\"],true,true],[[\"x\",\"x\",\"x\",\"x\",\"<undefined>\"],[[0,1],[0,1],[0,1],[0,1],\"<undefined>\"],0,\"x\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"x\",\"x\",\"x\",\"<undefined>\"],[[0,1],[0,1],[0,1],\"<undefined>\"],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var e=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<iteration>(?<e>)|(?<c>[\q{ab}&&\q{cd}])){1})$/dv.exec(''),c=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<c>[\p{Any}\q{}]){1})$/dv.exec('');return JSON.stringify([row(e),row(c)]);})()==="[[[\"\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\"],0,\"\",[\"whole\",\"iteration\",\"e\",\"c\"],[\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\"],true,true],[[\"\",\"\",\"\"],[[0,0],[0,0],[0,0]],0,\"\",[\"whole\",\"c\"],[\"\",\"\"],[[0,0],[0,0]],true,true]]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?:(?:\b)|(?:\B)){0}^(?<whole>(?<iteration>(?<c>[\p{Any}\q{|ababa|a|ab}])|(?<e>)){1})(?<after>)$/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('');m.groups.whole==='ababa'&&m.groups.c==='ababa'&&m.groups.e===undefined&&m.indices.groups.c===m.indices[3]&&m.indices[3][0]===0&&m.indices[3][1]===5&&n[0]===''&&n.groups.iteration===''&&n.groups.e===undefined&&n.groups.c===''&&n.indices.groups.e===undefined&&n.indices.groups.c===n.indices[3]&&n.indices[3][0]===0&&n.indices[3][1]===0&&clone.source===r.source&&clone.lastIndex===0&&r.lastIndex===5"),Ok(Value::Boolean(true)));
}
#[test]
fn zero_count_asserted_alternative_string_class_deep_skips_defined_points_and_member_priority_remain_flat()
 {
    let pattern = JsString::from(
        format!(
            r"^{}\b{}(([\q{{}}])|())$",
            "(".repeat(1000),
            "){0}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('');m.length===1004&&m[0]===''&&m[1]===undefined&&m[1000]===undefined&&m[1001]===''&&m[1002]===''&&m[1003]===undefined&&m.indices[1002][0]===0&&m.indices[1002][1]===0&&m.indices[1003]===undefined"
    ));
    check(
        r"var r=/\b((\b){0})(?<whole>(?<c>[\q{|a|ab}])|(?<e>))((^){0})/dv,m=r.exec('ab!'),n=r.exec('x');m[1]===''&&m[2]===undefined&&m.groups.c==='ab'&&m.groups.e===undefined&&m.indices.groups.c===m.indices[4]&&m[6]===''&&m[7]===undefined&&m.indices[6][0]===2&&n.groups.c===''&&n.groups.e===undefined&&n.indices.groups.c[0]===0&&n.indices.groups.c[1]===0",
    );
    check(
        r"var r=/\b((\b){0})(?<whole>(?<e>)|(?<c>[\q{|a|ab}]))((^){0})/dv,m=r.exec('ab!');m[0]===''&&m[1]===''&&m[2]===undefined&&m.groups.e===''&&m.groups.c===undefined&&m.indices.groups.e===m.indices[4]&&m.indices.groups.c===undefined&&m[6]===''&&m[7]===undefined",
    );
    check(
        r"var r=/^((\b)|(\B)){0}(?<whole>(?<c>[\q{ab!|a|X|!}])|(?<e>)){1}\B/dv,m=r.exec('ab!Xc');m[0]==='a'&&m.groups.whole==='a'&&m.groups.c==='a'&&m.groups.e===undefined&&m[1]===undefined&&m[2]===undefined&&m[3]===undefined&&m.indices.groups.c===m.indices[5]&&m.indices.groups.c[0]===0&&m.indices.groups.c[1]===1",
    );
    check(
        r"var r=/^(?:(?:\b)|(?:\B)){0}(?<whole>(?<e>)|(?<c>[\q{a|ab}]))$/dv,m=r.exec('ab'),n=r.exec('');m[0]==='ab'&&m.groups.e===undefined&&m.groups.c==='ab'&&m.indices.groups.c===m.indices[3]&&n[0]===''&&n.groups.e===''&&n.groups.c===undefined&&n.indices.groups.e===n.indices[2]&&n.indices.groups.c===undefined",
    );
    let word = "x".repeat(20_000);
    let pattern = JsString::from(
        format!(r"^(?:(?:\b)|(?:\B)){{0}}()(([\p{{Any}}\q{{{word}|{word}x}}])|()){{1,1}}()$")
            .as_str(),
    );
    let input = JsString::from("x".repeat(20_000).as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===20000&&m[1]===''&&m[3].length===20000&&m[4]===undefined&&m[5]===''&&m.indices[3][0]===0&&m.indices[3][1]===20000&&m.indices[5][0]===20000"
    ));
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec('x');m[0]==='x'&&m[3]==='x'&&m[4]===undefined&&m.indices[3][1]===1"
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
fn zero_count_asserted_alternative_string_class_unselected_unknown_leaves_and_unproved_counts_remain_closed()
 {
    assert_eq!(Realm::default().eval(r####"var pattern="^(?:(?:\\b)|(?:\\B)){0}((\\b)|(\\B))?([\\q{|a|ab}]){3}$",input="aba",r=new RegExp(pattern,"dv"),m=r.exec(input);r.source===pattern&&r.flags==="dv"&&r.hasIndices&&r.unicodeSets&&m!==null&&m.input===input&&m.groups===undefined&&m.indices.groups===undefined&&Object.hasOwn(m,"groups")&&Object.hasOwn(m.indices,"groups")&&JSON.stringify(m===null?null:[Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,3],\"<undefined>\",\"<undefined>\",\"<undefined>\",[3,3]],0,0]""####),Ok(Value::Boolean(true)));
    for pattern in [
        r"((\b)|(\B)){1}([\q{|a|ab}]){3}",
        r"(^([\q{|a|ab}])){3}",
        r"([\q{|a|ab}]){3}(\b|)",
        r"([\q{|a|ab}]){3}\1",
        r"(([\q{|a|ab}]){3}){2}",
        r"([\q{|a|ab}]){3}[\q{ab}]",
        r"(()|([\q{ab}&&\p{RGI_Emoji}])){1}",
    ] {
        let pattern = JsString::from(format!(r"^(?:(?:\b)|(?:\B)){{0}}{pattern}$").as_str());
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({pattern:?},'v').test('aba')}}catch{{marker=1}}finally{{marker=2}}")),Err(Error::Unsupported{..})),"{pattern:?}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
