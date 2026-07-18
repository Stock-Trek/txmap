use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyOrInsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    mutate: Box<dyn Fn(&K, &mut V)>,
    value_generator: Box<dyn Fn(&K) -> V>,
}

impl<K, V> ModifyOrInsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<M, G>(indexer: &Indexer, key: K, mutate: M, value_generator: G) -> Self
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutate: Box::new(mutate),
            value_generator: Box::new(value_generator),
        }
    }
}

impl<K, V> OpTrait<K, V> for ModifyOrInsertWithOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some((key, value)) = mutex_guard.get_key_value_mut(&self.key) {
            (self.mutate)(key, value);
        } else {
            let new_value = (self.value_generator)(&self.key);
            mutex_guard.insert(self.key.clone(), new_value);
        }
    }
}
