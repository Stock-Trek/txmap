use crate::{
    indexed_keys::IndexedKeys, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RemoveWhereOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RemoveWhereOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<I, C>(shard_count: u8, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let indexed_keys = ShardCount::indexes(shard_count, keys, |k| k);
        Self {
            indexed_keys,
            condition: Box::new(condition),
        }
    }
}

impl<K, V> RemoveWhereOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<I, C>(shard_count: u8, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_params(shard_count, keys, move |k, v, _| condition(k, v))
    }
}

impl<K, V, P> OpTrait<K, V, P> for RemoveWhereOp<K, V, P>
where
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P) {
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref(mutex_guards);
            if let Some(v) = value_ref
                && (self.condition)(&indexed_key.3, v, params)
            {
                indexed_key.remove_entry(mutex_guards);
            }
        }
    }
}
