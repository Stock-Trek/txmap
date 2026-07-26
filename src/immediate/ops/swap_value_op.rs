use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct SwapValueOp<K>
where
    K: Hash + Eq,
{
    pub key_a: TxKey<K>,
    pub key_b: TxKey<K>,
}

impl<K, V, L, STATE> OpTrait<K, V, L, STATE> for SwapValueOp<K>
where
    K: Clone + Hash + Eq,
    V:,
    L: LockPolicy,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            self.key_a.shard_index.bitmask() | self.key_b.shard_index.bitmask(),
        )
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, _: &mut STATE) {
        let a = lock_guards.remove_entry(&self.key_a);
        let b = lock_guards.remove_entry(&self.key_b);
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    lock_guards.insert_with_duplicate_key(&self.key_a, a_key, b_value);
                    lock_guards.insert_with_duplicate_key(&self.key_b, b_key, a_value);
                }
                None => {
                    lock_guards.insert(&self.key_b, a_value);
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    lock_guards.insert(&self.key_a, b_value);
                }
            }
        }
    }
}
