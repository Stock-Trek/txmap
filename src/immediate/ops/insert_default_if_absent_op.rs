use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOpApply;

impl InsertDefaultIfAbsentOpApply {
    pub(crate) fn apply<K, V, L>(key: &TxKey<K>, lock_guards: &mut LockGuards<'_, K, V, L>)
    where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || V::default());
    }
}

pub(crate) struct InsertDefaultIfAbsentOp<K>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
}

impl<K, V, L, STATE> OpTrait<K, V, L, STATE> for InsertDefaultIfAbsentOp<K>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, _state: &mut STATE) {
        InsertDefaultIfAbsentOpApply::apply::<K, V, L>(&self.key, lock_guards)
    }
}
