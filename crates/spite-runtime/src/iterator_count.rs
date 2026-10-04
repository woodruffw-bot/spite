//! Exact mathematical iterator indices, converted only at callback calls.

use spite_bigint::{BigInt, Budget, Error as IntegerError};

#[derive(Debug)]
pub(crate) enum Counter {
    Small(u64),
    Large(BigInt),
}

impl Counter {
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
