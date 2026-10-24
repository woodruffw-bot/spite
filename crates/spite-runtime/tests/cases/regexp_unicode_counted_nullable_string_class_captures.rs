//! Larger nullable counts retain required-empty capture resets.
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
fn counted_nullable_string_class_native_tables_consumers_and_required_empty_identity() {
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abab\",\"\",\"abab\",\"ab\",\"ab\",\"<undefined>\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],\"<undefined>\",[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2,3})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],\"<undefined>\",[0,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2,3}?)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2,3}?)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"\",\"<undefined>\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],\"<undefined>\",[2,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>)){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"\",\"\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,2],[2,2],[2,2],\"<undefined>\",[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],[2,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2,3})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){3}?)(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])|(?<e>)){2})(?<after>)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaa\",\"\",\"aaa\",\"\",\"<undefined>\",\"\",\"\"],[[0,3],[0,0],[0,3],[3,3],\"<undefined>\",[3,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){2})(?<after>)",input="!ababX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){2,4})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abab\",\"\",\"abab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,4],[0,0],[0,4],[2,4],\"<undefined>\",[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a}])|(?<e>)){2})(?<after>)",input="\ud83d\ude00a\ud83d\ude00!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00a\ud83d\ude00\",\"\",\"\ud83d\ude00a\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"<undefined>\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],\"<undefined>\",[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{s|sa|k|ka}])|(?<e>)){2})(?<after>)",input="\u017fa\u212a!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017fa\u212a\",\"\",\"\u017fa\u212a\",\"\u212a\",\"\u212a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab}])|(?<e>)){2})(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{}]{2}))(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{|a|ab}]{2}))(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],[2,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{|a|ab}]{2,3}?))(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abab\",\"\",\"abab\",\"abab\",\"\"],[[0,4],[0,0],[0,4],[0,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\n|\\na}])|(?<e>)){2})(?<after>)",input="\na",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\\na\",\"\",\"\\na\",\"\",\"<undefined>\",\"\",\"\"],[[0,2],[0,0],[0,2],[2,2],\"<undefined>\",[2,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}])|(?<e>)){2})(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){99999999999999999999,100000000000000000000})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],\"<undefined>\",[0,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>)){99999999999999999999,100000000000000000000})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"\",\"<undefined>\",\"\",\"\"],[[0,5],[0,0],[0,5],[5,5],\"<undefined>\",[5,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])){99999999999999999999,100000000000000000000})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"\",\"\",\"\"],[[0,5],[0,0],[0,5],[5,5],[5,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){99999999999999999999})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}])){000099999999999999999999,0000100000000000000000001})(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abab\",\"\",\"abab\",\"ab\",\"<undefined>\",\"ab\",\"\"],[[0,4],[0,0],[0,4],[2,4],\"<undefined>\",[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){2,3})/dvg,a=Array.from('abXaa'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"ab\",\"ab\",\"\",\"\",\"<undefined>\"],[[0,2],[0,2],[2,2],[2,2],\"<undefined>\"],0,\"abXaa\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"ab\",\"\",\"\",\"<undefined>\"],[[0,2],[2,2],[2,2],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"\",\"<undefined>\"],[[2,2],[2,2],[2,2],[2,2],\"<undefined>\"],2,\"abXaa\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"\",\"\",\"\",\"<undefined>\"],[[2,2],[2,2],[2,2],\"<undefined>\"],true,true],[[\"aa\",\"aa\",\"a\",\"a\",\"<undefined>\"],[[3,5],[3,5],[4,5],[4,5],\"<undefined>\"],3,\"abXaa\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"aa\",\"a\",\"a\",\"<undefined>\"],[[3,5],[4,5],[4,5],\"<undefined>\"],true,true],[[\"\",\"\",\"\",\"\",\"<undefined>\"],[[5,5],[5,5],[5,5],[5,5],\"<undefined>\"],5,\"abXaa\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"\",\"\",\"\",\"<undefined>\"],[[5,5],[5,5],[5,5],\"<undefined>\"],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){2,3})/vg,out='abXaa'.replace(r,function(all,w,it,e,c,offset,input,g){calls.push([all,w,it,e===undefined?'<undefined>':e,c===undefined?'<undefined>':c,offset,input,arguments.length,g.c===c&&g.e===e,Object.getPrototypeOf(g)===null]);return c===undefined?'[E]':'['+c+']'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"[ab][E]X[a][a][E]\",[[\"ab\",\"ab\",\"ab\",\"<undefined>\",\"ab\",0,\"abXaa\",8,true,true],[\"\",\"\",\"\",\"\",\"<undefined>\",2,\"abXaa\",8,true,true],[\"a\",\"a\",\"a\",\"<undefined>\",\"a\",3,\"abXaa\",8,true,true],[\"a\",\"a\",\"a\",\"<undefined>\",\"a\",4,\"abXaa\",8,true,true],[\"\",\"\",\"\",\"\",\"<undefined>\",5,\"abXaa\",8,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}])){2,3})/dvg;r.lastIndex=9;var a='abXaa'.split(r),at='!ab'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"\",\"ab\",\"ab\",\"<undefined>\",\"ab\",\"X\",\"a\",\"a\",\"<undefined>\",\"a\",\"\",\"a\",\"a\",\"<undefined>\",\"a\",\"\"],0,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/[\q{|a|ab}]{2}/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('ab')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<\u{1D49C}>)(?<whole>(?<iteration>(?<c>[\q{😀|😀a}])|(?<e>)){2,3})(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀a😀!');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[[[\"\ud83d\ude00a\ud83d\ude00\",\"\",\"\ud83d\ude00a\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"<undefined>\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],\"<undefined>\",[5,5]],0,\"\ud83d\ude00a\ud83d\ude00!\",[\"\ud835\udc9c\",\"whole\",\"iteration\",\"c\",\"e\",\"after\"],[\"\",\"\ud83d\ude00a\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"<undefined>\",\"\"],[[0,0],[0,5],[3,5],[3,5],\"<undefined>\",[5,5]],true,true],5,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>)(?<whole>(?<iteration>(?<c>[\q{|s|sa|k|ka}])|(?<e>)){2,3})(?<after>)/dvi,m=r.exec('ſaK!');return JSON.stringify([row(m)]);})()==="[[[\"\u017fa\u212a\",\"\",\"\u017fa\u212a\",\"\u212a\",\"\u212a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,\"\u017fa\u212a!\",[\"before\",\"whole\",\"iteration\",\"c\",\"e\",\"after\"],[\"\",\"\u017fa\u212a\",\"\u212a\",\"\u212a\",\"<undefined>\",\"\"],[[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var z=/(?<whole>(?<iteration>(?<c>[\q{|a|ab}])){2,3}?)/dv.exec('ab'),p=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{|a|ab}])){2,3}?)/dv.exec('ab');return JSON.stringify([row(z),row(p)]);})()==="[[[\"ab\",\"ab\",\"\",\"\"],[[0,2],[0,2],[2,2],[2,2]],0,\"ab\",[\"whole\",\"iteration\",\"c\"],[\"ab\",\"\",\"\"],[[0,2],[2,2],[2,2]],true,true],[[\"\",\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\"],0,\"ab\",[\"whole\",\"iteration\",\"e\",\"c\"],[\"\",\"\",\"\",\"<undefined>\"],[[0,0],[0,0],[0,0],\"<undefined>\"],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var e=/(?<whole>(?<iteration>(?<c>[\q{aa|aaa}])|(?<e>)){2,3})/dv.exec('aaaa'),c=/(?<whole>(?<iteration>(?<c>[\q{|aa|aaa}])){2,3})/dv.exec('aaaa');return JSON.stringify([row(e),row(c)]);})()==="[[[\"aaa\",\"aaa\",\"\",\"<undefined>\",\"\"],[[0,3],[0,3],[3,3],\"<undefined>\",[3,3]],0,\"aaaa\",[\"whole\",\"iteration\",\"c\",\"e\"],[\"aaa\",\"\",\"<undefined>\",\"\"],[[0,3],[3,3],\"<undefined>\",[3,3]],true,true],[[\"aaa\",\"aaa\",\"\",\"\"],[[0,3],[0,3],[3,3],[3,3]],0,\"aaaa\",[\"whole\",\"iteration\",\"c\"],[\"aaa\",\"\",\"\"],[[0,3],[3,3],[3,3]],true,true]]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>)){2,3})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('!');m.groups.whole==='ababa'&&m.groups.c==='a'&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===4&&n.groups.whole===''&&n.groups.iteration===''&&n.groups.c===''&&n.groups.e===undefined&&n.indices.groups.c===n.indices[4]&&clone.source===r.source&&r.lastIndex===5"),Ok(Value::Boolean(true)));
}
#[test]
fn counted_nullable_string_class_deep_scopes_long_members_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"{}(([\q{{a|ab}}])|()){{4,5}}{}",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('ababaX');m.length===1004&&m[0]==='ababa'&&m[1]==='ababa'&&m[1000]==='ababa'&&m[1001]===''&&m[1002]===undefined&&m[1003]===''&&m.indices[1001][0]===5&&m.indices[1001][1]===5&&m.indices[1003][0]===5"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!(r"()((([\q{{|{word}}}])|()){{3,4}})()").as_str());
    let input = JsString::from(format!("{word}{word}!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===40000&&m[1]===''&&m[2].length===40000&&m[3]===''&&m[4]===''&&m[5]===undefined&&m[6]===''&&m.indices[3][0]===40000&&m.indices[3][1]===40000&&m.indices[6][0]===40000"
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
    realm
        .eval(r"var r=/[\q{|xx}]{20000}/vg,marker=0;r.lastIndex=1")
        .unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(input)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/(|[\q{ab}]){2}/dvgy,marker=0;r.lastIndex=1")
        .unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===2&&m.index===1&&m[0]===''&&m[1]===''&&m.indices[0][0]===1&&m.indices[1][0]===1&&m.indices[1][1]===1&&r.lastIndex===1"),Ok(Value::Boolean(true)));
}
#[test]
fn counted_nullable_string_class_proofs_keep_backtracking_predicates_references_and_unknown_leaves_closed()
 {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="[\\q{ab}&&\\q{cd}]{2}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",null,0],[false,\"\",null,0],[false,\"\\n\",null,0],[false,\"x\\nab\",null,0],[true,\"ab!\",null,0],[true,\"\",null,0],[true,\"\\n\",null,0],[true,\"x\\nab\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{a|ab}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[false,\"ab\",null,0],[false,\"\u017faX\",null,0],[false,\"\ud83d\ude00aX\",null,0],[true,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[true,\"ab\",null,0],[true,\"\u017faX\",null,0],[true,\"\ud83d\ude00aX\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()(([\\q{a|ab}])|()){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[2,3],\"<undefined>\",[3,3]],\"index\":0},3],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[false,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[2,3],\"<undefined>\",[3,3]],\"index\":0},3],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()(([\\q{a|ab}])|()){3}()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[3,6],[3,3],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":3},6],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[false,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[4,4],[4,4],[4,4],\"<undefined>\",[4,4],[4,4]],\"index\":4},4],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[3,6],[3,3],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":3},6],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()([[\q{a|ab}]&&\p{RGI_Emoji}]){2}()",
        r"[[\q{ab}&&\q{cd}]&&\p{RGI_Emoji}]{2}",
        r"^()(([[\q{a|ab}]&&\p{RGI_Emoji}])|()){3}()",
        r"()(([[\q{a|ab}]&&\p{RGI_Emoji}])|()){3}()$",
        r"()(([\q{a|ab}])|()){3}()\1",
        r"()(([\q{a|ab}])|()){3}()a",
        r"a()(([\q{a|ab}])|()){3}()",
        r"()((([\q{a|ab}])|()){2}){3}()",
        r"()(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){99999999999999999999}()",
        r"()(([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()){3}?()",
        r"()(([\q{a|ab}])|[\q{}]){3}()",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('ababa')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
