use crate::{
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    ops::op_trait::OpTrait,
    params::{TxKey, TxKeySelector},
    result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct UpdateOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub transform: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> OpTrait<K, V, L, KEYS, PARAMS, STATE>
    for UpdateOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            self.key_selector.get(keys).shard_index.bitmask(),
        )
    }
    fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) {
        let key = self.key_selector.get(keys);
        if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(key.hash_code.0, |entry| entry.0 == key.key)
                .map(|(_key, value)| value);
            let new_value = (self.transform)(&key.key, value_ref, params, state);
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
            let new_value = (self.transform)(&key.key, value_ref, params, state);
            match new_value {
                Some(v) => lock_guards.insert(key, v),
                None => {
                    lock_guards.remove_entry(key);
                }
            };
        }
    }
}
