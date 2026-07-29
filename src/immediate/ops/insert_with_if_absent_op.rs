use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask,
};
use std::hash::{BuildHasher, Hash};

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}

impl<'tx, K, V, L, S, STATE> OpTrait<K, V, L, S, STATE> for InsertWithIfAbsentOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L, S>, state: &mut STATE) {
        lock_guards.insert_if_absent(&self.key, || (self.value_generator)(&self.key.key, state));
    }
}
