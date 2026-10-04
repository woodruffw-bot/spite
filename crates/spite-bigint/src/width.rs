//! Reduction modulo powers of two without constructing the modulus.

use crate::{BigInt, Budget, Error};

impl BigInt {
    /// Reduces modulo 2^bits, interpreting the result as a signed integer.
    pub fn as_int_n(&self, bits: u64, budget: &mut Budget) -> Result<Self, Error> {
        self.truncate_width(bits, true, budget)
    }

    /// Reduces modulo 2^bits, interpreting the result as an unsigned integer.
    pub fn as_uint_n(&self, bits: u64, budget: &mut Budget) -> Result<Self, Error> {
        self.truncate_width(bits, false, budget)
    }

    fn truncate_width(&self, bits: u64, signed: bool, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(1)?;
        if bits == 0 || self.is_zero() {
            return budget.finish(Self::default());
        }
        let length = self.bit_length() as u64;
        if (signed && length < bits) || (!signed && !self.negative && length <= bits) {
            // Huge widths need no width-sized allocation when the value fits.
            if budget
                .max_bits
                .is_some_and(|limit| self.bit_length() > limit)
            {
                return Err(Error::Limit);
            }
            budget.charge(self.words.len())?;
            let mut words = Self::zero_words(self.words.len())?;
            words.copy_from_slice(&self.words);
            return budget.finish(Self {
                negative: self.negative,
                words,
            });
        }
        if !signed
            && self.negative
            && length < bits
            && budget.max_bits.is_some_and(|limit| bits > limit as u64)
        {
            // Here 2^bits - |value| has exactly bits magnitude bits. Reject an
            // opted-in size quota before reserving the potentially huge output.
            return Err(Error::Limit);
        }
        let count = usize::try_from(bits.div_ceil(32)).map_err(|_| Error::Limit)?;
        budget.charge(count)?;
        let mut words = Self::zero_words(count)?;
        let copied = count.min(self.words.len());
        budget.charge(copied)?;
        words[..copied].copy_from_slice(&self.words[..copied]);
        if self.negative {
            budget.charge(count)?;
            Self::negate_words(&mut words);
        }
        let mask = if bits % 32 == 0 {
            u32::MAX
        } else {
            (1u32 << (bits % 32)) - 1
        };
        words[count - 1] &= mask;
        let negative = signed && words[count - 1] & (1u32 << ((bits - 1) % 32)) != 0;
        if negative {
            budget.charge(count)?;
            Self::negate_words(&mut words);
            words[count - 1] &= mask;
        }
        budget.charge(count)?;
        budget.finish(Self::normalized(negative, words))
    }
}
