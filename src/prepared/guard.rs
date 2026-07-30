use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, prepared::schema::TxKeySelector, shard_ops::ShardOps,
};
use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub(crate) struct Guard<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
{
    pub name: String,
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx>,
    pub _phantom: PhantomData<STATE>,
}

impl<'tx, K, V, KEYS, PARAMS, STATE> Guard<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
{
    pub fn read_bitmask(&self, keys: &KEYS) -> BitMask {
        let key = self.key_selector.get(keys);
        key.shard_index.bitmask()
    }
    pub fn is_condition_met<L>(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) -> bool
    where
        L: LockPolicy,
    {
        let key = self.key_selector.get(keys);
        let shard = if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            lock_guards.write_guard(key).deref_mut()
        } else {
            lock_guards.read_guard(key).deref()
        };
        let value_ref = ShardOps::value_ref(shard, key);
        (self.condition)(&key.key, value_ref, params, state)
    }
}
