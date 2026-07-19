use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub trait FinisherTrait<K, V> {
    type Output;

    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Self::Output;
}
