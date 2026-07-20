use crate::{
    finishers::finisher_trait::FinisherTrait,
    indexer::{IndexedData, Indexer},
    result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub struct CopyAllFinisher<K, V> {
    indexed_keys: IndexedData<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> CopyAllFinisher<K, V>
where
    K: Hash,
    V: Copy,
{
    pub fn new<I>(indexer: Indexer, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        Self {
            indexed_keys: indexer.indexes(keys, |k| k),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> FinisherTrait<K, V> for CopyAllFinisher<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    type Output = Vec<Option<V>>;

    fn guards_bitmask(&self) -> u128 {
        self.indexed_keys.bitmask
    }
    fn to_result(
        &self,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Vec<Option<V>> {
        let mut result = Vec::with_capacity(self.indexed_keys.indexed.len());
        for (key_index, key) in &self.indexed_keys.indexed {
            let mutex_guard = mutex_guards
                .get(*key_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            let value = mutex_guard.get(key).copied();
            result.push(value);
        }
        result
    }
}
