use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct MoveValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    from_index: u8,
    to_index: u8,
    from: K,
    to: K,
    _phantom: PhantomData<V>,
}

impl<K, V> MoveValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new(indexer: &Indexer, from: K, to: K) -> Self {
        let from_index = indexer.index(&from);
        let to_index = indexer.index(&to);
        Self {
            guards_bitmask: (1 << from_index) | (1 << to_index),
            from_index,
            to_index,
            from,
            to,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> OpTrait<K, V> for MoveValueOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let from_guard = mutex_guards
            .get_mut(self.from_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        let value = from_guard.remove(&self.from);
        drop(from_guard);

        let to_guard = mutex_guards
            .get_mut(self.to_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some(v) = value {
            to_guard.insert(self.to.clone(), v);
        }
    }
}
