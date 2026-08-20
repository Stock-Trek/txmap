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
        K: Hash + Eq,
    {
        let hash_code = self.hash(&key);
        let shard_index = Self::shard_index(shard_count, hash_code);
        TxKey {
            hash_code,
            shard_index,
            key,
        }
    }

    pub(crate) fn shard_index(shard_count: ShardCount, hash_code: HashCode) -> ShardIndex {
        // Select the shard from the middle bits of the hash (skipping the low
        // 7 bits) rather than the low bits. hashbrown's per-shard table uses
        // the low bits for the initial probe position (`h1(hash) & bucket_mask`)
        // and the top 7 bits for the control tag (`h2(hash)`). Selecting the
        // shard from the low bits would make every entry in a shard share
        // identical low bits, collapsing probe positions and clustering the
        // swiss table; middle bits keep full entropy in both the probe bits
        // and the tag bits.
        ShardIndex(((hash_code.0 >> 7) & (shard_count.0 as u64 - 1)) as u8)
    }

    pub(crate) fn hash<K>(&self, key: &K) -> HashCode
    where
        K: Hash,
    {
        HashCode(self.hasher_builder.hash_one(key))
    }
}
