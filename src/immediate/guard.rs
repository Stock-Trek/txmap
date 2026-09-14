use crate::{
    key::TxKey, lock_guards::LockGuard, lock_policies::lock_policy::LockPolicy, new_types::BitMask,
    shard_ops::ShardOps,
};
use std::marker::PhantomData;

pub(crate) struct ImmediateGuard<'tx, K, V, STATE> {
    pub name: String,
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub condition: Option<Box<dyn FnOnce(&K, Option<&V>, &mut STATE) -> bool + 'tx>>,
    pub _phantom: PhantomData<STATE>,
}

impl<'tx, K, V, STATE> ImmediateGuard<'tx, K, V, STATE> {
    pub fn read_bitmask(&self) -> BitMask {
        self.key.shard_index.bitmask()
    }
}

impl<'tx, K, V, STATE> ImmediateGuard<'tx, K, V, STATE>
where
    K: Eq,
{
    pub fn condition_is_met<L>(
        &mut self,
        lock_guards: &mut LockGuard<'_, K, V, L>,
        state: &mut STATE,
    ) -> bool
    where
        L: LockPolicy,
    {
        let condition = self
            .condition
            .take()
            .expect("guard condition already evaluated");
        let key = &self.key;
        let shard = lock_guards.read_guard(key);
        let value_ref = ShardOps::value_ref(shard, key.hash_code, &key.key);
        (condition)(&key.key, value_ref, state)
    }
}
