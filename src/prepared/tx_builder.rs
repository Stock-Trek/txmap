use crate::{
    custodian::Custodian,
    indexer::Indexer,
    key::TxKey,
    prepared::{
        guard::Guard, op::PreparedOp, schema::TxKeySelector, schema::TxSchema,
        transaction::PreparedTransaction,
    },
};
use hashbrown::HashSet;
use std::{hash::Hash, marker::PhantomData};

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
///
/// The only generic parameter is the transaction's [`TxSchema`]; all other
/// types (key, value, keys, params, state, lock policy, hasher) are
/// associated types of the schema.
pub struct PreparedTxBuilder<'tx, SCHEMA, PHASE = PreparedBuilderPhase>
where
    SCHEMA: TxSchema + 'tx,
{
    pub(crate) custodian: &'tx Custodian<SCHEMA::Key, SCHEMA::Value, SCHEMA::LockPolicy>,
    pub(crate) indexer: &'tx Indexer<SCHEMA::Hasher>,
    #[allow(clippy::type_complexity)]
    pub(crate) guards: Vec<
        Guard<'tx, SCHEMA::Key, SCHEMA::Value, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>,
    >,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<
        PreparedOp<
            'tx,
            SCHEMA::Key,
            SCHEMA::Value,
            SCHEMA::IndexedKeys,
            SCHEMA::Params,
            SCHEMA::State,
        >,
    >,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, SCHEMA> PreparedTxBuilder<'tx, SCHEMA, PreparedBuilderPhase>
where
    SCHEMA: TxSchema + 'tx,
{
    /// Adds a guard precondition using a key selector.
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        condition: impl Fn(
            &SCHEMA::Key,
            Option<&SCHEMA::Value>,
            &SCHEMA::Params,
            &mut SCHEMA::State,
        ) -> bool
        + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuilderPhase> {
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

impl<'tx, SCHEMA, PHASE> PreparedTxBuilder<'tx, SCHEMA, PHASE>
where
    SCHEMA: TxSchema + 'tx,
{
    /// Reads a value and passes it (or `None`) to the callback.
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        get: impl Fn(&SCHEMA::Key, Option<&SCHEMA::Value>, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        value: SCHEMA::Value,
        get: impl Fn(&SCHEMA::Key, &SCHEMA::Value, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase>
    where
        SCHEMA::Value: Clone,
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&SCHEMA::Key, &SCHEMA::Params, &mut SCHEMA::State) -> SCHEMA::Value
        + 'tx,
        get: impl Fn(&SCHEMA::Key, &SCHEMA::Value, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&SCHEMA::Key, &SCHEMA::Params, &mut SCHEMA::State) -> SCHEMA::Value
        + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        value_generator: impl Fn(&SCHEMA::Key, &SCHEMA::Params, &mut SCHEMA::State) -> SCHEMA::Value
        + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        mutate: impl Fn(&SCHEMA::Key, &mut SCHEMA::Value, &SCHEMA::Params, &mut SCHEMA::State) + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector_from: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        condition: impl Fn(&SCHEMA::Key, &SCHEMA::Value, &SCHEMA::Params, &mut SCHEMA::State) -> bool
        + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector_a: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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
        key_selector: impl TxKeySelector<TxKey<SCHEMA::Key>, SCHEMA::IndexedKeys> + 'tx,
        transform: impl Fn(
            &SCHEMA::Key,
            Option<&SCHEMA::Value>,
            &SCHEMA::Params,
            &mut SCHEMA::State,
        ) -> Option<SCHEMA::Value>
        + 'tx,
    ) -> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase> {
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

impl<'tx, SCHEMA> PreparedTxBuilder<'tx, SCHEMA, PreparedBuildablePhase>
where
    SCHEMA: TxSchema + 'tx,
    SCHEMA::Key: Clone + Hash + Eq + 'tx,
{
    #[must_use]
    /// Consumes the builder and returns a [`PreparedTransaction`].
    ///
    /// The transaction is erased to a single [`TxSchema`] generic, so all
    /// other types (value, lock policy, hasher, keys, params, state) are
    /// associated types of the schema and do not appear in the returned
    /// type.
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
        PreparedTransaction {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops,
        }
    }
}
