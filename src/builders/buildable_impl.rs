use crate::{
    builders::{
        builder_traits::{IntoTransaction, TxBuildable, TxOpBuilder, TxResultBuilder},
        finishable_impl::TxFinishableImpl,
    },
    custodian::Custodian,
    finisher::Finisher,
    finishers::{
        none_finisher::NoneFinisher, value_finisher::ValueFinisher, values_finisher::ValuesFinisher,
    },
    guard::Guard,
    indexer::Indexer,
    ops::{
        clear_op::ClearOp, insert_default_if_absent_op::InsertDefaultIfAbsentOp,
        insert_default_op::InsertDefaultOp, insert_with_if_absent_op::InsertWithIfAbsentOp,
        insert_with_op::InsertWithOp, modify_op::ModifyOp, modify_peek_op::ModifyPeekOp,
        move_value_op::MoveValueOp, op_trait::OpTrait, remove_if_op::RemoveIfOp,
        remove_op::RemoveOp, remove_where_op::RemoveWhereOp, retain_only_op::RetainOnlyOp,
        retain_op::RetainOp, retain_where_op::RetainWhereOp, swap_value_op::SwapValueOp,
        update_op::UpdateOp, update_peek_op::UpdatePeekOp,
    },
    transaction::{Transaction, TransactionBase},
};
use std::hash::Hash;

pub struct TxBuildableImpl<'txmap, K, V> {
    pub(crate) indexer: Indexer,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, ()>>>,
}

impl<'txmap, K, V> TxBuildable<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq + 'static,
    V: 'static,
{
}

impl<'txmap, K, V> TxOpBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq + 'static,
    V: 'static,
{
    // single key ops
    fn insert_default(mut self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp::new(&self.indexer, key);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_default_if_absent(mut self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp::new(&self.indexer, key);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with<G>(mut self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let op = InsertWithOp::new(&self.indexer, key, value_generator);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with_if_absent<G>(
        mut self,
        key: K,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let op = InsertWithIfAbsentOp::new(&self.indexer, key, value_generator);
        self.ops.push(Box::new(op));
        self
    }
    fn modify<M>(mut self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let op = ModifyOp::new(&self.indexer, key, mutate);
        self.ops.push(Box::new(op));
        self
    }
    fn modify_peek<const N: usize, M>(
        mut self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        K: Clone,
    {
        let op = ModifyPeekOp::new(&self.indexer, key, peek_keys, mutate);
        self.ops.push(Box::new(op));
        self
    }
    fn update<T>(mut self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
        K: Clone,
    {
        let op = UpdateOp::new(&self.indexer, key, transform);
        self.ops.push(Box::new(op));
        self
    }
    fn update_peek<const N: usize, T>(
        mut self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
        K: Clone,
    {
        let op = UpdatePeekOp::new(&self.indexer, key, peek_keys, transform);
        self.ops.push(Box::new(op));
        self
    }

    // multi key ops
    fn move_value(mut self, from: K, to: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
    {
        let op = MoveValueOp::new(&self.indexer, from, to);
        self.ops.push(Box::new(op));
        self
    }
    fn swap_value(mut self, a: K, b: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
    {
        let op = SwapValueOp::new(&self.indexer, a, b);
        self.ops.push(Box::new(op));
        self
    }

    // batch ops
    fn remove<I>(mut self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let op = RemoveOp::new(&self.indexer, keys);
        self.ops.push(Box::new(op));
        self
    }
    fn remove_where<I, C>(mut self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RemoveWhereOp::new(&self.indexer, keys, condition);
        self.ops.push(Box::new(op));
        self
    }
    fn retain_only<I>(mut self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let op = RetainOnlyOp::new(&self.indexer, keys);
        self.ops.push(Box::new(op));
        self
    }
    fn retain_where<I, C>(mut self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RetainWhereOp::new(&self.indexer, keys, condition);
        self.ops.push(Box::new(op));
        self
    }

    // global ops
    fn clear(mut self) -> impl TxBuildable<'txmap, K, V> {
        let op = ClearOp::new(&self.indexer);
        self.ops.push(Box::new(op));
        self
    }
    fn remove_if<C>(mut self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RemoveIfOp::new(&self.indexer, condition);
        self.ops.push(Box::new(op));
        self
    }
    fn retain<C>(mut self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RetainOp::new(&self.indexer, condition);
        self.ops.push(Box::new(op));
        self
    }
}

impl<'txmap, K, V> TxResultBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn get<T, R>(
        self,
        key: K,
        transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, ValueFinisher<K, V, R>>
    where
        T: Fn(&K, &V) -> R + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
            ops,
            ..
        } = self;
        let value_finisher = ValueFinisher::new(indexer, key, transform);
        let finisher = Finisher::new(value_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_all<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, ValuesFinisher<K, V, R>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static,
    {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let values_finisher = ValuesFinisher::new(self.indexer, keys, transform);
        let finisher = Finisher::new(values_finisher);
        TxFinishableImpl {
            custodian,
            guards,
            ops,
            finisher,
        }
    }
}

impl<'txmap, K, V> IntoTransaction<'txmap, K, V, NoneFinisher> for TxBuildableImpl<'txmap, K, V> {
    fn into_transaction(self) -> Transaction<'txmap, K, V, NoneFinisher> {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let mut guards_bitmask: u128 = 0;
        for guard in &guards {
            guards_bitmask |= guard.guards_bitmask;
        }
        for op in &ops {
            guards_bitmask |= op.guards_bitmask();
        }
        let base = TransactionBase {
            custodian,
            guards_bitmask,
            guards,
            ops,
            finisher: Finisher::new(NoneFinisher),
        };
        Transaction { base }
    }
}
