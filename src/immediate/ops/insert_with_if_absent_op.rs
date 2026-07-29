use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOpApply;

impl InsertWithIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &dyn Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, state));
    }
}

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for InsertWithIfAbsentOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        InsertWithIfAbsentOpApply::apply(&self.key, &*self.value_generator, lock_guards, state)
    }
}
