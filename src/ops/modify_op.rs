use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V)>,
}

impl<K, V> ModifyOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<M>(indexer: &Indexer, key: K, mutate: M) -> Self
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutate: Box::new(mutate),
        }
    }
}

impl<K, V> OpTrait<K, V> for ModifyOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some(key_mut_value) = mutex_guard.get_key_value_mut(&self.key) {
            (self.mutate)(key_mut_value.0, key_mut_value.1)
        }
    }
}
