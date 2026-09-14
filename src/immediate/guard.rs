use crate::{
    custodian::Custodian, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask, shard_ops::ShardOps,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

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

    /// Re-routes the guard key after a routing change.
    pub fn reroute<L>(&mut self, custodian: &Custodian<K, V, L>)
    where
        L: LockPolicy,
    {
        self.key.shard_index = custodian.route(self.key.hash_code);
        self.key.version = custodian.version(self.key.shard_index);
    }
}

impl<'tx, K, V, STATE> ImmediateGuard<'tx, K, V, STATE>
where
    K: Eq,
{
    pub fn condition_is_met<L>(
        &mut self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
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
        let shard = if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            lock_guards.write_guard(key).deref_mut()
        } else {
            lock_guards.read_guard(key).deref()
        };
        let value_ref = ShardOps::value_ref(shard, key.hash_code, &key.key);
        (condition)(&key.key, value_ref, state)
    }
}
