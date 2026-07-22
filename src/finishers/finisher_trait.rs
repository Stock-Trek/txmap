use crate::{locks::lock_policy::LockPolicy, new_types::BitMask, shard::Shard};
use intmap::IntMap;

pub trait FinisherTrait<K, V> {
    type Output;

    fn guards_bitmask(&self) -> BitMask;
    fn to_result<L>(
        &self,
        mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
    ) -> Self::Output
    where
        L: LockPolicy;
}
