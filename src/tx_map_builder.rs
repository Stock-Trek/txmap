use crate::{
    custodian::Custodian,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::ShardCount,
    shards::Shards,
    tx_map::TxMap,
};
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

pub struct TxMapBuilder<L, S = RandomState>
where
    L: LockPolicy,
    S: BuildHasher,
{
    shards: Shards,
    capacity: usize,
    hash_builder: S,
    _phantom_l: PhantomData<L>,
}

impl<L, S> TxMapBuilder<L, S>
where
    L: LockPolicy,
    S: BuildHasher,
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
    pub fn with_hash_builder<NS>(self, hash_builder: NS) -> TxMapBuilder<L, NS>
    where
        NS: BuildHasher,
    {
        TxMapBuilder::<L, NS> {
            shards: self.shards,
            capacity: self.capacity,
            hash_builder,
            _phantom_l: PhantomData,
        }
    }
    #[must_use]
    pub fn with_lock_policy<LP>(self) -> TxMapBuilder<LP, S>
    where
        LP: LockPolicy,
    {
        TxMapBuilder::<LP, S> {
            shards: self.shards,
            capacity: self.capacity,
            hash_builder: self.hash_builder,
            _phantom_l: PhantomData,
        }
    }
    #[must_use]
    pub fn build<K, V>(self) -> TxMap<K, V, L, S>
    where
        K: Hash + Eq,
    {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count, self.hash_builder),
        }
    }
}

impl Default for TxMapBuilder<MutexPolicy, RandomState> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            hash_builder: RandomState::new(),
            _phantom_l: PhantomData,
        }
    }
}
