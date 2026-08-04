use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, shard_ops::ShardOps,
};
use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub(crate) struct Guard<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
{
    pub name: String,
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn FnOnce(&K, Option<&V>, &mut STATE) -> bool + 'tx>,
    pub _phantom: PhantomData<STATE>,
}

impl<'tx, K, V, STATE> Guard<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq,
{
    pub fn read_bitmask(&self) -> BitMask {
        self.key.shard_index.bitmask()
    }
    pub fn condition_is_met<L>(
        self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) -> bool
    where
        L: LockPolicy,
    {
        let Self { key, condition, .. } = self;
        let shard = if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            lock_guards.write_guard(&key).deref_mut()
        } else {
            lock_guards.read_guard(&key).deref()
        };
        let value_ref = ShardOps::value_ref(shard, &key);
        (condition)(&key.key, value_ref, state)
    }
}
