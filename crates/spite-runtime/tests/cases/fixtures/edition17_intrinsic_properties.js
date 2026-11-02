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
// Exact names/arities follow section 17, the individual headings and explicit overrides.
// Alias headings retain the target function name and identity.
let builtinMetadataRows=[
["Object",Object,[["assign","method","assign",2],["create","method","create",2],["defineProperties","method","defineProperties",2],["defineProperty","method","defineProperty",3],["entries","method","entries",1],["freeze","method","freeze",1],["fromEntries","method","fromEntries",1],["getOwnPropertyDescriptor","method","getOwnPropertyDescriptor",2],["getOwnPropertyDescriptors","method","getOwnPropertyDescriptors",1],["getOwnPropertyNames","method","getOwnPropertyNames",1],["getOwnPropertySymbols","method","getOwnPropertySymbols",1],["getPrototypeOf","method","getPrototypeOf",1],["groupBy","method","groupBy",2],["hasOwn","method","hasOwn",2],["is","method","is",2],["isExtensible","method","isExtensible",1],["isFrozen","method","isFrozen",1],["isSealed","method","isSealed",1],["keys","method","keys",1],["preventExtensions","method","preventExtensions",1],["seal","method","seal",1],["setPrototypeOf","method","setPrototypeOf",2],["values","method","values",1]]],
["Object.prototype",Object.prototype,[["hasOwnProperty","method","hasOwnProperty",1],["isPrototypeOf","method","isPrototypeOf",1],["propertyIsEnumerable","method","propertyIsEnumerable",1],["toLocaleString","method","toLocaleString",0],["toString","method","toString",0],["valueOf","method","valueOf",0]]],
["Function.prototype",Function.prototype,[["apply","method","apply",2],["bind","method","bind",1],["call","method","call",1],["toString","method","toString",0],["@@hasInstance","method","[Symbol.hasInstance]",1]]],
["Boolean.prototype",Boolean.prototype,[["toString","method","toString",0],["valueOf","method","valueOf",0]]],
["Symbol",Symbol,[["for","method","for",1],["keyFor","method","keyFor",1]]],
["Symbol.prototype",Symbol.prototype,[["description","getter","get description",0],["toString","method","toString",0],["valueOf","method","valueOf",0],["@@toPrimitive","method","[Symbol.toPrimitive]",1]]],
["Error",Error,[["isError","method","isError",1]]],
["Error.prototype",Error.prototype,[["toString","method","toString",0]]],
["Number",Number,[["isFinite","method","isFinite",1],["isInteger","method","isInteger",1],["isNaN","method","isNaN",1],["isSafeInteger","method","isSafeInteger",1],["parseFloat","method","parseFloat",1],["parseInt","method","parseInt",2]]],
["Number.prototype",Number.prototype,[["toExponential","method","toExponential",1],["toFixed","method","toFixed",1],["toLocaleString","method","toLocaleString",0],["toPrecision","method","toPrecision",1],["toString","method","toString",1],["valueOf","method","valueOf",0]]],
["BigInt",BigInt,[["asIntN","method","asIntN",2],["asUintN","method","asUintN",2]]],
["BigInt.prototype",BigInt.prototype,[["toLocaleString","method","toLocaleString",0],["toString","method","toString",0],["valueOf","method","valueOf",0]]],
["Math",Math,[["abs","method","abs",1],["acos","method","acos",1],["acosh","method","acosh",1],["asin","method","asin",1],["asinh","method","asinh",1],["atan","method","atan",1],["atanh","method","atanh",1],["atan2","method","atan2",2],["cbrt","method","cbrt",1],["ceil","method","ceil",1],["clz32","method","clz32",1],["cos","method","cos",1],["cosh","method","cosh",1],["exp","method","exp",1],["expm1","method","expm1",1],["floor","method","floor",1],["fround","method","fround",1],["f16round","method","f16round",1],["hypot","method","hypot",2],["imul","method","imul",2],["log","method","log",1],["log1p","method","log1p",1],["log10","method","log10",1],["log2","method","log2",1],["max","method","max",2],["min","method","min",2],["pow","method","pow",2],["random","method","random",0],["round","method","round",1],["sign","method","sign",1],["sin","method","sin",1],["sinh","method","sinh",1],["sqrt","method","sqrt",1],["sumPrecise","method","sumPrecise",1],["tan","method","tan",1],["tanh","method","tanh",1],["trunc","method","trunc",1]]],
["Date",Date,[["now","method","now",0],["parse","method","parse",1],["UTC","method","UTC",7]]],
["Date.prototype",Date.prototype,[["getDate","method","getDate",0],["getDay","method","getDay",0],["getFullYear","method","getFullYear",0],["getHours","method","getHours",0],["getMilliseconds","method","getMilliseconds",0],["getMinutes","method","getMinutes",0],["getMonth","method","getMonth",0],["getSeconds","method","getSeconds",0],["getTime","method","getTime",0],["getTimezoneOffset","method","getTimezoneOffset",0],["getUTCDate","method","getUTCDate",0],["getUTCDay","method","getUTCDay",0],["getUTCFullYear","method","getUTCFullYear",0],["getUTCHours","method","getUTCHours",0],["getUTCMilliseconds","method","getUTCMilliseconds",0],["getUTCMinutes","method","getUTCMinutes",0],["getUTCMonth","method","getUTCMonth",0],["getUTCSeconds","method","getUTCSeconds",0],["setDate","method","setDate",1],["setFullYear","method","setFullYear",3],["setHours","method","setHours",4],["setMilliseconds","method","setMilliseconds",1],["setMinutes","method","setMinutes",3],["setMonth","method","setMonth",2],["setSeconds","method","setSeconds",2],["setTime","method","setTime",1],["setUTCDate","method","setUTCDate",1],["setUTCFullYear","method","setUTCFullYear",3],["setUTCHours","method","setUTCHours",4],["setUTCMilliseconds","method","setUTCMilliseconds",1],["setUTCMinutes","method","setUTCMinutes",3],["setUTCMonth","method","setUTCMonth",2],["setUTCSeconds","method","setUTCSeconds",2],["toDateString","method","toDateString",0],["toISOString","method","toISOString",0],["toJSON","method","toJSON",1],["toLocaleDateString","method","toLocaleDateString",0],["toLocaleString","method","toLocaleString",0],["toLocaleTimeString","method","toLocaleTimeString",0],["toString","method","toString",0],["toTimeString","method","toTimeString",0],["toUTCString","method","toUTCString",0],["valueOf","method","valueOf",0],["@@toPrimitive","method","[Symbol.toPrimitive]",1]]],
["String",String,[["fromCharCode","method","fromCharCode",1],["fromCodePoint","method","fromCodePoint",1],["raw","method","raw",1]]],
["String.prototype",String.prototype,[["at","method","at",1],["charAt","method","charAt",1],["charCodeAt","method","charCodeAt",1],["codePointAt","method","codePointAt",1],["concat","method","concat",1],["endsWith","method","endsWith",1],["includes","method","includes",1],["indexOf","method","indexOf",1],["isWellFormed","method","isWellFormed",0],["lastIndexOf","method","lastIndexOf",1],["localeCompare","method","localeCompare",1],["match","method","match",1],["matchAll","method","matchAll",1],["normalize","method","normalize",0],["padEnd","method","padEnd",1],["padStart","method","padStart",1],["repeat","method","repeat",1],["replace","method","replace",2],["replaceAll","method","replaceAll",2],["search","method","search",1],["slice","method","slice",2],["split","method","split",2],["startsWith","method","startsWith",1],["substring","method","substring",2],["toLocaleLowerCase","method","toLocaleLowerCase",0],["toLocaleUpperCase","method","toLocaleUpperCase",0],["toLowerCase","method","toLowerCase",0],["toString","method","toString",0],["toUpperCase","method","toUpperCase",0],["toWellFormed","method","toWellFormed",0],["trim","method","trim",0],["trimEnd","method","trimEnd",0],["trimStart","method","trimStart",0],["valueOf","method","valueOf",0],["@@iterator","method","[Symbol.iterator]",0]]],
["intrinsicObjects[\"StringIteratorPrototype\"]",intrinsicObjects["StringIteratorPrototype"],[["next","method","next",0]]],
["RegExp",RegExp,[["escape","method","escape",1],["@@species","getter","get [Symbol.species]",0]]],
["RegExp.prototype",RegExp.prototype,[["exec","method","exec",1],["dotAll","getter","get dotAll",0],["flags","getter","get flags",0],["global","getter","get global",0],["hasIndices","getter","get hasIndices",0],["ignoreCase","getter","get ignoreCase",0],["@@match","method","[Symbol.match]",1],["@@matchAll","method","[Symbol.matchAll]",1],["multiline","getter","get multiline",0],["@@replace","method","[Symbol.replace]",2],["@@search","method","[Symbol.search]",1],["source","getter","get source",0],["@@split","method","[Symbol.split]",2],["sticky","getter","get sticky",0],["test","method","test",1],["toString","method","toString",0],["unicode","getter","get unicode",0],["unicodeSets","getter","get unicodeSets",0]]],
["intrinsicObjects[\"RegExpStringIteratorPrototype\"]",intrinsicObjects["RegExpStringIteratorPrototype"],[["next","method","next",0]]],
["Array",Array,[["from","method","from",1],["fromAsync","method","fromAsync",1],["isArray","method","isArray",1],["of","method","of",0],["@@species","getter","get [Symbol.species]",0]]],
["Array.prototype",Array.prototype,[["at","method","at",1],["concat","method","concat",1],["copyWithin","method","copyWithin",2],["entries","method","entries",0],["every","method","every",1],["fill","method","fill",1],["filter","method","filter",1],["find","method","find",1],["findIndex","method","findIndex",1],["findLast","method","findLast",1],["findLastIndex","method","findLastIndex",1],["flat","method","flat",0],["flatMap","method","flatMap",1],["forEach","method","forEach",1],["includes","method","includes",1],["indexOf","method","indexOf",1],["join","method","join",1],["keys","method","keys",0],["lastIndexOf","method","lastIndexOf",1],["map","method","map",1],["pop","method","pop",0],["push","method","push",1],["reduce","method","reduce",1],["reduceRight","method","reduceRight",1],["reverse","method","reverse",0],["shift","method","shift",0],["slice","method","slice",2],["some","method","some",1],["sort","method","sort",1],["splice","method","splice",2],["toLocaleString","method","toLocaleString",0],["toReversed","method","toReversed",0],["toSorted","method","toSorted",1],["toSpliced","method","toSpliced",2],["toString","method","toString",0],["unshift","method","unshift",1],["values","method","values",0],["with","method","with",2],["@@iterator","method","values",0]]],
["intrinsicObjects[\"ArrayIteratorPrototype\"]",intrinsicObjects["ArrayIteratorPrototype"],[["next","method","next",0]]],
["TA",TA,[["from","method","from",1],["of","method","of",0],["@@species","getter","get [Symbol.species]",0]]],
["TA.prototype",TA.prototype,[["at","method","at",1],["buffer","getter","get buffer",0],["byteLength","getter","get byteLength",0],["byteOffset","getter","get byteOffset",0],["copyWithin","method","copyWithin",2],["entries","method","entries",0],["every","method","every",1],["fill","method","fill",1],["filter","method","filter",1],["find","method","find",1],["findIndex","method","findIndex",1],["findLast","method","findLast",1],["findLastIndex","method","findLastIndex",1],["forEach","method","forEach",1],["includes","method","includes",1],["indexOf","method","indexOf",1],["join","method","join",1],["keys","method","keys",0],["lastIndexOf","method","lastIndexOf",1],["length","getter","get length",0],["map","method","map",1],["reduce","method","reduce",1],["reduceRight","method","reduceRight",1],["reverse","method","reverse",0],["set","method","set",1],["slice","method","slice",2],["some","method","some",1],["sort","method","sort",1],["subarray","method","subarray",2],["toLocaleString","method","toLocaleString",0],["toReversed","method","toReversed",0],["toSorted","method","toSorted",1],["toString","method","toString",0],["values","method","values",0],["with","method","with",2],["@@iterator","method","values",0],["@@toStringTag","getter","get [Symbol.toStringTag]",0]]],
["Uint8Array",Uint8Array,[["fromBase64","method","fromBase64",1],["fromHex","method","fromHex",1]]],
["Uint8Array.prototype",Uint8Array.prototype,[["setFromBase64","method","setFromBase64",1],["setFromHex","method","setFromHex",1],["toBase64","method","toBase64",0],["toHex","method","toHex",0]]],
["Map",Map,[["groupBy","method","groupBy",2],["@@species","getter","get [Symbol.species]",0]]],
["Map.prototype",Map.prototype,[["clear","method","clear",0],["delete","method","delete",1],["entries","method","entries",0],["forEach","method","forEach",1],["get","method","get",1],["getOrInsert","method","getOrInsert",2],["getOrInsertComputed","method","getOrInsertComputed",2],["has","method","has",1],["keys","method","keys",0],["set","method","set",2],["size","getter","get size",0],["values","method","values",0],["@@iterator","method","entries",0]]],
["intrinsicObjects[\"MapIteratorPrototype\"]",intrinsicObjects["MapIteratorPrototype"],[["next","method","next",0]]],
["Set",Set,[["@@species","getter","get [Symbol.species]",0]]],
["Set.prototype",Set.prototype,[["add","method","add",1],["clear","method","clear",0],["delete","method","delete",1],["difference","method","difference",1],["entries","method","entries",0],["forEach","method","forEach",1],["has","method","has",1],["intersection","method","intersection",1],["isDisjointFrom","method","isDisjointFrom",1],["isSubsetOf","method","isSubsetOf",1],["isSupersetOf","method","isSupersetOf",1],["keys","method","values",0],["size","getter","get size",0],["symmetricDifference","method","symmetricDifference",1],["union","method","union",1],["values","method","values",0],["@@iterator","method","values",0]]],
["intrinsicObjects[\"SetIteratorPrototype\"]",intrinsicObjects["SetIteratorPrototype"],[["next","method","next",0]]],
["WeakMap.prototype",WeakMap.prototype,[["delete","method","delete",1],["get","method","get",1],["getOrInsert","method","getOrInsert",2],["getOrInsertComputed","method","getOrInsertComputed",2],["has","method","has",1],["set","method","set",2]]],
["WeakSet.prototype",WeakSet.prototype,[["add","method","add",1],["delete","method","delete",1],["has","method","has",1]]],
["ArrayBuffer",ArrayBuffer,[["isView","method","isView",1],["@@species","getter","get [Symbol.species]",0]]],
["ArrayBuffer.prototype",ArrayBuffer.prototype,[["byteLength","getter","get byteLength",0],["detached","getter","get detached",0],["maxByteLength","getter","get maxByteLength",0],["resizable","getter","get resizable",0],["resize","method","resize",1],["slice","method","slice",2],["transfer","method","transfer",0],["transferToFixedLength","method","transferToFixedLength",0]]],
["SharedArrayBuffer",SharedArrayBuffer,[["@@species","getter","get [Symbol.species]",0]]],
["SharedArrayBuffer.prototype",SharedArrayBuffer.prototype,[["byteLength","getter","get byteLength",0],["grow","method","grow",1],["growable","getter","get growable",0],["maxByteLength","getter","get maxByteLength",0],["slice","method","slice",2]]],
["DataView.prototype",DataView.prototype,[["buffer","getter","get buffer",0],["byteLength","getter","get byteLength",0],["byteOffset","getter","get byteOffset",0],["getBigInt64","method","getBigInt64",1],["getBigUint64","method","getBigUint64",1],["getFloat16","method","getFloat16",1],["getFloat32","method","getFloat32",1],["getFloat64","method","getFloat64",1],["getInt8","method","getInt8",1],["getInt16","method","getInt16",1],["getInt32","method","getInt32",1],["getUint8","method","getUint8",1],["getUint16","method","getUint16",1],["getUint32","method","getUint32",1],["setBigInt64","method","setBigInt64",2],["setBigUint64","method","setBigUint64",2],["setFloat16","method","setFloat16",2],["setFloat32","method","setFloat32",2],["setFloat64","method","setFloat64",2],["setInt8","method","setInt8",2],["setInt16","method","setInt16",2],["setInt32","method","setInt32",2],["setUint8","method","setUint8",2],["setUint16","method","setUint16",2],["setUint32","method","setUint32",2]]],
["Atomics",Atomics,[["add","method","add",3],["and","method","and",3],["compareExchange","method","compareExchange",4],["exchange","method","exchange",3],["isLockFree","method","isLockFree",1],["load","method","load",2],["or","method","or",3],["store","method","store",3],["sub","method","sub",3],["wait","method","wait",4],["waitAsync","method","waitAsync",4],["notify","method","notify",3],["xor","method","xor",3]]],
["JSON",JSON,[["isRawJSON","method","isRawJSON",1],["parse","method","parse",2],["rawJSON","method","rawJSON",1],["stringify","method","stringify",3]]],
["WeakRef.prototype",WeakRef.prototype,[["deref","method","deref",0]]],
["FinalizationRegistry.prototype",FinalizationRegistry.prototype,[["register","method","register",2],["unregister","method","unregister",1]]],
["intrinsicObjects[\"IteratorHelperPrototype\"]",intrinsicObjects["IteratorHelperPrototype"],[["next","method","next",0],["return","method","return",0]]],
["Iterator",Iterator,[["concat","method","concat",0],["from","method","from",1]]],
["intrinsicObjects[\"WrapForValidIteratorPrototype\"]",intrinsicObjects["WrapForValidIteratorPrototype"],[["next","method","next",0],["return","method","return",0]]],
["Iterator.prototype",Iterator.prototype,[["constructor","getter","get constructor",0],["constructor","setter","set constructor",1],["drop","method","drop",1],["every","method","every",1],["filter","method","filter",1],["find","method","find",1],["flatMap","method","flatMap",1],["forEach","method","forEach",1],["map","method","map",1],["reduce","method","reduce",1],["some","method","some",1],["take","method","take",1],["toArray","method","toArray",0],["@@iterator","method","[Symbol.iterator]",0],["@@toStringTag","getter","get [Symbol.toStringTag]",0],["@@toStringTag","setter","set [Symbol.toStringTag]",1]]],
["intrinsicObjects[\"AsyncIteratorPrototype\"]",intrinsicObjects["AsyncIteratorPrototype"],[["@@asyncIterator","method","[Symbol.asyncIterator]",0]]],
["Promise",Promise,[["all","method","all",1],["allSettled","method","allSettled",1],["any","method","any",1],["race","method","race",1],["reject","method","reject",1],["resolve","method","resolve",1],["try","method","try",1],["withResolvers","method","withResolvers",0],["@@species","getter","get [Symbol.species]",0]]],
["Promise.prototype",Promise.prototype,[["catch","method","catch",1],["finally","method","finally",1],["then","method","then",2]]],
["intrinsicObjects[\"GeneratorPrototype\"]",intrinsicObjects["GeneratorPrototype"],[["next","method","next",1],["return","method","return",1],["throw","method","throw",1]]],
["intrinsicObjects[\"AsyncGeneratorPrototype\"]",intrinsicObjects["AsyncGeneratorPrototype"],[["next","method","next",1],["return","method","return",1],["throw","method","throw",1]]],
["Reflect",Reflect,[["apply","method","apply",3],["construct","method","construct",2],["defineProperty","method","defineProperty",3],["deleteProperty","method","deleteProperty",2],["get","method","get",2],["getOwnPropertyDescriptor","method","getOwnPropertyDescriptor",2],["getPrototypeOf","method","getPrototypeOf",1],["has","method","has",2],["isExtensible","method","isExtensible",1],["ownKeys","method","ownKeys",1],["preventExtensions","method","preventExtensions",1],["set","method","set",3],["setPrototypeOf","method","setPrototypeOf",2]]],
["Proxy",Proxy,[["revocable","method","revocable",2]]],
 ];
