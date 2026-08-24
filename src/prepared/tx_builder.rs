use crate::{
    key::TxKey,
    lock_policies::lock_policy::LockPolicy,
    prepared::{guard::Guard, op::PreparedOp, schema::TxKeySelector},
};
use hashbrown::HashSet;
use std::{hash::BuildHasher, marker::PhantomData};

/// Fluent builder for a prepared (re-usable) transaction, still accepting
/// guard requirements.
///
/// Start with [`TxMap::prepared_tx`](crate::tx_map::TxMap::prepared_tx), add guard
/// requirements with [`require`](PreparedTxRequirementsBuilder::require), add
/// operations (e.g. [`modify`](PreparedTxOperationsBuilder::modify)), then call
/// [`into_transaction`](PreparedTxBuilder::into_transaction) to obtain a
/// transaction that can be executed repeatedly.
///
/// Implemented by the `Builder` struct generated for a schema by the
/// [`tx_schema`](macro@crate::tx_schema) macro.
pub trait PreparedTxRequirementsBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>
where
    Self: Sized,
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: 'tx,
{
    type Builder: PreparedTxRequirementsBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>
        + PreparedTxOperationsBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>;

    fn with_guard(self, guard: Guard<'tx, K, V, KEYS, PARAMS, STATE>) -> Self::Builder;

    /// Adds a guard precondition using a key selector.
    fn require(
        self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> Self::Builder {
        let guard = Guard {
            name: name.as_ref().into(),
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.with_guard(guard)
    }
}

/// Fluent builder for a prepared (re-usable) transaction, adding operations.
///
/// Once at least one operation has been added, the builder is also buildable
/// via [`into_transaction`](PreparedTxBuilder::into_transaction).
///
/// Implemented by the `Builder` and `Buildable` structs generated for a
/// schema by the [`tx_schema`](macro@crate::tx_schema) macro.
pub trait PreparedTxOperationsBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>
where
    Self: Sized,
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: 'tx,
{
    type Builder: PreparedTxOperationsBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>
        + PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>;

    fn with_operation(self, op: PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>) -> Self::Builder;

    /// Reads a value and passes it (or `None`) to the callback.
    fn get(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::Get {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        })
    }
    /// Ensures the key has a value, inserting `value` if the key is absent,
    /// then passes the resulting value to `get`.
    ///
    /// If the key is present, its value is passed to `get` and `value` is
    /// discarded. If the key is absent, `value` is inserted and then passed
    /// to `get`. Requires `V: Clone` because the transaction is re-usable and
    /// `value` must be duplicated for each execution that needs to insert it.
    fn get_or_insert(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value: V,
        get: impl Fn(&K, &V, &PARAMS, &mut STATE) + 'tx,
    ) -> Self::Builder
    where
        V: Clone,
    {
        self.with_operation(PreparedOp::GetOrInsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(move |_k, _p, _s| value.clone()),
            get: Box::new(get),
        })
    }

    /// Ensures the key has a value, inserting a generated value if the key is
    /// absent, then passes the resulting value to `get`.
    ///
    /// If the key is present, its value is passed to `get` and
    /// `value_generator` is never called. If the key is absent,
    /// `value_generator` is called with the key, params and state and its
    /// result is inserted and then passed to `get`.
    fn get_or_insert_with(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
        get: impl Fn(&K, &V, &PARAMS, &mut STATE) + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::GetOrInsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            get: Box::new(get),
        })
    }

    /// Inserts a value generated from the key and parameters.
    fn insert_with(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::InsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            take_key: false,
        })
    }
    /// Inserts a value only if the key is absent.
    fn insert_with_if_absent(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::InsertWithIfAbsent {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            take_key: false,
        })
    }
    /// Mutates an existing value in-place.
    fn modify(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::Modify {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        })
    }
    /// Moves a value from one key to another atomically.
    fn move_value(
        self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::MoveValue {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        })
    }
    /// Removes a key-value pair.
    fn remove(self, key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx) -> Self::Builder {
        self.with_operation(PreparedOp::Remove {
            key_selector: Box::new(key_selector),
        })
    }
    /// Removes a key if the condition is satisfied.
    fn remove_if(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::RemoveIf {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        })
    }
    /// Swaps the values of two keys atomically.
    fn swap_value(
        self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::SwapValue {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        })
    }
    /// Updates or removes an entry via a transform closure.
    fn update(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> Self::Builder {
        self.with_operation(PreparedOp::Update {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
            take_key: false,
        })
    }
}

/// A prepared transaction builder that has accumulated at least one operation
/// and can be turned into a transaction with
/// [`into_transaction`](PreparedTxBuilder::into_transaction).
///
/// Implemented by the `Buildable` struct generated for a schema by the
/// [`tx_schema`](macro@crate::tx_schema) macro.
pub trait PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE>
where
    Self: Sized,
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: 'tx,
{
    type Tx;

    fn guards(&self) -> Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>;
    fn ops(&self) -> Vec<PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>>;
    fn tx(
        self,
        guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
        ops: Vec<PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>>,
    ) -> Self::Tx;
    #[must_use]
    /// Consumes the builder and returns the schema's transaction type.
    fn into_transaction(self) -> Self::Tx {
        // Ops apply in order, so a consuming op may move its key out of the
        // per-execution keys container instead of cloning it only when no
        // later op references the same key handle. Iterating backwards and
        // keeping the handles already seen by later ops, the final op to see
        // a key always takes it; earlier uses fall back to cloning.
        let mut taken: HashSet<&'static str> = HashSet::new();
        let mut ops = self.ops();
        for op in ops.iter_mut().rev() {
            match op {
                PreparedOp::InsertWith {
                    key_selector,
                    take_key,
                    ..
                }
                | PreparedOp::InsertWithIfAbsent {
                    key_selector,
                    take_key,
                    ..
                }
                | PreparedOp::Update {
                    key_selector,
                    take_key,
                    ..
                } => {
                    *take_key = taken.insert(key_selector.key_id());
                }
                _ => {}
            }
            op.insert_key_ids(&mut taken);
        }
        let guards = self.guards();
        self.tx(guards, ops)
    }
}
