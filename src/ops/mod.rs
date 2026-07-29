use crate::{
    key::TxKey, lock_guards::LockGuards, lock_policies::lock_policy::LockPolicy,
    new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

/// Zero-sized struct containing the core logic for the Get operation.
pub(crate) struct GetOpFn;
impl GetOpFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (key.shard_index.bitmask(), BitMask::ZERO)
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        get: &impl Fn(&K, Option<&V>, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
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
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        get: &impl Fn(&K, Option<&V>, &PARAMS, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
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
}

/// Zero-sized struct containing the core logic for the InsertDefault operation.
pub(crate) struct InsertDefaultFn;
impl InsertDefaultFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L>(key: &TxKey<K>, lock_guards: &mut LockGuards<'a, K, V, L>)
    where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        lock_guards.insert(key, V::default());
    }
}

/// Zero-sized struct containing the core logic for the InsertDefaultIfAbsent operation.
pub(crate) struct InsertDefaultIfAbsentFn;
impl InsertDefaultIfAbsentFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L>(key: &TxKey<K>, lock_guards: &mut LockGuards<'a, K, V, L>)
    where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || V::default());
    }
}

/// Zero-sized struct containing the core logic for the InsertWith operation.
pub(crate) struct InsertWithFn;
impl InsertWithFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &impl Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let new_value = (value_generator)(&key.key, state);
        lock_guards.insert(key, new_value);
    }
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        value_generator: &impl Fn(&K, &PARAMS, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let new_value = (value_generator)(&key.key, params, state);
        lock_guards.insert(key, new_value);
    }
}

/// Zero-sized struct containing the core logic for the InsertWithIfAbsent operation.
pub(crate) struct InsertWithIfAbsentFn;
impl InsertWithIfAbsentFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        value_generator: &impl Fn(&K, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, state));
    }
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        value_generator: &impl Fn(&K, &PARAMS, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, params, state));
    }
}

/// Zero-sized struct containing the core logic for the Remove operation.
pub(crate) struct RemoveFn;
impl RemoveFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        on_remove: &impl Fn(Option<(K, V)>, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let removed_entry = lock_guards.remove_entry(key);
        (on_remove)(removed_entry, state)
    }
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        on_remove: &impl Fn(Option<(K, V)>, &PARAMS, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let removed_entry = lock_guards.remove_entry(key);
        (on_remove)(removed_entry, params, state)
    }
}

/// Zero-sized struct containing the core logic for the RemoveIf operation.
pub(crate) struct RemoveIfFn;
impl RemoveIfFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        condition: &impl Fn(&K, &V, &mut STATE) -> bool,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
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
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        condition: &impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
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
}

/// Zero-sized struct containing the core logic for the Update operation.
pub(crate) struct UpdateFn;
impl UpdateFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        transform: &impl Fn(&K, Option<&V>, &mut STATE) -> Option<V>,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
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
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        transform: &impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V>,
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
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
}

/// Zero-sized struct containing the core logic for the Modify operation.
pub(crate) struct ModifyFn;
impl ModifyFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(key: &TxKey<K>) -> (BitMask, BitMask) {
        (BitMask::ZERO, key.shard_index.bitmask())
    }
    pub(crate) fn apply<'a, K, V, L, STATE>(
        key: &TxKey<K>,
        mutate: &impl Fn(&K, &mut V, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = lock_guards
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(mut_entry) = write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            (mutate)(&mut_entry.0, &mut mut_entry.1, state)
        }
    }
    pub(crate) fn apply_prepared<'a, K, V, L, KEYS, PARAMS, STATE>(
        key: &TxKey<K>,
        mutate: &impl Fn(&K, &mut V, &PARAMS, &mut STATE),
        lock_guards: &mut LockGuards<'a, K, V, L>,
        _keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = lock_guards
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(mut_entry) = write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            (mutate)(&mut_entry.0, &mut mut_entry.1, params, state)
        }
    }
}

/// Zero-sized struct containing the core logic for the MoveValue operation.
pub(crate) struct MoveValueFn;
impl MoveValueFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
    ) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            key_from.shard_index.bitmask() | key_to.shard_index.bitmask(),
        )
    }
    pub(crate) fn apply<'a, K, V, L>(
        key_from: &TxKey<K>,
        key_to: &TxKey<K>,
        lock_guards: &mut LockGuards<'a, K, V, L>,
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

/// Zero-sized struct containing the core logic for the SwapValue operation.
pub(crate) struct SwapValueFn;
impl SwapValueFn {
    pub(crate) fn read_write_bitmasks<K: Hash + Eq>(
        key_a: &TxKey<K>,
        key_b: &TxKey<K>,
    ) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            key_a.shard_index.bitmask() | key_b.shard_index.bitmask(),
        )
    }
    pub(crate) fn apply<'a, K, V, L>(
        key_a: &TxKey<K>,
        key_b: &TxKey<K>,
        lock_guards: &mut LockGuards<'a, K, V, L>,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
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
}
