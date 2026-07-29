use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct RemoveOpApply;

impl RemoveOpApply {
    #[allow(clippy::type_complexity)]
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        on_remove: &dyn Fn(Option<(K, V)>, &mut STATE),
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let removed_entry = lock_guards.remove_entry(key);
        (on_remove)(removed_entry, state)
    }
}

pub(crate) struct RemoveOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub on_remove: Box<dyn Fn(Option<(K, V)>, &mut STATE) + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for RemoveOp<'tx, K, V, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        RemoveOpApply::apply(&self.key, &*self.on_remove, lock_guards, state)
    }
}
