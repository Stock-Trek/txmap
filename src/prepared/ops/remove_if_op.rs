use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, prepared::schema::TxKeySelector, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct RemoveIfOpApply;

impl RemoveIfOpApply {
    pub(crate) fn apply<K, V, L, KEYS, PARAMS, STATE>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        condition: &dyn Fn(&K, &V, &PARAMS, &mut STATE) -> bool,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);

        if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            if let Some(v) = value_ref
                && (condition)(&key.key, v, params, state)
            {
                lock_guards.remove_entry(key);
            }
        } else {
            let value_ref = lock_guards
                .read
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            if let Some(v) = value_ref
                && (condition)(&key.key, v, params, state)
            {
                lock_guards.remove_entry(key);
            }
        }
    }
}

pub(crate) struct RemoveIfOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx>,
}
