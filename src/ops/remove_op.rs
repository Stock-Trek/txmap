use crate::{
    indexer::{IndexedData, Indexer},
    ops::op_trait::OpTrait,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct RemoveOp<K, V>
where
    K: Clone + Hash + Eq,
{
    indexed_keys: IndexedData<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> RemoveOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<I>(indexer: &Indexer, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        let indexed_keys = indexer.indexes(keys, |k| k);
        Self {
            indexed_keys,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> OpTrait<K, V> for RemoveOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.indexed_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for (key_index, key) in &self.indexed_keys.indexed {
            if let Some(guard) = mutex_guards.get_mut(*key_index) {
                guard.remove(key);
            }
        }
    }
}
