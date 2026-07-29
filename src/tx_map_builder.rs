use crate::{
    custodian::Custodian,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::ShardCount,
    shards::Shards,
    tx_map::TxMap,
};
use std::hash::{Hash, RandomState};

pub struct TxMapBuilder<L = MutexPolicy>
where
    L: LockPolicy,
{
    build_hasher: RandomState,
    capacity: usize,
    #[allow(dead_code)]
    lock_policy: L,
    shards: Shards,
}

impl<L> TxMapBuilder<L>
where
    L: LockPolicy + Default,
{
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn build<K, V>(self) -> TxMap<K, V, L>
    where
        K: Hash + Eq,
    {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count),
        }
    }
    #[must_use]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }
    #[must_use]
    pub fn with_shards(mut self, shards: Shards) -> Self {
        self.shards = shards;
        self
    }
    #[must_use]
    pub fn with_hasher(mut self, build_hasher: RandomState) -> Self {
        self.build_hasher = build_hasher;
        self
    }
}

impl<L> Default for TxMapBuilder<L>
where
    L: LockPolicy + Default,
{
    fn default() -> Self {
        Self {
            build_hasher: RandomState::new(),
            capacity: 0,
            lock_policy: L::default(),
            shards: Shards::_32,
        }
    }
}
