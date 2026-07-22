use crate::{
    indexed_key::IndexedKey, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct UpdateOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>, &P) -> Option<V>>,
}

impl<K, V, P> UpdateOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<T>(shard_count: u8, key: K, transform: T) -> Self
    where
        T: Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            transform: Box::new(transform),
        }
    }
}

impl<K, V> UpdateOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<T>(shard_count: u8, key: K, transform: T) -> Self
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
    {
        Self::new_with_params(shard_count, key, move |k, v, _| transform(k, v))
    }
}

impl<K, V, P> OpTrait<K, V, P> for UpdateOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P) {
        let value_ref = self.indexed_key.value_ref(mutex_guards);
        let new_value = (self.transform)(&self.indexed_key.3, value_ref, params);
        match new_value {
            Some(v) => self.indexed_key.insert(mutex_guards, v),
            None => {
                self.indexed_key.remove_entry(mutex_guards);
            }
        };
    }
}
