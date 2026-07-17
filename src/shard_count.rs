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
