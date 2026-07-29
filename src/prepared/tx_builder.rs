use crate::{
    custodian::Custodian,
    key::TxKey,
    lock_policies::lock_policy::LockPolicy,
    prepared::{
        guard::Guard,
        ops::{
            get_op::GetOp, insert_default_if_absent_op::InsertDefaultIfAbsentOp,
            insert_default_op::InsertDefaultOp, insert_with_if_absent_op::InsertWithIfAbsentOp,
            insert_with_op::InsertWithOp, modify_op::ModifyOp, move_value_op::MoveValueOp,
            op_trait::OpTrait, remove_if_op::RemoveIfOp, remove_op::RemoveOp,
            swap_value_op::SwapValueOp, update_op::UpdateOp,
        },
        schema::TxKeySelector,
        transaction::PreparedTransaction,
    },
};
use std::{hash::BuildHasher, hash::Hash, marker::PhantomData};

pub struct PreparedBuilderPhase;
pub struct PreparedBuildablePhase;

pub struct PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE = PreparedBuilderPhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L, S>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, S, KEYS, PARAMS, STATE> + 'tx>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuilderPhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
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
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PHASE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        let op = GetOp {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_default(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_default_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_with(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn insert_with_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
    {
        let op = InsertWithIfAbsentOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn modify(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        let op = ModifyOp {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn move_value(
        mut self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
    {
        let op = MoveValueOp {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn remove(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        on_remove: impl Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        let op = RemoveOp {
            key_selector: Box::new(key_selector),
            on_remove: Box::new(on_remove),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn remove_if(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        let op = RemoveIfOp {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn swap_value(
        mut self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
    {
        let op = SwapValueOp {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
    pub fn update(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
    where
        K: Clone,
    {
        let op = UpdateOp {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
        };
        self.ops.push(Box::new(op));
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuildablePhase>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    pub fn into_transaction(self) -> PreparedTransaction<'tx, K, V, L, S, KEYS, PARAMS, STATE> {
        PreparedTransaction {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
        }
    }
}
