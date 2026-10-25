//! Simple repetition retains whole and final-iteration capture scopes.
use spite_core::JsString;
use spite_runtime::{Realm, Value};
fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}
#[test]
fn simple_repeated_string_class_native_tables_and_consumers_keep_iteration_and_empty_identity() {
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>))+)(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"a\",\"a\",\"<undefined>\",\"\"],[[0,5],[0,0],[0,5],[4,5],[4,5],\"<undefined>\",[5,5]],0,5]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}]))+)(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"a\",\"<undefined>\",\"a\",\"\"],[[0,5],[0,0],[0,5],[4,5],\"<undefined>\",[4,5],[5,5]],0,5]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}]))+?)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>))*?)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}])|(?<e>))?)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"ab\",\"ab\",\"<undefined>\",\"\"],[[0,2],[0,0],[0,2],[0,2],[0,2],\"<undefined>\",[2,2]],0,2]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<e>)|(?<c>[\\q{a|ab}]))??)(?<after>)",input="ab",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "e", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>))+)(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{|a|ab}])|(?<e>))*)(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"<undefined>\",\"<undefined>\",\"<undefined>\",\"\"],[[0,0],[0,0],[0,0],\"<undefined>\",\"<undefined>\",\"<undefined>\",[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{aa|aaa}])|(?<e>))*)(?<after>)",input="aaaa",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"aaa\",\"\",\"aaa\",\"aaa\",\"aaa\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[0,3],[0,3],\"<undefined>\",[3,3]],0,3]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}]))+)(?<after>)",input="!ababX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"abab\",\"\",\"abab\",\"ab\",\"ab\",\"\"],[[1,5],[1,1],[1,5],[3,5],[3,5],[5,5]],1,5]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{a|ab}]))+)(?<after>)",input="!ababX",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\ud83d\ude00|\ud83d\ude00a}]))+)(?<after>)",input="\ud83d\ude00a\ud83d\ude00!",r=new RegExp(pattern,"dgvy"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=1;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\ud83d\ude00a\ud83d\ude00\",\"\",\"\ud83d\ude00a\ud83d\ude00\",\"\ud83d\ude00\",\"\ud83d\ude00\",\"\"],[[0,5],[0,0],[0,5],[3,5],[3,5],[5,5]],0,5]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{s|sa|k|ka}])|(?<e>))+)(?<after>)",input="\u017fa\u212a!",r=new RegExp(pattern,"dgvi"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\u017fa\u212a\",\"\",\"\u017fa\u212a\",\"\u212a\",\"\u212a\",\"<undefined>\",\"\"],[[0,3],[0,0],[0,3],[2,3],[2,3],\"<undefined>\",[3,3]],0,3]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab}]))+)(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);ok&&m===null&&r.lastIndex===0"###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<c>[\\q{}]?))(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<c>[\\q{}]+))(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],[0,0]],0,0]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<c>[\\q{a|ab}]+))(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ababa\",\"\",\"ababa\",\"ababa\",\"\"],[[0,5],[0,0],[0,5],[0,5],[5,5]],0,5]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<c>[\\q{a|ab}]+?))(?<after>)",input="ababaX",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"ab\",\"\",\"ab\",\"ab\",\"\"],[[0,2],[0,0],[0,2],[0,2],[2,2]],0,2]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{\\n|\\na}]))*)(?<after>)",input="\na",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\\na\",\"\",\"\\na\",\"\\na\",\"\\na\",\"\"],[[0,2],[0,0],[0,2],[0,2],[0,2],[2,2]],0,2]""###,
    );
    check(
        r###"var pattern="(?<before>)(?<whole>(?<iteration>(?<c>[\\q{ab}&&\\q{cd}])|(?<e>))+)(?<after>)",input="!",r=new RegExp(pattern,"dgv"),ok=r.source===pattern&&r.global&&r.hasIndices&&r.unicodeSets===true&&r.unicode===false;r.lastIndex=0;var m=r.exec(input);if(m!==null){var names=["before", "whole", "iteration", "c", "e", "after"];for(var n of names){var slot=names.indexOf(n)+1;ok=ok&&Object.hasOwn(m.groups,n)&&Object.hasOwn(m.indices.groups,n)&&m.groups[n]===m[slot]&&m.indices.groups[n]===m.indices[slot]}ok=ok&&Object.getPrototypeOf(m.groups)===null&&m.input===input}else ok=false;ok&&m!==null&&JSON.stringify([Array.from(m,v=>v===undefined?"<undefined>":v),Array.from(m.indices,v=>v===undefined?"<undefined>":v),m.index,r.lastIndex])==="[[\"\",\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],[[0,0],[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],0,0]""###,
    );
    check(
        r###"var r=/(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>))+)/dvg,a=Array.from('abXaa'.matchAll(r));a.length===4&&a.map(m=>m.index).join(',')==='0,2,3,5'&&a.map(m=>m[0]).join('|')==='ab||aa|'&&a.map(m=>m.groups.iteration).join('|')==='ab||a|'&&a.every(m=>m.groups.whole===m[0]&&m.groups.iteration===m.groups.c&&m.groups.e===undefined&&m.indices.groups.c===m.indices[3])&&r.lastIndex===0"###,
    );
    check(
        r###"var calls=0,r=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}]))+)/vg,out='abXaa'.replace(r,function(all,w,it,e,c,offset,input,groups){calls++;return arguments.length===8&&w===all&&it===(c===undefined?'':c)&&groups.c===c&&input==='abXaa'?(c===undefined?'<E>':'<C:'+c+'>'):'bad'});out==='<C:ab><E>X<C:a><E>'&&calls===4"###,
    );
    check(
        r###"var r=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{a|ab}]))+)/vg;r.lastIndex=9;var a='abXaa'.split(r);'!ab'.search(r)===0&&r.lastIndex===9&&a.length===11&&a[0]===''&&a[1]==='ab'&&a[2]==='ab'&&a[3]===undefined&&a[4]==='ab'&&a[5]==='X'&&a[6]==='aa'&&a[7]==='a'&&a[8]===undefined&&a[9]==='a'&&a[10]===''"###,
    );
    check(
        r###"var r=/[\q{a|ab}]+/vg;Object.defineProperty(r,'lastIndex',{writable:false});var caught=false;try{r.exec('ab')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0"###,
    );
    check(
        r###"var r=/(?<\u{1D49C}>)(?<whole>(?<iteration>(?<c>[\q{😀|😀a}]))+)(?<after>)/dvgy;r.lastIndex=1;var m=r.exec('😀a😀!');m.index===0&&m.groups.𝒜===''&&m.groups.whole==='😀a😀'&&m.groups.iteration==='😀'&&m.groups.c==='😀'&&m.groups.after===''&&m.indices.groups.iteration===m.indices[3]&&m.indices[3][0]===3&&m.indices[3][1]===5&&m.indices.groups.after[0]===5&&r.lastIndex===5&&r.source.indexOf('1D49C')!==-1"###,
    );
    check(
        r###"var m=/(?<before>)(?<whole>(?<iteration>(?<c>[\q{s|sa|k|ka}]))+)(?<after>)/dvi.exec('ſaK!');m.groups.whole==='ſaK'&&m.groups.iteration==='K'&&m.groups.c==='K'&&m.groups.before===''&&m.groups.after===''&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===2&&m.indices[4][1]===3"###,
    );
    check(
        r###"var z=/(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>))*?)/dv.exec('ab'),p=/(?<whole>(?<iteration>(?<e>)|(?<c>[\q{|a|ab}]))+?)/dv.exec('ab');z.groups.whole===''&&z.groups.iteration===undefined&&z.groups.c===undefined&&z.groups.e===undefined&&z.indices[1][0]===0&&z.indices[2]===undefined&&p.groups.whole===''&&p.groups.iteration===''&&p.groups.e===''&&p.groups.c===undefined&&p.indices.groups.e[0]===0"###,
    );
    check(
        r###"var m=/(?<whole>(?<iteration>(?<c>[\q{aa|aaa}])|(?<e>))*)/dv.exec('aaaa');m[0]==='aaa'&&m.groups.whole==='aaa'&&m.groups.iteration==='aaa'&&m.groups.c==='aaa'&&m.groups.e===undefined&&m.indices.groups.whole[1]===3"###,
    );

    let mut realm = Realm::default();
    realm.eval(r"var r=/(?<before>)(?<whole>(?<iteration>(?<c>[\q{|a|ab}])|(?<e>))+)(?<after>)/dvg,clone=new RegExp(r),m=r.exec('ababa')").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("var n=clone.exec('!');m.groups.whole==='ababa'&&m.groups.c==='a'&&m.indices.groups.c===m.indices[4]&&m.indices[4][0]===4&&n.groups.whole===''&&n.groups.iteration===''&&n.groups.c===''&&n.groups.e===undefined&&n.indices.groups.c===n.indices[4]&&clone.source===r.source&&r.lastIndex===5"),Ok(Value::Boolean(true)));
}

