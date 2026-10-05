//! MakeDay with exact mathematical normalization and finite calendar witnesses.

use spite_bigint::{BigInt, Budget, Error};

const MS: f64 = super::MS_PER_DAY as f64;

/// Applies MakeDay to numeric year, month, and day components.
///
/// Components truncate before normalization. Month division and Gregorian year
/// arithmetic round only at the specified Number boundaries, including huge
/// components that cancel. Calendar existence is checked before adding the day
/// component; only the eventual Date caller applies TimeClip.
///
/// The budget accounts for native integer work and can be unlimited. Runtime
/// callers must not apply a JavaScript BigInt magnitude quota to this Number
/// operation. See [MakeDay](https://262.ecma-international.org/17.0/#sec-makeday).
pub fn make_day(year: f64, month: f64, date: f64, budget: &mut Budget) -> Result<f64, Error> {
    budget.charge(1)?;
    if !year.is_finite() || !month.is_finite() || !date.is_finite() {
        return Ok(f64::NAN);
    }
    // 21.4.1.28: floor and modulo act on the mathematical integer month,
    // before converting the quotient to Number and adding the year.
    let month = month.trunc();
    let (shift, month) = if (i64::MIN as f64..-(i64::MIN as f64)).contains(&month) {
        let month = month as i64;
        (month.div_euclid(12) as f64, month.rem_euclid(12) as usize)
    } else {
        let integer = BigInt::from_f64(month, budget)?.expect("finite integral month");
        let (mut quotient, mut remainder) = integer.div_rem(&BigInt::from(12), budget)?;
        if remainder.is_negative() {
            quotient = quotient.sub(&BigInt::from(1), budget)?;
            remainder = remainder.add(&BigInt::from(12), budget)?;
        }
        (
            quotient.to_f64(budget)?,
            remainder
                .to_usize()
                .expect("Euclidean month remainder is 0–11"),
        )
    };
    let year = year.trunc() + shift;
    if !year.is_finite() {
        return Ok(f64::NAN);
    }
    let leap = year % 400.0 == 0.0 || (year % 4.0 == 0.0 && year % 100.0 != 0.0);
    let prefix = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][month]
        + i32::from(leap && month >= 2);
    // Every i32 year has a finite first-day witness: at this magnitude,
    // adjacent time values are less than one UTC day apart. The integer
    // calendar day and month prefix are exact Numbers. Larger years use the
    // same algorithm with explicit witness checks, without an input cap.
    if (i32::MIN as f64..=i32::MAX as f64).contains(&year) {
        let day = super::day_from_year(year as i32) + i64::from(prefix);
        return Ok((day as f64 + date.trunc()) - 1.0);
    }
    let start = day_from_year(year, budget)?;
    let day = start + f64::from(prefix);
    if !day.is_finite() || day - start != f64::from(prefix) {
        return Ok(f64::NAN);
    }
    // YearFromTime chooses the largest integral Number whose TimeFromYear
    // is at most t. These neighboring year boundaries handle rounded plateaus.
    let lower = start * MS;
    let next_year = if year.abs() < (1_u64 << 53) as f64 {
        year + 1.0
    } else {
        next_up(year)
    };
    let upper = day_from_year(next_year, budget)? * MS;
    let rounded = day * MS;
    let center = if rounded.is_infinite() {
        f64::MAX.copysign(rounded)
    } else {
        rounded
    };
    // Day(t) floors Number division (21.4.1.3). Testing the rounded month
    // boundary and its immediate neighbors decides whether a witness exists:
    // division is monotone, and values farther away cannot be the first one
    // mapping to the target day.
    // Saturation supplies the finite endpoint when a negative product
    // overflows; the year/day checks still decide whether a witness exists.
    for time in [center, next_down(center), next_up(center)] {
        if time.is_finite()
            && time.trunc() == time
            && lower <= time
            && time < upper
            && (time / MS).floor() == day
        {
            return Ok((day + date.trunc()) - 1.0);
        }
    }
    Ok(f64::NAN)
}

fn floor_div(integer: BigInt, divisor: i64, budget: &mut Budget) -> Result<BigInt, Error> {
    let (q, r) = integer.div_rem(&BigInt::from(divisor), budget)?;
    if r.is_negative() {
        q.sub(&BigInt::from(1), budget)
    } else {
        Ok(q)
    }
}

fn day_from_year(year: f64, budget: &mut Budget) -> Result<f64, Error> {
    if !year.is_finite() {
        return Ok(year);
    }
    if (i32::MIN as f64..=i32::MAX as f64).contains(&year) {
        return Ok(super::day_from_year(year as i32) as f64);
    }
    // DayFromYear (21.4.1.6) performs all of this integer arithmetic in the
    // mathematical domain and rounds once. Ordered floating additions here
    // would lose calendar corrections near Number rounding boundaries.
    let y = BigInt::from_f64(year, budget)?.expect("finite integral year");
    let ones = y
        .sub(&BigInt::from(1970), budget)?
        .mul(&BigInt::from(365), budget)?;
    let fours = floor_div(y.sub(&BigInt::from(1969), budget)?, 4, budget)?;
    let hundreds = floor_div(y.sub(&BigInt::from(1901), budget)?, 100, budget)?;
    let four_hundreds = floor_div(y.sub(&BigInt::from(1601), budget)?, 400, budget)?;
    ones.add(&fours, budget)?
        .sub(&hundreds, budget)?
        .add(&four_hundreds, budget)?
        .to_f64(budget)
}

fn next_up(value: f64) -> f64 {
    if value == f64::INFINITY {
        value
    } else if value == 0.0 {
        f64::from_bits(1)
    } else if value > 0.0 {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    }
}

fn next_down(value: f64) -> f64 {
    -next_up(-value)
}

#[cfg(test)]
mod tests;
