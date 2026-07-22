use crate::{locks::lock_policy::LockPolicy, new_types::BitMask, shard::Shard};
use intmap::IntMap;

pub(crate) trait OpTrait<L, K, V, P> {
    fn guards_bitmask(&self) -> BitMask;
    fn apply(&self, mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>, params: &P)
    where
        L: LockPolicy;
}
