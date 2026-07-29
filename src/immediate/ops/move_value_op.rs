use crate::{key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy};
use std::hash::Hash;

pub(crate) struct MoveValueOpApply;

impl MoveValueOpApply {
    pub(crate) fn apply<K, V, L>(
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let removed = lock_guards.remove_entry(key_from);
        if let Some(entry) = removed {
            lock_guards.insert(key_to, entry.1);
        } else {
            lock_guards.remove_entry(key_to);
        }
    }
}

pub(crate) struct MoveValueOp<K>
where
    K: Hash + Eq,
{
    pub key_from: TxKey<K>,
    pub key_to: TxKey<K>,
}
