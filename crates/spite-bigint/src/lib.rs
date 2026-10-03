//! Safe arbitrary-precision integers with explicit size and work budgets.

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
    /// An operation exceeds its size or work budget.
    Limit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRadix => "invalid integer radix",
            Self::InvalidDigit => "invalid integer digit",
            Self::DivisionByZero => "integer division by zero",
            Self::Limit => "integer resource limit exceeded",
        })
    }
}
impl std::error::Error for Error {}

/// Limits integer result size and remaining arithmetic work.
#[derive(Clone, Debug)]
pub struct Budget {
    max_bits: usize,
    remaining_work: usize,
}

impl Budget {
    /// Sets the maximum result magnitude in bits and available work units.
    pub fn new(max_bits: usize, work: usize) -> Self {
        Self {
            max_bits,
            remaining_work: work,
        }
    }

    /// Returns the unspent work units.
    pub fn remaining_work(&self) -> usize {
        self.remaining_work
    }

    fn charge(&mut self, work: usize) -> Result<(), Error> {
        self.remaining_work = self.remaining_work.checked_sub(work).ok_or(Error::Limit)?;
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
