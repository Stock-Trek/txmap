use crate::{key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy};
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOpApply;

impl InsertWithIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &dyn Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, state));
    }
}

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}
