//! Exact conversion to and from native 64-bit integer words.

use crate::{BigInt, Budget, Error};

impl BigInt {
    /// Constructs an exact unsigned integer, checking result size and work limits.
    pub fn from_u64(value: u64, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(1)?;
        let bits = (u64::BITS - value.leading_zeros()) as usize;
        if budget.max_bits.is_some_and(|limit| bits > limit) {
            return Err(Error::Limit);
        }
        let count = bits.div_ceil(32);
        budget.charge(count)?;
        let mut words = Self::zero_words(count)?;
        if let Some(low) = words.first_mut() {
            *low = value as u32;
        }
        if let Some(high) = words.get_mut(1) {
            *high = (value >> 32) as u32;
        }
        Ok(Self {
            negative: false,
            words,
        })
    }

    /// Constructs an exact signed integer, including the minimum native value.
    /// Size limits apply to the result's magnitude rather than its raw encoding.
    pub fn from_i64(value: i64, budget: &mut Budget) -> Result<Self, Error> {
        let mut result = Self::from_u64(value.unsigned_abs(), budget)?;
        result.negative = value < 0;
        Ok(result)
    }

    /// Returns the non-negative residue modulo 2^64 without allocating.
    /// Only the low two magnitude words contribute, even for very large inputs.
    /// The native result does not consume the BigInt result-size allowance.
    pub fn modulo_u64(&self, budget: &mut Budget) -> Result<u64, Error> {
        budget.charge(1 + self.words.len().min(2))?;
        let low = u64::from(self.words.first().copied().unwrap_or(0));
        let high = u64::from(self.words.get(1).copied().unwrap_or(0));
        let value = low | (high << 32);
        Ok(if self.negative {
            value.wrapping_neg()
        } else {
            value
        })
    }
}
