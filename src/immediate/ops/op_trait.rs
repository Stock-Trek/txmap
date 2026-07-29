use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::{hash::Hash, marker::PhantomData};

#[allow(clippy::type_complexity)]
pub(crate) enum ImmediateOp<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    Get {
        key: TxKey<K>,
        get: Box<dyn Fn(&K, Option<&V>, &mut STATE) + 'tx>,
    },
    InsertDefault {
        key: TxKey<K>,
    },
    InsertDefaultIfAbsent {
        key: TxKey<K>,
    },
    InsertWith {
        key: TxKey<K>,
        value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
    },
    InsertWithIfAbsent {
        key: TxKey<K>,
        value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
    },
    Modify {
        key: TxKey<K>,
        mutate: Box<dyn Fn(&K, &mut V, &mut STATE) + 'tx>,
    },
    MoveValue {
        key_from: TxKey<K>,
        key_to: TxKey<K>,
    },
    Remove {
        key: TxKey<K>,
        on_remove: Box<dyn Fn(Option<(K, V)>, &mut STATE) + 'tx>,
    },
    RemoveIf {
        key: TxKey<K>,
        condition: Box<dyn Fn(&K, &V, &mut STATE) -> bool + 'tx>,
    },
    SwapValue {
        key_a: TxKey<K>,
        key_b: TxKey<K>,
    },
    Update {
        key: TxKey<K>,
        transform: Box<dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
    },
    #[doc(hidden)]
    _Phantom(PhantomData<L>),
}

impl<'tx, K, V, L, STATE> ImmediateOp<'tx, K, V, L, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    pub fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        match self {
            Self::Get { key, .. } => (key.shard_index.bitmask(), BitMask::ZERO),
            Self::InsertDefault { key, .. }
            | Self::InsertDefaultIfAbsent { key, .. }
            | Self::InsertWith { key, .. }
            | Self::InsertWithIfAbsent { key, .. }
            | Self::Modify { key, .. }
            | Self::Remove { key, .. }
            | Self::RemoveIf { key, .. }
            | Self::Update { key, .. } => (BitMask::ZERO, key.shard_index.bitmask()),
            Self::MoveValue {
                key_from, key_to, ..
            } => (
                BitMask::ZERO,
                key_from.shard_index.bitmask() | key_to.shard_index.bitmask(),
            ),
            Self::SwapValue { key_a, key_b, .. } => (
                BitMask::ZERO,
                key_a.shard_index.bitmask() | key_b.shard_index.bitmask(),
            ),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}

impl<'tx, K, V, L, STATE> ImmediateOp<'tx, K, V, L, STATE>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
{
    pub fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        match self {
            Self::Get { key, get } => {
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_, value)| value);
                    (get)(&key.key, value_ref, state)
                } else {
                    let value_ref = lock_guards
                        .read
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_, value)| value);
                    (get)(&key.key, value_ref, state)
                }
            }
            Self::InsertDefault { key } => {
                lock_guards.insert(key, V::default());
            }
            Self::InsertDefaultIfAbsent { key } => {
                lock_guards.insert_if_absent(key, || V::default());
            }
            Self::InsertWith {
                key,
                value_generator,
            } => {
                let new_value = (value_generator)(&key.key, state);
                lock_guards.insert(key, new_value);
            }
            Self::InsertWithIfAbsent {
                key,
                value_generator,
            } => {
                lock_guards.insert_if_absent(key, || (value_generator)(&key.key, state));
            }
            Self::Modify { key, mutate } => {
                let write_guard = lock_guards
                    .write
                    .get_mut(key.shard_index.0)
                    .expect(MISSING_LOCK_GUARD_ERROR);
                if let Some(mut_entry) =
                    write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key)
                {
                    (mutate)(&mut_entry.0, &mut mut_entry.1, state)
                }
            }
            Self::MoveValue { key_from, key_to } => {
                let removed = lock_guards.remove_entry(key_from);
                if let Some(entry) = removed {
                    lock_guards.insert(key_to, entry.1);
                } else {
                    lock_guards.remove_entry(key_to);
                }
            }
            Self::Remove { key, on_remove } => {
                let removed_entry = lock_guards.remove_entry(key);
                (on_remove)(removed_entry, state)
            }
            Self::RemoveIf { key, condition } => {
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_key, value)| value);
                    if let Some(v) = value_ref
                        && (condition)(&key.key, v, state)
                    {
                        lock_guards.remove_entry(key);
                    }
                } else {
                    let value_ref = lock_guards
                        .read
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_key, value)| value);
                    if let Some(v) = value_ref
                        && (condition)(&key.key, v, state)
                    {
                        lock_guards.remove_entry(key);
                    }
                }
            }
            Self::SwapValue { key_a, key_b } => {
                let a = lock_guards.remove_entry(key_a);
                let b = lock_guards.remove_entry(key_b);
                match a {
                    Some((a_key, a_value)) => match b {
                        Some((b_key, b_value)) => {
                            lock_guards.insert_with_duplicate_key(key_a, a_key, b_value);
                            lock_guards.insert_with_duplicate_key(key_b, b_key, a_value);
                        }
                        None => {
                            lock_guards.insert(key_b, a_value);
                        }
                    },
                    None => {
                        if let Some((_, b_value)) = b {
                            lock_guards.insert(key_a, b_value);
                        }
                    }
                }
            }
            Self::Update { key, transform } => {
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_key, value)| value);
                    let new_value = (transform)(&key.key, value_ref, state);
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
                    let new_value = (transform)(&key.key, value_ref, state);
                    match new_value {
                        Some(v) => lock_guards.insert(key, v),
                        None => {
                            lock_guards.remove_entry(key);
                        }
                    };
                }
            }
            Self::_Phantom(_) => unreachable!(),
        }
    }
}
