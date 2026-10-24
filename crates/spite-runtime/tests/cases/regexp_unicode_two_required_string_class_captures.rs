//! Two-required counts retain successful starts and final-word captures.
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
fn two_required_string_class_native_tables_consumers_and_successful_start_identity() {
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2})(?<after>)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2,3}?)(?<after>)",input="aaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaa\",\"\",\"aaaaa\",\"aa\",\"aa\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){2})(?<after>)",input="aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2})(?<after>)",input="abc",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){2,3})(?<after>)",input="abcab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcab\",\"\",\"abcab\",\"ab\",\"ab\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){2,3})(?<after>)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){2,3}?)(?<after>)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2})(?<after>)",input="!aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aa\",\"aa\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2})(?<after>)",input="!aaaa",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){2})(?<after>)",input="\ud83d\ude00aX",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){2})(?<after>)",input="\u017faX",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2}?)(?<after>)",input="abc",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"b\",\"b\",\"\"],[[0,2],[0,0],[0,2],[1,2],[1,2],[2,2]],0,2]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){2})(?<after>)",input="\ud83d\ude00ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00ab\",\"\",\"\ud83d\ude00ab\",\"ab\",\"ab\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){2,999999999999999999999999999999})(?<after>)",input="aaaaab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaa\",\"\",\"aaaaa\",\"aa\",\"aa\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{aa|aaa}]{2}))(?<after>)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaa\",\"\",\"aaaa\",\"aaaa\",\"\"],[[0,4],[0,0],[0,4],[0,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|aba}])){2})(?<after>)",input="ababa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"aba\",\"aba\",\"\"],[[0,5],[0,0],[0,5],[2,5],[2,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\(|\\(a|aX}])){2})(?<after>)",input="(aX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"(aX\",\"\",\"(aX\",\"aX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{aa|aaa}]){2,3})/dvg,a=Array.from('!aaaaXaaaaaa!'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"aaaa\",\"aaaa\",\"aa\"],[[1,5],[1,5],[3,5]],1,\"!aaaaXaaaaaa!\",[\"whole\",\"word\"],[\"aaaa\",\"aa\"],[[1,5],[3,5]],true,true],[[\"aaaaaa\",\"aaaaaa\",\"aaa\"],[[6,12],[6,12],[9,12]],6,\"!aaaaXaaaaaa!\",[\"whole\",\"word\"],[\"aaaaaa\",\"aaa\"],[[6,12],[9,12]],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/(?<whole>(?<word>[\q{aa|aaa}]){2,3})/vg,out='!aaaaXaaaaaa!'.replace(r,function(all,w,word,offset,input,g){calls.push([all,w,word,offset,input,arguments.length,g.word===word,Object.getPrototypeOf(g)===null]);return '['+word+']'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"![aa]X[aaa]!\",[[\"aaaa\",\"aaaa\",\"aa\",1,\"!aaaaXaaaaaa!\",6,true,true],[\"aaaaaa\",\"aaaaaa\",\"aaa\",6,\"!aaaaXaaaaaa!\",6,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{ab|abc|c}]){2})/dvg;r.lastIndex=9;var a='!abc'.split(r),at='!abc'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"!\",\"abc\",\"c\",\"\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/[\q{aa|aaa}]{2}/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('xaaaa')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<\u{1D49C}>)(?<whole>(?<word>[\q{😀|😀a|aX}]){2})(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀aX!');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[[[\"\ud83d\ude00aX\",\"\",\"\ud83d\ude00aX\",\"aX\",\"\"],[[0,4],[0,0],[0,4],[2,4],[4,4]],0,\"\ud83d\ude00aX!\",[\"\ud835\udc9c\",\"whole\",\"word\",\"after\"],[\"\",\"\ud83d\ude00aX\",\"aX\",\"\"],[[0,0],[0,4],[2,4],[4,4]],true,true],4,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>)(?<whole>(?<word>[\q{S|sa|aX}]){2,3})(?<after>)/dvi,m=r.exec('ſaX!');return JSON.stringify([row(m)]);})()==="[[[\"\u017faX\",\"\",\"\u017faX\",\"aX\",\"\"],[[0,3],[0,0],[0,3],[1,3],[3,3]],0,\"\u017faX!\",[\"before\",\"whole\",\"word\",\"after\"],[\"\",\"\u017faX\",\"aX\",\"\"],[[0,0],[0,3],[1,3],[3,3]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var g=/(?<whole>(?<word>[\q{aa|aaa}]){2,3})/dv.exec('aaaaaa!'),l=/(?<whole>(?<word>[\q{aa|aaa}]){2,3}?)/dv.exec('aaaaaa!');return JSON.stringify([row(g),row(l)]);})()==="[[[\"aaaaaa\",\"aaaaaa\",\"aaa\"],[[0,6],[0,6],[3,6]],0,\"aaaaaa!\",[\"whole\",\"word\"],[\"aaaaaa\",\"aaa\"],[[0,6],[3,6]],true,true],[[\"aaaaaa\",\"aaaaaa\",\"aaa\"],[[0,6],[0,6],[3,6]],0,\"aaaaaa!\",[\"whole\",\"word\"],[\"aaaaaa\",\"aaa\"],[[0,6],[3,6]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{ab|abc|c}]){2})/dvg,s=/(?<whole>(?<word>[\q{ab|abc|c}]){2})/dvgy,m=r.exec('!abc'),n=s.exec('!abc');s.lastIndex=1;var p=s.exec('!abc');return JSON.stringify([row(m),row(n),row(p),r.lastIndex,s.lastIndex]);})()==="[[[\"abc\",\"abc\",\"c\"],[[1,4],[1,4],[3,4]],1,\"!abc\",[\"whole\",\"word\"],[\"abc\",\"c\"],[[1,4],[3,4]],true,true],null,[[\"abc\",\"abc\",\"c\"],[[1,4],[1,4],[3,4]],1,\"!abc\",[\"whole\",\"word\"],[\"abc\",\"c\"],[[1,4],[3,4]],true,true],4,4]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{aa|aaa}]){2,3})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('aaaaaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.whole==='aaaaaa'&&m.groups.word==='aaa'&&m.indices.groups.word===m.indices[3]&&m.indices[3][0]===3&&m.indices[3][1]===6&&clone.exec('!')===null&&clone.source===r.source&&clone.lastIndex===0&&r.lastIndex===6"),Ok(Value::Boolean(true)));
}
#[test]
fn two_required_string_class_deep_scopes_long_members_search_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"{}([\q{{aa|aaa}}]){{2,3}}{}",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('aaaaaa!');m.length===1002&&m[0]==='aaaaaa'&&m[1]==='aaaaaa'&&m[1000]==='aaaaaa'&&m[1001]==='aaa'&&m.indices[1001][0]===3&&m.indices[1001][1]===6"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!(r"()([\q{{{word}|{word}x}}]){{2}}()").as_str());
    let input = JsString::from(format!("{word}{word}!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===40000&&m[1]===''&&m[2].length===20000&&m[3]===''&&m.indices[2][0]===20000&&m.indices[2][1]===40000&&m.indices[3][0]===40000"
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
    realm.eval(r"var r=/[\q{aa|aaa}]{2,}/vg,marker=0").unwrap();
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
    realm.eval(r"var r=/[\q{a|aa}]{2}?/dvgy,marker=0").unwrap();
    realm
        .eval(&format!("var input='aa{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===1&&m.index===0&&m[0]==='aa'&&m.indices[0][0]===0&&m.indices[0][1]===2&&r.lastIndex===2"),Ok(Value::Boolean(true)));
}
#[test]
fn two_required_string_class_proofs_keep_other_minima_predicates_references_and_unknown_leaves_closed()
 {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','ſaXſ','😀aX😀']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\u017faX\u017f\",null,0],[false,\"\ud83d\ude00aX\ud83d\ude00\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\u017faX\u017f\",null,0],[true,\"\ud83d\ude00aX\ud83d\ude00\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\p{Any}\\q{ab}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','ſaXſ','😀aX😀']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[5,5]],\"index\":0},5],[false,\"\u017faX\u017f\",{\"values\":[\"\u017faX\",\"\",\"X\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[false,\"\ud83d\ude00aX\ud83d\ude00\",{\"values\":[\"\ud83d\ude00aX\",\"\",\"X\",\"\"],\"indices\":[[0,4],[0,0],[3,4],[4,4]],\"index\":0},4],[true,\"aaaaaa\",{\"values\":[\"aaa\",\"\",\"a\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[5,5]],\"index\":0},5],[true,\"\u017faX\u017f\",{\"values\":[\"\u017faX\",\"\",\"X\",\"\"],\"indices\":[[0,3],[0,0],[2,3],[3,3]],\"index\":0},3],[true,\"\ud83d\ude00aX\ud83d\ude00\",{\"values\":[\"\ud83d\ude00aX\",\"\",\"X\",\"\"],\"indices\":[[0,4],[0,0],[3,4],[4,4]],\"index\":0},4]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()",
        r"()([[\p{Any}\q{ab}]&&\p{RGI_Emoji}]){3}()",
        r"^()([\q{aa|aaa}]){2}()",
        r"()([\q{aa|aaa}]){2}()$",
        r"()([\q{aa|aaa}]){2}()\1",
        r"()([\q{aa|aaa}]){2}()a",
        r"a()([\q{aa|aaa}]){2}()",
        r"()(([\q{aa|aaa}]){2}){3}()",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2}()",
        r"()([\q{aa|aaa}]){2}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('aaaa')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
