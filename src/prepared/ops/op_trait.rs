use crate::{lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy, new_types::BitMask};
use std::hash::Hash;

pub(crate) trait OpTrait<K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask);
    fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    );
}
