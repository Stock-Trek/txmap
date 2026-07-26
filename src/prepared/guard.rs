use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, prepared::schema::TxKeySelector, result::MISSING_LOCK_GUARD_ERROR,
};
use std::{hash::Hash, marker::PhantomData};

pub(crate) struct Guard<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub name: String,
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx>,
    pub _phantom: PhantomData<STATE>,
}

impl<'tx, K, V, KEYS, PARAMS, STATE> Guard<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
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
        if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            (self.condition)(&key.key, value_ref, params, state)
        } else {
            let value_ref = lock_guards
                .read
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            (self.condition)(&key.key, value_ref, params, state)
        }
    }
}
