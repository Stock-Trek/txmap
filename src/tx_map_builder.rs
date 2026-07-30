use crate::{
    custodian::Custodian,
    indexer::Indexer,
    lock_policies::{
        lock_policy::LockPolicy, mutex_policy::MutexPolicy, rwlock_policy::RwLockPolicy,
    },
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
    S: BuildHasher + Clone,
{
    shards: Shards,
    capacity: usize,
    _phantom_l: PhantomData<L>,
    hasher_builder: S,
}

impl<L, S> TxMapBuilder<L, S>
where
    L: LockPolicy,
    S: BuildHasher + Clone,
{
    pub(crate) fn new(shards: Shards, hasher_builder: S) -> Self {
        Self {
            capacity: 0,
            shards,
            _phantom_l: PhantomData,
            hasher_builder,
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
    pub fn with_hasher<H>(self, hasher_builder: H) -> TxMapBuilder<L, H>
    where
        H: BuildHasher + Clone,
    {
        let Self {
            capacity, shards, ..
        } = self;
        TxMapBuilder::<L, H> {
            capacity,
            shards,
            _phantom_l: PhantomData,
            hasher_builder,
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
            _phantom_l: PhantomData,
            hasher_builder,
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

impl Default for TxMapBuilder<MutexPolicy, RandomState> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            _phantom_l: PhantomData,
            hasher_builder: RandomState::default(),
        }
    }
}

impl Default for TxMapBuilder<RwLockPolicy, RandomState> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            _phantom_l: PhantomData,
            hasher_builder: RandomState::default(),
        }
    }
}
