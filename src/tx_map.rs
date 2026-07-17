use crate::{
    custodian::Custodian, fat_transaction::FatTransactionBuilder, indexer::Indexer,
    shard_count::ShardCount, skeleton_transaction::SkeletonTransactionBuilder,
};
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::hash::{DefaultHasher, Hash};

#[allow(dead_code)]
pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    shard_count: usize,
    shards: Vec<Shard<K, V>>,
    owned_key: fn(&K) -> K,
    custodian: Custodian<K, V>,
}

type Shard<K, V> = Mutex<HashMap<K, V>>;

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
        let shard_count_u8 = u8::from(shard_count);
        let indexer = Indexer {
            shard_count: shard_count_u8 as u64,
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        let mut shards = Vec::with_capacity(shard_count_u8 as usize);
        for _ in 0..shard_count_u8 {
            shards.push(Mutex::new(HashMap::new()));
        }
        Self {
            indexer,
            shard_count: shard_count_u8 as usize,
            shards,
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
