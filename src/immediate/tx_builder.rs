use crate::{
    custodian::Custodian,
    immediate::{
        guard::Guard,
        ops::{
            get_op::GetOp, insert_default_if_absent_op::InsertDefaultIfAbsentOp,
            insert_default_op::InsertDefaultOp, insert_with_if_absent_op::InsertWithIfAbsentOp,
            insert_with_op::InsertWithOp, modify_op::ModifyOp, move_value_op::MoveValueOp,
            op_trait::OpTrait, remove_if_op::RemoveIfOp, remove_op::RemoveOp,
            swap_value_op::SwapValueOp, update_op::UpdateOp,
        },
        transaction::ImmediateTransaction,
    },
    indexer::Indexer,
    lock_policies::lock_policy::LockPolicy,
    result::TxResult,
};
use std::{hash::Hash, marker::PhantomData};

pub struct ImmediateBuilderPhase;
pub struct ImmediateBuildablePhase;

pub struct ImmediateTxBuilder<'tx, K, V, L, STATE, PHASE = ImmediateBuilderPhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, STATE> + 'tx>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, STATE> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuilderPhase>
where
    K: Hash + Eq + 'tx,
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
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    STATE: Default + 'tx,
{
    pub fn get(
        mut self,
        key: K,
        get: impl Fn(&K, Option<&V>, &mut STATE) + 'tx,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase> {
        let op = GetOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            get: Box::new(get),
        };
        self.ops.push(Box::new(op));
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_default(
        mut self,
        key: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
        };
        self.ops.push(Box::new(op));
        ImmediateTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_default_if_absent(
        mut self,
        key: K,
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
        };
        self.ops.push(Box::new(op));
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
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
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
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithIfAbsentOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
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
        let op = ModifyOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            mutate: Box::new(mutate),
        };
        self.ops.push(Box::new(op));
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
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
    {
        let op = MoveValueOp {
            key_from: Indexer::indexed_key(self.custodian.shard_count, key_from),
            key_to: Indexer::indexed_key(self.custodian.shard_count, key_to),
        };
        self.ops.push(Box::new(op));
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
        let op = RemoveOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            on_remove: Box::new(on_remove),
        };
        self.ops.push(Box::new(op));
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
        let op = RemoveIfOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            condition: Box::new(condition),
        };
        self.ops.push(Box::new(op));
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
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
    {
        let op = SwapValueOp {
            key_a: Indexer::indexed_key(self.custodian.shard_count, key_a),
            key_b: Indexer::indexed_key(self.custodian.shard_count, key_b),
        };
        self.ops.push(Box::new(op));
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
    ) -> ImmediateTxBuilder<'tx, K, V, L, STATE, ImmediateBuildablePhase>
    where
        K: Clone,
    {
        let op = UpdateOp {
            key: Indexer::indexed_key(self.custodian.shard_count, key),
            transform: Box::new(transform),
        };
        self.ops.push(Box::new(op));
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
    K: Hash + Eq + 'tx,
    V: 'tx,
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
