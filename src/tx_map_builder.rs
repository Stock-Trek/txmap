use crate::{
    custodian::Custodian,
    indexer::Indexer,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::ShardCount,
    shards::Shards,
    tx_map::TxMap,
};
use std::{
    hash::{BuildHasher, Hash, RandomState},
    marker::PhantomData,
};

pub struct TxMapBuilder<L = MutexPolicy, S = RandomState>
where
    L: LockPolicy,
    S: BuildHasher,
{
    shards: Shards,
    capacity: usize,
    hasher_builder: S,
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
    pub fn with_hasher<BH>(self, hasher_builder: BH) -> TxMapBuilder<L, BH>
    where
        BH: BuildHasher,
    {
        let Self {
            capacity,
            shards,
            _phantom_l,
            ..
        } = self;
        TxMapBuilder::<L, BH> {
            capacity,
            shards,
            hasher_builder,
            _phantom_l,
        }
    }

    #[must_use]
    pub fn with_lock_policy<LP>(self) -> TxMapBuilder<LP, S>
    where
        LP: LockPolicy,
    {
        let Self {
            capacity,
            shards,
            hasher_builder,
            ..
        } = self;
        TxMapBuilder::<LP, S> {
            capacity,
            shards,
            hasher_builder,
            _phantom_l: PhantomData,
        }
    }

    #[must_use]
    pub fn build<K, V>(self) -> TxMap<K, V, L, S>
    where
        K: Clone + Hash + Eq,
    {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count),
            indexer: Indexer::new(self.hasher_builder),
        }
    }
}

impl<L> Default for TxMapBuilder<L, RandomState>
where
    L: LockPolicy,
{
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            _phantom_l: PhantomData,
            hasher_builder: RandomState::default(),
        }
    }
}
