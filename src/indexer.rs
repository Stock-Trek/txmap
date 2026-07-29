use crate::{
    key::TxKey,
    new_types::{HashCode, ShardCount, ShardIndex},
};
use std::hash::{BuildHasher, Hash};

pub struct Indexer;

impl Indexer {
    pub fn indexed_key<K, S>(shard_count: ShardCount, key: K, hash_builder: &S) -> TxKey<K>
    where
        K: Hash + Eq,
        S: BuildHasher,
    {
        let hash_code = Self::hash(&key, hash_builder);
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
    pub(crate) fn hash<K, S>(key: &K, hash_builder: &S) -> HashCode
    where
        K: Hash,
        S: BuildHasher,
    {
        HashCode(hash_builder.hash_one(key))
    }
}
