use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
    result::{INCORRECT_PEEK_VALUES_LENGTH, MISSING_MUTEX_GUARD_ERROR},
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyPeekOrDefaultOp<K, V, P = ()> {
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    indexed_peek_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V, &[Option<&V>], &P)>,
}

impl<K, V, P> ModifyPeekOrDefaultOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_params<const N: usize, M>(
        indexer: &Indexer,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> Self
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_peek_keys = indexer.indexes(peek_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_peek_keys.bitmask,
            key_index,
            key,
            indexed_peek_keys,
            mutate: Box::new(move |key, value, peek_values, params| {
                let peek_array: [Option<&V>; N] =
                    peek_values.try_into().expect(INCORRECT_PEEK_VALUES_LENGTH);
                (mutate)(key, value, peek_array, params)
            }),
        }
    }
    fn remove_value(
        &self,
        mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Option<V> {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard.remove(&self.key)
    }
    fn insert_value(
        &self,
        value: V,
        mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Option<V> {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard.insert(self.key.clone(), value)
    }
}

impl<K, V> ModifyPeekOrDefaultOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new<const N: usize, M>(indexer: &Indexer, key: K, peek_keys: [K; N], mutate: M) -> Self
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    {
        Self::new_with_params(indexer, key, peek_keys, move |k, v, pks, _| {
            mutate(k, v, pks)
        })
    }
}

impl<K, V, P> OpTrait<K, V, P> for ModifyPeekOrDefaultOp<K, V, P>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        let old_value = self.remove_value(mutex_guards);
        let mut peek_values = Vec::with_capacity(self.indexed_peek_keys.indexed.len());
        for (shard_index, peek_key) in &self.indexed_peek_keys.indexed {
            let peek_guard = mutex_guards.get(*shard_index);
            let peek_shard = peek_guard.expect(MISSING_MUTEX_GUARD_ERROR);
            let peek_value = peek_shard.get(peek_key);
            peek_values.push(peek_value);
        }
        let mut value = if let Some(v) = old_value {
            v
        } else {
            V::default()
        };
        (self.mutate)(&self.key, &mut value, peek_values.as_slice(), params);
        self.insert_value(value, mutex_guards);
    }
}
