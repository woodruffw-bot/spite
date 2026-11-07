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
        name: "radix_overflow",
        source: "(()=>{const text='0x'+'f'.repeat(8192);let sum=0;for(let i=0;i<16;i++){if(Number(text)===Infinity)sum++;if(Number.isNaN(Number(text+'g')))sum++}return sum})()",
        expected: 32.0,
        jobs: false,
    },
    Case {
        name: "properties",
        source: "(()=>{const object={};for(let i=0;i<128;i++)object['k'+i]=i;let sum=0;for(let i=0;i<128;i++)sum+=object['k'+i];return sum})()",
        expected: 8128.0,
        jobs: false,
    },
    Case {
        name: "string_search",
        source: "(()=>{const text='a'.repeat(4096)+'b',needle='a'.repeat(128)+'b';let sum=0;for(let i=0;i<8;i++)sum+=text.indexOf(needle);return sum})()",
        expected: 31744.0,
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
        name: "map_iteration",
        source: "(()=>{const map=new Map();for(let i=0;i<256;i++)map.set(i,i);let sum=0;for(const value of map.values())sum+=value;return sum})()",
        expected: 32640.0,
        jobs: false,
    },
    Case {
        name: "bigint",
        source: "(()=>{let x=1n<<256n;for(let i=0;i<64;i++)x=(x*3n+BigInt(i))/2n;return Number(x&65535n)})()",
        expected: 5661.0,
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
