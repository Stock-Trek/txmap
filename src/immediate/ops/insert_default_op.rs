use crate::{key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy};
use std::hash::Hash;

pub(crate) struct InsertDefaultOpApply;

impl InsertDefaultOpApply {
    pub(crate) fn apply<K, V, L>(key: &TxKey<K>, lock_guards: &mut LockGuards<'_, K, V, L>)
    where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        lock_guards.insert(key, V::default());
    }
}

pub(crate) struct InsertDefaultOp<K>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
}
