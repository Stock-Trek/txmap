use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::marker::PhantomData;

pub(crate) struct ClearOp<K, V> {
    guards_bitmask: u128,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> ClearOp<K, V> {
    pub fn new(indexer: &Indexer) -> Self {
        let guards_bitmask = indexer.all_bitmask();
        Self {
            guards_bitmask,
            _phantom: PhantomData,
        }
    }
}

impl<K, V, P> OpTrait<K, V, P> for ClearOp<K, V> {
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, _params: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.clear();
        }
    }
}
