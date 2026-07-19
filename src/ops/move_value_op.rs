use crate::{
    indexer::Indexer, ops::op_trait::ParameterizedOpTrait, result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct MoveValueOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    from_index: u8,
    to_index: u8,
    from: K,
    to: K,
    _phantom: PhantomData<(V, P)>,
}

impl<K, V, P> MoveValueOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_param(indexer: &Indexer, from: K, to: K, _param_placeholder: P) -> Self {
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

impl<K, V> MoveValueOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new(indexer: &Indexer, from: K, to: K) -> Self {
        Self::new_with_param(indexer, from, to, ())
    }
}

impl<K, V, P> ParameterizedOpTrait<K, V, P> for MoveValueOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, _params: &P) {
        let value = {
            let from_guard = mutex_guards
                .get_mut(self.from_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            from_guard.remove(&self.from)
        };
        if let Some(v) = value {
            let to_guard = mutex_guards
                .get_mut(self.to_index)
                .expect(MISSING_MUTEX_GUARD_ERROR);
            to_guard.insert(self.to.clone(), v);
        }
    }
}
