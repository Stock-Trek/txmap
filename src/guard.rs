use crate::{
    indexer::{IndexedData, Indexer},
    result::{INCORRECT_GUARD_VALUES_LENGTH, MISSING_MUTEX_GUARD_ERROR},
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct Guard<K, V, P = ()> {
    pub guards_bitmask: u128,
    pub name: String,
    pub indexed_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub is_condition_met: Box<dyn Fn(&[Option<&V>], &P) -> bool>,
}

impl<K, V> Guard<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<const N: usize, C>(
        indexer: Indexer,
        name: String,
        keys: [K; N],
        condition: C,
    ) -> Self
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
    {
        Self::new_with_params(indexer, name, keys, move |k, _| condition(k))
    }
}

impl<K, V, P> Guard<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<const N: usize, C>(
        indexer: Indexer,
        name: String,
        keys: [K; N],
        condition: C,
    ) -> Self
    where
        C: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let indexed_keys = indexer.indexes(keys, |k| k);
        let is_condition_met = Box::new(move |values: &[Option<&V>], params: &P| {
            let array: [Option<&V>; N] = values.try_into().expect(INCORRECT_GUARD_VALUES_LENGTH);
            (condition)(array, params)
        });
        Self {
            guards_bitmask: indexed_keys.bitmask,
            name,
            indexed_keys,
            is_condition_met,
        }
    }
    pub fn is_condition_met(
        &self,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
        params: &P,
    ) -> bool {
        let mut values = Vec::with_capacity(self.indexed_keys.indexed.len());
        for (shard_index, key) in &self.indexed_keys.indexed {
            let mutex_guard = mutex_guards.get(*shard_index);
            let shard = mutex_guard.expect(MISSING_MUTEX_GUARD_ERROR);
            let value = shard.get(key);
            values.push(value);
            if !(self.is_condition_met)(&values, params) {
                return false;
            }
        }
        true
    }
}
