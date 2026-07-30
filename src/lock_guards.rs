use crate::{
    key::TxKey, lock_policies::lock_policy::LockPolicy, new_types::BitMask,
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
    K: Clone + Hash + Eq + 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub fn read_guard(&self, key: &TxKey<K>) -> &L::ReadGuard<'ex, Shard<K, V>> {
        self.read
            .get(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
    pub fn write_guard(&mut self, key: &TxKey<K>) -> &mut L::WriteGuard<'ex, Shard<K, V>> {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
}
