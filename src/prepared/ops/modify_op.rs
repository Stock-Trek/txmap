use crate::{
    key::TxKey,
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{ops::op_trait::OpTrait, schema::TxKeySelector},
    result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct ModifyOpApply;

impl ModifyOpApply {
    pub(crate) fn apply<K, V, L, KEYS, PARAMS, STATE>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        mutate: &dyn Fn(&K, &mut V, &PARAMS, &mut STATE),
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        let write_guard = lock_guards
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(mut_entry) = write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            (mutate)(&mut_entry.0, &mut mut_entry.1, params, state)
        }
    }
}

pub(crate) struct ModifyOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub mutate: Box<dyn Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> OpTrait<K, V, L, KEYS, PARAMS, STATE>
    for ModifyOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
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
        ModifyOpApply::apply(
            &*self.key_selector,
            &*self.mutate,
            lock_guards,
            keys,
            params,
            state,
        )
    }
}
