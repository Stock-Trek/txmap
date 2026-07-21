use crate::{finishers::finisher_trait::FinisherTrait, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub struct NoneFinisher;

impl<K, V> FinisherTrait<K, V> for NoneFinisher {
    type Output = ();

    fn guards_bitmask(&self) -> BitMask {
        BitMask::default()
    }
    fn to_result(&self, _: &IntMap<u8, MutexGuard<HashTable<(K, V)>>>) -> Self::Output {}
}
