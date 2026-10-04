use super::*;

#[test]
fn exact_sums_match_independent_rational_rounding_in_both_orders() {
    let fixtures = include_str!("../../../../tests/fixtures/number_sum.tsv");
    let mut count = 0;
    for line in fixtures.lines().filter(|line| !line.starts_with('#')) {
        let (inputs, expected) = line.split_once('\t').unwrap();
        let inputs: Vec<_> = inputs
            .split(',')
            .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
            .collect();
        let expected = u64::from_str_radix(expected, 16).unwrap();
        for reverse in [false, true] {
            let mut sum = ExactSum::new();
            for index in 0..inputs.len() {
                sum.add(
                    inputs[if reverse {
                        inputs.len() - 1 - index
                    } else {
                        index
                    }],
                );
            }
            assert_eq!(sum.round().to_bits(), expected, "{line}, reverse={reverse}");
        }
        count += 1;
    }
    assert_eq!(count, 464);
}

#[test]
fn individual_values_round_trip_through_every_normal_exponent_and_subnormal_bit() {
    for exponent in 1..=2046u64 {
        for fraction in [0, 1, 1 << 51, FRACTION_MASK] {
            for sign in [0, 1 << 63] {
                let bits = sign | (exponent << 52) | fraction;
                let mut sum = ExactSum::new();
                sum.add(f64::from_bits(bits));
                assert_eq!(sum.round().to_bits(), bits);
            }
        }
    }
    for shift in 0..52 {
        for sign in [0, 1 << 63] {
            let bits = sign | (1 << shift);
            let mut sum = ExactSum::new();
            sum.add(f64::from_bits(bits));
            assert_eq!(sum.round().to_bits(), bits);
        }
    }
}

#[test]
fn carry_and_cancellation_cross_all_words_without_intermediate_float_overflow() {
    let mut sum = ExactSum::new();
    for _ in 0..65536 {
        sum.add(f64::MAX);
        sum.add(f64::from_bits(1));
    }
    for _ in 0..65536 {
        sum.add(-f64::MAX);
    }
    assert_eq!(sum.round().to_bits(), 65536);
}

#[test]
fn specification_count_bound_steps_then_closes_and_allows_done_at_the_bound() {
    let mut realm = Realm::default();
    let items = realm.eval("let log='';let items={[Symbol.iterator](){return {next(){log+='n';return {get done(){log+='d';return false;},get value(){log+='v';return 'bad';}};},return(){log+='r';throw 7;}};}};items").unwrap();
    assert!(matches!(
        realm.precise_sum(items, MAX_COUNT, Span::new(0, 0)),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    assert_eq!(realm.eval("log==='ndvr'"), Ok(Value::Boolean(true)));
    let done = realm.eval("({[Symbol.iterator](){return {next(){return {done:true,get value(){throw 8;}};},return(){throw 9;}};}})").unwrap();
    let Value::Number(result) = realm.precise_sum(done, MAX_COUNT, Span::new(0, 0)).unwrap() else {
        panic!("sumPrecise returns a Number");
    };
    assert_eq!(result.to_bits(), (-0.0_f64).to_bits());
}
