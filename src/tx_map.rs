use crate::{
    custodian::Custodian, fat_transaction::FatTransactionBuilder, indexer::Indexer,
    shard_count::ShardCount, skeleton_transaction::SkeletonTransactionBuilder,
};
use std::hash::{DefaultHasher, Hash};

pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    owned_key: fn(&K) -> K,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn with_cloneable_key(shard_count: ShardCount) -> Self {
        Self::new(shard_count, |k| k.clone())
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: ShardCount, owned_key: fn(&K) -> K) -> Self {
        let indexer = Indexer {
            shard_count: u8::from(shard_count) as u64,
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        Self {
            indexer,
            owned_key,
            custodian: Custodian::new(shard_count),
        }
    }
    pub fn transaction<'txmap>(&'txmap self) -> FatTransactionBuilder<'txmap, K, V> {
        FatTransactionBuilder::new(self.indexer, self.owned_key, &self.custodian)
    }
    pub fn parameterized_transaction<'txmap, P>(
        &'txmap self,
    ) -> SkeletonTransactionBuilder<'txmap, K, V, P> {
        SkeletonTransactionBuilder::new(self.indexer, self.owned_key, &self.custodian)
    }
}
