use crate::{
    indexed_keys::IndexedKeys, locks::lock_policy::LockPolicy, new_types::BitMask,
    result::INCORRECT_GUARD_VALUES_LENGTH, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct Guard<K, V, P = ()>
where
    K: Hash + Eq,
{
    pub guards_bitmask: BitMask,
    pub name: String,
    indexed_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&[Option<&V>], &P) -> bool>,
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
        let condition = Box::new(move |values: &[Option<&V>], params: &P| {
            let array: [Option<&V>; N] = values.try_into().expect(INCORRECT_GUARD_VALUES_LENGTH);
            (condition)(array, params)
        });
        Self {
            guards_bitmask: indexed_keys.bitmask,
            name,
            indexed_keys,
            condition,
        }
    }
    pub fn is_condition_met<L>(
        &self,
        mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
        params: &P,
    ) -> bool
    where
        L: LockPolicy,
    {
        let mut values = Vec::with_capacity(self.indexed_keys.indexed.len());
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref::<L, V>(mutex_guards);
            values.push(value_ref);
        }
        (self.condition)(&values, params)
    }
}
