use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct Guard<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub name: String,
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(Option<&V>, &mut STATE) -> bool + 'tx>,
    pub _phantom: PhantomData<STATE>,
}

impl<'tx, K, V, STATE> Guard<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub fn read_bitmask(&self) -> BitMask {
        self.key.shard_index.bitmask()
    }
    pub fn is_condition_met<L>(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) -> bool
    where
        L: LockPolicy,
    {
        if (self.key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_key, value)| value);
            (self.condition)(value_ref, state)
        } else {
            let value_ref = lock_guards
                .read
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_key, value)| value);
            (self.condition)(value_ref, state)
        }
    }
}
