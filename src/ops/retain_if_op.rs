use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    keys: Vec<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V) -> bool>,
}

impl<K, V> RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<I, C>(indexer: &Indexer, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self {
            guards_bitmask: indexer.all_bitmask(),
            keys: keys.into_iter().collect(),
            condition: Box::new(condition),
        }
    }
}

impl<K, V> OpTrait<K, V> for RetainIfOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        for mutex_guard in mutex_guards.values_mut() {
            mutex_guard.retain(|k, v| self.keys.contains(k) && (self.condition)(k, v));
        }
    }
}
