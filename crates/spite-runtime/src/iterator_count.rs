//! Exact mathematical iterator indices and take/drop countdowns.

use spite_bigint::{BigInt, Budget, Error as IntegerError};

#[derive(Debug)]
pub(crate) enum Counter {
    Small(u64),
    Large(BigInt),
}

impl Counter {
    pub(crate) fn from_number(value: f64, budget: &mut Budget) -> Result<Self, IntegerError> {
        debug_assert!(value.is_finite() && value >= 0.0 && value.fract() == 0.0);
        budget.charge(1)?;
        if value < 18_446_744_073_709_551_616.0 {
            Ok(Self::Small(value as u64))
        } else {
            Ok(Self::Large(
                BigInt::from_f64(value, budget)?.expect("integral Number"),
            ))
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Self::Small(value) => *value == 0,
            Self::Large(value) => value.is_zero(),
        }
    }

    pub(crate) fn decrement(&mut self, budget: &mut Budget) -> Result<(), IntegerError> {
        debug_assert!(!self.is_zero());
        budget.charge(1)?;
        match self {
            Self::Small(value) => *value -= 1,
            Self::Large(value) => *value = value.sub(&BigInt::from(1), budget)?,
        }
        Ok(())
    }

    pub(crate) fn number(&self, budget: &mut Budget) -> Result<f64, IntegerError> {
        match self {
            Self::Small(value) => {
                budget.charge(1)?;
                Ok(*value as f64)
            }
            Self::Large(value) => value.to_f64(budget),
        }
    }

    pub(crate) fn advance(&mut self, budget: &mut Budget) -> Result<(), IntegerError> {
        budget.charge(1)?;
        match self {
            Self::Small(value) if *value != u64::MAX => *value += 1,
            Self::Small(_) => {
                // The first value beyond u64 is exactly representable as 2^64.
                *self = Self::Large(
                    BigInt::from_f64(18_446_744_073_709_551_616.0, budget)?
                        .expect("integral Number"),
                );
            }
            Self::Large(value) => *value = value.add(&BigInt::from(1), budget)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_countdowns_subtract_exactly_above_binary64_precision_and_u64() {
        let mut budget = Budget::with_limits(None, None);
        let mut counter = Counter::from_number(9_007_199_254_740_994.0, &mut budget).unwrap();
        counter.decrement(&mut budget).unwrap();
        assert!(matches!(counter, Counter::Small(9_007_199_254_740_993)));
        let mut counter = Counter::from_number(18_446_744_073_709_551_616.0, &mut budget).unwrap();
        counter.decrement(&mut budget).unwrap();
        let Counter::Large(value) = counter else {
            panic!("large count")
        };
        assert_eq!(
            value.to_radix(10, &mut budget).unwrap(),
            "18446744073709551615"
        );
        let counter = Counter::from_number(f64::MAX, &mut budget).unwrap();
        assert_eq!(counter.number(&mut budget), Ok(f64::MAX));
        let mut counter = Counter::Large(BigInt::from(1));
        counter.decrement(&mut budget).unwrap();
        assert!(counter.is_zero());
        let mut counter = Counter::Small(7);
        assert_eq!(
            counter.decrement(&mut Budget::with_limits(None, Some(0))),
            Err(IntegerError::Limit)
        );
        assert!(matches!(counter, Counter::Small(7)));
    }
}
