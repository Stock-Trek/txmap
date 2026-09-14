use crate::{
    custodian::Custodian, hasher::DefaultBuildHasher, indexer::Indexer, new_types::ShardCount,
    shards::Shards, tx_map::TxMap,
};
use std::hash::BuildHasher;

/// Builder for configuring and constructing a [`TxMap`].
///
/// Use [`TxMapBuilder::default`] to get a builder with sensible defaults
/// (32 shards, default hasher), then customise as needed.
pub struct TxMapBuilder<S = DefaultBuildHasher>
where
    S: BuildHasher,
{
    shards: Shards,
    capacity: usize,
    hasher_builder: S,
}

impl<S> TxMapBuilder<S>
where
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
    pub fn with_hasher<BH>(self, hasher_builder: BH) -> TxMapBuilder<BH>
    where
        BH: BuildHasher,
    {
        let Self {
            capacity, shards, ..
        } = self;
        TxMapBuilder::<BH> {
            capacity,
            shards,
            hasher_builder,
        }
    }

    #[must_use]
    /// Consumes the builder and returns a [`TxMap`].
    pub fn build<K, V>(self) -> TxMap<K, V, S> {
        let shard_count: ShardCount = self.shards.into();
        TxMap {
            shard_count,
            custodian: Custodian::new(shard_count, self.capacity),
            indexer: Indexer::new(self.hasher_builder),
        }
    }
}

impl Default for TxMapBuilder<DefaultBuildHasher> {
    fn default() -> Self {
        Self {
            capacity: 0,
            shards: Shards::_32,
            hasher_builder: DefaultBuildHasher::default(),
        }
    }
}
