use crate::{
    custodian::Custodian,
    indexer::Indexer,
    key::TxKey,
    lock_policies::lock_policy::LockPolicy,
    prepared::{
        guard::Guard, op::PreparedOp, schema::TxKeySelector, transaction::PreparedTransaction,
    },
};
use std::{
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

/// Phase marker: transaction is still accepting guard requirements.
pub struct PreparedBuilderPhase;
/// Phase marker: transaction has at least one operation and can be built.
pub struct PreparedBuildablePhase;

/// Fluent builder for a prepared (re-usable) transaction.
///
/// Start with [`TxMap::prepared_tx`](crate::tx_map::TxMap::prepared_tx), add guard requirements with
/// [`require`](PreparedTxBuilder::require), add operations (e.g.
/// [`modify`](PreparedTxBuilder::modify)), then call
/// [`into_transaction`](PreparedTxBuilder::into_transaction) to
/// obtain a [`PreparedTransaction`] that can be executed repeatedly.
pub struct PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE = PreparedBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    /// Adds a guard precondition using a key selector.
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuilderPhase> {
        let guard = Guard {
            name: name.as_ref().into(),
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.guards.push(guard);
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    /// Reads a value and passes it (or `None`) to the callback.
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Get {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Ensures the key has a value, inserting `value` if the key is absent,
    /// then passes the resulting value to `get`.
    ///
    /// If the key is present, its value is passed to `get` and `value` is
    /// discarded. If the key is absent, `value` is inserted and then passed
    /// to `get`. Requires `V: Clone` because the transaction is re-usable and
    /// `value` must be duplicated for each execution that needs to insert it.
    pub fn get_or_insert(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value: V,
        get: impl Fn(&K, &V, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        V: Clone,
    {
        self.ops.push(PreparedOp::GetOrInsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(move |_k, _p, _s| value.clone()),
            get: Box::new(get),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    /// Ensures the key has a value, inserting a generated value if the key is
    /// absent, then passes the resulting value to `get`.
    ///
    /// If the key is present, its value is passed to `get` and
    /// `value_generator` is never called. If the key is absent,
    /// `value_generator` is called with the key, params and state and its
    /// result is inserted and then passed to `get`.
    pub fn get_or_insert_with(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
        get: impl Fn(&K, &V, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::GetOrInsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            get: Box::new(get),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    /// Inserts a value generated from the key and parameters.
    pub fn insert_with(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::InsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            take_key: false,
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Inserts a value only if the key is absent.
    pub fn insert_with_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::InsertWithIfAbsent {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
            take_key: false,
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Mutates an existing value in-place.
    pub fn modify(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Modify {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Moves a value from one key to another atomically.
    pub fn move_value(
        mut self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::MoveValue {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Removes a key-value pair.
    pub fn remove(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Remove {
            key_selector: Box::new(key_selector),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Removes a key if the condition is satisfied.
    pub fn remove_if(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::RemoveIf {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Swaps the values of two keys atomically.
    pub fn swap_value(
        mut self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::SwapValue {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    /// Updates or removes an entry via a transform closure.
    pub fn update(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Update {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
            take_key: false,
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    /// Consumes the builder and returns a [`PreparedTransaction`].
    pub fn into_transaction(self) -> PreparedTransaction<'tx, K, V, L, S, KEYS, PARAMS, STATE> {
        // Count how many times each key handle is referenced across guards and
        // operations. A handle used exactly once is "provably last-used", so
        // consuming operations can move the key out of the per-execution keys
        // container instead of cloning it on every execution.
        let mut usage: std::collections::HashMap<&'static str, usize> =
            std::collections::HashMap::new();
        for guard in &self.guards {
            *usage.entry(guard.key_id()).or_default() += 1;
        }
        let mut key_ids = Vec::new();
        for op in &self.ops {
            op.push_key_ids(&mut key_ids);
        }
        for id in key_ids {
            *usage.entry(id).or_default() += 1;
        }
        let mut ops = self.ops;
        for op in &mut ops {
            match op {
                PreparedOp::InsertWith { key_selector, take_key, .. }
                | PreparedOp::InsertWithIfAbsent { key_selector, take_key, .. }
                | PreparedOp::Update { key_selector, take_key, .. } => {
                    *take_key = usage.get(key_selector.key_id()) == Some(&1);
                }
                _ => {}
            }
        }
        PreparedTransaction {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops,
        }
    }
}
