#[derive(Debug, Clone, Copy)]
pub enum ShardCount {
    _8,
    _16,
    _32,
    _64,
    _128,
}

impl From<ShardCount> for u8 {
    fn from(value: ShardCount) -> Self {
        match value {
            ShardCount::_8 => 8,
            ShardCount::_16 => 16,
            ShardCount::_32 => 32,
            ShardCount::_64 => 64,
            ShardCount::_128 => 128,
        }
    }
}

impl std::fmt::Display for ShardCount {
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

impl ShardCount {
    pub fn all() -> Vec<ShardCount> {
        vec![
            ShardCount::_8,
            ShardCount::_16,
            ShardCount::_32,
            ShardCount::_64,
            ShardCount::_128,
        ]
    }
    pub fn all_guards_bitmask(shard_count: u8) -> u128 {
        if shard_count == 128 {
            !0u128
        } else {
            (1 << shard_count) - 1
        }
    }
}
