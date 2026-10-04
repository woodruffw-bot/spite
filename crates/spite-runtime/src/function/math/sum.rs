//! Exact iterable summation (sec-math.sumprecise).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{Span, WellKnownSymbol};
use std::cmp::Ordering;

const MAX_COUNT: u64 = 9_007_199_254_740_991;

impl Realm {
    pub(in crate::function) fn math_sum_precise(
        &mut self,
        items: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.precise_sum(items, 0, span)
    }

    fn precise_sum(&mut self, items: Value, mut count: u64, span: Span) -> Result<Value, Error> {
        Self::require_object_coercible(&items, span)?;
        let method = self
            .get_method(&items, &WellKnownSymbol::Iterator.symbol(), span)?
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "sumPrecise input is not iterable",
                )
            })?;
        let mut iterator = self.get_iterator_from_method(items, method, span)?;
        let mut sum = ExactSum::new();
        let mut only_minus_zero = true;
        let mut nan = false;
        let mut positive_infinity = false;
        let mut negative_infinity = false;
        loop {
            self.tick(span)?;
            // IteratorStepValue failures do not close. The specification count
            // bound follows stepping, including its done/value property reads.
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                break;
            };
            if count >= MAX_COUNT {
                let error = Self::exception(
                    ExceptionKind::RangeError,
                    span,
                    "sumPrecise exceeds the maximum safe integer count",
                );
                return Err(self.iterator_close_error(&iterator, error, span));
            }
            let Value::Number(number) = value else {
                let error = Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "sumPrecise elements must be Numbers",
                );
                return Err(self.iterator_close_error(&iterator, error, span));
            };
            if number.is_nan() {
                nan = true;
            } else if number == f64::INFINITY {
                positive_infinity = true;
            } else if number == f64::NEG_INFINITY {
                negative_infinity = true;
            } else {
                only_minus_zero &= number.to_bits() == (-0.0_f64).to_bits();
                if !nan && !positive_infinity && !negative_infinity {
                    sum.add(number);
                }
            }
            count += 1;
        }
        let result = if nan || (positive_infinity && negative_infinity) {
            f64::NAN
        } else if positive_infinity {
            f64::INFINITY
        } else if negative_infinity {
            f64::NEG_INFINITY
        } else if only_minus_zero {
            -0.0
        } else {
            sum.round()
        };
        Ok(Value::Number(result))
    }
}

// Each finite binary64 is an integer multiple of 2^-1074. For fewer than 2^53
// inputs, either sign's total magnitude is below 2^1077, requiring at most 2151
// bits in these units. 34 words provide 2176 bits. This capacity follows the
// specification count bound; it is not a host resource quota.
const WORDS: usize = 34;
const FRACTION_MASK: u64 = (1 << 52) - 1;

struct ExactSum {
    positive: [u64; WORDS],
    negative: [u64; WORDS],
}

impl ExactSum {
    fn new() -> Self {
        Self {
            positive: [0; WORDS],
            negative: [0; WORDS],
        }
    }

    fn add(&mut self, number: f64) {
        debug_assert!(number.is_finite());
        let bits = number.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as usize;
        let fraction = bits & FRACTION_MASK;
        let (significand, shift) = if exponent == 0 {
            (fraction, 0)
        } else {
            (fraction | (1 << 52), exponent - 1)
        };
        let words = if number.is_sign_negative() {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let index = shift / 64;
        let scaled = u128::from(significand) << (shift % 64);
        Self::add_word(words, index, scaled as u64);
        Self::add_word(words, index + 1, (scaled >> 64) as u64);
    }

    fn add_word(words: &mut [u64; WORDS], index: usize, mut carry: u64) {
        for word in &mut words[index..] {
            if carry == 0 {
                return;
            }
            let (total, overflow) = word.overflowing_add(carry);
            *word = total;
            carry = u64::from(overflow);
        }
        assert_eq!(
            carry, 0,
            "the specification count bound guarantees capacity"
        );
    }

    fn round(&self) -> f64 {
        let (greater, lesser, sign) =
            match self.positive.iter().rev().cmp(self.negative.iter().rev()) {
                Ordering::Equal => return 0.0,
                Ordering::Greater => (&self.positive, &self.negative, 0),
                Ordering::Less => (&self.negative, &self.positive, 1u64 << 63),
            };
        let mut magnitude = [0u64; WORDS];
        let mut borrow = false;
        for index in 0..WORDS {
            let (difference, first) = greater[index].overflowing_sub(lesser[index]);
            let (difference, second) = difference.overflowing_sub(u64::from(borrow));
            magnitude[index] = difference;
            borrow = first || second;
        }
        debug_assert!(!borrow);
        let high = magnitude.iter().rposition(|word| *word != 0).unwrap();
        let mut length = high * 64 + (64 - magnitude[high].leading_zeros()) as usize;
        if length <= 52 {
            // Subnormal sums are already exact integer multiples of 2^-1074.
            return f64::from_bits(sign | magnitude[0]);
        }
        if length > 2098 {
            return f64::from_bits(sign | f64::INFINITY.to_bits());
        }
        let discarded = length - 53;
        let bit = |index: usize| (magnitude[index / 64] >> (index % 64)) & 1;
        let mut significand = 0;
        for index in (discarded..length).rev() {
            significand = (significand << 1) | bit(index);
        }
        if discarded != 0 {
            let guard = bit(discarded - 1) != 0;
            let sticky = (0..discarded - 1).any(|index| bit(index) != 0);
            if guard && (sticky || significand & 1 != 0) {
                significand += 1;
                if significand == 1 << 53 {
                    significand >>= 1;
                    length += 1;
                }
            }
        }
        // Bias the highest set bit after accounting for the 2^-1074 unit.
        let exponent = length - 52;
        if exponent >= 0x7ff {
            return f64::from_bits(sign | f64::INFINITY.to_bits());
        }
        f64::from_bits(sign | ((exponent as u64) << 52) | (significand & FRACTION_MASK))
    }
}

#[cfg(test)]
mod tests;
