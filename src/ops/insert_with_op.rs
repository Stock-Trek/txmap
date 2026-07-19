use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct InsertWithOp<K, V, P = ()> {
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    #[allow(clippy::type_complexity)]
    value_generator: Box<dyn Fn(&K, &P) -> V>,
}

impl<K, V, P> InsertWithOp<K, V, P>
where
    K: Hash,
{
    pub fn new_with_params<G>(indexer: &Indexer, key: K, value_generator: G) -> Self
    where
        G: Fn(&K, &P) -> V + 'static,
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

impl<K, V> InsertWithOp<K, V, ()>
where
    K: Hash,
{
    pub fn new<G>(indexer: &Indexer, key: K, value_generator: G) -> Self
    where
        G: Fn(&K) -> V + 'static,
    {
        Self::new_with_params(indexer, key, move |k, _| value_generator(k))
    }
}

impl<K, V, P> OpTrait<K, V, P> for InsertWithOp<K, V, P>
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
        let new_value = (self.value_generator)(&self.key, params);
        mutex_guard.insert(self.key.clone(), new_value);
    }
}
