use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyOrDefaultOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    mutate: Box<dyn Fn(&K, &mut V)>,
}

impl<K, V> ModifyOrDefaultOp<K, V>
where
    K: Clone + Hash + Eq,
    V: Default,
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

impl<K, V> OpTrait<K, V> for ModifyOrDefaultOp<K, V>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some((key, value)) = mutex_guard.get_key_value_mut(&self.key) {
            (self.mutate)(key, value);
        } else {
            let mut value = V::default();
            (self.mutate)(&self.key, &mut value);
            mutex_guard.insert(self.key.clone(), value);
        }
    }
}
