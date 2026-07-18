use crate::{
    builder_traits::{IntoTransaction, TxBuildable, TxOpBuilder, TxResultBuilder},
    custodian::Custodian,
    guard::Guard,
    indexer::Indexer,
    op::Op,
    ops::{
        clear_op::ClearOp, insert_default_op::InsertDefaultOp, insert_with_op::InsertWithOp,
        map_op::MapOp, map_peek_op::MapPeekOp, modify_op::ModifyOp,
        modify_or_default_op::ModifyOrDefaultOp, modify_or_insert_with_op::ModifyOrInsertWithOp,
        modify_peek_op::ModifyPeekOp, modify_peek_or_default_op::ModifyPeekOrDefaultOp,
        modify_peek_or_insert_with_op::ModifyPeekOrInsertWithOp, move_value_op::MoveValueOp,
        remove_any_if_op::RemoveAnyIfOp, remove_if_op::RemoveIfOp, remove_op::RemoveOp,
        retain_any_if_op::RetainAnyIfOp, retain_if_op::RetainIfOp, retain_op::RetainOp,
        swap_value_op::SwapValueOp,
    },
    transaction::Transaction,
};
use std::hash::Hash;

pub struct TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Op<K, V>>,
}

impl<'txmap, K, V> TxBuildable<'txmap, K, V> for TxBuildableImpl<'txmap, K, V> where
    K: Clone + Hash + Eq
{
}

impl<'txmap, K, V> TxOpBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    // single key ops
    fn insert_with<G>(mut self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
    {
        let op = InsertWithOp::new(&self.indexer, key, value_generator);
        self.ops.push(Op::InsertWith(op));
        self
    }
    fn insert_default(mut self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        V: Default,
    {
        let op = InsertDefaultOp::new(&self.indexer, key);
        self.ops.push(Op::InsertDefault(op));
        self
    }
    fn modify<M>(mut self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let op = ModifyOp::new(&self.indexer, key, mutate);
        self.ops.push(Op::Modify(op));
        self
    }
    fn modify_peek<const N: usize, M>(
        mut self,
        key: K,
        context_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    {
        let op = ModifyPeekOp::new(&self.indexer, key, context_keys, mutate);
        self.ops.push(Op::ModifyPeek(op));
        self
    }
    fn modify_or_insert_with<M, G>(
        mut self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        let op = ModifyOrInsertWithOp::new(&self.indexer, key, mutate, value_generator);
        self.ops.push(Op::ModifyOrInsertWith(op));
        self
    }
    fn modify_peek_or_insert_with<const N: usize, M, G>(
        mut self,
        key: K,
        context_keys: [K; N],
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        let op = ModifyPeekOrInsertWithOp::new(
            &self.indexer,
            key,
            context_keys,
            mutate,
            value_generator,
        );
        self.ops.push(Op::ModifyPeekOrInsertWith(op));
        self
    }
    fn modify_or_default<M>(mut self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        V: Default,
    {
        let op = ModifyOrDefaultOp::new(&self.indexer, key, mutate);
        self.ops.push(Op::ModifyOrDefault(op));
        self
    }
    fn modify_peek_or_default<const N: usize, M>(
        mut self,
        key: K,
        context_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        V: Default,
    {
        let op = ModifyPeekOrDefaultOp::new(&self.indexer, key, context_keys, mutate);
        self.ops.push(Op::ModifyPeekOrDefault(op));
        self
    }
    fn map<T>(mut self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
    {
        let map_op = MapOp::new(&self.indexer, key, transform);
        self.ops.push(Op::Map(map_op));
        self
    }
    fn map_peek<const N: usize, T>(
        mut self,
        key: K,
        transform: T,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let map_peek_op = MapPeekOp::new(&self.indexer, key, context_keys, transform);
        self.ops.push(Op::MapPeek(map_peek_op));
        self
    }

    // multi key ops
    fn swap_value(mut self, a: K, b: K) -> impl TxBuildable<'txmap, K, V> {
        let op = SwapValueOp::new(&self.indexer, a, b);
        self.ops.push(Op::SwapValue(op));
        self
    }
    fn move_value(mut self, from: K, to: K) -> impl TxBuildable<'txmap, K, V> {
        let op = MoveValueOp::new(&self.indexer, from, to);
        self.ops.push(Op::MoveValue(op));
        self
    }

    // batch ops
    fn remove<I>(mut self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let op = RemoveOp::new(&self.indexer, keys);
        self.ops.push(Op::Remove(op));
        self
    }
    fn remove_if<I, C>(mut self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RemoveIfOp::new(&self.indexer, keys, condition);
        self.ops.push(Op::RemoveIf(op));
        self
    }
    fn retain<I>(mut self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let op = RetainOp::new(&self.indexer, keys);
        self.ops.push(Op::Retain(op));
        self
    }
    fn retain_if<I, C>(mut self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RetainIfOp::new(&self.indexer, keys, condition);
        self.ops.push(Op::RetainIf(op));
        self
    }

    // global ops
    fn clear(mut self) -> impl TxBuildable<'txmap, K, V> {
        let op = ClearOp::new(&self.indexer);
        self.ops.push(Op::Clear(op));
        self
    }
    fn remove_any_if<C>(mut self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RemoveAnyIfOp::new(&self.indexer, condition);
        self.ops.push(Op::RemoveAnyIf(op));
        self
    }
    fn retain_any_if<C>(mut self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let op = RetainAnyIfOp::new(&self.indexer, condition);
        self.ops.push(Op::RetainAnyIf(op));
        self
    }
}

impl<'txmap, K, V> TxResultBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn get<T, R>(self, _key: K, _transform: T) -> impl IntoTransaction<'txmap, K, V, Option<R>>
    where
        T: Fn(&K, &V) -> R,
    {
        // Create a MapOp that preserves the key, then convert to transaction
        // For the result, we store a dummy getter since the actual value
        // extraction happens outside the transaction execution
        let Self {
            owned_key,
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
        Transaction {
            owned_key,
            custodian,
            guards_bitmask,
            guards,
            ops,
            get: Box::new(|_: &V| -> Option<R> { None }),
        }
    }
    fn get_all<I, T, R>(
        self,
        _keys: I,
        _transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, Vec<Option<R>>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R,
    {
        let Self {
            owned_key,
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
        Transaction {
            owned_key,
            custodian,
            guards_bitmask,
            guards,
            ops,
            get: Box::new(|_: &V| -> Vec<Option<R>> { Vec::new() }),
        }
    }
}

impl<'txmap, K, V, R> IntoTransaction<'txmap, K, V, R> for TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, R> {
        let Self {
            owned_key,
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
        Transaction {
            owned_key,
            custodian,
            guards_bitmask,
            guards,
            ops,
            get: Box::new(|_: &V| -> R { panic!("No getter set") }),
        }
    }
}
