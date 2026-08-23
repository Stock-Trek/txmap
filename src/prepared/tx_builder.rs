use crate::{
    custodian::Custodian,
    indexer::Indexer,
    key::TxKey,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{
        guard::Guard, op::PreparedOp, schema::TxKeySelector, schema::TxKeys, schema::TxSchema,
        transaction::PreparedTransaction,
    },
    result::TxResult,
};
use hashbrown::HashSet;
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
pub struct PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PHASE = PreparedBuilderPhase>
where
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    SCHEMA: TxSchema<Key = K> + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    #[allow(clippy::type_complexity)]
    pub(crate) guards: Vec<Guard<'tx, K, V, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<PreparedOp<'tx, K, V, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, S, SCHEMA> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuilderPhase>
where
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    SCHEMA: TxSchema<Key = K> + 'tx,
{
    /// Adds a guard precondition using a key selector.
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        condition: impl Fn(&K, Option<&V>, &SCHEMA::Params, &mut SCHEMA::State) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuilderPhase> {
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

impl<'tx, K, V, L, S, SCHEMA, PHASE> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PHASE>
where
    K: 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    SCHEMA: TxSchema<Key = K> + 'tx,
{
    /// Reads a value and passes it (or `None`) to the callback.
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        get: impl Fn(&K, Option<&V>, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        value: V,
        get: impl Fn(&K, &V, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase>
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&K, &SCHEMA::Params, &mut SCHEMA::State) -> V + 'tx,
        get: impl Fn(&K, &V, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&K, &SCHEMA::Params, &mut SCHEMA::State) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&K, &SCHEMA::Params, &mut SCHEMA::State) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        mutate: impl Fn(&K, &mut V, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector_from: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        condition: impl Fn(&K, &V, &SCHEMA::Params, &mut SCHEMA::State) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector_a: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<K>, SCHEMA::IndexedKeys> + 'tx,
        transform: impl Fn(&K, Option<&V>, &SCHEMA::Params, &mut SCHEMA::State) -> Option<V> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase> {
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

impl<'tx, K, V, L, S, SCHEMA> PreparedTxBuilder<'tx, K, V, L, S, SCHEMA, PreparedBuildablePhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    SCHEMA: TxSchema<Key = K> + 'tx,
    SCHEMA::Keys: TxKeys<SCHEMA::Key, SCHEMA::IndexedKeys, S> + 'tx,
{
    #[must_use]
    /// Consumes the builder and returns a [`PreparedTransaction`].
    ///
    /// The transaction is erased to a single [`TxSchema`] generic, so all
    /// other types (value, lock policy, hasher, keys, params, state) are
    /// captured internally and do not appear in the returned type.
    pub fn into_transaction(self) -> PreparedTransaction<'tx, SCHEMA> {
        // Ops apply in order, so a consuming op may move its key out of the
        // per-execution keys container instead of cloning it only when no
        // later op references the same key handle. Iterating backwards and
        // keeping the handles already seen by later ops, the final op to see
        // a key always takes it; earlier uses fall back to cloning.
        let mut taken: HashSet<&'static str> = HashSet::new();
        let mut ops = self.ops;
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
        let guards = self.guards;
        let custodian = self.custodian;
        let indexer = self.indexer;
        let shard_count = custodian.shard_count;
        let exec = move |keys: SCHEMA::Keys, params: SCHEMA::Params| -> TxResult<SCHEMA::State> {
            let mut keys = keys.into_indexed(shard_count, indexer);
            let mut total_read_bitmask = BitMask::ZERO;
            let mut total_write_bitmask = BitMask::ZERO;

            // get all bitmasks
            for guard in guards.iter() {
                total_read_bitmask |= guard.read_bitmask(&keys);
            }
            for op in ops.iter() {
                let (read_bitmask, write_bitmask) = op.read_write_bitmasks(&keys);
                total_read_bitmask |= read_bitmask;
                total_write_bitmask |= write_bitmask;
            }
            // ensure locks are either read or write, not both
            total_read_bitmask &= !total_write_bitmask;

            let mut lock_guards = custodian.lock_guards(total_read_bitmask, total_write_bitmask);
            let mut state = SCHEMA::State::default();
            for (i, guard) in guards.iter().enumerate() {
                if !guard.is_condition_met::<L>(&mut lock_guards, &keys, &params, &mut state) {
                    return TxResult::RequirementNotMet {
                        index: i,
                        requirement: guard.name.clone(),
                        state,
                    };
                }
            }
            for op in ops.iter() {
                op.apply::<L, S>(&mut lock_guards, &mut keys, &params, indexer, &mut state);
            }
            TxResult::Completed { state }
        };
        PreparedTransaction {
            exec: Box::new(exec),
        }
    }
}
