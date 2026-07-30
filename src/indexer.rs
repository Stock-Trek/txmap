use crate::{
    hasher::DefaultBuildHasher,
    key::TxKey,
    new_types::{HashCode, ShardCount, ShardIndex},
};
use std::hash::{BuildHasher, Hash};

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
        ShardIndex((hash_code.0 & (shard_count.0 as u64 - 1)) as u8)
    }

    pub(crate) fn hash<K>(&self, key: &K) -> HashCode
    where
        K: Hash,
    {
        HashCode(self.hasher_builder.hash_one(key))
    }
}
