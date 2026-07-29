use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    prepared::schema::TxKeySelector,
};
use std::hash::Hash;

pub(crate) struct MoveValueOpApply;

impl MoveValueOpApply {
    pub(crate) fn apply<K, V, L, KEYS>(
        key_selector_from: &dyn TxKeySelector<TxKey<K>, KEYS>,
        key_selector_to: &dyn TxKeySelector<TxKey<K>, KEYS>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let key_from = key_selector_from.get(keys);
        let key_to = key_selector_to.get(keys);
        let removed = lock_guards.remove_entry(key_from);
        if let Some(entry) = removed {
            lock_guards.insert(key_to, entry.1);
        } else {
            lock_guards.remove_entry(key_to);
        }
    }
}

pub(crate) struct MoveValueOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector_from: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    pub key_selector_to: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}
