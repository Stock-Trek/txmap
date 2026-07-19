use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
    result::{INCORRECT_PEEK_VALUES_LENGTH, MISSING_MUTEX_GUARD_ERROR},
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct MapPeekOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    indexed_peek_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>, &[Option<&V>], &P) -> Option<V>>,
}

impl<K, V, P> MapPeekOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_param<const N: usize, T>(
        indexer: &Indexer,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> Self
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_peek_keys = indexer.indexes(peek_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_peek_keys.bitmask,
            key_index,
            key,
            indexed_peek_keys,
            transform: Box::new(move |key, value, peek_values, params| {
                let peek_array: [Option<&V>; N] =
                    peek_values.try_into().expect(INCORRECT_PEEK_VALUES_LENGTH);
                (transform)(key, value, peek_array, params)
            }),
        }
    }
    fn mapped_value(
        &self,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
        params: &P,
    ) -> Option<V> {
        let mut peek_values = Vec::with_capacity(self.indexed_peek_keys.indexed.len());
        for (shard_index, peek_key) in &self.indexed_peek_keys.indexed {
            let peek_guard = mutex_guards.get(*shard_index);
            let peek_shard = peek_guard.expect(MISSING_MUTEX_GUARD_ERROR);
            let peek_value = peek_shard.get(peek_key);
            peek_values.push(peek_value);
        }
        let key_guard = mutex_guards.get(self.key_index);
        let key_shard = key_guard.expect(MISSING_MUTEX_GUARD_ERROR);
        let key_value = key_shard.get(&self.key);
        (self.transform)(&self.key, key_value, peek_values.as_slice(), params)
    }
}

impl<K, V> MapPeekOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new<const N: usize, T>(
        indexer: &Indexer,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> Self
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        Self::new_with_param(indexer, key, peek_keys, move |k, v, pks, _| {
            transform(k, v, pks)
        })
    }
}

impl<K, V, P> OpTrait<K, V, P> for MapPeekOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        let new_value = self.mapped_value(mutex_guards, params);
        let guard = mutex_guards.get_mut(self.key_index);
        let shard = guard.expect(MISSING_MUTEX_GUARD_ERROR);
        match new_value {
            Some(v) => shard.insert(self.key.clone(), v),
            None => shard.remove(&self.key),
        };
    }
}
