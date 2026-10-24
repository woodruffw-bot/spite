//! Required counts retain successful starts and final-word captures.
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
fn required_string_class_native_tables_consumers_and_successful_start_identity() {
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3,4}?)(?<after>)",input="aaaaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaaaa\",\"\",\"aaaaaaaa\",\"aa\",\"aa\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){3})(?<after>)",input="ababa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])){3})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){3})(?<after>)",input="abcab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcab\",\"\",\"abcab\",\"ab\",\"ab\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|abc|c}])){3,4})(?<after>)",input="abcabcab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcabcab\",\"\",\"abcabcab\",\"ab\",\"ab\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){3,4})(?<after>)",input="aaaaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaaaa\",\"\",\"aaaaaaaa\",\"aa\",\"aa\",\"\"],[[0,8],[0,0],[0,8],[6,8],[6,8],[8,8]],0,8]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|aa}])){3,4}?)(?<after>)",input="aaaaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>)",input="!aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[1,7],[1,1],[1,7],[5,7],[5,7],[7,7]],1,7]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3})(?<after>)",input="!aaaaaa",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a|aX}])){3})(?<after>)",input="\ud83d\ude00aX\ud83d\ude00",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00aX\ud83d\ude00\",\"\",\"\ud83d\ude00aX\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{S|sa|aX}])){3})(?<after>)",input="\u017faX\u017f",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017faX\u017f\",\"\",\"\u017faX\u017f\",\"\u017f\",\"\u017f\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3})(?<after>)",input="aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3}?)(?<after>)",input="abc",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abc\",\"\",\"abc\",\"c\",\"c\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[a-z\\q{ab}])){3})(?<after>)",input="aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aba\",\"\",\"aba\",\"a\",\"a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\p{Any}\\q{ab}])){3})(?<after>)",input="\ud83d\ude00aba",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00aba\",\"\",\"\ud83d\ude00aba\",\"a\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])){3,999999999999999999999999999999})(?<after>)",input="aaaaaab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aa\",\"aa\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{aa|aaa}]{3}))(?<after>)",input="aaaaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaaaaa\",\"\",\"aaaaaa\",\"aaaaaa\",\"\"],[[0,6],[0,0],[0,6],[0,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|aba}])){3})(?<after>)",input="abababa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abababa\",\"\",\"abababa\",\"aba\",\"aba\",\"\"],[[0,7],[0,0],[0,7],[4,7],[4,7],[7,7]],0,7]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\(|\\(a|aX}])){3})(?<after>)",input="(aX(",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"(aX(\",\"\",\"(aX(\",\"(\",\"(\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{aa|aaa}]){3,4})/dvg,a=Array.from('!aaaaaaXaaaaaaaaaa!'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"aaaaaa\",\"aaaaaa\",\"aa\"],[[1,7],[1,7],[5,7]],1,\"!aaaaaaXaaaaaaaaaa!\",[\"whole\",\"word\"],[\"aaaaaa\",\"aa\"],[[1,7],[5,7]],true,true],[[\"aaaaaaaaa\",\"aaaaaaaaa\",\"aaa\"],[[8,17],[8,17],[14,17]],8,\"!aaaaaaXaaaaaaaaaa!\",[\"whole\",\"word\"],[\"aaaaaaaaa\",\"aaa\"],[[8,17],[14,17]],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/(?<whole>(?<word>[\q{aa|aaa}]){3,4})/vg,out='!aaaaaaXaaaaaaaaaa!'.replace(r,function(all,w,word,offset,input,g){calls.push([all,w,word,offset,input,arguments.length,g.word===word,Object.getPrototypeOf(g)===null]);return '['+word+']'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"![aa]X[aaa]a!\",[[\"aaaaaa\",\"aaaaaa\",\"aa\",1,\"!aaaaaaXaaaaaaaaaa!\",6,true,true],[\"aaaaaaaaa\",\"aaaaaaaaa\",\"aaa\",8,\"!aaaaaaXaaaaaaaaaa!\",6,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{ab|abc|c}]){3})/dvg;r.lastIndex=9;var a='!abcab'.split(r),at='!abcab'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"!\",\"abcab\",\"ab\",\"\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/[\q{aa|aaa}]{3}/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('xaaaa')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<\u{1D49C}>)(?<whole>(?<word>[\q{😀|😀a|aX}]){3})(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀aX😀!');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[[[\"\ud83d\ude00aX\ud83d\ude00\",\"\",\"\ud83d\ude00aX\ud83d\ude00\",\"\ud83d\ude00\",\"\"],[[0,6],[0,0],[0,6],[4,6],[6,6]],0,\"\ud83d\ude00aX\ud83d\ude00!\",[\"\ud835\udc9c\",\"whole\",\"word\",\"after\"],[\"\",\"\ud83d\ude00aX\ud83d\ude00\",\"\ud83d\ude00\",\"\"],[[0,0],[0,6],[4,6],[6,6]],true,true],6,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>)(?<whole>(?<word>[\q{S|sa|aX}]){3,4})(?<after>)/dvi,m=r.exec('ſaXſ!');return JSON.stringify([row(m)]);})()==="[[[\"\u017faX\u017f\",\"\",\"\u017faX\u017f\",\"\u017f\",\"\"],[[0,4],[0,0],[0,4],[3,4],[4,4]],0,\"\u017faX\u017f!\",[\"before\",\"whole\",\"word\",\"after\"],[\"\",\"\u017faX\u017f\",\"\u017f\",\"\"],[[0,0],[0,4],[3,4],[4,4]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var g=/(?<whole>(?<word>[\q{aa|aaa}]){3,4})/dv.exec('aaaaaaaa!'),l=/(?<whole>(?<word>[\q{aa|aaa}]){3,4}?)/dv.exec('aaaaaaaa!');return JSON.stringify([row(g),row(l)]);})()==="[[[\"aaaaaaaa\",\"aaaaaaaa\",\"aa\"],[[0,8],[0,8],[6,8]],0,\"aaaaaaaa!\",[\"whole\",\"word\"],[\"aaaaaaaa\",\"aa\"],[[0,8],[6,8]],true,true],[[\"aaaaaaaa\",\"aaaaaaaa\",\"aa\"],[[0,8],[0,8],[6,8]],0,\"aaaaaaaa!\",[\"whole\",\"word\"],[\"aaaaaaaa\",\"aa\"],[[0,8],[6,8]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{ab|abc|c}]){3})/dvg,s=/(?<whole>(?<word>[\q{ab|abc|c}]){3})/dvgy,m=r.exec('!abcab'),n=s.exec('!abcab');s.lastIndex=1;var p=s.exec('!abcab');return JSON.stringify([row(m),row(n),row(p),r.lastIndex,s.lastIndex]);})()==="[[[\"abcab\",\"abcab\",\"ab\"],[[1,6],[1,6],[4,6]],1,\"!abcab\",[\"whole\",\"word\"],[\"abcab\",\"ab\"],[[1,6],[4,6]],true,true],null,[[\"abcab\",\"abcab\",\"ab\"],[[1,6],[1,6],[4,6]],1,\"!abcab\",[\"whole\",\"word\"],[\"abcab\",\"ab\"],[[1,6],[4,6]],true,true],6,6]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{aa|aaa}]){3,4})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('aaaaaaaa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.whole==='aaaaaaaa'&&m.groups.word==='aa'&&m.indices.groups.word===m.indices[3]&&m.indices[3][0]===6&&m.indices[3][1]===8&&clone.exec('!')===null&&clone.source===r.source&&clone.lastIndex===0&&r.lastIndex===8"),Ok(Value::Boolean(true)));
}
#[test]
fn required_string_class_deep_scopes_long_members_required_storage_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"{}([\q{{aa|aaa}}]){{3,4}}{}",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('aaaaaa!');m.length===1002&&m[0]==='aaaaaa'&&m[1]==='aaaaaa'&&m[1000]==='aaaaaa'&&m[1001]==='aa'&&m.indices[1001][0]===4&&m.indices[1001][1]===6"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!(r"()([\q{{{word}|{word}x}}]){{3}}()").as_str());
    let input = JsString::from(format!("{word}{word}{word}!").as_str());
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
    realm.eval(r"var r=/[\q{a|aa}]{3,}/vg,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "a".repeat(60_000)))
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
    realm.eval(r"var r=/[\q{a|aa}]{3}/vgy,marker=0").unwrap();
    realm
        .eval(&format!("var input='aaa{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===1&&m.index===0&&m[0]==='aaa'&&r.lastIndex===3"),Ok(Value::Boolean(true)));
}
#[test]
fn required_string_class_proofs_keep_predicates_references_consuming_siblings_and_unknown_leaves_closed()
 {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()([\\q{aa|aaa}]){3}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){3}()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[false,\"ababa\",null,0],[false,\"\\nab\\n\",null,0],[false,\"\",null,0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,6],[0,0],[4,6],[6,6]],\"index\":0},6],[true,\"ababa\",null,0],[true,\"\\nab\\n\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"^()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()",
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){3}()$",
        r"()([\q{aa|aaa}]){3}()\1",
        r"()([\q{aa|aaa}]){3}()a",
        r"a()([\q{aa|aaa}]){3}()",
        r"()(([\q{aa|aaa}]){3}){3}()",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){3}()",
        r"()([\q{aa|aaa}]){3}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('aaaaaa')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
}
