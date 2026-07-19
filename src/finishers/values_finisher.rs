use crate::{
    finishers::finisher_trait::FinisherTrait,
    indexer::{IndexedData, Indexer},
    result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct ValuesFinisher<K, V, R> {
    indexed_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>) -> Option<R>>,
}

impl<K, V, R> ValuesFinisher<K, V, R>
where
    K: Hash,
{
    pub fn new<I, T>(indexer: Indexer, keys: I, transform: T) -> Self
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static,
    {
        Self {
            indexed_keys: indexer.indexes(keys, |k| k),
            transform: Box::new(move |key, value_opt| value_opt.map(|value| transform(key, value))),
        }
    }
}

impl<K, V, R> FinisherTrait<K, V> for ValuesFinisher<K, V, R>
where
    K: Hash + Eq,
{
    type Output = Vec<Option<R>>;

    fn to_result(
        &self,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Vec<Option<R>> {
        let mut result = Vec::with_capacity(self.indexed_keys.indexed.len());
        for (key_index, key) in &self.indexed_keys.indexed {
            let mutex_guard = mutex_guards
                .get(*key_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            let value = mutex_guard.get(key);
            let result_value = (self.transform)(key, value);
            result.push(result_value);
        }
        result
    }
}
