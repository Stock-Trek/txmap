use crate::{
    key::TxKey, lock_policies::lock_policy::LockPolicy, new_types::BitMask, new_types::MAX_SHARDS,
    result::MISSING_LOCK_GUARD_ERROR, shard::Shard,
};
use std::hash::Hash;

pub(crate) struct LockGuards<'ex, K, V, L>
where
    K: 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub read: [Option<L::ReadGuard<'ex, Shard<K, V>>>; MAX_SHARDS],
    pub write: [Option<L::WriteGuard<'ex, Shard<K, V>>>; MAX_SHARDS],
    pub write_bitmask: BitMask,
}

impl<'ex, K, V, L> LockGuards<'ex, K, V, L>
where
    K: Clone + Hash + Eq + 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub fn read_guard(&self, key: &TxKey<K>) -> &L::ReadGuard<'ex, Shard<K, V>> {
        self.read[key.shard_index.0 as usize]
            .as_ref()
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
    pub fn write_guard(&mut self, key: &TxKey<K>) -> &mut L::WriteGuard<'ex, Shard<K, V>> {
        self.write[key.shard_index.0 as usize]
            .as_mut()
            .expect(MISSING_LOCK_GUARD_ERROR)
    }
}
