//! Manual parser, Realm initialization, evaluation and collection measurements.

use spite_parser::parse_script;
use spite_runtime::{PromiseJobOutcome, Realm, Value};
use std::{
    error::Error,
    hint::black_box,
    num::NonZeroUsize,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Case {
    name: &'static str,
    source: &'static str,
    expected: f64,
    jobs: bool,
}

const CASES: &[Case] = &[
    Case {
        name: "arithmetic",
        source: "(()=>{let sum=0;for(let i=0;i<1000;i++)sum=(sum+i)|0;return sum})()",
        expected: 499500.0,
        jobs: false,
    },
    Case {
        name: "arguments_body_scan",
        source: "(()=>{function f(value=1){let a=value,b=a+1;return b}function g(arguments){let a=arguments,b=a+1;return b}let sum=0;for(let i=0;i<2048;i++)sum+=f(i)+g(i);return sum})()",
        expected: 4_196_352.0,
        jobs: false,
    },
    Case {
        name: "var_bindings",
        source: "(()=>{function f(value){var a=value,b=a+1;return b}let sum=0;for(let i=0;i<2048;i++)sum+=f(i);return sum})()",
        expected: 2_098_176.0,
        jobs: false,
    },
    Case {
        name: "radix_overflow",
        source: "(()=>{const text='0x'+'f'.repeat(8192);let sum=0;for(let i=0;i<16;i++){if(Number(text)===Infinity)sum++;if(Number.isNaN(Number(text+'g')))sum++}return sum})()",
        expected: 32.0,
        jobs: false,
    },
    Case {
        name: "parse_int_overflow",
        source: "(()=>{const text='0x'+'f'.repeat(8192);let sum=0;for(let i=0;i<16;i++){if(parseInt(text)===Infinity)sum++;if(parseInt(text+'g',16)===Infinity)sum++}return sum})()",
        expected: 32.0,
        jobs: false,
    },
    Case {
        name: "parse_int_small",
        source: "(()=>{let sum=0;for(let i=0;i<1024;i++)sum+=parseInt(i.toString(16),16);return sum})()",
        expected: 523_776.0,
        jobs: false,
    },
    Case {
        name: "properties",
        source: "(()=>{const object={};for(let i=0;i<128;i++)object['k'+i]=i;let sum=0;for(let i=0;i<128;i++)sum+=object['k'+i];return sum})()",
        expected: 8128.0,
        jobs: false,
    },
    Case {
        name: "unresolved_globals",
        source: "(()=>{let sum=0;for(let i=0;i<4096;i++)if(typeof missingName==='undefined')sum++;return sum})()",
        expected: 4096.0,
        jobs: false,
    },
    Case {
        name: "string_search",
        source: "(()=>{const text='a'.repeat(4096)+'b',needle='a'.repeat(128)+'b';let sum=0;for(let i=0;i<8;i++)sum+=text.indexOf(needle);return sum})()",
        expected: 31744.0,
        jobs: false,
    },
    Case {
        name: "string_box",
        source: "(()=>{const text='ab漢\\ud800'.repeat(512);let sum=0;for(let i=0;i<8;i++){const boxed=Object(text);sum+=boxed.length+boxed[0].charCodeAt(0)+boxed[boxed.length-1].charCodeAt(0)}return sum})()",
        expected: 459_528.0,
        jobs: false,
    },
    Case {
        name: "string_character",
        source: "(()=>{const text='ab漢\\ud800';let sum=0;for(let i=0;i<4096;i++){const c=i%2?text.at(i%4):text.charAt(i%4);sum+=c.charCodeAt(0)}return sum})()",
        expected: 85_955_584.0,
        jobs: false,
    },
    Case {
        name: "string_slice",
        source: "(()=>{const text='ab漢\\ud800'.repeat(4096);let sum=0;for(let i=0;i<16;i++){const output=i%2?text.slice(1,-1):text.substring(1,text.length-1);sum+=output.length+output.charCodeAt(0)+output.charCodeAt(output.length-1)}return sum})()",
        expected: 718_880.0,
        jobs: false,
    },
    Case {
        name: "string_slice_full",
        source: "(()=>{const base='ab漢\\ud800'.repeat(4096);let sum=0;for(let i=0;i<64;i++){const out=i%2?base.slice():base.substring();sum+=out.length+out.charCodeAt(out.length-1)}return sum})()",
        expected: 4_587_520.0,
        jobs: false,
    },
    Case {
        name: "string_normalize_ascii",
        source: "(()=>{const text='ASCII 09'.repeat(2048),forms=['NFC','NFD','NFKC','NFKD'];let sum=0;for(let i=0;i<16;i++){const output=text.normalize(forms[i%4]);sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 263_056.0,
        jobs: false,
    },
    Case {
        name: "string_iteration",
        source: "(()=>{const text='ab漢💩\\ud800'.repeat(128);let sum=0;for(const c of text)sum+=c.length+c.charCodeAt(0);return sum})()",
        expected: 17_830_912.0,
        jobs: false,
    },
    Case {
        name: "string_add",
        source: "(()=>{let text='';for(let i=0;i<2048;i++)text+='ab漢\\ud800';return text.length+text.charCodeAt(text.length-1)})()",
        expected: 63_488.0,
        jobs: false,
    },
    Case {
        name: "string_add_empty",
        source: "(()=>{const base='ab漢\\ud800'.repeat(4096);let sum=0;for(let i=0;i<64;i++){const out=i%2?''+base:base+'';sum+=out.length+out.charCodeAt(out.length-1)}return sum})()",
        expected: 4_587_520.0,
        jobs: false,
    },
    Case {
        name: "string_concat_no_args",
        source: "(()=>{const base='ab漢\\ud800'.repeat(4096);let sum=0;for(let i=0;i<64;i++){const out=base.concat();sum+=out.length+out.charCodeAt(out.length-1)}return sum})()",
        expected: 4_587_520.0,
        jobs: false,
    },
    Case {
        name: "string_trim_noop",
        source: "(()=>{const base='ab漢\\ud800'.repeat(4096);let sum=0;for(let i=0;i<64;i++){const out=i%3===0?base.trim():i%3===1?base.trimStart():base.trimEnd();sum+=out.length+out.charCodeAt(out.length-1)}return sum})()",
        expected: 4_587_520.0,
        jobs: false,
    },
    Case {
        name: "string_repeat",
        source: "(()=>{let sum=0;for(let i=0;i<16;i++){const text=String.fromCharCode(97+i).repeat(65536);sum+=text.length+text.charCodeAt(text.length-1)}return sum})()",
        expected: 1_050_248.0,
        jobs: false,
    },
    Case {
        name: "string_repeat_wide",
        source: "(()=>{let sum=0;for(let i=0;i<16;i++){const text='ab漢\\ud800'.repeat(16384);sum+=text.length+text.charCodeAt(text.length-1)}return sum})()",
        expected: 1_933_312.0,
        jobs: false,
    },
    Case {
        name: "string_pad",
        source: "(()=>{let sum=0;for(let i=0;i<16;i++){const text=i%2?'ab'.padStart(65536):'ab'.padEnd(65536);sum+=text.length+text.charCodeAt(i%2?0:text.length-1)}return sum})()",
        expected: 1_049_088.0,
        jobs: false,
    },
    Case {
        name: "string_pad_wide",
        source: "(()=>{let sum=0;for(let i=0;i<16;i++){const text=i%2?'ab'.padStart(65536,'é漢\\ud800'):'ab'.padEnd(65536,'é漢\\ud800');sum+=text.length+text.charCodeAt(i%2?0:text.length-1)}return sum})()",
        expected: 1_278_040.0,
        jobs: false,
    },
    Case {
        name: "string_wellformed",
        source: "(()=>{const input='ab漢💩'.repeat(2048);let sum=0;for(let i=0;i<16;i++){const output=input.toWellFormed();sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 1_067_664.0,
        jobs: false,
    },
    Case {
        name: "string_wellformed_mixed",
        source: "(()=>{const base='ab漢💩'.repeat(2048);let sum=0;for(let i=0;i<16;i++){const input=i%2?'\\ud800'+base:base+'\\ud800',output=input.toWellFormed();sum+=output.length+output.charCodeAt(i%2?0:output.length-1)}return sum})()",
        expected: 1_212_384.0,
        jobs: false,
    },
    Case {
        name: "string_case_identity_ascii",
        source: "(()=>{const lower='abc123_-'.repeat(2048),upper='ABC123_-'.repeat(2048);let sum=0;for(let i=0;i<8;i++)sum+=lower.toLowerCase().length+upper.toUpperCase().length;return sum})()",
        expected: 262_144.0,
        jobs: false,
    },
    Case {
        name: "string_case_ascii",
        source: "(()=>{const text=\"AbC09 .:^`'z\".repeat(1024);let sum=0;for(let i=0;i<8;i++){const lower=text.toLowerCase(),upper=text.toUpperCase();sum+=lower.length+upper.length+lower.charCodeAt(2)+upper.charCodeAt(2)}return sum})()",
        expected: 197_936.0,
        jobs: false,
    },
    Case {
        name: "string_case_context",
        source: "(()=>{const text=\"AΣ\\u0345B \\ud800aΣ: .\".repeat(1024);let sum=0;for(let i=0;i<8;i++){const lower=text.toLowerCase(),upper=text.toUpperCase();sum+=lower.length+upper.length+lower.charCodeAt(2)+upper.charCodeAt(2)}return sum})()",
        expected: 194_288.0,
        jobs: false,
    },
    Case {
        name: "string_case_unicode",
        source: "(()=>{const text='Σ\\u0345𐐀ß漢\\u200d'.repeat(1024);let sum=0;for(let i=0;i<8;i++){const lower=text.toLowerCase(),upper=text.toUpperCase();sum+=lower.length+upper.length+lower.charCodeAt(2)+upper.charCodeAt(2)}return sum})()",
        expected: 1_007_632.0,
        jobs: false,
    },
    Case {
        name: "object_group_by",
        source: "(()=>{const values=Array.from({length:1024},(_,i)=>i),groups=Object.groupBy(values,v=>'g'+v);return Object.keys(groups).length+groups.g0[0]+groups.g1023[0]})()",
        expected: 2047.0,
        jobs: false,
    },
    Case {
        name: "typed_array_indices",
        source: "(()=>{const values=Array.from({length:1024},(_,i)=>i);let sum=0;for(let i=0;i<8;i++){const a=Float64Array.from(values),b=a.map(v=>v+1);sum+=b.reduce((x,y)=>x+y,0)}return sum})()",
        expected: 4_198_400.0,
        jobs: false,
    },
    Case {
        name: "object_group_by_few",
        source: "(()=>{const values=Array.from({length:4096},(_,i)=>i),groups=Object.groupBy(values,v=>'g'+v%4);return groups.g0.length+groups.g1.length+groups.g2.length+groups.g3.length+groups.g3[1023]})()",
        expected: 8191.0,
        jobs: false,
    },
    Case {
        name: "map_group_by",
        source: "(()=>{const values=Array.from({length:1024},(_,i)=>i),groups=Map.groupBy(values,v=>'g'+v);return groups.size+groups.get('g0')[0]+groups.get('g1023')[0]})()",
        expected: 2047.0,
        jobs: false,
    },
    Case {
        name: "map_iteration",
        source: "(()=>{const map=new Map();for(let i=0;i<256;i++)map.set(i,i);let sum=0;for(const value of map.values())sum+=value;return sum})()",
        expected: 32640.0,
        jobs: false,
    },
    Case {
        name: "function_source",
        source: "(()=>{const f=Function('/*'+'ab漢💩'.repeat(2048)+'*/return 1');let sum=0;for(let i=0;i<16;i++){const text=f.toString();sum+=text.length+text.charCodeAt(text.length-1)}return sum})()",
        expected: 166_448.0,
        jobs: false,
    },
    Case {
        name: "function_source_ascii",
        source: "(()=>{const f=Function('/*'+'ab'.repeat(8192)+'*/return 1');let sum=0;for(let i=0;i<16;i++){const text=f.toString();sum+=text.length+text.charCodeAt(text.length-1)}return sum})()",
        expected: 264_752.0,
        jobs: false,
    },
    Case {
        name: "function_source_surrogates",
        source: "(()=>{const f=Function('/*'+'ab漢\\ud800x\\udc00'.repeat(2048)+'*/return 1');let sum=0;for(let i=0;i<16;i++){const text=f.toString();sum+=text.length+text.charCodeAt(text.length-1)}return sum})()",
        expected: 199_216.0,
        jobs: false,
    },
    Case {
        name: "bigint",
        source: "(()=>{let x=1n<<256n;for(let i=0;i<64;i++)x=(x*3n+BigInt(i))/2n;return Number(x&65535n)})()",
        expected: 5661.0,
        jobs: false,
    },
    Case {
        name: "bigint_bitwise",
        source: "(()=>{let x=-(1n<<2048n)+1n,sum=0;const mask=(1n<<2048n)-1n;for(let i=0;i<64;i++){x=((x&mask)^(x>>13n))|(1n<<BigInt(i));sum+=Number(x&65535n)}return sum})()",
        expected: 2_258_539.0,
        jobs: false,
    },
    Case {
        name: "bigint_divide_u64",
        source: "(()=>{let x=(1n<<2048n)-1n,sum=0;const d=(1n<<63n)+17n;for(let i=0;i<32;i++){sum+=Number(((x/d)^(x%d))&65535n);x=x*3n+BigInt(i)}return sum})()",
        expected: 813_570.0,
        jobs: false,
    },
    Case {
        name: "bigint_radix",
        source: "(()=>{let x=(1n<<2048n)-1n,sum=0;for(let i=0;i<32;i++){const hex=x.toString(16),base32=x.toString(32);sum+=hex.length+base32.length+parseInt(hex.slice(-6),16)+parseInt(base32.slice(-6),32);x=x*3n+BigInt(i)}return sum})()",
        expected: 26_296_828_883.0,
        jobs: false,
    },
    Case {
        name: "bigint_decimal",
        source: "(()=>{let x=(1n<<2048n)-1n,sum=0;for(let i=0;i<32;i++){const digits=x.toString();sum+=digits.length+parseInt(digits.slice(-8),10);x=x*3n+BigInt(i)}return sum})()",
        expected: 1_654_203_804.0,
        jobs: false,
    },
    Case {
        name: "bigint_small_decimal",
        source: "(()=>{let x=0n,sum=0;for(let i=0;i<1024;i++){const digits=x.toString();sum+=digits.length+parseInt(digits,10);x++}return sum})()",
        expected: 526_762.0,
        jobs: false,
    },
    Case {
        name: "bigint_parse_decimal",
        source: "(()=>{const text='9'.repeat(2048);let sum=0;for(let i=0;i<16;i++)sum+=Number(BigInt.asUintN(32,BigInt(text)));return sum})()",
        expected: 68_719_476_720.0,
        jobs: false,
    },
    Case {
        name: "bigint_parse_hex",
        source: "(()=>{const text='0x'+'abcdef0123456789'.repeat(128);let sum=0;for(let i=0;i<16;i++)sum+=Number(BigInt.asUintN(32,BigInt(text)));return sum})()",
        expected: 9_468_016_784.0,
        jobs: false,
    },
    Case {
        name: "bigint_parse_small",
        source: "(()=>{let sum=0;for(let i=0;i<1024;i++)sum+=Number(BigInt(String(i)));return sum})()",
        expected: 523_776.0,
        jobs: false,
    },
    Case {
        name: "json_strings",
        source: "(()=>{const text='plain café 漢字 '.repeat(256);let sum=0;for(let i=0;i<16;i++){const output=JSON.stringify([text,text]);sum+=output.length+output.charCodeAt(output.length-2)}return sum})()",
        expected: 115344.0,
        jobs: false,
    },
    Case {
        name: "json_escaped",
        source: "(()=>{const text='plain café 漢字 '.repeat(256)+String.fromCharCode(10,34,0xd800);let sum=0;for(let i=0;i<16;i++){const output=JSON.stringify([text,text]);sum+=output.length+output.charCodeAt(output.length-2)}return sum})()",
        expected: 115664.0,
        jobs: false,
    },
    Case {
        name: "uri_decode",
        source: "(()=>{const input='https://example.test/'+'abé漢'.repeat(2048)+'\\ud800';let sum=0;for(let i=0;i<16;i++){const output=decodeURI(input);sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 1_016_160.0,
        jobs: false,
    },
    Case {
        name: "uri_decode_mixed",
        source: "(()=>{const input='https://example.test/'+'abé漢'.repeat(2048)+'%2f%20%E2%82%AC';let sum=0;for(let i=0;i<16;i++){const output=(i%2?decodeURIComponent:decodeURI)(input);sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 265_296.0,
        jobs: false,
    },
    Case {
        name: "uri_encode",
        source: "(()=>{const input='https://example.test/'+'abc_-.'.repeat(2048);let sum=0;for(let i=0;i<16;i++){const output=encodeURI(input);sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 197_680.0,
        jobs: false,
    },
    Case {
        name: "uri_encode_mixed",
        source: "(()=>{const input='abc_DEF-123~'.repeat(1024)+' /é漢💩';let sum=0;for(let i=0;i<16;i++){const output=(i%2?encodeURIComponent:encodeURI)(input);sum+=output.length+output.charCodeAt(output.length-1)}return sum})()",
        expected: 198_032.0,
        jobs: false,
    },
    Case {
        name: "regexp",
        source: "(()=>{const text='aaaab'.repeat(128),re=/(?<prefix>a+)(?<suffix>b)/gu;let sum=0,match;while((match=re.exec(text))!==null)sum+=match[0].length;return sum})()",
        expected: 640.0,
        jobs: false,
    },
    Case {
        name: "generator",
        source: "(()=>{function* values(){for(let i=0;i<128;i++)yield i}let sum=0;for(const value of values())sum+=value;return sum})()",
        expected: 8128.0,
        jobs: false,
    },
    Case {
        name: "async_jobs",
        source: "globalThis.__spite_bench_result=-1;(async()=>{let sum=0;for(let i=0;i<64;i++)sum+=await i;return sum})().then(value=>{globalThis.__spite_bench_result=value});0",
        expected: 2016.0,
        jobs: true,
    },
];

fn measure(
    name: &str,
    samples: usize,
    iterations: usize,
    mut run: impl FnMut() -> Result<(Duration, Duration)>,
) -> Result<()> {
    // Warm-up and validation use the same path as each measured invocation.
    run()?;
    let mut evaluation = Vec::new();
    let mut collection = Vec::new();
    for _ in 0..samples {
        let (mut elapsed, mut collected) = (Duration::ZERO, Duration::ZERO);
        for _ in 0..iterations {
            let (run, gc) = run()?;
            elapsed += run;
            collected += gc;
        }
        evaluation.push(elapsed);
        collection.push(collected);
    }
    evaluation.sort_unstable();
    collection.sort_unstable();
    let micros = |times: &[Duration]| {
        let middle = times.len() / 2;
        let seconds = if times.len().is_multiple_of(2) {
            (times[middle - 1].as_secs_f64() + times[middle].as_secs_f64()) / 2.0
        } else {
            times[middle].as_secs_f64()
        };
        seconds * 1e6 / iterations as f64
    };
    println!(
        "{name},{samples},{iterations},{:.3},{:.3}",
        micros(&evaluation),
        micros(&collection),
    );
    Ok(())
}

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let samples = arguments
        .next()
        .unwrap_or_else(|| "7".into())
        .parse::<NonZeroUsize>()?
        .get();
    let iterations = arguments
        .next()
        .unwrap_or_else(|| "10".into())
        .parse::<NonZeroUsize>()?
        .get();
    if arguments.next().is_some() {
        return Err("usage: bench [positive sample count] [positive iterations per sample]".into());
    }
    println!("case,samples,iterations,median_us,collection_median_us");
    measure("parse_all", samples, iterations, || {
        let start = Instant::now();
        for case in CASES {
            drop(black_box(parse_script(black_box(case.source))?));
        }
        Ok((start.elapsed(), Duration::ZERO))
    })?;
    let empty = parse_script("")?;
    measure("initialize_and_drop", samples, iterations, || {
        let start = Instant::now();
        let mut realm = Realm::default();
        black_box(realm.evaluate(&empty)?);
        drop(realm);
        Ok((start.elapsed(), Duration::ZERO))
    })?;
    let result_script = parse_script("globalThis.__spite_bench_result")?;
    for case in CASES {
        let script = parse_script(case.source)?;
        let mut realm = Realm::default();
        // Initialize outside evaluation measurements; warm-up also collects.
        realm.evaluate(&empty)?;
        measure(case.name, samples, iterations, || {
            let start = Instant::now();
            let mut result = realm.evaluate(black_box(&script))?;
            if case.jobs {
                loop {
                    match realm.run_next_promise_job()? {
                        PromiseJobOutcome::NoJob => break,
                        PromiseJobOutcome::Completed => (),
                        PromiseJobOutcome::Reported(error) => return Err(error.into()),
                    }
                }
                result = realm.evaluate(&result_script)?;
            }
            let elapsed = start.elapsed();
            if !black_box(result).same_value(&Value::Number(case.expected)) {
                return Err(format!("{} produced an incorrect result", case.name).into());
            }
            let start = Instant::now();
            realm.clear_kept_objects()?;
            realm.collect(usize::MAX)?;
            Ok((elapsed, start.elapsed()))
        })?;
    }
    Ok(())
}