#[test]
fn simple_repeated_string_class_proofs_keep_counts_predicates_references_and_unknown_leaves_closed()
{
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()(([\\q{a|ab}])|()){2}()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['ab!','','\n','x\nab']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"ab!\",{\"values\":[\"ab\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,2],[0,0],[2,2],\"<undefined>\",[2,2],[2,2]],\"index\":0},2],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[false,\"\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[false,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"ab!\",{\"values\":[\"ab\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,2],[0,0],[2,2],\"<undefined>\",[2,2],[2,2]],\"index\":0},2],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"x\\nab\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="^()(([\\q{a|ab}])|())+()",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[0,0],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[false,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[0,0],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    check(
        r#"var marker=0,ok=false;try{ok=(function(){var pattern="()(([\\q{a|ab}])|())+()$",rows=[],metadata=true;
for(var multi of [false,true]){var r=new RegExp(pattern,'dvg'+(multi?'m':''));metadata=metadata&&r.source===pattern;
for(var input of ['aaaaaa','ababa','\nab\n','']){r.lastIndex=0;var m=r.exec(input),result=null;if(m!==null)result={values:Array.from(m,v=>v===undefined?'<undefined>':v),indices:Array.from(m.indices,v=>v===undefined?'<undefined>':v),index:m.index};rows.push([multi,input,result,r.lastIndex]);}}
return metadata&&JSON.stringify(rows)==="[[false,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[0,0],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[false,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[false,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[4,4],[4,4],[4,4],\"<undefined>\",[4,4],[4,4]],\"index\":4},4],[false,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"aaaaaa\",{\"values\":[\"aaaaaa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,6],[0,0],[5,6],[5,6],\"<undefined>\",[6,6]],\"index\":0},6],[true,\"ababa\",{\"values\":[\"ababa\",\"\",\"a\",\"a\",\"<undefined>\",\"\"],\"indices\":[[0,5],[0,0],[4,5],[4,5],\"<undefined>\",[5,5]],\"index\":0},5],[true,\"\\nab\\n\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0],[true,\"\",{\"values\":[\"\",\"\",\"\",\"<undefined>\",\"\",\"\"],\"indices\":[[0,0],[0,0],[0,0],\"<undefined>\",[0,0],[0,0]],\"index\":0},0]]"})();}finally{marker=3}ok&&marker===3"#,
    );
    for pattern in [
        r"()(([[\q{a|ab}]&&\p{RGI_Emoji}])|()){2}()",
        r"^()(([[\q{a|ab}]&&\p{RGI_Emoji}])|())+()",
        r"()(([[\q{a|ab}]&&\p{RGI_Emoji}])|())+()$",
        r"()(([\q{a|ab}])|())+()\1",
        r"()(([\q{a|ab}])|())+()a",
        r"a()(([\q{a|ab}])|())+()",
        r"()((([\q{a|ab}])|())+)+()",
        r"()(([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|())*()",
        r"()(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]))+?()",
        r"()(([\q{a|ab}])|[\q{}])+()",
    ] {
        let mut realm = Realm::default();
        assert!(
            crate::common::regexp_oracle::check(
                &mut realm,
                &format!(
                    "var marker=0;try{{new RegExp({:?},'v').test('ababa')}}catch{{marker=1}}finally{{marker=2}};marker",
                    JsString::from(pattern)
                )
            ),
            "{pattern}"
        );
    }
}
