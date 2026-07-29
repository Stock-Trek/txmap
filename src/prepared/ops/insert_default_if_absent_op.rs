use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    prepared::schema::TxKeySelector,
};
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOpApply;

impl InsertDefaultIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, KEYS>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
    ) where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        lock_guards.insert_if_absent(key, || V::default());
    }
}

pub(crate) struct InsertDefaultIfAbsentOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}
