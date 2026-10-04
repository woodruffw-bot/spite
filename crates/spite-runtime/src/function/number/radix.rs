//! Shortest round-tripping radix output from exact rounding intervals (6.1.6.1.20).

use spite_bigint::{BigInt, Budget, Error};
use std::cmp::Ordering;

// All four quantities share a denominator. The two margins describe distances
// from the exact value to the midpoints with its adjacent binary64 values.
struct Interval {
    numerator: BigInt,
    denominator: BigInt,
    lower: BigInt,
    upper: BigInt,
}

impl Interval {
    fn new(value: f64, budget: &mut Budget) -> Result<(Self, i32), Error> {
        let (significand, exponent) = super::format::components(value);
        let binary_exponent = exponent + 63 - significand.leading_zeros() as i32;
        // A normal power of two has half the spacing below it, except at the
        // minimum normal where the subnormal spacing is the same on both sides.
        let asymmetric = significand == 1 << 52 && exponent > -1074;
        let mut interval = Self {
            numerator: BigInt::from(significand * 4),
            denominator: BigInt::from(1),
            lower: BigInt::from(if asymmetric { 1 } else { 2 }),
            upper: BigInt::from(2),
        };
        let binary_scale = exponent - 2;
        if binary_scale >= 0 {
            let power = BigInt::from(1).shl(&BigInt::from(i64::from(binary_scale)), budget)?;
            interval.scale_numerator(&power, budget)?;
        } else {
            interval.denominator =
                BigInt::from(1).shl(&BigInt::from(i64::from(-binary_scale)), budget)?;
        }
        Ok((interval, binary_exponent))
    }

    fn scale_numerator(&mut self, factor: &BigInt, budget: &mut Budget) -> Result<(), Error> {
        self.numerator = self.numerator.mul(factor, budget)?;
        self.lower = self.lower.mul(factor, budget)?;
        self.upper = self.upper.mul(factor, budget)?;
        Ok(())
    }

    fn normalize(
        &mut self,
        binary_exponent: i32,
        radix: u32,
        budget: &mut Budget,
    ) -> Result<i32, Error> {
        let base = BigInt::from(i64::from(radix));
        // Only a seed: platform rounding in log2 cannot affect the output,
        // because exact comparisons below establish 1 <= numerator/den < radix.
        let mut exponent = (f64::from(binary_exponent) / f64::from(radix).log2()).floor() as i32;
        let power = base.pow(&BigInt::from(i64::from(exponent.unsigned_abs())), budget)?;
        if exponent >= 0 {
            self.denominator = self.denominator.mul(&power, budget)?;
        } else {
            self.scale_numerator(&power, budget)?;
        }
        while compare(&self.numerator, &self.denominator, budget)?.is_lt() {
            self.scale_numerator(&base, budget)?;
            exponent -= 1;
        }
        loop {
            let next = self.denominator.mul(&base, budget)?;
            if compare(&self.numerator, &next, budget)?.is_lt() {
                break;
            }
            self.denominator = next;
            exponent += 1;
        }
        Ok(exponent + 1)
    }
}

fn compare(left: &BigInt, right: &BigInt, budget: &mut Budget) -> Result<Ordering, Error> {
    budget.charge(left.bit_length().max(right.bit_length()).div_ceil(32) + 1)?;
    Ok(left.cmp(right))
}

pub(super) fn format(value: f64, radix: u32, budget: &mut Budget) -> Result<String, Error> {
    debug_assert!(value.is_finite() && value != 0.0 && (2..=36).contains(&radix));
    let (mut interval, binary_exponent) = Interval::new(value.abs(), budget)?;
    let mut point = interval.normalize(binary_exponent, radix, budget)?;
    let inclusive = value.to_bits() & 1 == 0;
    let base = BigInt::from(i64::from(radix));
    let mut significand = BigInt::default();
    let mut length = 0;
    loop {
        // The quotient is one digit. At most radix-1 subtractions avoid a
        // general long division of the potentially 1076-bit denominator.
        let mut digit = 0;
        while !compare(&interval.numerator, &interval.denominator, budget)?.is_lt() {
            interval.numerator = interval.numerator.sub(&interval.denominator, budget)?;
            digit += 1;
        }
        debug_assert!(digit < radix);
        significand = significand
            .mul(&base, budget)?
            .add(&BigInt::from(i64::from(digit)), budget)?;
        length += 1;
        let below = compare(&interval.numerator, &interval.lower, budget)?;
        let above = compare(
            &interval.numerator.add(&interval.upper, budget)?,
            &interval.denominator,
            budget,
        )?;
        let low = below.is_lt() || inclusive && below.is_eq();
        let high = above.is_gt() || inclusive && above.is_eq();
        if low || high {
            // The first accepted prefix is shortest. When both neighbors fit,
            // use the nearest one, then an even significand on an exact tie.
            let round_up = if low && high {
                let twice = interval.numerator.mul(&BigInt::from(2), budget)?;
                let midpoint = compare(&twice, &interval.denominator, budget)?;
                midpoint.is_gt() || midpoint.is_eq() && significand.is_odd()
            } else {
                high
            };
            if round_up {
                significand = significand.add(&BigInt::from(1), budget)?;
            }
            break;
        }
        interval.scale_numerator(&base, budget)?;
    }
    let mut digits = significand.to_radix(radix, budget)?;
    if digits.len() > length {
        // Rounding 99...9 to 100...0 (using the requested radix).
        debug_assert_eq!(digits.len(), length + 1);
        point += 1;
    }
    while digits.ends_with('0') {
        digits.pop();
    }
    let mut result = String::new();
    if value < 0.0 {
        result.push('-');
    }
    // Non-decimal output always uses fixed notation, including extreme values.
    if point <= 0 {
        budget.charge((-point) as usize + digits.len() + 2)?;
        result.push_str("0.");
        result.extend(std::iter::repeat_n('0', (-point) as usize));
        result.push_str(&digits);
    } else if point as usize >= digits.len() {
        budget.charge(point as usize)?;
        result.push_str(&digits);
        result.extend(std::iter::repeat_n('0', point as usize - digits.len()));
    } else {
        budget.charge(digits.len() + 1)?;
        result.push_str(&digits[..point as usize]);
        result.push('.');
        result.push_str(&digits[point as usize..]);
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
