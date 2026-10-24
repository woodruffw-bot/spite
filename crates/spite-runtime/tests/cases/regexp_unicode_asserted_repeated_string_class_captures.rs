//! Unconditional predicates retain member/count backtracking and lexical captures.
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
fn asserted_repeated_string_class_native_tables_consumers_and_zero_iteration_scopes() {
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))*)(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))*?)(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))+)(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))+?)(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>$)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3,4}?)(?<after>$)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){0})(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){0})(?<after>$)",input="a",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){0,2}?)(?<after>$)",input="aaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))?)(?<after>$)",input="aaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}]))??)(?<after>$)",input="aaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2,})(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\b)",input="aaaa!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*?)(?<after>\\b)",input="aaaa!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2})(?<after>$)",input="abc",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>$)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){2})(?<after>$)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){2})(?<after>$)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){2})(?<after>$)",input="\u017faX",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))+?)(?<after>$)",input="\naaaa\n",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""####,
    );
    check(
        r####"var pattern="(?<before>$)(?<whole>(?<iteration>(?<c>[\\q{\\n|\\na|aX}]))*)(?<after>^)",input="\n\naX\n",r=new RegExp(pattern,"dgvm"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\\n\\naX\\n\",\"\",\"\\n\\naX\\n\",\"\\n\",\"\\n\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\b)",input="!aaaa!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""####,
    );
    check(
        r####"var pattern="(?<before>\\b)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\b)",input="!aaaa!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*?)(?<after>\\b)",input="x",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>\\B)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>\\B)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>\\b\\B)(?<whole>(?<iteration>(?<c>[\\q{a|aa}]))*)(?<after>)",input="a",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<c>[\\q{ab|abc|c}]*?))(?<after>$)",input="abcab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcab\",\"\",\"abcab\",\"abcab\",\"\"],[[0,5],[0,0],[0,5],[0,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){2,999999999999999999999999})(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){999999999999999999999999})(?<after>$)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}]))*)(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}]))+)(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>^)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}])){0})(?<after>$)",input="",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/^(?<whole>(?<word>[\q{aa|aaa}])*)$/dvgm,a=Array.from('aaaa\n\naaaaaa'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"aaaa\",\"aaaa\",\"aa\"],[[0,4],[0,4],[2,4]],0,\"aaaa\\n\\naaaaaa\",[\"whole\",\"word\"],[\"aaaa\",\"aa\"],[[0,4],[2,4]],true,true],[[\"\",\"\",\"<undefined>\"],[[5,5],[5,5],\"<undefined>\"],5,\"aaaa\\n\\naaaaaa\",[\"whole\",\"word\"],[\"\",\"<undefined>\"],[[5,5],\"<undefined>\"],true,true],[[\"aaaaaa\",\"aaaaaa\",\"aaa\"],[[6,12],[6,12],[9,12]],6,\"aaaa\\n\\naaaaaa\",[\"whole\",\"word\"],[\"aaaaaa\",\"aaa\"],[[6,12],[9,12]],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/^(?<whole>(?<word>[\q{a|aa}])*?)$/vgm,out='aaa\n\nbbbb'.replace(r,function(all,w,word,offset,input,g){calls.push([all,w,word===undefined?'<undefined>':word,offset,input,arguments.length,g.word===word,Object.getPrototypeOf(g)===null]);return word===undefined?'[E]':'['+word+']'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"[a]\\n[E]\\nbbbb\",[[\"aaa\",\"aaa\",\"a\",0,\"aaa\\n\\nbbbb\",6,true,true],[\"\",\"\",\"<undefined>\",4,\"aaa\\n\\nbbbb\",6,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>\b)(?<whole>(?<word>[\q{a|aa}])*)(?<after>\b)/dvg;r.lastIndex=9;var a='!aaaa!'.split(r),at='!aaaa!'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"!\",\"\",\"aaaa\",\"aa\",\"\",\"!\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/^[\q{aa|aaa}]*$/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('aaaa')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/^(?<\u{1D49C}>)(?<whole>(?<word>[\q{😀|😀a|aX}]){2})(?<after>)$/dvgy;r.lastIndex=1;var m=r.exec('😀aX');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],[4,4]],0,\"\ud83d\ude00aX\",[\"\ud835\udc9c\",\"whole\",\"word\",\"after\"],[\"\",\"\ud83d\ude00aX\",\"aX\",\"\"],[[0,0],[0,4],[2,4],[4,4]],true,true],4,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>\b)(?<whole>(?<word>[\q{S|sa|aX}]){2})(?<after>\b)/dvi,m=r.exec('ſaX!');return JSON.stringify([row(m)]);})()==="[[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[3,3]],0,\"\u017faX!\",[\"before\",\"whole\",\"word\",\"after\"],[\"\",\"\u017faX\",\"aX\",\"\"],[[0,0],[0,3],[1,3],[3,3]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var g=/^(?<whole>(?<word>[\q{aa|aaa}]){0,2})\b/dv.exec('aaaa!'),l=/^(?<whole>(?<word>[\q{aa|aaa}]){0,2}?)\b/dv.exec('aaaa!');return JSON.stringify([row(g),row(l)]);})()==="[[[\"aaaa\",\"aaaa\",\"aa\"],[[0,4],[0,4],[2,4]],0,\"aaaa!\",[\"whole\",\"word\"],[\"aaaa\",\"aa\"],[[0,4],[2,4]],true,true],[[\"\",\"\",\"<undefined>\"],[[0,0],[0,0],\"<undefined>\"],0,\"aaaa!\",[\"whole\",\"word\"],[\"\",\"<undefined>\"],[[0,0],\"<undefined>\"],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>^)(?:(?<zero>)|(?<never>))(?<whole>(?<word>[\q{aa|aaa}])*)(?<after>$)/dv,m=r.exec('aaaa');return JSON.stringify([row(m)]);})()==="[[[\"aaaa\",\"\",\"\",\"<undefined>\",\"aaaa\",\"aa\",\"\"],[[0,4],[0,0],[0,0],\"<undefined>\",[0,4],[2,4],[4,4]],0,\"aaaa\",[\"before\",\"zero\",\"never\",\"whole\",\"word\",\"after\"],[\"\",\"\",\"<undefined>\",\"aaaa\",\"aa\",\"\"],[[0,0],[0,0],\"<undefined>\",[0,4],[2,4],[4,4]],true,true]]""####,
    );

    let mut realm = Realm::default();
    realm
        .eval(r"var r=/^(?<whole>(?<word>[\q{aa|aaa}])*)$/dvg,clone=new RegExp(r),m=r.exec('aaaa')")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('');m.groups.whole==='aaaa'&&m.groups.word==='aa'&&m.indices.groups.word===m.indices[2]&&m.indices[2][0]===2&&m.indices[2][1]===4&&n[0]===''&&n.groups.whole===''&&n.groups.word===undefined&&n.indices.groups.word===undefined&&clone.source===r.source&&clone.lastIndex===0&&r.lastIndex===4"),Ok(Value::Boolean(true)));
}
#[test]
fn asserted_repeated_string_class_deep_scopes_continuations_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"^{}([\q{{aa|aaa}}]){{3,4}}{}$",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('aaaaaa');m.length===1002&&m[0]==='aaaaaa'&&m[1000]==='aaaaaa'&&m[1001]==='aa'&&m.indices[1001][0]===4&&m.indices[1001][1]===6"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!(r"^()([\q{{{word}|{word}x}}]){{3}}()$").as_str());
    let input = JsString::from(format!("{word}{word}{word}").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===60000&&m[1]===''&&m[2].length===20000&&m[3]===''&&m.indices[2][0]===40000&&m.indices[2][1]===60000&&m.indices[3][0]===60000"
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
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval(r"var r=/^[\q{a|aa}]*$/vg,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "a".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(input)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm.eval(r"var r=/^[\q{a|aa}]*?\b/vg,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "a".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===1&&m.index===0&&m[0]===''&&r.lastIndex===0"),Ok(Value::Boolean(true)));
}
#[test]
fn asserted_repeated_string_class_proofs_keep_conditional_predicates_and_unproved_leaves_closed() {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^([\\q{|aa|aaa}])*$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"aaa\"],\"indices\":[[0,6],[3,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"aaa\"],\"indices\":[[0,6],[3,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^(([\\q{aa|aaa}])|())*$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"aaa\",\"aaa\",\"<undefined>\"],\"indices\":[[0,6],[3,6],[3,6],\"<undefined>\"],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"aaa\",\"aaa\",\"<undefined>\"],\"indices\":[[0,6],[3,6],[3,6],\"<undefined>\"],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"indices\":[[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\"],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"^([[\q{|aa|aaa}]&&\p{RGI_Emoji}])*$",
        r"^(([[\q{aa|aaa}]&&\p{RGI_Emoji}])|())*$",
        r"(^([\q{aa|aaa}]))*$",
        r"(^|)([\q{aa|aaa}])*$",
        r"(^)*([\q{aa|aaa}])*$",
        r"^([\q{aa|aaa}])*(\b|)",
        r"^([\q{aa|aaa}])*\1",
        r"a^([\q{aa|aaa}])*$",
        r"^([\q{aa|aaa}])*a$",
        r"^(([\q{aa|aaa}])*){2}$",
        r"^([\q{aa|aaa}])*[\q{ab}]$",
        r"^([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}$",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('aaaa')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
