use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct MutOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    key_index: u8,
    key: K,
    mutator: Box<dyn Fn(&mut V)>,
    value_generator: Option<Box<dyn Fn() -> V>>,
}

impl<K, V> MutOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<M>(indexer: &Indexer, key: K, mutate: M) -> Self
    where
        M: Fn(&mut V) + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutator: Box::new(mutate),
            value_generator: None,
        }
    }
    pub fn new_or_insert_with<M, G>(
        indexer: &Indexer,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> Self
    where
        M: Fn(&mut V) + 'static,
        G: Fn() -> V + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutator: Box::new(mutate),
            value_generator: Some(Box::new(value_generator)),
        }
    }
}

impl<K, V> OpTrait<K, V> for MutOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some((_, value)) = mutex_guard.get_key_value_mut(&self.key) {
            (self.mutator)(value);
        } else if let Some(value_generator) = &self.value_generator {
            let new_value = value_generator();
            mutex_guard.insert(self.key.clone(), new_value);
        }
    }
}
