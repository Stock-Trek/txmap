#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{
    hash::Hash,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Not},
};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct HashCode(pub u64);

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ShardCount(pub u8);

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ShardIndex(pub u8);

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BitMask(pub u128);

impl ShardIndex {
    pub fn bitmask(&self) -> BitMask {
        BitMask(1 << self.0)
    }
}

impl BitMask {
    pub const ZERO: BitMask = BitMask(0);
}

impl Deref for HashCode {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ShardCount {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ShardIndex {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BitMask {
    type Target = u128;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Not for BitMask {
    type Output = BitMask;
    fn not(self) -> Self::Output {
        BitMask(!self.0)
    }
}

impl BitOr for BitMask {
    type Output = BitMask;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitMask(self.0 | rhs.0)
    }
}

impl BitAnd for BitMask {
    type Output = BitMask;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitMask(self.0 & rhs.0)
    }
}

impl BitXor for BitMask {
    type Output = BitMask;
    fn bitxor(self, rhs: Self) -> Self::Output {
        BitMask(self.0 ^ rhs.0)
    }
}

impl BitOrAssign for BitMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for BitMask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitXorAssign for BitMask {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
