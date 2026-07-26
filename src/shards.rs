#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::new_types::ShardCount;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Shards {
    _8,
    _16,
    _32,
    _64,
    _128,
}

impl From<Shards> for ShardCount {
    fn from(value: Shards) -> Self {
        match value {
            Shards::_8 => ShardCount(8),
            Shards::_16 => ShardCount(16),
            Shards::_32 => ShardCount(32),
            Shards::_64 => ShardCount(64),
            Shards::_128 => ShardCount(128),
        }
    }
}

impl std::fmt::Display for Shards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::_8 => write!(f, "ShardCount::_8"),
            Self::_16 => write!(f, "ShardCount::_16"),
            Self::_32 => write!(f, "ShardCount::_32"),
            Self::_64 => write!(f, "ShardCount::_64"),
            Self::_128 => write!(f, "ShardCount::_128"),
        }
    }
}