let builtinMetadataCount=0;
for(let [label,owner,entries] of builtinMetadataRows){
  for(let [name,kind,expectedName,expectedLength] of entries){
    builtinMetadataCount++;
    let key=name.startsWith("@@")?Symbol[name.slice(2)]:name;
    let descriptor=Object.getOwnPropertyDescriptor(owner,key);
    let fn=kind==="getter"?descriptor.get:kind==="setter"?descriptor.set:descriptor.value;
    if(fn.name!==expectedName||fn.length!==expectedLength)propertyFailures.push(label+":"+name+":name-length:"+fn.name+"/"+fn.length);
  }
}
for(let [label,value,target] of [
  ["Array.iterator",Array.prototype[Symbol.iterator],Array.prototype.values],
  ["TypedArray.toString",TA.prototype.toString,Array.prototype.toString],
  ["TypedArray.iterator",TA.prototype[Symbol.iterator],TA.prototype.values],
  ["Map.iterator",Map.prototype[Symbol.iterator],Map.prototype.entries],
  ["Set.keys",Set.prototype.keys,Set.prototype.values],
  ["Set.iterator",Set.prototype[Symbol.iterator],Set.prototype.values]
])if(value!==target)propertyFailures.push(label+":alias");
// Section 17 defaults, with the individual clauses' explicit attribute overrides.
let propertyAttributeOverrides=new Map();
for(let [owner,keys,writable,configurable] of [
["Object",["prototype"],false,false],
["Function",["prototype"],false,false],
["Function.prototype",["@@hasInstance"],false,false],
["Boolean",["prototype"],false,false],
["Symbol",["asyncIterator","hasInstance","isConcatSpreadable","iterator","match","matchAll","prototype","replace","search","species","split","toPrimitive","toStringTag","unscopables"],false,false],
["Symbol.prototype",["@@toPrimitive","@@toStringTag"],false,true],
["Error",["prototype"],false,false],
["EvalError",["prototype"],false,false],
["RangeError",["prototype"],false,false],
["ReferenceError",["prototype"],false,false],
["SyntaxError",["prototype"],false,false],
["TypeError",["prototype"],false,false],
["URIError",["prototype"],false,false],
["AggregateError",["prototype"],false,false],
["Number",["EPSILON","MAX_SAFE_INTEGER","MAX_VALUE","MIN_SAFE_INTEGER","MIN_VALUE","NaN","NEGATIVE_INFINITY","POSITIVE_INFINITY","prototype"],false,false],
["BigInt",["prototype"],false,false],
["BigInt.prototype",["@@toStringTag"],false,true],
["Math",["E","LN10","LN2","LOG10E","LOG2E","PI","SQRT1_2","SQRT2"],false,false],
["Math",["@@toStringTag"],false,true],
["Date",["prototype"],false,false],
["Date.prototype",["@@toPrimitive"],false,true],
["String",["prototype"],false,false],
["intrinsicObjects[\"StringIteratorPrototype\"]",["@@toStringTag"],false,true],
["RegExp",["prototype"],false,false],
["intrinsicObjects[\"RegExpStringIteratorPrototype\"]",["@@toStringTag"],false,true],
["Array",["prototype"],false,false],
["Array.prototype",["@@unscopables"],false,true],
["intrinsicObjects[\"ArrayIteratorPrototype\"]",["@@toStringTag"],false,true],
["TA",["prototype"],false,false],
["Int8Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Uint8Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Uint8ClampedArray",["BYTES_PER_ELEMENT","prototype"],false,false],
["Int16Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Uint16Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Int32Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Uint32Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["BigInt64Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["BigUint64Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Float16Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Float32Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Float64Array",["BYTES_PER_ELEMENT","prototype"],false,false],
["Int8Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Uint8Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Uint8ClampedArray.prototype",["BYTES_PER_ELEMENT"],false,false],
["Int16Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Uint16Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Int32Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Uint32Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["BigInt64Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["BigUint64Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Float16Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Float32Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Float64Array.prototype",["BYTES_PER_ELEMENT"],false,false],
["Map",["prototype"],false,false],
["Map.prototype",["@@toStringTag"],false,true],
["intrinsicObjects[\"MapIteratorPrototype\"]",["@@toStringTag"],false,true],
["Set",["prototype"],false,false],
["Set.prototype",["@@toStringTag"],false,true],
["intrinsicObjects[\"SetIteratorPrototype\"]",["@@toStringTag"],false,true],
["WeakMap",["prototype"],false,false],
["WeakMap.prototype",["@@toStringTag"],false,true],
["WeakSet",["prototype"],false,false],
["WeakSet.prototype",["@@toStringTag"],false,true],
["ArrayBuffer",["prototype"],false,false],
["ArrayBuffer.prototype",["@@toStringTag"],false,true],
["SharedArrayBuffer",["prototype"],false,false],
["SharedArrayBuffer.prototype",["@@toStringTag"],false,true],
["DataView",["prototype"],false,false],
["DataView.prototype",["@@toStringTag"],false,true],
["Atomics",["@@toStringTag"],false,true],
["JSON",["@@toStringTag"],false,true],
["WeakRef",["prototype"],false,false],
["WeakRef.prototype",["@@toStringTag"],false,true],
["FinalizationRegistry",["prototype"],false,false],
["FinalizationRegistry.prototype",["@@toStringTag"],false,true],
["intrinsicObjects[\"IteratorHelperPrototype\"]",["@@toStringTag"],false,true],
["Iterator",["prototype"],false,false],
["Promise",["prototype"],false,false],
["Promise.prototype",["@@toStringTag"],false,true],
["GeneratorFunction",["prototype"],false,false],
["GeneratorFunction.prototype",["constructor","prototype","@@toStringTag"],false,true],
["AsyncGeneratorFunction",["prototype"],false,false],
["AsyncGeneratorFunction.prototype",["constructor","prototype","@@toStringTag"],false,true],
["intrinsicObjects[\"GeneratorPrototype\"]",["constructor","@@toStringTag"],false,true],
["intrinsicObjects[\"AsyncGeneratorPrototype\"]",["constructor","@@toStringTag"],false,true],
["AsyncFunction",["prototype"],false,false],
["AsyncFunction.prototype",["constructor","@@toStringTag"],false,true],
["Reflect",["@@toStringTag"],false,true],
])for(let key of keys)propertyAttributeOverrides.set(owner+":"+key,{writable,configurable});
for(let [label,owner,methods,getters,setters,data] of propertyRows){
  for(let [kind,keys] of [["method",methods],["getter",getters],["setter",setters],["data",data]]){
    for(let name of keys){
      let key=name.startsWith("@@")?Symbol[name.slice(2)]:name;
      let d=Object.getOwnPropertyDescriptor(owner,key),override=propertyAttributeOverrides.get(label+":"+name);
      let expectedConfigurable=override?override.configurable:true;
      if(d.configurable!==expectedConfigurable)propertyFailures.push(label+":"+name+":configurable");
      if(kind==="data"||kind==="method"){
        let expectedWritable=override?override.writable:true;
        if(d.writable!==expectedWritable)propertyFailures.push(label+":"+name+":writable");
      }
      if(kind==="getter"&&!setters.includes(name)&&d.set!==undefined)propertyFailures.push(label+":"+name+":unexpected-setter");
      if(kind==="setter"&&!getters.includes(name)&&d.get!==undefined)propertyFailures.push(label+":"+name+":unexpected-getter");
    }
  }
}
JSON.stringify({checked:propertyCount,functions:builtinMetadataCount,failures:propertyFailures});
