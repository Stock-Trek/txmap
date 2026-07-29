use crate::{
    custodian::Custodian,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::ShardCount,
    shards::Shards,
    tx_map::TxMap,
};
use std::{hash::Hash, marker::PhantomData};

pub struct TxMapBuilder<L>
where
    L: LockPolicy,
{
    shards: Shards,
    capacity: usize,
    _phantom_l: PhantomData<L>,
}

impl<L> TxMapBuilder<L>
where
    L: LockPolicy,
{
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
    pub fn with_lock_policy<LP>(self) -> TxMapBuilder<LP>
    where
        LP: LockPolicy,
    {
        let Self {
            capacity, shards, ..
        } = self;
        TxMapBuilder::<LP> {
            capacity,
            shards,
            _phantom_l: PhantomData,
        }
    }
    #[must_use]
    pub fn build<K, V>(self) -> TxMap<K, V, L>
    where
        K: Clone + Hash + Eq,
    {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count),
        }
    }
}

impl Default for TxMapBuilder<MutexPolicy> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            _phantom_l: PhantomData,
        }
    }
}
