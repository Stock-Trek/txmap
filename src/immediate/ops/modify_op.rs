use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct ModifyOpApply;

impl ModifyOpApply {
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        mutate: &dyn Fn(&K, &mut V, &mut STATE),
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = lock_guards
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(mut_entry) = write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            (mutate)(&mut_entry.0, &mut mut_entry.1, state)
        }
    }
}

pub(crate) struct ModifyOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub mutate: Box<dyn Fn(&K, &mut V, &mut STATE) + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for ModifyOp<'tx, K, V, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        ModifyOpApply::apply(&self.key, &*self.mutate, lock_guards, state)
    }
}
