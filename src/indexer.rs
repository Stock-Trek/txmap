use crate::{
    key::TxKey,
    new_types::{HashCode, ShardCount, ShardIndex},
};
use rapidhash::fast::RapidHasher;
use std::hash::{Hash, Hasher};

pub struct Indexer;

impl Indexer {
    pub fn indexed_key<K>(shard_count: ShardCount, key: K) -> TxKey<K>
    where
        K: Hash + Eq,
    {
        let hash_code = Self::hash(&key);
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
    pub(crate) fn hash<K>(key: &K) -> HashCode
    where
        K: Hash,
    {
        let mut state = RapidHasher::default();
        key.hash(&mut state);
        HashCode(state.finish())
    }
}
