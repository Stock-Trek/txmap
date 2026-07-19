use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub trait FinisherTrait<K, V>
where
    K: Clone + Hash + Eq,
{
    type Output;

    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Self::Output;
}
