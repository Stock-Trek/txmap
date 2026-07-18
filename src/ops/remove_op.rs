use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct RemoveOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    keys: Vec<(u8, K)>,
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
        let mut guards_bitmask: u128 = 0;
        let mut indexed_keys = Vec::new();
        for key in keys {
            let key_index = indexer.index(&key);
            guards_bitmask |= 1 << key_index;
            indexed_keys.push((key_index, key));
        }
        Self {
            guards_bitmask,
            keys: indexed_keys,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> OpTrait<K, V> for RemoveOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for (key_index, key) in &self.keys {
            if let Some(guard) = mutex_guards.get_mut(*key_index) {
                guard.remove(key);
            }
        }
    }
}
