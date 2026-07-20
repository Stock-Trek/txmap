use crate::{
    finishers::finisher_trait::FinisherTrait, indexer::Indexer, result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub struct CopyFinisher<K, V> {
    key_index: u8,
    key: K,
    _phantom: PhantomData<V>,
}

impl<K, V> CopyFinisher<K, V>
where
    K: Hash,
    V: Copy,
{
    pub fn new(indexer: Indexer, key: K) -> Self {
        Self {
            key_index: indexer.index(&key),
            key,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> FinisherTrait<K, V> for CopyFinisher<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    type Output = Option<V>;

    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Option<V> {
        let mutex_guard = mutex_guards
            .get(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        let value = mutex_guard.get(&self.key);
        value.copied()
    }
}
