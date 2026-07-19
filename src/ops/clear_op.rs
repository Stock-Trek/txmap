use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct ClearOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    _phantom: PhantomData<(K, V, P)>,
}

impl<K, V, P> ClearOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_param(indexer: &Indexer, _param_placeholder: P) -> Self {
        let guards_bitmask = indexer.all_bitmask();
        Self {
            guards_bitmask,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> ClearOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new(indexer: &Indexer) -> Self {
        Self::new_with_param(indexer, ())
    }
}

impl<K, V, P> OpTrait<K, V, P> for ClearOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, _params: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.clear();
        }
    }
}
