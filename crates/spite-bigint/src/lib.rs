//! Safe arbitrary-precision integers with explicit size and optional work budgets.

use std::{cmp::Ordering, fmt};

/// An arithmetic or resource failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Radix is outside 2 through 36.
    InvalidRadix,
    /// Input is empty or contains a character outside the requested radix.
    InvalidDigit,
    /// Division or remainder by zero.
    DivisionByZero,
    /// Exponentiation requires a non-negative exponent.
    NegativeExponent,
    /// An operation exceeds its size or work budget.
    Limit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRadix => "invalid integer radix",
            Self::InvalidDigit => "invalid integer digit",
            Self::DivisionByZero => "integer division by zero",
            Self::NegativeExponent => "negative integer exponent",
            Self::Limit => "integer resource limit exceeded",
        })
    }
}
impl std::error::Error for Error {}

/// An operation on infinitely sign-extended two's complement integers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitwiseOp {
    /// Set bits present in both operands.
    And,
    /// Set bits present in either operand.
    Or,
    /// Set bits present in exactly one operand.
    Xor,
}

/// Limits integer result size and remaining arithmetic work.
#[derive(Clone, Debug)]
pub struct Budget {
    max_bits: usize,
    remaining_work: Option<usize>,
}

impl Budget {
    /// Sets the maximum result magnitude in bits and available work units.
    pub fn new(max_bits: usize, work: usize) -> Self {
        Self::with_work_limit(max_bits, Some(work))
    }

    /// Sets the maximum result magnitude and an optional work limit.
    /// `None` disables work accounting while retaining the result size limit.
    pub fn with_work_limit(max_bits: usize, work: Option<usize>) -> Self {
        Self {
            max_bits,
            remaining_work: work,
        }
    }

    /// Returns the unspent work units, or `None` when work is unlimited.
    pub fn remaining_work(&self) -> Option<usize> {
        self.remaining_work
    }

    /// Reserves work before an operation, including caller-owned conversion work.
    pub fn charge(&mut self, work: usize) -> Result<(), Error> {
        if let Some(remaining) = &mut self.remaining_work {
            *remaining = remaining.checked_sub(work).ok_or(Error::Limit)?;
        }
        Ok(())
    }

    fn finish(&self, value: BigInt) -> Result<BigInt, Error> {
        if value.bit_length() > self.max_bits {
            Err(Error::Limit)
        } else {
            Ok(value)
        }
    }
}

/// A signed integer with no fixed precision.
///
/// Magnitudes use normalized little-endian 32-bit words. Zero is non-negative.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BigInt {
    negative: bool,
    words: Vec<u32>,
}

impl BigInt {
    /// Parses unsigned digits without whitespace, a sign, prefixes, or separators.
    pub fn parse_digits(digits: &str, radix: u32, budget: &mut Budget) -> Result<Self, Error> {
        if !(2..=36).contains(&radix) {
            return Err(Error::InvalidRadix);
        }
        if digits.is_empty() {
            return Err(Error::InvalidDigit);
        }
        budget.charge(1)?;
        let mut value = Self::default();
        for byte in digits.bytes() {
            let digit = char::from(byte)
                .to_digit(radix)
                .ok_or(Error::InvalidDigit)?;
            budget.charge(value.words.len().max(1))?;
            value.multiply_add_small(radix, digit);
            if value.bit_length() > budget.max_bits {
                return Err(Error::Limit);
            }
        }
        Ok(value)
    }

    /// Formats the integer using lowercase digits in radix 2 through 36.
    pub fn to_radix(&self, radix: u32, budget: &mut Budget) -> Result<String, Error> {
        if !(2..=36).contains(&radix) {
            return Err(Error::InvalidRadix);
        }
        budget.charge(self.words.len() + 1)?;
        if self.is_zero() {
            return Ok("0".into());
        }
        let mut magnitude = self.words.clone();
        let mut digits = Vec::new();
        while !magnitude.is_empty() {
            budget.charge(magnitude.len())?;
            let mut remainder = 0u64;
            for word in magnitude.iter_mut().rev() {
                let n = (remainder << 32) | u64::from(*word);
                *word = (n / u64::from(radix)) as u32;
                remainder = n % u64::from(radix);
            }
            while magnitude.last() == Some(&0) {
                magnitude.pop();
            }
            digits
                .push(char::from_digit(remainder as u32, radix).expect("valid radix digit") as u8);
        }
        if self.negative {
            digits.push(b'-');
        }
        digits.reverse();
        Ok(String::from_utf8(digits).expect("radix digits are ASCII"))
    }

