use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RetainAnyIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    shard_count: u8,
    condition: Box<dyn Fn(&K, &V) -> bool>,
}

impl<K, V> RetainAnyIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<C>(indexer: &Indexer, condition: C) -> Self
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let shard_count = indexer.shard_count as u8;
        let guards_bitmask = if shard_count == 128 {
            !0u128
        } else {
            (1 << shard_count) - 1
        };
        Self {
            guards_bitmask,
            shard_count,
            condition: Box::new(condition),
        }
    }
}

impl<K, V> OpTrait<K, V> for RetainAnyIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for i in 0..self.shard_count {
            if let Some(guard) = mutex_guards.get_mut(i) {
                guard.retain(|k, v| (self.condition)(k, v));
            }
        }
    }
}
