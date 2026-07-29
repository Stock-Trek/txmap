use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct UpdateOpApply;

impl UpdateOpApply {
    #[allow(clippy::type_complexity)]
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        transform: &dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            let new_value = (transform)(&key.key, value_ref, state);
            match new_value {
                Some(v) => lock_guards.insert(key, v),
                None => {
                    lock_guards.remove_entry(key);
                }
            };
        } else {
            let value_ref = lock_guards
                .read
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            let new_value = (transform)(&key.key, value_ref, state);
            match new_value {
                Some(v) => lock_guards.insert(key, v),
                None => {
                    lock_guards.remove_entry(key);
                }
            };
        }
    }
}

pub(crate) struct UpdateOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub transform: Box<dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
}
