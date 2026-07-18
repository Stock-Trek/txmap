use crate::{indexer::Indexer, ops::op_trait::OpTrait};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct InsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    value_generator: Box<dyn Fn(&K) -> V>,
}

impl<K, V> InsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<G>(indexer: &Indexer, key: K, value_generator: G) -> Self
    where
        G: Fn(&K) -> V + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            value_generator: Box::new(value_generator),
        }
    }
}

impl<K, V> OpTrait<K, V> for InsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn mutex_guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let mutex_guard = mutex_guards.get_mut(self.key_index).expect("No Guard");
        let new_value = (self.value_generator)(&self.key);
        mutex_guard.insert(self.key.clone(), new_value);
    }
}
