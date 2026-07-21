use crate::{
    indexed_key::IndexedKey, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct InsertWithOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    #[allow(clippy::type_complexity)]
    value_generator: Box<dyn Fn(&K, &P) -> V>,
}

impl<K, V, P> InsertWithOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<G>(shard_count: u8, key: K, value_generator: G) -> Self
    where
        G: Fn(&K, &P) -> V + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            value_generator: Box::new(value_generator),
        }
    }
}

impl<K, V> InsertWithOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<G>(shard_count: u8, key: K, value_generator: G) -> Self
    where
        G: Fn(&K) -> V + 'static,
    {
        Self::new_with_params(shard_count, key, move |k, _| value_generator(k))
    }
}

impl<K, V, P> OpTrait<K, V, P> for InsertWithOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P) {
        let new_value = (self.value_generator)(&self.indexed_key.3, params);
        self.indexed_key.insert(mutex_guards, new_value);
    }
}
