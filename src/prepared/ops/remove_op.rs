use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    prepared::schema::TxKeySelector,
};
use std::hash::Hash;

pub(crate) struct RemoveOpApply;

impl RemoveOpApply {
    #[allow(clippy::type_complexity)]
    pub(crate) fn apply<K, V, L, KEYS, PARAMS, STATE>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        on_remove: &dyn Fn(Option<(K, V)>, &PARAMS, &mut STATE),
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        let removed_entry = lock_guards.remove_entry(key);
        (on_remove)(removed_entry, params, state)
    }
}

pub(crate) struct RemoveOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub on_remove: Box<dyn Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx>,
}
