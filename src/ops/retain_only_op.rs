use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::marker::PhantomData;

pub(crate) struct RetainOnlyOp<K, V> {
    guards_bitmask: u128,
    keys: Vec<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> RetainOnlyOp<K, V> {
    pub fn new<I>(indexer: &Indexer, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        Self {
            guards_bitmask: indexer.all_bitmask(),
            keys: keys.into_iter().collect(),
            _phantom: PhantomData,
        }
    }
}

impl<K, V, P> OpTrait<K, V, P> for RetainOnlyOp<K, V>
where
    K: Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, _: &P) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.retain(|k, _| self.keys.contains(k));
        }
    }
}
