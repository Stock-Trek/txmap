use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::{BuildHasher, Hash};

pub(crate) struct InsertDefaultIfAbsentOp<K>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
}

impl<K, V, L, S, STATE> OpTrait<K, V, L, S, STATE> for InsertDefaultIfAbsentOp<K>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
    S: BuildHasher,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L, S>, _state: &mut STATE) {
        lock_guards.insert_if_absent(&self.key, || V::default());
    }
}
