/*---
description: Supplemental published-algorithm probes; no original Test262 credit.
---*/
for(const C of [ArrayBuffer,SharedArrayBuffer]) {
 for(const n of [7*1125899906842624,9007199254740991]) {
  for(const growable of [false,true]) {
   const b=growable?new C(n,{maxByteLength:n}):new C(n);
   assert.sameValue(b.byteLength,n);
   const d=new DataView(b);assert.sameValue(d.getUint8(n-1),0);
   d.setUint8(n-1,123);assert.sameValue(d.getUint8(n-1),123);
  }
 }
}
for(const C of [Int8Array,Uint8Array,Uint8ClampedArray]) {
 const marker={};let reached=false;
 try { new C({length:9007199254740992,get 0(){reached=true;throw marker}});throw 1; }
 catch(e){assert.sameValue(e,marker)}
 assert.sameValue(reached,true);
}
for(const method of ['take','drop']) {
 for(const limit of [9007199254740992,1e100,Number.MAX_VALUE,Infinity]) {
  const trace=[];let closed=0;
  const iterator={get next(){trace.push('next');return ()=>({done:true})},return(){closed++;return {done:true}}};
  const helper=Iterator.prototype[method].call(iterator,{valueOf(){trace.push('limit');return limit}});
  assert.sameValue(trace.join(','),'limit,next');assert.sameValue(closed,0);
  assert.sameValue(helper.next().done,true);assert.sameValue(closed,0);
 }
}
for(const key of ['__defineGetter__','__defineSetter__','__lookupGetter__','__lookupSetter__','__proto__'])assert.sameValue(Object.hasOwn(Object.prototype,key),false);
function f(){};assert.sameValue(Object.hasOwn(f,'caller'),false);assert.sameValue(Object.hasOwn(f,'arguments'),false);
const caller=Object.getOwnPropertyDescriptor(Function.prototype,'caller');const args=Object.getOwnPropertyDescriptor(Function.prototype,'arguments');
assert.sameValue(caller.get,caller.set);assert.sameValue(caller.get,args.get);assert.sameValue(caller.get,args.set);
assert.throws(TypeError,()=>f.caller);
for(const cp of [0x200c,0x200d]) {
 assert.sameValue(new RegExp('^\\p{ID_Continue}$','u').test(String.fromCharCode(cp)),true);
 assert.throws(SyntaxError,()=>new RegExp('\\'+String.fromCharCode(cp)));
}
// Published Date.parse permits declining this non-standard fallback format.
assert.sameValue(Date.parse('1997-03-08 1:1:1.01'),NaN);
// EscapeRegExpPattern permits equivalent, deterministic source representations.
for(const flags of ['', 'u']) {
 const re=new RegExp('[/]',flags),source=re.source;
 const literal=eval('/'+source+'/'+flags);
 assert.sameValue(re.test('/'),true);assert.sameValue(literal.test('/'),true);
 assert.sameValue(re.test('x'),false);assert.sameValue(literal.test('x'),false);
 assert.sameValue(new RegExp('[/]',flags).source,source);
}
// All eight BMP mappings changed between the original Unicode17 table and UCD18.
for(const [small,capital] of [[0x0277,0xA7DD],[0x027C,0xA7E2],[0xAB4B,0xAB6C],[0xAB4C,0xAB6D]]) {
 assert.sameValue(String.fromCharCode(small).toUpperCase(),String.fromCharCode(capital));
 assert.sameValue(String.fromCharCode(capital).toLowerCase(),String.fromCharCode(small));
}
