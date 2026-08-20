use crate::{
    custodian::Custodian,
    hasher::DefaultBuildHasher,
    indexer::Indexer,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::ShardCount,
    shards::Shards,
    tx_map::TxMap,
};
use std::{hash::BuildHasher, marker::PhantomData};

/// Builder for configuring and constructing a [`TxMap`].
///
/// Use [`TxMapBuilder::default`] to get a builder with sensible defaults
/// (32 shards, `MutexPolicy`, default hasher), then customise as needed.
pub struct TxMapBuilder<L = MutexPolicy, S = DefaultBuildHasher>
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
    /// Sets the initial capacity hint (total across all shards).
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    #[must_use]
    /// Sets the number of shards.
    pub fn with_shards(mut self, shards: Shards) -> Self {
        self.shards = shards;
        self
    }

    #[must_use]
    /// Replaces the hasher builder.
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
    /// Replaces the lock policy.
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
    /// Consumes the builder and returns a [`TxMap`].
    pub fn build<K, V>(self) -> TxMap<K, V, L, S> {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count, self.capacity),
            indexer: Indexer::new(self.hasher_builder),
        }
    }
}

impl Default for TxMapBuilder<MutexPolicy, DefaultBuildHasher> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            _phantom_l: PhantomData,
            hasher_builder: DefaultBuildHasher::default(),
        }
    }
}
