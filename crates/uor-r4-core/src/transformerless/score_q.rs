//! ScoreQ: Quantized fixed-point log-domain score representation (Q16.16 in i32).
//!
//! Replaces floating-point route scores in deployed inference artifacts.
//! Arithmetic is pure integer saturating add/sub — multiplication-free.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Q16.16 fixed-point log-domain score representation.
/// 16 integer bits (signed), 16 fractional bits. Scale factor = 65536.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ScoreQ(pub i32);

impl ScoreQ {
    pub const SCALE: f32 = 65536.0;
    pub const ZERO: ScoreQ = ScoreQ(0);
    pub const MIN: ScoreQ = ScoreQ(i32::MIN);
    pub const MAX: ScoreQ = ScoreQ(i32::MAX);

    /// Construct ScoreQ from log probability (float).
    pub fn from_logprob(lp: f32) -> Self {
        let val = (lp * Self::SCALE).clamp(i32::MIN as f32, i32::MAX as f32);
        ScoreQ(val as i32)
    }

    /// Convert ScoreQ back to log probability (float).
    pub fn to_logprob(self) -> f32 {
        self.0 as f32 / Self::SCALE
    }

    /// Construct ScoreQ directly from raw i32 Q16.16 representation.
    pub const fn from_raw(raw: i32) -> Self {
        ScoreQ(raw)
    }

    /// Raw i32 Q16.16 value.
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Saturating addition.
    pub fn saturating_add(self, rhs: Self) -> Self {
        ScoreQ(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction.
    pub fn saturating_sub(self, rhs: Self) -> Self {
        ScoreQ(self.0.saturating_sub(rhs.0))
    }
}

impl Add for ScoreQ {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl AddAssign for ScoreQ {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(rhs);
    }
}

impl Sub for ScoreQ {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

impl SubAssign for ScoreQ {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.saturating_sub(rhs);
    }
}

impl fmt::Display for ScoreQ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ScoreQ({:.4})", self.to_logprob())
    }
}

/// Dyadic storage descriptor for compact residual table decoding.
/// `{ width: i8|i16|i32, shift: i8, zero_point: i32 }`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageDescriptor {
    pub width_bits: u8,
    pub shift: i8,
    pub zero_point: i32,
}

impl StorageDescriptor {
    pub const fn new(width_bits: u8, shift: i8, zero_point: i32) -> Self {
        StorageDescriptor {
            width_bits,
            shift,
            zero_point,
        }
    }

    /// Decode raw integer entry into ScoreQ using shift + zero_point (mul-free).
    pub fn decode(&self, raw_entry: i32) -> ScoreQ {
        let centered = raw_entry.saturating_sub(self.zero_point);
        let raw_score = if self.shift >= 0 {
            centered.wrapping_shl(self.shift as u32)
        } else {
            centered.wrapping_shr((-self.shift) as u32)
        };
        ScoreQ(raw_score)
    }
}
