use crate::{lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy, new_types::BitMask};
use std::hash::Hash;

pub(crate) trait OpTrait<K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask);
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE);
}