    /// Returns whether the integer is zero.
    pub fn is_zero(&self) -> bool {
        self.words.is_empty()
    }

    /// Returns whether the integer is less than zero.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns the number of bits in the magnitude, or zero for zero.
    pub fn bit_length(&self) -> usize {
        self.words.last().map_or(0, |word| {
            (self.words.len() - 1) * 32 + (32 - word.leading_zeros()) as usize
        })
    }

    /// Returns whether the integer is odd, regardless of its sign.
    pub fn is_odd(&self) -> bool {
        self.words.first().is_some_and(|word| word & 1 != 0)
    }

    /// Converts a non-negative integer to a host index when it fits exactly.
    pub fn to_usize(&self) -> Option<usize> {
        (!self.negative)
            .then(|| self.magnitude_to_usize())
            .flatten()
    }

    /// Rounds to binary64 using nearest, ties to even, including signed overflow.
    ///
    /// This implements the mathematical integer-to-Number conversion used by
    /// ECMA-262 21.1.1.1. It does not round intermediate words or allocate a copy.
    pub fn to_f64(&self, budget: &mut Budget) -> Result<f64, Error> {
        budget.charge(1)?;
        let mut length = self.bit_length();
        let sign = u64::from(self.negative) << 63;
        if length == 0 {
            return Ok(0.0);
        }
        if length > 1024 {
            return Ok(f64::from_bits(sign | f64::INFINITY.to_bits()));
        }
        budget.charge(length)?;
        let kept = length.min(53);
        let discarded = length - kept;
        let bit = |index: usize| (self.words[index / 32] >> (index % 32)) & 1;
        let mut significand = 0u64;
        for index in (discarded..length).rev() {
            significand = (significand << 1) | u64::from(bit(index));
        }
        if discarded > 0 {
            let guard = bit(discarded - 1) != 0;
            let sticky = (0..discarded - 1).any(|index| bit(index) != 0);
            if guard && (sticky || significand & 1 != 0) {
                significand += 1;
                if significand == 1u64 << 53 {
                    significand >>= 1;
                    length += 1;
                }
            }
        }
        if length > 1024 {
            return Ok(f64::from_bits(sign | f64::INFINITY.to_bits()));
        }
        let exponent = (length as u64 - 1 + 1023) << 52;
        let fraction = (significand << (53 - kept)) & ((1u64 << 52) - 1);
        Ok(f64::from_bits(sign | exponent | fraction))
    }

    /// Compares mathematical values exactly, without rounding this integer to binary64.
    ///
    /// NaN is unordered. Infinities and fractional finite numbers are supported.
    pub fn cmp_f64(&self, number: f64, budget: &mut Budget) -> Result<Option<Ordering>, Error> {
        budget.charge(1)?;
        if number.is_nan() {
            return Ok(None);
        }
        if number.is_infinite() {
            return Ok(Some(if number.is_sign_positive() {
                Ordering::Less
            } else {
                Ordering::Greater
            }));
        }
        if self.is_zero() {
            return Ok(0.0f64.partial_cmp(&number));
        }
        if number == 0.0 || self.negative != number.is_sign_negative() {
            return Ok(Some(if self.negative {
                Ordering::Less
            } else {
                Ordering::Greater
            }));
        }
        let exponent = ((number.to_bits() >> 52) & 0x7ff) as i32 - 1023;
        let magnitude_order = if exponent < 0 {
            // Every nonzero integer has magnitude at least one.
            Ordering::Greater
        } else {
            let length = exponent as usize + 1;
            let mut order = self.bit_length().cmp(&length);
            if order.is_eq() {
                budget.charge(length)?;
                let significand = (number.to_bits() & ((1u64 << 52) - 1)) | (1u64 << 52);
                for bit in (0..length).rev() {
                    let integer_bit = (self.words[bit / 32] >> (bit % 32)) & 1;
                    let significand_bit = bit as i32 - (exponent - 52);
                    let number_bit = if (0..53).contains(&significand_bit) {
                        ((significand >> significand_bit) & 1) as u32
                    } else {
                        0
                    };
                    order = integer_bit.cmp(&number_bit);
                    if !order.is_eq() {
                        break;
                    }
                }
                if order.is_eq()
                    && exponent < 52
                    && significand & ((1u64 << (52 - exponent)) - 1) != 0
                {
                    order = Ordering::Less;
                }
            }
            order
        };
        Ok(Some(if self.negative {
            magnitude_order.reverse()
        } else {
            magnitude_order
        }))
    }

