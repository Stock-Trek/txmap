use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    prepared::schema::TxKeySelector,
};
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOpApply;

impl InsertWithIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, KEYS, PARAMS, STATE>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        value_generator: &dyn Fn(&K, &PARAMS, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, params, state));
    }
}

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &PARAMS, &mut STATE) -> V + 'tx>,
}
