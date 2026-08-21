use crate::{
    hasher::DefaultBuildHasher,
    key::TxKey,
    new_types::{HashCode, ShardCount, ShardIndex},
};
use std::hash::{BuildHasher, Hash};

/// Hashes keys and maps them to shards.
pub struct Indexer<S: BuildHasher = DefaultBuildHasher> {
    hasher_builder: S,
}

impl Default for Indexer<DefaultBuildHasher> {
    fn default() -> Self {
        Self {
            hasher_builder: DefaultBuildHasher::default(),
        }
    }
}

impl<S: BuildHasher> Indexer<S> {
    pub(crate) fn new(hasher_builder: S) -> Self {
        Self { hasher_builder }
    }

    pub(crate) fn hasher_builder(&self) -> &S {
        &self.hasher_builder
    }

    pub fn indexed_key<K>(&self, shard_count: ShardCount, key: K) -> TxKey<K>
    where
        K: Hash,
    {
        let hash_code = self.hash(&key);
        let shard_index = Self::shard_index(shard_count, hash_code);
        TxKey {
            hash_code,
            shard_index,
            key,
        }
    }

    /// Uses the top `shard_count.trailing_zeros()` bits of the hash.
    ///
    /// Hashbrown stores a 7-bit control byte/tag in the low bits of the hash
    /// (`hash & 0x7F`). If sharding consumed those bits, all keys in the same
    /// shard would share the same control byte, hurting SwissTable's ability to
    /// reject non-matching entries quickly. Taking the high bits instead leaves
    /// the control byte intact, so per-shard tables retain full tag entropy.
    pub(crate) fn shard_index(shard_count: ShardCount, hash_code: HashCode) -> ShardIndex {
        let shift = 64 - shard_count.trailing_zeros();
        ShardIndex((hash_code.0 >> shift) as u8)
    }

    pub(crate) fn hash<K>(&self, key: &K) -> HashCode
    where
        K: Hash,
    {
        HashCode(self.hasher_builder.hash_one(key))
    }
}
