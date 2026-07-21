use crate::new_types::BitMask;
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub(crate) trait OpTrait<K, V, P> {
    fn guards_bitmask(&self) -> BitMask;
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P);
}
