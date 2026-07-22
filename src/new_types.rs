use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HashCode(pub u64);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShardIndex(pub u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BitMask(pub u128);

impl ShardIndex {
    pub fn bitmask(&self) -> BitMask {
        BitMask(1 << self.0)
    }
}

impl Deref for HashCode {
    type Target = u64;
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

impl Default for BitMask {
    fn default() -> Self {
        BitMask(0)
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
