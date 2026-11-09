//! Retained BigInt result storage; measure the built executable's peak RSS.

use spite_bigint::{BigInt, BitwiseOp, Budget};
use std::{error::Error, hint::black_box};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let bits = args
        .next()
        .map(|v| v.parse::<usize>())
        .transpose()?
        .unwrap_or(1_000_000);
    let count = args
        .next()
        .map(|v| v.parse::<usize>())
        .transpose()?
        .unwrap_or(512);
    let operation = args.next().unwrap_or_else(|| "and".into());
    if bits == 0
        || count == 0
        || args.next().is_some()
        || !["and", "or"].contains(&operation.as_str())
    {
        return Err("usage: zero_storage [positive-bits] [positive-count] [and|or]".into());
    }
    let mut unlimited = Budget::with_limits(None, None);
    let shift = BigInt::from_u64(u64::try_from(bits)?, &mut unlimited)?;
    let high = BigInt::from(1).shl(&shift, &mut unlimited)?;
    let low = high.sub(&BigInt::from(1), &mut unlimited)?;
    let op = if operation == "and" {
        BitwiseOp::And
    } else {
        BitwiseOp::Or
    };
    let expected_bits = if op == BitwiseOp::And {
        0
    } else {
        high.bit_length()
    };
    let mut budget = Budget::with_limits(Some(expected_bits), None);
    let mut results = Vec::new();
    results.try_reserve_exact(count)?;
    for _ in 0..count {
        results.push(high.bitwise(&low, op, &mut budget)?);
    }
    black_box(&results);
    assert!(
        results
            .iter()
            .all(|value| value.bit_length() == expected_bits)
    );
    println!("operation={operation},retained={count},result_bits={expected_bits}");
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status")?;
        let peak = status
            .lines()
            .find(|line| line.starts_with("VmHWM:"))
            .ok_or("missing VmHWM")?;
        println!("{peak}");
    }
    Ok(())
}
