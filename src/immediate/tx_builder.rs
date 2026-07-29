use crate::{
    custodian::Custodian,
    immediate::{guard::Guard, op::ImmediateOp, transaction::ImmediateTransaction},
    indexer::Indexer,
    lock_policies::lock_policy::LockPolicy,
    result::TxResult,
};
use std::{hash::Hash, marker::PhantomData};

pub struct ImmediateBuilderPhase;
pub struct ImmediateBuildablePhase;

pub struct ImmediateTxBuilder<'tx, K, V, L, STATE, PHASE = ImmediateBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<ImmediateOp<'tx, K, V, STATE>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, STATE> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key: K,
        condition: impl Fn(&K, Option<&V>, &mut STATE) -> bool + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuilderPhase> {
        let guard = Guard {
            name: name.as_ref().into(),
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.guards.push(guard);
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, STATE, PHASE> ImmediateTxBuilder<'tx, K, V, L, STATE, PHASE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    pub fn get(
        mut self,
        key: K,
        get: impl Fn(&K, Option<&V>, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Get {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            get: Box::new(get),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_with(
        mut self,
        key: K,
        value_generator: impl Fn(&K, &mut STATE) -> V + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::InsertWith {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_with_if_absent(
        mut self,
        key: K,
        value_generator: impl Fn(&K, &mut STATE) -> V + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::InsertWithIfAbsent {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn modify(
        mut self,
        key: K,
        mutate: impl Fn(&K, &mut V, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Modify {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            mutate: Box::new(mutate),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn move_value(
        mut self,
        key_from: K,
        key_to: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::MoveValue {
            key_from: Indexer::indexed_key(self.custodian.shard_count, key_from),
            key_to: Indexer::indexed_key(self.custodian.shard_count, key_to),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn remove(
        mut self,
        key: K,
        on_remove: impl Fn(Option<(K, V)>, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Remove {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            on_remove: Box::new(on_remove),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn remove_if(
        mut self,
        key: K,
        condition: impl Fn(&K, &V, &mut STATE) -> bool + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::RemoveIf {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            condition: Box::new(condition),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn swap_value(
        mut self,
        key_a: K,
        key_b: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::SwapValue {
            key_a: Indexer::indexed_key(self.custodian.shard_count, key_a),
            key_b: Indexer::indexed_key(self.custodian.shard_count, key_b),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn update(
        mut self,
        key: K,
        transform: impl Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        self.ops.push(ImmediateOp::Update {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            transform: Box::new(transform),
        });
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, STATE> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: Default + 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    pub fn execute(self) -> TxResult<STATE> {
        let Self {
            custodian,
            guards,
            ops,
            _phase,
        } = self;
        ImmediateTransaction {
            custodian,
            guards,
            ops,
        }
        .execute()
    }
}
