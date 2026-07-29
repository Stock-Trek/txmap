use crate::{lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy, new_types::BitMask};
use std::hash::{BuildHasher, Hash};

pub(crate) trait OpTrait<K, V, L, S, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
    STATE: Default,
{
    fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask);
    fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L, S>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    );
}
