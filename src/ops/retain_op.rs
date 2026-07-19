use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct RetainOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    keys: Vec<(u8, K)>,
    _phantom: PhantomData<V>,
}

impl<K, V> RetainOp<K, V>
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

impl<K, V> OpTrait<K, V> for RetainOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        // Collect all shard indices this operation touches
        let mut shard_indices: Vec<u8> = self.keys.iter().map(|(idx, _)| *idx).collect();
        shard_indices.sort();
        shard_indices.dedup();

        for shard_index in &shard_indices {
            if let Some(guard) = mutex_guards.get_mut(*shard_index) {
                let key_refs: Vec<&K> = self
                    .keys
                    .iter()
                    .filter(|(idx, _)| idx == shard_index)
                    .map(|(_, k)| k)
                    .collect();
                guard.retain(|k, _| key_refs.contains(&k));
            }
        }
    }
}
