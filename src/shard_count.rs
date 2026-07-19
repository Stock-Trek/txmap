#[derive(Clone, Copy)]
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

pub fn all_guards_bitmask(shard_count: u8) -> u128 {
    if shard_count == 128 {
        !0u128
    } else {
        (1 << shard_count) - 1
    }
}
