use crate::{
    indexer::Indexer, lock_policies::lock_policy::LockPolicy, new_types::BitMask, params::TxKey,
    result::MISSING_LOCK_GUARD_ERROR, shard::Shard,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct LockGuards<'ex, K, V, L>
where
    K: 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub read: IntMap<u8, L::ReadGuard<'ex, Shard<K, V>>>,
    pub write: IntMap<u8, L::WriteGuard<'ex, Shard<K, V>>>,
    pub write_bitmask: BitMask,
}

impl<'ex, K, V, L> LockGuards<'ex, K, V, L>
where
    K: 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub fn insert(&mut self, key: &TxKey<K>, value: V)
    where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| Indexer::hash(&entry.0).0,
            )
            .insert((key.key.clone(), value));
    }
    pub fn insert_if_absent(&mut self, key: &TxKey<K>, value_gen: impl FnOnce() -> V)
    where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| Indexer::hash(&entry.0).0,
            )
            .or_insert_with(|| (key.key.clone(), value_gen()));
    }
    pub fn insert_with_duplicate_key(&mut self, key: &TxKey<K>, duplicate_key: K, value: V)
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| Indexer::hash(&entry.0).0,
            )
            .insert((duplicate_key, value));
    }
    pub fn remove_entry(&mut self, key: &TxKey<K>) -> Option<(K, V)>
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .find_entry(key.hash_code.0, |entry| entry.0 == key.key)
            .ok()
            .map(|entry| entry.remove().0)
    }
}
