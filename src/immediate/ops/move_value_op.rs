use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct MoveValueOpApply;

impl MoveValueOpApply {
    pub(crate) fn apply<K, V, L>(
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let removed = lock_guards.remove_entry(key_from);
        if let Some(entry) = removed {
            lock_guards.insert(key_to, entry.1);
        } else {
            lock_guards.remove_entry(key_to);
        }
    }
}

pub(crate) struct MoveValueOp<K>
where
    K: Hash + Eq,
{
    pub key_from: TxKey<K>,
    pub key_to: TxKey<K>,
}

impl<K, V, L, STATE> OpTrait<K, V, L, STATE> for MoveValueOp<K>
where
    K: Clone + Hash + Eq,
    V:,
    L: LockPolicy,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            self.key_from.shard_index.bitmask() | self.key_to.shard_index.bitmask(),
        )
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, _: &mut STATE) {
        MoveValueOpApply::apply::<K, V, L>(&self.key_from, &self.key_to, lock_guards)
    }
}
