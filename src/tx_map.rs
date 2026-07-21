use crate::{
    builders::stem_builder::TxStemBuilder, custodian::Custodian, indexer::Indexer,
    shard_count::ShardCount,
};
use std::hash::{DefaultHasher, Hash};

pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    #[must_use]
    pub fn new(shard_count: ShardCount) -> Self {
        let indexer = Indexer {
            shard_count: u8::from(shard_count),
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        Self {
            indexer,
            custodian: Custodian::new(shard_count),
        }
    }
    pub fn clear(&self) {
        let all_guards = self.custodian.all_guards();
        for mut mutex_guard in all_guards {
            mutex_guard.1.clear();
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        let all_guards = self.custodian.all_guards();
        for mutex_guard in all_guards {
            total_length += mutex_guard.1.len();
        }
        total_length
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_index = self.indexer.index(&key);
        let mut mutex_guard = self.custodian.guard_at(shard_index);
        mutex_guard.insert(key, value)
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        let shard_index = self.indexer.index(key);
        let mut mutex_guard = self.custodian.guard_at(shard_index);
        mutex_guard.remove(key)
    }
    #[must_use]
    pub fn get_with<T, R>(&self, key: &K, transform: T) -> Option<R>
    where
        T: FnOnce(&V) -> R,
    {
        let shard_index = self.indexer.index(key);
        let mutex_guard = self.custodian.guard_at(shard_index);
        mutex_guard.get(key).map(transform)
    }
    #[must_use]
    pub fn fold<T, R, C, A>(&self, initial: R, convert: C, accumulate: A) -> R
    where
        C: Fn(&K, &V) -> Option<T>,
        A: Fn(R, T) -> R,
    {
        self.custodian
            .all_guards()
            .iter()
            .flat_map(|guard| guard.1.iter())
            .filter_map(|(key, value)| convert(key, value))
            .fold(initial, accumulate)
    }
    #[must_use]
    pub fn transaction<'txmap>(&'txmap self) -> TxStemBuilder<'txmap, K, V> {
        TxStemBuilder {
            indexer: self.indexer,
            custodian: &self.custodian,
        }
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    #[must_use]
    pub fn get_copied(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| *v)
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    #[must_use]
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| v.clone())
    }
}
