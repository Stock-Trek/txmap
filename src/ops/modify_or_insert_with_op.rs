use crate::{
    indexer::Indexer, ops::op_trait::ParameterizedOpTrait, result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyOrInsertWithOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V, &P)>,
    #[allow(clippy::type_complexity)]
    value_generator: Box<dyn Fn(&K, &P) -> V>,
}

impl<K, V, P> ModifyOrInsertWithOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new_with_param<M, G>(indexer: &Indexer, key: K, mutate: M, value_generator: G) -> Self
    where
        M: Fn(&K, &mut V, &P) + 'static,
        G: Fn(&K, &P) -> V + 'static,
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

impl<K, V> ModifyOrInsertWithOp<K, V, ()>
where
    K: Clone + Hash + Eq,
{
    pub fn new<M, G>(indexer: &Indexer, key: K, mutate: M, value_generator: G) -> Self
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        Self::new_with_param(
            indexer,
            key,
            move |k, v, _| mutate(k, v),
            move |k, _| value_generator(k),
        )
    }
}

impl<K, V, P> ParameterizedOpTrait<K, V, P> for ModifyOrInsertWithOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P) {
        let mutex_guard = mutex_guards
            .get_mut(self.key_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        if let Some((key, value)) = mutex_guard.get_key_value_mut(&self.key) {
            (self.mutate)(key, value, params);
        } else {
            let new_value = (self.value_generator)(&self.key, params);
            mutex_guard.insert(self.key.clone(), new_value);
        }
    }
}
