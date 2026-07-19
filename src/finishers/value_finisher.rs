use crate::{
    finishers::finisher_trait::FinisherTrait, indexer::Indexer, result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ValueFinisher<K, V, R>
where
    K: Clone + Hash + Eq,
{
    key_index: u8,
    key: K,
    transform: Box<dyn Fn(&K, Option<&V>) -> Option<R>>,
}

impl<K, V, R> ValueFinisher<K, V, R>
where
    K: Clone + Hash + Eq,
{
    pub fn new<T>(indexer: Indexer, key: K, transform: T) -> Self
    where
        T: Fn(&K, &V) -> R + 'static,
    {
        Self {
            key_index: indexer.index(&key),
            key,
            transform: Box::new(move |key, value_opt| value_opt.map(|value| transform(key, value))),
        }
    }
}

impl<K, V, R> FinisherTrait<K, V> for ValueFinisher<K, V, R>
where
    K: Clone + Hash + Eq,
{
    type Output = Option<R>;

    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Option<R> {
        let mutex_guard = mutex_guards
            .get(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        let value = mutex_guard.get(&self.key);
        (self.transform)(&self.key, value)
    }
}
