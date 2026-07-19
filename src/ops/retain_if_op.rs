use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    keys: Vec<(u8, K)>,
    condition: Box<dyn Fn(&K, &V) -> bool>,
}

impl<K, V> RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
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
            condition: Box::new(condition),
        }
    }
}

impl<K, V> OpTrait<K, V> for RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for (key_index, key) in &self.keys {
            if let Some(guard) = mutex_guards.get_mut(*key_index) {
                if let Some(value) = guard.get(key) {
                    if !(self.condition)(key, value) {
                        guard.remove(key);
                    }
                }
            }
        }
    }
}
