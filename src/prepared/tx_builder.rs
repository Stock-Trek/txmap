use crate::{
    custodian::Custodian,
    key::TxKey,
    lock_policies::lock_policy::LockPolicy,
    prepared::{
        guard::Guard, op::PreparedOp, schema::TxKeySelector, transaction::PreparedTransaction,
    },
};
use std::{hash::Hash, marker::PhantomData};

pub struct PreparedBuilderPhase;
pub struct PreparedBuildablePhase;

pub struct PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE = PreparedBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>>,
    pub(crate) _phase: PhantomData<PHASE>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuilderPhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuilderPhase> {
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

impl<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE>
    PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PHASE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Get {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::InsertWith {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::InsertWithIfAbsent {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Modify {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::MoveValue {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Remove {
            key_selector: Box::new(key_selector),
            on_remove: Box::new(on_remove),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::RemoveIf {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::SwapValue {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        });
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
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase> {
        self.ops.push(PreparedOp::Update {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
        });
        PreparedTxBuilder {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
            _phase: PhantomData,
        }
    }
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE>
    PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuildablePhase>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    pub fn into_transaction(self) -> PreparedTransaction<'tx, K, V, L, KEYS, PARAMS, STATE> {
        PreparedTransaction {
            custodian: self.custodian,
            guards: self.guards,
            ops: self.ops,
        }
    }
}
