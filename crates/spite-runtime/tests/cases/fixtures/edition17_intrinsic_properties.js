// Property headings from ECMA-262 edition 17, tc39/ecma262@0248456c758431e4bb8e5d26333ff1865123c9cd.
// Run after edition17_intrinsic_inventory.js has initialized rows, GF, AGF, AF and TA.
// Rows: label, owner, method/getter/setter/data keys. @@ keys are well-known Symbols.
// Expands NativeError and TypedArray templates; excludes normative-optional legacy Object accessors,
// Annex B web features and the internal-only AsyncFromSync/ForIn iterator prototypes.
'use strict';
let intrinsicObjects=Object.fromEntries(rows.map(r=>[r[0],r[1]]));
let GeneratorFunction=GF.constructor,AsyncGeneratorFunction=AGF.constructor,AsyncFunction=AF.constructor;
let propertyRows=[
["Object",Object,["assign","create","defineProperties","defineProperty","entries","freeze","fromEntries","getOwnPropertyDescriptor","getOwnPropertyDescriptors","getOwnPropertyNames","getOwnPropertySymbols","getPrototypeOf","groupBy","hasOwn","is","isExtensible","isFrozen","isSealed","keys","preventExtensions","seal","setPrototypeOf","values"],[],[],["prototype"]],
["Object.prototype",Object.prototype,["hasOwnProperty","isPrototypeOf","propertyIsEnumerable","toLocaleString","toString","valueOf"],[],[],["constructor"]],
["Function",Function,[],[],[],["prototype"]],
["Function.prototype",Function.prototype,["apply","bind","call","toString","@@hasInstance"],[],[],["constructor"]],
["Boolean",Boolean,[],[],[],["prototype"]],
["Boolean.prototype",Boolean.prototype,["toString","valueOf"],[],[],["constructor"]],
["Symbol",Symbol,["for","keyFor"],[],[],["asyncIterator","hasInstance","isConcatSpreadable","iterator","match","matchAll","prototype","replace","search","species","split","toPrimitive","toStringTag","unscopables"]],
["Symbol.prototype",Symbol.prototype,["toString","valueOf","@@toPrimitive"],["description"],[],["constructor","@@toStringTag"]],
["Error",Error,["isError"],[],[],["prototype"]],
["Error.prototype",Error.prototype,["toString"],[],[],["constructor","message","name"]],
["EvalError",EvalError,[],[],[],["prototype"]],
["RangeError",RangeError,[],[],[],["prototype"]],
["ReferenceError",ReferenceError,[],[],[],["prototype"]],
["SyntaxError",SyntaxError,[],[],[],["prototype"]],
["TypeError",TypeError,[],[],[],["prototype"]],
["URIError",URIError,[],[],[],["prototype"]],
["EvalError.prototype",EvalError.prototype,[],[],[],["constructor","message","name"]],
["RangeError.prototype",RangeError.prototype,[],[],[],["constructor","message","name"]],
["ReferenceError.prototype",ReferenceError.prototype,[],[],[],["constructor","message","name"]],
["SyntaxError.prototype",SyntaxError.prototype,[],[],[],["constructor","message","name"]],
["TypeError.prototype",TypeError.prototype,[],[],[],["constructor","message","name"]],
["URIError.prototype",URIError.prototype,[],[],[],["constructor","message","name"]],
["AggregateError",AggregateError,[],[],[],["prototype"]],
["AggregateError.prototype",AggregateError.prototype,[],[],[],["constructor","message","name"]],
["Number",Number,["isFinite","isInteger","isNaN","isSafeInteger","parseFloat","parseInt"],[],[],["EPSILON","MAX_SAFE_INTEGER","MAX_VALUE","MIN_SAFE_INTEGER","MIN_VALUE","NaN","NEGATIVE_INFINITY","POSITIVE_INFINITY","prototype"]],
["Number.prototype",Number.prototype,["toExponential","toFixed","toLocaleString","toPrecision","toString","valueOf"],[],[],["constructor"]],
["BigInt",BigInt,["asIntN","asUintN"],[],[],["prototype"]],
["BigInt.prototype",BigInt.prototype,["toLocaleString","toString","valueOf"],[],[],["constructor","@@toStringTag"]],
["Math",Math,["abs","acos","acosh","asin","asinh","atan","atanh","atan2","cbrt","ceil","clz32","cos","cosh","exp","expm1","floor","fround","f16round","hypot","imul","log","log1p","log10","log2","max","min","pow","random","round","sign","sin","sinh","sqrt","sumPrecise","tan","tanh","trunc"],[],[],["E","LN10","LN2","LOG10E","LOG2E","PI","SQRT1_2","SQRT2","@@toStringTag"]],
["Date",Date,["now","parse","UTC"],[],[],["prototype"]],
["Date.prototype",Date.prototype,["getDate","getDay","getFullYear","getHours","getMilliseconds","getMinutes","getMonth","getSeconds","getTime","getTimezoneOffset","getUTCDate","getUTCDay","getUTCFullYear","getUTCHours","getUTCMilliseconds","getUTCMinutes","getUTCMonth","getUTCSeconds","setDate","setFullYear","setHours","setMilliseconds","setMinutes","setMonth","setSeconds","setTime","setUTCDate","setUTCFullYear","setUTCHours","setUTCMilliseconds","setUTCMinutes","setUTCMonth","setUTCSeconds","toDateString","toISOString","toJSON","toLocaleDateString","toLocaleString","toLocaleTimeString","toString","toTimeString","toUTCString","valueOf","@@toPrimitive"],[],[],["constructor"]],
["String",String,["fromCharCode","fromCodePoint","raw"],[],[],["prototype"]],
["String.prototype",String.prototype,["at","charAt","charCodeAt","codePointAt","concat","endsWith","includes","indexOf","isWellFormed","lastIndexOf","localeCompare","match","matchAll","normalize","padEnd","padStart","repeat","replace","replaceAll","search","slice","split","startsWith","substring","toLocaleLowerCase","toLocaleUpperCase","toLowerCase","toString","toUpperCase","toWellFormed","trim","trimEnd","trimStart","valueOf","@@iterator"],[],[],["constructor"]],
["intrinsicObjects[\"StringIteratorPrototype\"]",intrinsicObjects["StringIteratorPrototype"],["next"],[],[],["@@toStringTag"]],
["RegExp",RegExp,["escape"],["@@species"],[],["prototype"]],
["RegExp.prototype",RegExp.prototype,["exec","@@match","@@matchAll","@@replace","@@search","@@split","test","toString"],["dotAll","flags","global","hasIndices","ignoreCase","multiline","source","sticky","unicode","unicodeSets"],[],["constructor"]],
["intrinsicObjects[\"RegExpStringIteratorPrototype\"]",intrinsicObjects["RegExpStringIteratorPrototype"],["next"],[],[],["@@toStringTag"]],
["Array",Array,["from","fromAsync","isArray","of"],["@@species"],[],["prototype"]],
["Array.prototype",Array.prototype,["at","concat","copyWithin","entries","every","fill","filter","find","findIndex","findLast","findLastIndex","flat","flatMap","forEach","includes","indexOf","join","keys","lastIndexOf","map","pop","push","reduce","reduceRight","reverse","shift","slice","some","sort","splice","toLocaleString","toReversed","toSorted","toSpliced","toString","unshift","values","with","@@iterator"],[],[],["constructor","@@unscopables"]],
["intrinsicObjects[\"ArrayIteratorPrototype\"]",intrinsicObjects["ArrayIteratorPrototype"],["next"],[],[],["@@toStringTag"]],
["TA",TA,["from","of"],["@@species"],[],["prototype"]],
["TA.prototype",TA.prototype,["at","copyWithin","entries","every","fill","filter","find","findIndex","findLast","findLastIndex","forEach","includes","indexOf","join","keys","lastIndexOf","map","reduce","reduceRight","reverse","set","slice","some","sort","subarray","toLocaleString","toReversed","toSorted","toString","values","with","@@iterator"],["buffer","byteLength","byteOffset","length","@@toStringTag"],[],["constructor"]],
["Int8Array",Int8Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Uint8Array",Uint8Array,["fromBase64","fromHex"],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Uint8ClampedArray",Uint8ClampedArray,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Int16Array",Int16Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Uint16Array",Uint16Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Int32Array",Int32Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Uint32Array",Uint32Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["BigInt64Array",BigInt64Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["BigUint64Array",BigUint64Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Float16Array",Float16Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Float32Array",Float32Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Float64Array",Float64Array,[],[],[],["BYTES_PER_ELEMENT","prototype"]],
["Int8Array.prototype",Int8Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Uint8Array.prototype",Uint8Array.prototype,["setFromBase64","setFromHex","toBase64","toHex"],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Uint8ClampedArray.prototype",Uint8ClampedArray.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Int16Array.prototype",Int16Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Uint16Array.prototype",Uint16Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Int32Array.prototype",Int32Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Uint32Array.prototype",Uint32Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["BigInt64Array.prototype",BigInt64Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["BigUint64Array.prototype",BigUint64Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Float16Array.prototype",Float16Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Float32Array.prototype",Float32Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Float64Array.prototype",Float64Array.prototype,[],[],[],["BYTES_PER_ELEMENT","constructor"]],
["Map",Map,["groupBy"],["@@species"],[],["prototype"]],
["Map.prototype",Map.prototype,["clear","delete","entries","forEach","get","getOrInsert","getOrInsertComputed","has","keys","set","values","@@iterator"],["size"],[],["constructor","@@toStringTag"]],
["intrinsicObjects[\"MapIteratorPrototype\"]",intrinsicObjects["MapIteratorPrototype"],["next"],[],[],["@@toStringTag"]],
["Set",Set,[],["@@species"],[],["prototype"]],
["Set.prototype",Set.prototype,["add","clear","delete","difference","entries","forEach","has","intersection","isDisjointFrom","isSubsetOf","isSupersetOf","keys","symmetricDifference","union","values","@@iterator"],["size"],[],["constructor","@@toStringTag"]],
["intrinsicObjects[\"SetIteratorPrototype\"]",intrinsicObjects["SetIteratorPrototype"],["next"],[],[],["@@toStringTag"]],
["WeakMap",WeakMap,[],[],[],["prototype"]],
["WeakMap.prototype",WeakMap.prototype,["delete","get","getOrInsert","getOrInsertComputed","has","set"],[],[],["constructor","@@toStringTag"]],
["WeakSet",WeakSet,[],[],[],["prototype"]],
["WeakSet.prototype",WeakSet.prototype,["add","delete","has"],[],[],["constructor","@@toStringTag"]],
["ArrayBuffer",ArrayBuffer,["isView"],["@@species"],[],["prototype"]],
["ArrayBuffer.prototype",ArrayBuffer.prototype,["resize","slice","transfer","transferToFixedLength"],["byteLength","detached","maxByteLength","resizable"],[],["constructor","@@toStringTag"]],
["SharedArrayBuffer",SharedArrayBuffer,[],["@@species"],[],["prototype"]],
["SharedArrayBuffer.prototype",SharedArrayBuffer.prototype,["grow","slice"],["byteLength","growable","maxByteLength"],[],["constructor","@@toStringTag"]],
["DataView",DataView,[],[],[],["prototype"]],
["DataView.prototype",DataView.prototype,["getBigInt64","getBigUint64","getFloat16","getFloat32","getFloat64","getInt8","getInt16","getInt32","getUint8","getUint16","getUint32","setBigInt64","setBigUint64","setFloat16","setFloat32","setFloat64","setInt8","setInt16","setInt32","setUint8","setUint16","setUint32"],["buffer","byteLength","byteOffset"],[],["constructor","@@toStringTag"]],
["Atomics",Atomics,["add","and","compareExchange","exchange","isLockFree","load","or","store","sub","wait","waitAsync","notify","xor"],[],[],["@@toStringTag"]],
["JSON",JSON,["isRawJSON","parse","rawJSON","stringify"],[],[],["@@toStringTag"]],
["WeakRef",WeakRef,[],[],[],["prototype"]],
["WeakRef.prototype",WeakRef.prototype,["deref"],[],[],["constructor","@@toStringTag"]],
["FinalizationRegistry",FinalizationRegistry,[],[],[],["prototype"]],
["FinalizationRegistry.prototype",FinalizationRegistry.prototype,["register","unregister"],[],[],["constructor","@@toStringTag"]],
["intrinsicObjects[\"IteratorHelperPrototype\"]",intrinsicObjects["IteratorHelperPrototype"],["next","return"],[],[],["@@toStringTag"]],
["Iterator",Iterator,["concat","from"],[],[],["prototype"]],
["intrinsicObjects[\"WrapForValidIteratorPrototype\"]",intrinsicObjects["WrapForValidIteratorPrototype"],["next","return"],[],[],[]],
["Iterator.prototype",Iterator.prototype,["drop","every","filter","find","flatMap","forEach","map","reduce","some","take","toArray","@@iterator"],["constructor","@@toStringTag"],["constructor","@@toStringTag"],[]],
["intrinsicObjects[\"AsyncIteratorPrototype\"]",intrinsicObjects["AsyncIteratorPrototype"],["@@asyncIterator"],[],[],[]],
["Promise",Promise,["all","allSettled","any","race","reject","resolve","try","withResolvers"],["@@species"],[],["prototype"]],
["Promise.prototype",Promise.prototype,["catch","finally","then"],[],[],["constructor","@@toStringTag"]],
["GeneratorFunction",GeneratorFunction,[],[],[],["prototype"]],
["GeneratorFunction.prototype",GeneratorFunction.prototype,[],[],[],["constructor","prototype","@@toStringTag"]],
["AsyncGeneratorFunction",AsyncGeneratorFunction,[],[],[],["prototype"]],
["AsyncGeneratorFunction.prototype",AsyncGeneratorFunction.prototype,[],[],[],["constructor","prototype","@@toStringTag"]],
["intrinsicObjects[\"GeneratorPrototype\"]",intrinsicObjects["GeneratorPrototype"],["next","return","throw"],[],[],["constructor","@@toStringTag"]],
["intrinsicObjects[\"AsyncGeneratorPrototype\"]",intrinsicObjects["AsyncGeneratorPrototype"],["next","return","throw"],[],[],["constructor","@@toStringTag"]],
["AsyncFunction",AsyncFunction,[],[],[],["prototype"]],
["AsyncFunction.prototype",AsyncFunction.prototype,[],[],[],["constructor","@@toStringTag"]],
["Reflect",Reflect,["apply","construct","defineProperty","deleteProperty","get","getOwnPropertyDescriptor","getPrototypeOf","has","isExtensible","ownKeys","preventExtensions","set","setPrototypeOf"],[],[],["@@toStringTag"]],
["Proxy",Proxy,["revocable"],[],[],[]],
];
let propertyFailures=[],propertyCount=0;
function checkBuiltin(label, fn){
  if(Object.getPrototypeOf(fn)!==Function.prototype)propertyFailures.push(label+":function-parent");
  if(constructor(fn))propertyFailures.push(label+":constructible");
  for(let key of ["name","length"]){
    let d=Object.getOwnPropertyDescriptor(fn,key);
    if(!d||d.writable||d.enumerable||!d.configurable)propertyFailures.push(label+":"+key+":attributes");
  }
}
for(let [label,owner,methods,getters,setters,data] of propertyRows){
  for(let [kind,keys] of [["method",methods],["getter",getters],["setter",setters],["data",data]]){
    for(let name of keys){
      propertyCount++;
      let key=name.startsWith("@@")?Symbol[name.slice(2)]:name;
      let descriptor=Object.getOwnPropertyDescriptor(owner,key);
      if(!descriptor){propertyFailures.push(label+":"+name+":missing");continue;}
      if(descriptor.enumerable)propertyFailures.push(label+":"+name+":enumerable");
      if(kind==="data"&&!Object.hasOwn(descriptor,"value"))propertyFailures.push(label+":"+name+":data");
      if(kind==="method"&&typeof descriptor.value!=="function")propertyFailures.push(label+":"+name+":method");
      if(kind==="getter"&&typeof descriptor.get!=="function")propertyFailures.push(label+":"+name+":getter");
      if(kind==="setter"&&typeof descriptor.set!=="function")propertyFailures.push(label+":"+name+":setter");
      let fn=kind==="getter"?descriptor.get:kind==="setter"?descriptor.set:kind==="method"?descriptor.value:null;
      if(typeof fn==="function")checkBuiltin(label+":"+name,fn);
    }
  }
}
JSON.stringify({checked:propertyCount,failures:propertyFailures});
