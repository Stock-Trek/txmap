use crate::{
    builders::builder_traits::{IntoTransaction, TxBuildable, TxOpBuilder},
    custodian::Custodian,
    guard::Guard,
    lock_policies::lock_policy::LockPolicy,
    ops::{
        get_op::GetOp, insert_default_if_absent_op::InsertDefaultIfAbsentOp,
        insert_default_op::InsertDefaultOp, insert_with_if_absent_op::InsertWithIfAbsentOp,
        insert_with_op::InsertWithOp, modify_op::ModifyOp, move_value_op::MoveValueOp,
        op_trait::OpTrait, remove_op::RemoveOp, remove_where_op::RemoveWhereOp,
        swap_value_op::SwapValueOp, update_op::UpdateOp,
    },
    params::{TxKey, TxKeySelector},
    transaction::Transaction,
};
use std::hash::Hash;

pub struct TxBuildableImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, L, KEYS, PARAMS, STATE> + 'tx>>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuildableImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxOpBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuildableImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn get(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        get: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let op = GetOp {
            key_selector: Box::new(key_selector),
            get: Box::new(get),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn insert_default(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn insert_default_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp {
            key_selector: Box::new(key_selector),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let op = InsertWithOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with_if_absent(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let op = InsertWithIfAbsentOp {
            key_selector: Box::new(key_selector),
            value_generator: Box::new(value_generator),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn modify(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let op = ModifyOp {
            key_selector: Box::new(key_selector),
            mutate: Box::new(mutate),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn move_value(
        mut self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let op = MoveValueOp {
            key_selector_from: Box::new(key_selector_from),
            key_selector_to: Box::new(key_selector_to),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn remove(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        on_remove: impl Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let op = RemoveOp {
            key_selector: Box::new(key_selector),
            on_remove: Box::new(on_remove),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn remove_where(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let op = RemoveWhereOp {
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn swap_value(
        mut self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let op = SwapValueOp {
            key_selector_a: Box::new(key_selector_a),
            key_selector_b: Box::new(key_selector_b),
        };
        self.ops.push(Box::new(op));
        self
    }
    fn update(
        mut self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let op = UpdateOp {
            key_selector: Box::new(key_selector),
            transform: Box::new(transform),
        };
        self.ops.push(Box::new(op));
        self
    }
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> IntoTransaction<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuildableImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn into_transaction(self) -> Transaction<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        Transaction {
            custodian,
            guards,
            ops,
        }
    }
}
