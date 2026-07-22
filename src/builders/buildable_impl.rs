use crate::{
    builders::{
        builder_traits::{IntoTransaction, TxBuildable, TxOpBuilder, TxResultBuilder},
        finishable_impl::TxFinishableImpl,
    },
    custodian::Custodian,
    finisher::Finisher,
    finishers::{
        clone_all_finisher::CloneAllFinisher, clone_finisher::CloneFinisher,
        copy_all_finisher::CopyAllFinisher, copy_finisher::CopyFinisher,
        none_finisher::NoneFinisher, value_finisher::ValueFinisher,
        values_finisher::ValuesFinisher,
    },
    guard::Guard,
    locks::lock_policy::LockPolicy,
    new_types::BitMask,
    ops::{
        insert_default_if_absent_op::InsertDefaultIfAbsentOp, insert_default_op::InsertDefaultOp,
        insert_with_if_absent_op::InsertWithIfAbsentOp, insert_with_op::InsertWithOp,
        modify_op::ModifyOp, modify_peek_op::ModifyPeekOp, move_value_op::MoveValueOp,
        op_trait::OpTrait, remove_op::RemoveOp, remove_where_op::RemoveWhereOp,
        swap_value_op::SwapValueOp, update_op::UpdateOp, update_peek_op::UpdatePeekOp,
    },
    transaction::{Transaction, TransactionBase},
};
use std::hash::Hash;

pub struct TxBuildableImpl<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    pub(crate) custodian: &'txmap Custodian<L, K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<L, K, V, ()>>>,
}

impl<'txmap, L, K, V> TxBuildable<'txmap, L, K, V> for TxBuildableImpl<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
}

impl<'txmap, L, K, V> TxOpBuilder<'txmap, L, K, V> for TxBuildableImpl<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
    // single key ops
    fn insert_default(mut self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultOp::new(self.custodian.shard_count, key);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_default_if_absent(mut self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default,
    {
        let op = InsertDefaultIfAbsentOp::new(self.custodian.shard_count, key);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with(
        mut self,
        key: K,
        value_generator: impl Fn(&K) -> V + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = InsertWithOp::new(self.custodian.shard_count, key, value_generator);
        self.ops.push(Box::new(op));
        self
    }
    fn insert_with_if_absent(
        mut self,
        key: K,
        value_generator: impl Fn(&K) -> V + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = InsertWithIfAbsentOp::new(self.custodian.shard_count, key, value_generator);
        self.ops.push(Box::new(op));
        self
    }
    fn modify(
        mut self,
        key: K,
        mutate: impl Fn(&K, &mut V) + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V> {
        let op = ModifyOp::new(self.custodian.shard_count, key, mutate);
        self.ops.push(Box::new(op));
        self
    }
    fn modify_peek<const N: usize>(
        mut self,
        key: K,
        peek_keys: [K; N],
        mutate: impl Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = ModifyPeekOp::new(self.custodian.shard_count, key, peek_keys, mutate);
        self.ops.push(Box::new(op));
        self
    }
    fn update(
        mut self,
        key: K,
        transform: impl Fn(&K, Option<&V>) -> Option<V> + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = UpdateOp::new(self.custodian.shard_count, key, transform);
        self.ops.push(Box::new(op));
        self
    }
    fn update_peek<const N: usize>(
        mut self,
        key: K,
        peek_keys: [K; N],
        transform: impl Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = UpdatePeekOp::new(self.custodian.shard_count, key, peek_keys, transform);
        self.ops.push(Box::new(op));
        self
    }

    // multi key ops
    fn move_value(mut self, from: K, to: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = MoveValueOp::new(self.custodian.shard_count, from, to);
        self.ops.push(Box::new(op));
        self
    }
    fn swap_value(mut self, a: K, b: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let op = SwapValueOp::new(self.custodian.shard_count, a, b);
        self.ops.push(Box::new(op));
        self
    }

    // batch ops
    fn remove(mut self, keys: impl IntoIterator<Item = K>) -> impl TxBuildable<'txmap, L, K, V> {
        let op = RemoveOp::new(self.custodian.shard_count, keys);
        self.ops.push(Box::new(op));
        self
    }
    fn remove_where(
        mut self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V) -> bool + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V> {
        let op = RemoveWhereOp::new(self.custodian.shard_count, keys, condition);
        self.ops.push(Box::new(op));
        self
    }
}

impl<'txmap, L, K, V> TxResultBuilder<'txmap, L, K, V> for TxBuildableImpl<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn get_copied(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CopyFinisher<K, V>>
    where
        V: Copy,
    {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let copy_finisher = CopyFinisher::new(self.custodian.shard_count, key);
        let finisher = Finisher::new(copy_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_all_copied(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoTransaction<'txmap, L, K, V, CopyAllFinisher<K, V>>
    where
        V: Copy,
    {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let copy_finisher = CopyAllFinisher::new(self.custodian.shard_count, keys);
        let finisher = Finisher::new(copy_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_cloned(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CloneFinisher<K, V>>
    where
        V: Clone,
    {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let clone_finisher = CloneFinisher::new(self.custodian.shard_count, key);
        let finisher = Finisher::new(clone_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_all_cloned(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoTransaction<'txmap, L, K, V, CloneAllFinisher<K, V>>
    where
        V: Clone,
    {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let clone_all_finisher = CloneAllFinisher::new(self.custodian.shard_count, keys);
        let finisher = Finisher::new(clone_all_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_with<R>(
        self,
        key: K,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValueFinisher<K, V, R>> {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let value_finisher = ValueFinisher::new(self.custodian.shard_count, key, transform);
        let finisher = Finisher::new(value_finisher);
        TxFinishableImpl {
            custodian,
            finisher,
            guards,
            ops,
        }
    }
    fn get_all_with<R>(
        self,
        keys: impl IntoIterator<Item = K>,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValuesFinisher<K, V, R>> {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let values_finisher = ValuesFinisher::new(self.custodian.shard_count, keys, transform);
        let finisher = Finisher::new(values_finisher);
        TxFinishableImpl {
            custodian,
            guards,
            ops,
            finisher,
        }
    }
}

impl<'txmap, L, K, V> IntoTransaction<'txmap, L, K, V, NoneFinisher>
    for TxBuildableImpl<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn into_transaction(self) -> Transaction<'txmap, L, K, V, NoneFinisher> {
        let Self {
            custodian,
            guards,
            ops,
            ..
        } = self;
        let mut guards_bitmask = BitMask::default();
        for guard in &guards {
            guards_bitmask |= guard.guards_bitmask;
        }
        for op in &ops {
            guards_bitmask |= op.guards_bitmask();
        }
        let base = TransactionBase::<L, K, V, (), NoneFinisher> {
            custodian,
            guards_bitmask,
            guards,
            ops,
            finisher: Finisher::new(NoneFinisher),
        };
        Transaction { base }
    }
}
