use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub(crate) trait OpTrait<K, V, P> {
    fn guards_bitmask(&self) -> u128;
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>, params: &P);
}
