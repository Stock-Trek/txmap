use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct MoveValueOp<K>
where
    K: Hash + Eq,
{
    pub key_from: TxKey<K>,
    pub key_to: TxKey<K>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for MoveValueOp<K>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            self.key_from.shard_index.bitmask() | self.key_to.shard_index.bitmask(),
        )
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, _: &mut STATE) {
        let removed = lock_guards.remove_entry(&self.key_from);
        if let Some(entry) = removed {
            lock_guards.insert(&self.key_to, entry.1);
        } else {
            lock_guards.remove_entry(&self.key_to);
        }
    }
}
