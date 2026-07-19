use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RemoveWhereOp<K, V, P = ()> {
    indexed_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RemoveWhereOp<K, V, P>
where
    K: Hash,
{
    pub fn new_with_params<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let indexed_keys = indexer.indexes(keys, |k| k);
        Self {
            indexed_keys,
            condition: Box::new(condition),
        }
    }
}

impl<K, V> RemoveWhereOp<K, V, ()>
where
    K: Hash,
{
    pub fn new<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_params(indexer, keys, move |k, v, _| condition(k, v))
    }
}

impl<K, V, P> OpTrait<K, V, P> for RemoveWhereOp<K, V, P>
where
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.indexed_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        for (key_index, key) in &self.indexed_keys.indexed {
            if let Some(guard) = mutex_guards.get_mut(*key_index)
                && let Some(value) = guard.get(key)
                && (self.condition)(key, value, params)
            {
                guard.remove(key);
            }
        }
    }
}
