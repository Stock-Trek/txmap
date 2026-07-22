use crate::{
    finishers::finisher_trait::FinisherTrait, locks::lock_policy::LockPolicy, new_types::BitMask,
    shard::Shard,
};
use intmap::IntMap;

pub struct NoneFinisher;

impl<K, V> FinisherTrait<K, V> for NoneFinisher {
    type Output = ();

    fn guards_bitmask(&self) -> BitMask {
        BitMask::default()
    }
    fn to_result<L>(&self, _: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>) -> Self::Output
    where
        L: LockPolicy,
    {
    }
}
