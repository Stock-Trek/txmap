use crate::{
    indexed_keys::IndexedKeys, new_types::BitMask, result::INCORRECT_GUARD_VALUES_LENGTH,
    shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct Guard<K, V, P = ()>
where
    K: Hash + Eq,
{
    pub guards_bitmask: BitMask,
    pub name: String,
    pub indexed_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    pub is_condition_met: Box<dyn Fn(&[Option<&V>], &P) -> bool>,
}

impl<K, V> Guard<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<const N: usize, C>(shard_count: u8, name: String, keys: [K; N], condition: C) -> Self
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
    {
        Self::new_with_params(shard_count, name, keys, move |k, _| condition(k))
    }
}

impl<K, V, P> Guard<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<const N: usize, C>(
        shard_count: u8,
        name: String,
        keys: [K; N],
        condition: C,
    ) -> Self
    where
        C: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let indexed_keys = ShardCount::indexes(shard_count, keys, |k| k);
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
        mutex_guards: &IntMap<u8, MutexGuard<HashTable<(K, V)>>>,
        params: &P,
    ) -> bool {
        let mut values = Vec::with_capacity(self.indexed_keys.indexed.len());
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref(mutex_guards);
            values.push(value_ref);
        }
        (self.is_condition_met)(&values, params)
    }
}
