use crate::{indexer::Indexer, ops::op_trait::OpTrait, result::MISSING_MUTEX_GUARD_ERROR};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct MapOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>) -> Option<V>>,
}

impl<K, V> MapOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<T>(indexer: &Indexer, key: K, transform: T) -> Self
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            transform: Box::new(transform),
        }
    }
    fn mapped_value(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Option<V> {
        let key_guard = mutex_guards.get(self.key_index);
        let key_shard = key_guard.expect(MISSING_MUTEX_GUARD_ERROR);
        let key_value = key_shard.get(&self.key);
        (self.transform)(&self.key, key_value)
    }
}

impl<K, V> OpTrait<K, V> for MapOp<K, V>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> u128 {
        self.guards_bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) {
        let new_value = self.mapped_value(&mutex_guards);
        let guard = mutex_guards.get_mut(self.key_index);
        let shard = guard.expect(MISSING_MUTEX_GUARD_ERROR);
        match new_value {
            Some(v) => shard.insert(self.key.clone(), v),
            None => shard.remove(&self.key),
        };
    }
}
