use crate::{
    custodian::Custodian,
    immediate::{guard::Guard, op::ImmediateOp, transaction::ImmediateTransaction},
    indexer::Indexer,
    lock_policies::lock_policy::LockPolicy,
    result::TxResult,
};
use std::{
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

/// Phase marker: transaction is still accepting guard requirements.
pub struct ImmediateBuilderPhase;
/// Phase marker: transaction has at least one operation and can be executed.
pub struct ImmediateBuildablePhase;

/// Fluent builder for an immediate transaction.
///
/// Start with [`TxMap::immediate_tx`](crate::tx_map::TxMap::immediate_tx), add guard requirements with
/// [`require`](ImmediateTxBuilder::require), add operations (e.g.
/// [`modify`](ImmediateTxBuilder::modify)), then call
/// [`execute`](ImmediateTxBuilder::execute).
pub struct ImmediateTxBuilder<'tx, K, V, L, S, STATE, PHASE = ImmediateBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<Guard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<ImmediateOp<'tx, K, V, STATE>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, S, STATE> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    STATE: Default + 'tx,
{
    /// Adds a guard precondition.
    ///
    /// The transaction will only execute if all guard conditions are
    /// met. Guards must be added before any operations. If a guard
    /// fails, [`TxResult::RequirementNotMet`] is returned with the
    /// guard index, name, and current state.
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key: K,
        condition: impl Fn(&K, Option<&V>, &mut STATE) -> bool + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuilderPhase> {
        let guard = Guard {
            name: name.as_ref().into(),
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.guards.push(guard);
        ImmediateTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, STATE, PHASE> ImmediateTxBuilder<'tx, K, V, L, S, STATE, PHASE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    STATE: Default + 'tx,
{
    /// Reads a value and passes it (or `None`) to the callback.
    ///
    /// Acquires either a read or write lock depending on whether the
    /// shard is already write-locked by a previous operation.
    pub fn get(
        mut self,
        key: K,
        get: impl Fn(&K, Option<&V>, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Get {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            get: Box::new(get),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }

    /// Inserts a value generated from the key.
    pub fn insert_with(
        mut self,
        key: K,
        value_generator: impl Fn(&K, &mut STATE) -> V + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::InsertWith {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        });
        ImmediateTxBuilder {
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
        key: K,
        value_generator: impl Fn(&K, &mut STATE) -> V + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::InsertWithIfAbsent {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        });
        ImmediateTxBuilder {
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
        key: K,
        mutate: impl Fn(&K, &mut V, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Modify {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            mutate: Box::new(mutate),
        });
        ImmediateTxBuilder {
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
        key_from: K,
        key_to: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::MoveValue {
            key_from: self
                .indexer
                .indexed_key(self.custodian.shard_count, key_from),
            key_to: self.indexer.indexed_key(self.custodian.shard_count, key_to),
        });
        ImmediateTxBuilder {
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
        key: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Remove {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
        });
        ImmediateTxBuilder {
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
        key: K,
        condition: impl Fn(&K, &V, &mut STATE) -> bool + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::RemoveIf {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            condition: Box::new(condition),
        });
        ImmediateTxBuilder {
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
        key_a: K,
        key_b: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::SwapValue {
            key_a: self.indexer.indexed_key(self.custodian.shard_count, key_a),
            key_b: self.indexer.indexed_key(self.custodian.shard_count, key_b),
        });
        ImmediateTxBuilder {
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
        key: K,
        transform: impl Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Update {
            key: self.indexer.indexed_key(self.custodian.shard_count, key),
            transform: Box::new(transform),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            indexer: self.indexer,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, STATE> ImmediateTxBuilder<'tx, K, V, L, S, STATE, ImmediateBuildablePhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    /// Executes the transaction.
    ///
    /// Acquires locks, checks guards, applies all operations, and
    /// returns the final state wrapped in [`TxResult`].
    pub fn execute(self) -> TxResult<STATE> {
        let Self {
            custodian,
            indexer,
            guards,
            ops,
            _phase,
        } = self;
        ImmediateTransaction {
            custodian,
            indexer,
            guards,
            ops,
        }
        .execute()
    }
}
