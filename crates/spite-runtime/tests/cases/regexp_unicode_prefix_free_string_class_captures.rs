//! Prefix-free counts retain successful starts and final-word captures.
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
fn prefix_free_string_class_native_tables_consumers_and_successful_start_identity() {
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",input="abcd!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2,3})(?<after>)",input="abcdab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcdab\",\"\",\"abcdab\",\"ab\",\"ab\",\"\"],[[0,6],[0,0],[0,6],[4,6],[4,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2,3}?)(?<after>)",input="abcdab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[0,4],[0,0],[0,4],[2,4],[2,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cde}])){2})(?<after>)",input="abcde!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcde\",\"\",\"abcde\",\"cde\",\"cde\",\"\"],[[0,5],[0,0],[0,5],[2,5],[2,5],[5,5]],0,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{abcd|bc|da}])){2})(?<after>)",input="abcda",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"bcda\",\"\",\"bcda\",\"da\",\"da\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{abcd|bc|da}])){2})(?<after>)",input="abcda",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",input="!abcd",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcd\",\"\",\"abcd\",\"cd\",\"cd\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",input="!abcd",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00a|\ud83d\ude00b}])){2})(?<after>)",input="\ud83d\ude00a\ud83d\ude00b!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00a\ud83d\ude00b\",\"\",\"\ud83d\ude00a\ud83d\ude00b\",\"\ud83d\ude00b\",\"\ud83d\ude00b\",\"\"],[[0,6],[0,0],[0,6],[3,6],[3,6],[6,6]],0,6]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|bc}])){2,3})(?<after>)",input="abca!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abca\",\"\",\"abca\",\"a\",\"a\",\"\"],[[0,4],[0,0],[0,4],[3,4],[3,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{Sx|k}])){2})(?<after>)",input="\u017fx\u212a!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017fx\u212a\",\"\",\"\u017fx\u212a\",\"\u212a\",\"\u212a\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cd}])){2})(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<c>[\\q{ab|cd}]{2}))(?<after>)",input="abcd!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abcd\",\"\",\"abcd\",\"abcd\",\"\"],[[0,4],[0,0],[0,4],[0,4],[4,4]],0,4]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\n|ab}])){2})(?<after>)",input="\nab!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\\nab\",\"\",\"\\nab\",\"ab\",\"ab\",\"\"],[[0,3],[0,0],[0,3],[1,3],[1,3],[3,3]],0,3]""####,
    );
    check(
        r####"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab|cde}])){99999999999999999999})(?<after>)",input="abcde",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{ab|cd}]){2,3})/dvg,a=Array.from('!abcdXcdabab!'.matchAll(r));return JSON.stringify([a.map(row),r.lastIndex]);})()==="[[[[\"abcd\",\"abcd\",\"cd\"],[[1,5],[1,5],[3,5]],1,\"!abcdXcdabab!\",[\"whole\",\"word\"],[\"abcd\",\"cd\"],[[1,5],[3,5]],true,true],[[\"cdabab\",\"cdabab\",\"ab\"],[[6,12],[6,12],[10,12]],6,\"!abcdXcdabab!\",[\"whole\",\"word\"],[\"cdabab\",\"ab\"],[[6,12],[10,12]],true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var calls=[],r=/(?<whole>(?<word>[\q{ab|cd}]){2,3})/vg,out='!abcdXcdabab!'.replace(r,function(all,w,word,offset,input,g){calls.push([all,w,word,offset,input,arguments.length,g.word===word,Object.getPrototypeOf(g)===null]);return '['+word+']'});return JSON.stringify([out,calls,r.lastIndex]);})()==="[\"![cd]X[ab]!\",[[\"abcd\",\"abcd\",\"cd\",1,\"!abcdXcdabab!\",6,true,true],[\"cdabab\",\"cdabab\",\"ab\",6,\"!abcdXcdabab!\",6,true,true]],0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{abcd|bc|da}]){2})/dvg;r.lastIndex=9;var a='abcda'.split(r),at='abcda'.search(r);return JSON.stringify([a.map(v=>v===undefined?'<undefined>':v),at,r.lastIndex]);})()==="[[\"a\",\"bcda\",\"da\",\"\"],1,9]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/[\q{ab|cd}]{2}/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('xabcd')}catch(e){caught=e instanceof TypeError}return JSON.stringify([caught,r.lastIndex]);})()==="[true,0]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<\u{1D49C}>)(?<whole>(?<word>[\q{😀a|😀b}]){2})(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀a😀b!');return JSON.stringify([row(m),r.lastIndex,r.source.indexOf('1D49C')!==-1]);})()==="[[[\"\ud83d\ude00a\ud83d\ude00b\",\"\",\"\ud83d\ude00a\ud83d\ude00b\",\"\ud83d\ude00b\",\"\"],[[0,6],[0,0],[0,6],[3,6],[6,6]],0,\"\ud83d\ude00a\ud83d\ude00b!\",[\"\ud835\udc9c\",\"whole\",\"word\",\"after\"],[\"\",\"\ud83d\ude00a\ud83d\ude00b\",\"\ud83d\ude00b\",\"\"],[[0,0],[0,6],[3,6],[6,6]],true,true],6,true]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<before>)(?<whole>(?<word>[\q{Sx|k}]){2,3})(?<after>)/dvi,m=r.exec('ſxKk!');return JSON.stringify([row(m)]);})()==="[[[\"\u017fx\u212ak\",\"\",\"\u017fx\u212ak\",\"k\",\"\"],[[0,4],[0,0],[0,4],[3,4],[4,4]],0,\"\u017fx\u212ak!\",[\"before\",\"whole\",\"word\",\"after\"],[\"\",\"\u017fx\u212ak\",\"k\",\"\"],[[0,0],[0,4],[3,4],[4,4]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var g=/(?<whole>(?<word>[\q{ab|cd}]){2,3})/dv.exec('abcdab!'),l=/(?<whole>(?<word>[\q{ab|cd}]){2,3}?)/dv.exec('abcdab!');return JSON.stringify([row(g),row(l)]);})()==="[[[\"abcdab\",\"abcdab\",\"ab\"],[[0,6],[0,6],[4,6]],0,\"abcdab!\",[\"whole\",\"word\"],[\"abcdab\",\"ab\"],[[0,6],[4,6]],true,true],[[\"abcd\",\"abcd\",\"cd\"],[[0,4],[0,4],[2,4]],0,\"abcdab!\",[\"whole\",\"word\"],[\"abcd\",\"cd\"],[[0,4],[2,4]],true,true]]""####,
    );
    check(
        r####"(function(){function row(m){if(m===null)return null;var names=m.groups===undefined?[]:Object.keys(m.groups);return [Array.from(m,v=>v===undefined?'<undefined>':v),Array.from(m.indices,v=>v===undefined?'<undefined>':v),m.index,m.input,names,names.map(n=>m.groups[n]===undefined?'<undefined>':m.groups[n]),names.map(n=>m.indices.groups[n]===undefined?'<undefined>':m.indices.groups[n]),names.every((n,i)=>Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[i+1]&&m.indices.groups[n]===m.indices[i+1]),m.groups===undefined||Object.getPrototypeOf(m.groups)===null]}var r=/(?<whole>(?<word>[\q{abcd|bc|da}]){2})/dvg,s=/(?<whole>(?<word>[\q{abcd|bc|da}]){2})/dvgy,m=r.exec('abcda'),n=s.exec('abcda');s.lastIndex=1;var p=s.exec('abcda');return JSON.stringify([row(m),row(n),row(p),r.lastIndex,s.lastIndex]);})()==="[[[\"bcda\",\"bcda\",\"da\"],[[1,5],[1,5],[3,5]],1,\"abcda\",[\"whole\",\"word\"],[\"bcda\",\"da\"],[[1,5],[3,5]],true,true],null,[[\"bcda\",\"bcda\",\"da\"],[[1,5],[1,5],[3,5]],1,\"abcda\",[\"whole\",\"word\"],[\"bcda\",\"da\"],[[1,5],[3,5]],true,true],5,5]""####,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<word>[\q{ab|cd}]){2,3})(?<after>)/dvg,clone=new RegExp(r),m=r.exec('abcdab')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("m.groups.whole==='abcdab'&&m.groups.word==='ab'&&m.indices.groups.word===m.indices[3]&&m.indices[3][0]===4&&m.indices[3][1]===6&&clone.exec('!')===null&&clone.source===r.source&&clone.lastIndex===0&&r.lastIndex===6"),Ok(Value::Boolean(true)));
}
#[test]
fn prefix_free_string_class_deep_scopes_long_members_search_and_host_aborts_remain_flat() {
    let pattern = JsString::from(
        format!(
            r"{}([\q{{ab|cd}}]){{2,3}}{}",
            "(".repeat(1000),
            "){1}".repeat(1000)
        )
        .as_str(),
    );
    check(&format!(
        "var r=new RegExp({pattern:?},'dv'),m=r.exec('abcdab!');m.length===1002&&m[0]==='abcdab'&&m[1]==='abcdab'&&m[1000]==='abcdab'&&m[1001]==='ab'&&m.indices[1001][0]===4&&m.indices[1001][1]===6"
    ));
    let word = "x".repeat(20_000);
    let pattern = JsString::from(format!(r"()([\q{{{word}a|{word}b}}]){{2}}()").as_str());
    let input = JsString::from(format!("{word}a{word}b!").as_str());
    check(&format!(
        "var m=new RegExp({pattern:?},'dv').exec({input:?});m[0].length===40002&&m[1]===''&&m[2].length===20001&&m[3]===''&&m.indices[2][0]===20001&&m.indices[2][1]===40002&&m.indices[3][0]===40002"
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
    realm.eval(r"var r=/[\q{xx}]{20000}/vg,marker=0").unwrap();
    realm
        .eval(&format!("var input='{}'", "x".repeat(40_000)))
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
    realm
        .eval(r"var r=/[\q{ab|cd}]{2,}?/dvgy,marker=0")
        .unwrap();
    realm
        .eval(&format!("var input='abcd{}'", "x".repeat(40_000)))
        .unwrap();
    assert_eq!(realm.eval("var m;try{m=r.exec(input)}finally{marker=3}marker===3&&m.length===1&&m.index===0&&m[0]==='abcd'&&m.indices[0][0]===0&&m.indices[0][1]===4&&r.lastIndex===4"),Ok(Value::Boolean(true)));
}
#[test]
fn prefix_free_string_class_proofs_keep_word_backtracking_predicates_references_and_unknown_leaves_closed()
 {
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{a|ab}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[false,\"ab\",null,0],[false,\"\u017faX\",null,0],[false,\"\ud83d\ude00aX\",null,0],[true,\"aaaa\",{\"values\":[\"aa\",\"\",\"a\",\"\"],\"indices\":[[0,2],[0,0],[1,2],[2,2]],\"index\":0},2],[true,\"ab\",null,0],[true,\"\u017faX\",null,0],[true,\"\ud83d\ude00aX\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()([\\q{aa|aaa}]){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[false,\"ab\",null,0],[false,\"\u017faX\",null,0],[false,\"\ud83d\ude00aX\",null,0],[true,\"aaaa\",{\"values\":[\"aaaa\",\"\",\"aa\",\"\"],\"indices\":[[0,4],[0,0],[2,4],[4,4]],\"index\":0},4],[true,\"ab\",null,0],[true,\"\u017faX\",null,0],[true,\"\ud83d\ude00aX\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\p{Any}\\q{ab}]){2}",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaa','ab','ſaX','😀aX']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaa\",{\"values\":[\"aa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"ab\",{\"values\":[\"ab\",\"b\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"\u017faX\",{\"values\":[\"\u017fa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[false,\"\ud83d\ude00aX\",{\"values\":[\"\ud83d\ude00a\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3],[true,\"aaaa\",{\"values\":[\"aa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"ab\",{\"values\":[\"ab\",\"b\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"\u017faX\",{\"values\":[\"\u017fa\",\"a\"],\"indices\":[[0,2],[1,2]],\"index\":0},2],[true,\"\ud83d\ude00aX\",{\"values\":[\"\ud83d\ude00a\",\"a\"],\"indices\":[[0,3],[2,3]],\"index\":0},3]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()([[\q{a|ab}]&&\p{RGI_Emoji}]){2}()",
        r"()([[\q{aa|aaa}]&&\p{RGI_Emoji}]){2}()",
        r"([[\p{Any}\q{ab}]&&\p{RGI_Emoji}]){2}",
        r"^()([\q{ab|cd}]){2}()",
        r"()([\q{ab|cd}]){2}()$",
        r"()([\q{ab|cd}]){2}()\1",
        r"()([\q{ab|cd}]){2}()a",
        r"a()([\q{ab|cd}]){2}()",
        r"()(([\q{ab|cd}]){2}){3}()",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){99999999999999999999}()",
        r"()([\q{ab|cd}]){2}()[\q{ef}]",
    ] {
        let mut realm = Realm::default();
        assert!(matches!(realm.eval(&format!("var marker=0;try{{new RegExp({:?},'v').test('abcd')}}catch{{marker=1}}finally{{marker=2}}",JsString::from(pattern))),Err(Error::Unsupported{..})),"{pattern}");
        assert_eq!(realm.eval("marker"), Ok(Value::Number(0.0)));
    }
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="([\\q{A|ab}]){2}",rows=[],metadata=true;for(var multi of [false,true]){var r=new RegExp(pattern,'dvgi'+(multi?'m':''));metadata=metadata&&r.source===pattern;for(var input of ['aab','Aab','ab','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}return metadata&&JSON.stringify(rows)==="[[false,\"aab\",{\"values\":[\"aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[false,\"Aab\",{\"values\":[\"Aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[false,\"ab\",null,0],[false,\"\",null,0],[true,\"aab\",{\"values\":[\"aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[true,\"Aab\",{\"values\":[\"Aab\",\"ab\"],\"indices\":[[0,3],[1,3]],\"index\":0},3],[true,\"ab\",null,0],[true,\"\",null,0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
}
