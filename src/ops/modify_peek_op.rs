use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyPeekOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    key_index: u8,
    key: K,
    indexed_peek_keys: IndexedData<K>,
    mutate: Box<dyn Fn(&K, &mut V, &[Option<&V>])>,
}

impl<K, V> ModifyPeekOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<const N: usize, M>(indexer: &Indexer, key: K, peek_keys: [K; N], mutate: M) -> Self
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_peek_keys = indexer.indexes(peek_keys, |k| k);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            indexed_peek_keys,
            mutate: Box::new(move |key, value, peek_values| {
                let peek_array: [Option<&V>; N] = peek_values
                    .try_into()
                    .expect("Incorrect operation values length");
                (mutate)(key, value, peek_array)
            }),
        }
    }
    fn remove_value(
        &self,
        mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Option<V> {
        let mutex_guard = mutex_guards.get_mut(self.key_index).expect("No Guard");
        mutex_guard.remove(&self.key)
    }
    fn insert_value(
        &self,
        value: V,
        mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Option<V> {
        let mutex_guard = mutex_guards.get_mut(self.key_index).expect("No Guard");
        mutex_guard.insert(self.key.clone(), value)
    }
}

impl<K, V> OpTrait<K, V> for ModifyPeekOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        if let Some(mut value) = self.remove_value(mutex_guards) {
            let mut peek_values = Vec::with_capacity(self.indexed_peek_keys.indexed.len());
            for (shard_index, peek_key) in &self.indexed_peek_keys.indexed {
                let peek_guard = mutex_guards.get(*shard_index);
                let peek_shard = peek_guard.expect("Missing shard lock");
                let peek_value = peek_shard.get(peek_key);
                peek_values.push(peek_value);
            }
            (self.mutate)(&self.key, &mut value, peek_values.as_slice());
            self.insert_value(value, mutex_guards);
        }
    }
}
