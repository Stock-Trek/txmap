use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct ClearOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    shard_count: u8,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> ClearOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new(indexer: &Indexer) -> Self {
        let shard_count = indexer.shard_count as u8;
        let guards_bitmask = indexer.all_bitmask();
        Self {
            guards_bitmask,
            shard_count,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> OpTrait<K, V> for ClearOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for i in 0..self.shard_count {
            if let Some(guard) = mutex_guards.get_mut(i) {
                guard.clear();
            }
        }
    }
}
