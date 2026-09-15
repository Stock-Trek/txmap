use crate::{
    custodian::Custodian, key::TxKey, lock_guards::LockGuards, new_types::BitMask,
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

    /// Re-routes the guard key after a routing change.
    pub fn reroute(&mut self, custodian: &Custodian<K, V>) {
        self.key.shard_index = custodian.route(self.key.hash_code);
        self.key.version = custodian.version(self.key.shard_index);
    }
}

impl<'tx, K, V, STATE> ImmediateGuard<'tx, K, V, STATE>
where
    K: Eq,
{
    pub fn condition_is_met(
        &mut self,
        lock_guards: &mut LockGuards<'_, K, V>,
        state: &mut STATE,
    ) -> bool {
        let condition = self
            .condition
            .take()
            .expect("guard condition already evaluated");
        let key = &self.key;
        let shard: &_ = if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO
        {
            &*lock_guards.write_guard(key)
        } else {
            lock_guards.read_guard(key)
        };
        let value_ref = ShardOps::value_ref(shard, key.hash_code, &key.key);
        (condition)(&key.key, value_ref, state)
    }
}
