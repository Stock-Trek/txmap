use crate::{
    new_types::{HashCode, ShardCount, ShardIndex},
    params::TxKey,
};
use rapidhash::fast::RapidHasher;
use std::hash::{Hash, Hasher};

pub struct Indexer;

impl Indexer {
    pub(crate) fn all_indexed_keys<E, K>(
        shard_count: ShardCount,
        keys: impl IntoIterator<Item = E>,
        element_to_key: fn(E) -> K,
    ) -> Vec<TxKey<K>>
    where
        K: Hash + Eq,
    {
        let iter = keys.into_iter();
        let mut indexed = Vec::with_capacity(iter.size_hint().0);
        for element in iter {
            let key = element_to_key(element);
            let indexed_key = Self::indexed_key(shard_count, key);
            indexed.push(indexed_key);
        }
        indexed
    }
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
