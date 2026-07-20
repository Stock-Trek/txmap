use crate::finishers::finisher_trait::FinisherTrait;
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;

pub struct NoneFinisher;

impl<K, V> FinisherTrait<K, V> for NoneFinisher {
    type Output = ();

    fn guards_bitmask(&self) -> u128 {
        0
    }
    fn to_result(&self, _: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Self::Output {}
}
