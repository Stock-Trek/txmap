use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::Hash;

pub(crate) struct InsertWithOpApply;

impl InsertWithOpApply {
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &dyn Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let new_value = (value_generator)(&key.key, state);
        lock_guards.insert(key, new_value);
    }
}

pub(crate) struct InsertWithOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for InsertWithOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        InsertWithOpApply::apply(&self.key, &*self.value_generator, lock_guards, state)
    }
}
