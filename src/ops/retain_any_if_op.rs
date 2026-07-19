use crate::{indexer::Indexer, ops::op_trait::ParameterizedOpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RetainAnyIfOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RetainAnyIfOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_param<C>(indexer: &Indexer, condition: C) -> Self
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

impl<K, V> RetainAnyIfOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new<C>(indexer: &Indexer, condition: C) -> Self
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_param(indexer, move |k, v, _| condition(k, v))
    }
}

impl<K, V, P> ParameterizedOpTrait<K, V, P> for RetainAnyIfOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.retain(|k, v| (self.condition)(k, v, params));
        }
    }
}