    /// Returns the negated integer.
    pub fn neg(&self, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(self.words.len() + 1)?;
        budget.finish(Self::normalized(!self.negative, self.words.clone()))
    }

    /// Adds two integers.
    pub fn add(&self, other: &Self, budget: &mut Budget) -> Result<Self, Error> {
        self.add_signed(other, other.negative, budget)
    }

    /// Subtracts another integer.
    pub fn sub(&self, other: &Self, budget: &mut Budget) -> Result<Self, Error> {
        self.add_signed(other, !other.negative, budget)
    }

    /// Multiplies two integers.
    pub fn mul(&self, other: &Self, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(1)?;
        if self.is_zero() || other.is_zero() {
            return Ok(Self::default());
        }
        let minimum_bits = self
            .bit_length()
            .checked_add(other.bit_length())
            .and_then(|n| n.checked_sub(1))
            .ok_or(Error::Limit)?;
        if minimum_bits > budget.max_bits {
            return Err(Error::Limit);
        }
        let work = self
            .words
            .len()
            .checked_mul(other.words.len())
            .and_then(|n| n.checked_add(self.words.len()))
            .ok_or(Error::Limit)?;
        budget.charge(work)?;
        let length = self
            .words
            .len()
            .checked_add(other.words.len())
            .ok_or(Error::Limit)?;
        let mut words = vec![0u32; length];
        for (i, a) in self.words.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in other.words.iter().enumerate() {
                // The product, destination word, and carry sum to at most u64::MAX.
                let n = u64::from(*a) * u64::from(*b) + u64::from(words[i + j]) + carry;
                words[i + j] = n as u32;
                carry = n >> 32;
            }
            words[i + other.words.len()] = carry as u32;
        }
        budget.finish(Self::normalized(self.negative != other.negative, words))
    }

    /// Divides with truncation toward zero, returning quotient and remainder.
    ///
    /// The remainder is zero or has the dividend's sign (ECMA-262 6.1.6.2.5–6).
    pub fn div_rem(&self, other: &Self, budget: &mut Budget) -> Result<(Self, Self), Error> {
        budget.charge(self.words.len() + 1)?;
        if other.is_zero() {
            return Err(Error::DivisionByZero);
        }
        if self.cmp_magnitude(other).is_lt() {
            return Ok((Self::default(), budget.finish(self.clone())?));
        }
        let mut quotient = vec![0u32; self.words.len()];
        let mut remainder = Self::default();
        let step_work = other
            .words
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(4))
            .ok_or(Error::Limit)?;
        // Binary long division maintains 0 <= remainder < divisor after each bit.
        for bit in (0..self.bit_length()).rev() {
            budget.charge(step_work)?;
            remainder.multiply_add_small(2, (self.words[bit / 32] >> (bit % 32)) & 1);
            if !remainder.cmp_magnitude(other).is_lt() {
                remainder =
                    Self::normalized(false, Self::sub_magnitudes(&remainder.words, &other.words));
                quotient[bit / 32] |= 1 << (bit % 32);
            }
        }
        let quotient =
            budget.finish(Self::normalized(self.negative != other.negative, quotient))?;
        remainder.negative = self.negative && !remainder.is_zero();
        Ok((quotient, budget.finish(remainder)?))
    }

    /// Raises this integer to a non-negative integer power (ECMA-262 6.1.6.2.3).
    pub fn pow(&self, exponent: &Self, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(self.words.len() + 1)?;
        if exponent.negative {
            return Err(Error::NegativeExponent);
        }
        if exponent.is_zero() {
            return budget.finish(Self::from(1));
        }
        if self.is_zero() {
            return Ok(Self::default());
        }
        if self.words == [1] {
            return budget.finish(Self::from(if self.negative && exponent.is_odd() {
                -1
            } else {
                1
            }));
        }
        let mut exponent = exponent.to_usize().ok_or(Error::Limit)?;
        let minimum_bits = (self.bit_length() - 1)
            .checked_mul(exponent)
            .and_then(|n| n.checked_add(1))
            .ok_or(Error::Limit)?;
        if minimum_bits > budget.max_bits {
            return Err(Error::Limit);
        }
        let mut base = self.clone();
        let mut result = Self::from(1);
        while exponent != 0 {
            if exponent & 1 != 0 {
                result = result.mul(&base, budget)?;
            }
            exponent >>= 1;
            if exponent != 0 {
                base = base.mul(&base, budget)?;
            }
        }
        Ok(result)
    }

    /// Shifts left; negative counts shift right with rounding toward minus infinity.
    pub fn shl(&self, count: &Self, budget: &mut Budget) -> Result<Self, Error> {
        self.shift(count, count.negative, budget)
    }

    /// Shifts right with rounding toward minus infinity; negative counts shift left.
    pub fn shr(&self, count: &Self, budget: &mut Budget) -> Result<Self, Error> {
        self.shift(count, !count.negative, budget)
    }

    /// Complements every bit, including the infinite sign extension.
    pub fn bitnot(&self, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(self.words.len() + 1)?;
        let words = if self.negative {
            Self::sub_magnitudes(&self.words, &[1])
        } else {
            Self::add_magnitudes(&self.words, &[1])
        };
        budget.finish(Self::normalized(!self.negative, words))
    }

    /// Combines infinitely sign-extended bits (ECMA-262 6.1.6.2.17).
    pub fn bitwise(&self, other: &Self, op: BitwiseOp, budget: &mut Budget) -> Result<Self, Error> {
        // An extra word preserves the sign after a finite two's complement operation.
        let width = self.words.len().max(other.words.len()) + 1;
        budget.charge(width.checked_mul(6).ok_or(Error::Limit)?)?;
        let mut left = self.twos_complement(width);
        let right = other.twos_complement(width);
        for (a, b) in left.iter_mut().zip(right) {
            *a = match op {
                BitwiseOp::And => *a & b,
                BitwiseOp::Or => *a | b,
                BitwiseOp::Xor => *a ^ b,
            };
        }
        let negative = left[width - 1] >> 31 != 0;
        if negative {
            Self::negate_words(&mut left);
        }
        budget.finish(Self::normalized(negative, left))
    }

    fn shift(&self, count: &Self, right: bool, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(self.words.len() + 1)?;
        if self.is_zero() {
            return Ok(Self::default());
        }
        let count = count.magnitude_to_usize();
        if right && count.is_none_or(|count| count >= self.bit_length()) {
            return budget.finish(Self::from(if self.negative { -1 } else { 0 }));
        }
        let count = count.ok_or(Error::Limit)?;
        let whole = count / 32;
        let part = count % 32;
        if right {
            let discarded = self.words[..whole].iter().any(|&word| word != 0)
                || (part != 0 && self.words[whole] & ((1 << part) - 1) != 0);
            let mut words = self.words[whole..].to_vec();
            if part != 0 {
                let mut carry = 0;
                for word in words.iter_mut().rev() {
                    let next = *word << (32 - part);
                    *word = (*word >> part) | carry;
                    carry = next;
                }
            }
            let mut result = Self::normalized(self.negative, words);
            if self.negative && discarded {
                result.multiply_add_small(1, 1);
                result.negative = true;
            }
            budget.finish(result)
        } else {
            let bits = self.bit_length().checked_add(count).ok_or(Error::Limit)?;
            if bits > budget.max_bits {
                return Err(Error::Limit);
            }
            let length = bits.div_ceil(32);
            budget.charge(length)?;
            let mut words = vec![0; length];
            let mut carry = 0u64;
            for (i, &word) in self.words.iter().enumerate() {
                let shifted = (u64::from(word) << part) | carry;
                words[i + whole] = shifted as u32;
                carry = shifted >> 32;
            }
            if carry != 0 {
                words[whole + self.words.len()] = carry as u32;
            }
            budget.finish(Self::normalized(self.negative, words))
        }
    }

    fn magnitude_to_usize(&self) -> Option<usize> {
        if self.bit_length() > usize::BITS as usize {
            return None;
        }
        Some(
            self.words
                .iter()
                .enumerate()
                .fold(0, |n, (i, &word)| n | ((word as usize) << (i * 32))),
        )
    }

    fn twos_complement(&self, width: usize) -> Vec<u32> {
        let mut words = self.words.clone();
        words.resize(width, 0);
        if self.negative {
            Self::negate_words(&mut words);
        }
        words
    }

    fn negate_words(words: &mut [u32]) {
        let mut carry = true;
        for word in words {
            let (n, overflow) = (!*word).overflowing_add(u32::from(carry));
            *word = n;
            carry = overflow;
        }
    }

    fn add_signed(&self, other: &Self, negative: bool, budget: &mut Budget) -> Result<Self, Error> {
        budget.charge(self.words.len().max(other.words.len()) + 1)?;
        let (negative, words) = if self.negative == negative {
            (negative, Self::add_magnitudes(&self.words, &other.words))
        } else {
            match self.cmp_magnitude(other) {
                Ordering::Equal => (false, Vec::new()),
                Ordering::Greater => (
                    self.negative,
                    Self::sub_magnitudes(&self.words, &other.words),
                ),
                Ordering::Less => (negative, Self::sub_magnitudes(&other.words, &self.words)),
            }
        };
        budget.finish(Self::normalized(negative, words))
    }

    fn normalized(negative: bool, mut words: Vec<u32>) -> Self {
        while words.last() == Some(&0) {
            words.pop();
        }
        Self {
            negative: negative && !words.is_empty(),
            words,
        }
    }

    fn cmp_magnitude(&self, other: &Self) -> Ordering {
        self.words
            .len()
            .cmp(&other.words.len())
            .then_with(|| self.words.iter().rev().cmp(other.words.iter().rev()))
    }

    fn multiply_add_small(&mut self, multiplier: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for word in &mut self.words {
            let n = u64::from(*word) * u64::from(multiplier) + carry;
            *word = n as u32;
            carry = n >> 32;
        }
        if carry != 0 {
            self.words.push(carry as u32);
        }
    }

    fn add_magnitudes(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut result = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let n = u64::from(a.get(i).copied().unwrap_or(0))
                + u64::from(b.get(i).copied().unwrap_or(0))
                + carry;
            result.push(n as u32);
            carry = n >> 32;
        }
        if carry != 0 {
            result.push(carry as u32);
        }
        result
    }

    // Requires a >= b. Each subtraction uses explicit unsigned borrow.
    fn sub_magnitudes(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut result = Vec::with_capacity(a.len());
        let mut borrow = false;
        for (i, word) in a.iter().enumerate() {
            let (n, b1) = word.overflowing_sub(b.get(i).copied().unwrap_or(0));
            let (n, b2) = n.overflowing_sub(u32::from(borrow));
            result.push(n);
            borrow = b1 || b2;
        }
        debug_assert!(!borrow);
        result
    }
}

impl From<i64> for BigInt {
    fn from(value: i64) -> Self {
        let magnitude = value.unsigned_abs();
        Self::normalized(value < 0, vec![magnitude as u32, (magnitude >> 32) as u32])
    }
}

// Hexadecimal host formatting is linear, streaming, and does not perform division.
impl fmt::LowerHex for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negative {
            f.write_str("-")?;
        }
        if f.alternate() {
            f.write_str("0x")?;
        }
        let mut words = self.words.iter().rev();
        write!(f, "{:x}", words.next().unwrap_or(&0))?;
        for word in words {
            write!(f, "{word:08x}")?;
        }
        Ok(())
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.cmp_magnitude(other),
            (true, true) => other.cmp_magnitude(self),
        }
    }
}
impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
