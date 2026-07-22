use crate::new_types::BitMask;
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub trait FinisherTrait<K, V> {
    type Output;

    fn guards_bitmask(&self) -> BitMask;
    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<HashTable<(K, V)>>>) -> Self::Output;
}
