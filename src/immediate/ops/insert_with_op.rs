use crate::{key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy};
use std::hash::Hash;

pub(crate) struct InsertWithOpApply;

impl InsertWithOpApply {
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &dyn Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let new_value = (value_generator)(&key.key, state);
        lock_guards.insert(key, new_value);
    }
}

pub(crate) struct InsertWithOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}
