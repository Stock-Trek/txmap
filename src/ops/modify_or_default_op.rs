use crate::{
    indexer::Indexer, ops::op_trait::ParameterizedOpTrait, result::MISSING_MUTEX_GUARD_ERROR,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyOrDefaultOp<K, V, P = ()>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V, &P)>,
}

impl<K, V, P> ModifyOrDefaultOp<K, V, P>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    pub fn new_with_param<M>(indexer: &Indexer, key: K, mutate: M) -> Self
    where
        M: Fn(&K, &mut V, &P) + 'static,
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

impl<K, V> ModifyOrDefaultOp<K, V, ()>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    pub fn new<M>(indexer: &Indexer, key: K, mutate: M) -> Self
    where
        M: Fn(&K, &mut V) + 'static,
    {
        Self::new_with_param(indexer, key, move |k, v, _| mutate(k, v))
    }
}

impl<K, V, P> ParameterizedOpTrait<K, V, P> for ModifyOrDefaultOp<K, V, P>
where
    K: Clone + Hash + Eq,
    V: Default,
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
            let mut value = V::default();
            (self.mutate)(&self.key, &mut value, params);
            mutex_guard.insert(self.key.clone(), value);
        }
    }
}
