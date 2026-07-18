use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct MapPeekOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    key_index: u8,
    key: K,
    indexed_peek_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>, &[Option<&V>]) -> Option<V>>,
}

impl<K, V> MapPeekOp<K, V>
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
        let key_index = indexer.index(&key);
        let indexed_peek_keys = indexer.indexes(peek_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_peek_keys.bitmask,
            key_index,
            key,
            indexed_peek_keys,
            transform: Box::new(move |key, value, peek_values| {
                let peek_array: [Option<&V>; N] = peek_values
                    .try_into()
                    .expect("Incorrect operation values length");
                (transform)(key, value, peek_array)
            }),
        }
    }
    fn mapped_value(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Option<V> {
        let mut peek_values = Vec::with_capacity(self.indexed_peek_keys.indexed.len());
        for (shard_index, peek_key) in &self.indexed_peek_keys.indexed {
            let peek_guard = mutex_guards.get(*shard_index);
            let peek_shard = peek_guard.expect("Missing shard lock");
            let peek_value = peek_shard.get(peek_key);
            peek_values.push(peek_value);
        }
        let key_guard = mutex_guards.get(self.key_index);
        let key_shard = key_guard.expect("Missing shard lock");
        let key_value = key_shard.get(&self.key);
        (self.transform)(&self.key, key_value, peek_values.as_slice())
    }
}

impl<K, V> OpTrait<K, V> for MapPeekOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let new_value = self.mapped_value(&mutex_guards);
        let guard = mutex_guards.get_mut(self.key_index);
        let shard = guard.expect("Missing shard lock");
        match new_value {
            Some(v) => shard.insert(self.key.clone(), v),
            None => shard.remove(&self.key),
        };
    }
}
