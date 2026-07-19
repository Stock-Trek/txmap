use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub(crate) struct RetainWhereOp<K, V, P = ()> {
    guards_bitmask: u128,
    keys: Vec<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RetainWhereOp<K, V, P> {
    pub fn new_with_params<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        Self {
            guards_bitmask: indexer.all_bitmask(),
            keys: keys.into_iter().collect(),
            condition: Box::new(condition),
        }
    }
}

impl<K, V> RetainWhereOp<K, V, ()> {
    pub fn new<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_params(indexer, keys, move |k, v, _| condition(k, v))
    }
}

impl<K, V, P> OpTrait<K, V, P> for RetainWhereOp<K, V, P>
where
    K: Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.retain(|k, v| self.keys.contains(k) && (self.condition)(k, v, params));
        }
    }
}
