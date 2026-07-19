use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub(crate) struct RetainOp<K, V, P = ()> {
    guards_bitmask: u128,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RetainOp<K, V, P> {
    pub fn new_with_params<C>(indexer: &Indexer, condition: C) -> Self
    where
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let guards_bitmask = indexer.all_bitmask();
        Self {
            guards_bitmask,
            condition: Box::new(condition),
        }
    }
}

impl<K, V> RetainOp<K, V, ()> {
    pub fn new<C>(indexer: &Indexer, condition: C) -> Self
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_params(indexer, move |k, v, _| condition(k, v))
    }
}

impl<K, V, P> OpTrait<K, V, P> for RetainOp<K, V, P> {
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.retain(|k, v| (self.condition)(k, v, params));
        }
    }
}
