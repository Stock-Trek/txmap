use crate::{
    indexed_key::IndexedKey,
    indexed_keys::IndexedKeys,
    new_types::{BitMask, HashCode, ShardIndex},
};
use rapidhash::fast::RapidHasher;
use std::hash::{Hash, Hasher};

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

impl ShardCount {
    pub(crate) fn indexes<E, K>(
        shard_count: u8,
        keys: impl IntoIterator<Item = E>,
        element_to_key: fn(E) -> K,
    ) -> IndexedKeys<K>
    where
        K: Hash + Eq,
    {
        let mut bitmask = BitMask::default();
        let iter = keys.into_iter();
        let mut indexed = Vec::with_capacity(iter.size_hint().0);
        for element in iter {
            let key = element_to_key(element);
            let hash_code = Self::hash(&key);
            let shard_index = Self::shard_index(shard_count, hash_code);
            let shard_bitmask = shard_index.bitmask();
            bitmask |= shard_bitmask;
            indexed.push(IndexedKey(hash_code, shard_index, shard_bitmask, key));
        }
        IndexedKeys { bitmask, indexed }
    }
    pub(crate) fn indexed_key<K>(shard_count: u8, key: K) -> IndexedKey<K>
    where
        K: Hash + Eq,
    {
        let hash_code = Self::hash(&key);
        let shard_index = Self::shard_index(shard_count, hash_code);
        let bitmask = shard_index.bitmask();
        IndexedKey(hash_code, shard_index, bitmask, key)
    }
    pub(crate) fn shard_index(shard_count: u8, hash_code: HashCode) -> ShardIndex {
        ShardIndex((hash_code.0 & (shard_count as u64 - 1)) as u8)
    }
    pub(crate) fn hash<K>(key: &K) -> HashCode
    where
        K: Hash,
    {
        let mut state = RapidHasher::default();
        key.hash(&mut state);
        HashCode(state.finish())
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
