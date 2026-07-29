use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, prepared::schema::TxKeySelector, result::MISSING_LOCK_GUARD_ERROR,
};
use std::{hash::Hash, marker::PhantomData};

#[allow(clippy::type_complexity)]
pub(crate) enum PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    Get {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        get: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx>,
    },
    InsertDefault {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    },
    InsertDefaultIfAbsent {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    },
    InsertWith {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        value_generator: Box<dyn Fn(&K, &PARAMS, &mut STATE) -> V + 'tx>,
    },
    InsertWithIfAbsent {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        value_generator: Box<dyn Fn(&K, &PARAMS, &mut STATE) -> V + 'tx>,
    },
    Modify {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        mutate: Box<dyn Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx>,
    },
    MoveValue {
        key_selector_from: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        key_selector_to: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    },
    Remove {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        on_remove: Box<dyn Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx>,
    },
    RemoveIf {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        condition: Box<dyn Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx>,
    },
    SwapValue {
        key_selector_a: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        key_selector_b: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    },
    Update {
        key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
        transform: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx>,
    },
    #[doc(hidden)]
    _Phantom(PhantomData<L>),
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    L: LockPolicy,
    STATE: Default,
{
    pub fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask) {
        match self {
            Self::Get { key_selector, .. } => {
                (key_selector.get(keys).shard_index.bitmask(), BitMask::ZERO)
            }
            Self::InsertDefault { key_selector, .. }
            | Self::InsertDefaultIfAbsent { key_selector, .. }
            | Self::InsertWith { key_selector, .. }
            | Self::InsertWithIfAbsent { key_selector, .. }
            | Self::Modify { key_selector, .. }
            | Self::Remove { key_selector, .. }
            | Self::RemoveIf { key_selector, .. }
            | Self::Update { key_selector, .. } => {
                (BitMask::ZERO, key_selector.get(keys).shard_index.bitmask())
            }
            Self::MoveValue {
                key_selector_from,
                key_selector_to,
                ..
            } => (
                BitMask::ZERO,
                key_selector_from.get(keys).shard_index.bitmask()
                    | key_selector_to.get(keys).shard_index.bitmask(),
            ),
            Self::SwapValue {
                key_selector_a,
                key_selector_b,
                ..
            } => (
                BitMask::ZERO,
                key_selector_a.get(keys).shard_index.bitmask()
                    | key_selector_b.get(keys).shard_index.bitmask(),
            ),
            Self::_Phantom(_) => unreachable!(),
        }
    }
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> PreparedOp<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
    V: Default,
    L: LockPolicy,
    STATE: Default,
{
    pub fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) {
        match self {
            Self::Get { key_selector, get } => {
                let key = key_selector.get(keys);
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_, value)| value);
                    (get)(&key.key, value_ref, params, state)
                } else {
                    let value_ref = lock_guards
                        .read
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_, value)| value);
                    (get)(&key.key, value_ref, params, state)
                }
            }
            Self::InsertDefault { key_selector } => {
                let key = key_selector.get(keys);
                lock_guards.insert(key, V::default());
            }
            Self::InsertDefaultIfAbsent { key_selector } => {
                let key = key_selector.get(keys);
                lock_guards.insert_if_absent(key, || V::default());
            }
            Self::InsertWith {
                key_selector,
                value_generator,
            } => {
                let key = key_selector.get(keys);
                let new_value = (value_generator)(&key.key, params, state);
                lock_guards.insert(key, new_value);
            }
            Self::InsertWithIfAbsent {
                key_selector,
                value_generator,
            } => {
                let key = key_selector.get(keys);
                lock_guards.insert_if_absent(key, || (value_generator)(&key.key, params, state));
            }
            Self::Modify {
                key_selector,
                mutate,
            } => {
                let key = key_selector.get(keys);
                let write_guard = lock_guards
                    .write
                    .get_mut(key.shard_index.0)
                    .expect(MISSING_LOCK_GUARD_ERROR);
                if let Some(mut_entry) =
                    write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key)
                {
                    (mutate)(&mut_entry.0, &mut mut_entry.1, params, state)
                }
            }
            Self::MoveValue {
                key_selector_from,
                key_selector_to,
            } => {
                let key_from = key_selector_from.get(keys);
                let key_to = key_selector_to.get(keys);
                let removed = lock_guards.remove_entry(key_from);
                if let Some(entry) = removed {
                    lock_guards.insert(key_to, entry.1);
                } else {
                    lock_guards.remove_entry(key_to);
                }
            }
            Self::Remove {
                key_selector,
                on_remove,
            } => {
                let key = key_selector.get(keys);
                let removed_entry = lock_guards.remove_entry(key);
                (on_remove)(removed_entry, params, state)
            }
            Self::RemoveIf {
                key_selector,
                condition,
            } => {
                let key = key_selector.get(keys);
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_key, value)| value);
                    if let Some(v) = value_ref
                        && (condition)(&key.key, v, params, state)
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
                        && (condition)(&key.key, v, params, state)
                    {
                        lock_guards.remove_entry(key);
                    }
                }
            }
            Self::SwapValue {
                key_selector_a,
                key_selector_b,
            } => {
                let key_a = key_selector_a.get(keys);
                let key_b = key_selector_b.get(keys);
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
            Self::Update {
                key_selector,
                transform,
            } => {
                let key = key_selector.get(keys);
                if (key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
                    let value_ref = lock_guards
                        .write
                        .get(key.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .find(key.hash_code.0, |entry| entry.0 == key.key)
                        .map(|(_key, value)| value);
                    let new_value = (transform)(&key.key, value_ref, params, state);
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
                    let new_value = (transform)(&key.key, value_ref, params, state);
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
