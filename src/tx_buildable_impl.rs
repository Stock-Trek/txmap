use crate::{
    builder_traits::{IntoTransaction, TxBuildable, TxOpBuilder, TxResultBuilder},
    custodian::Custodian,
    guard::Guard,
    indexer::Indexer,
    map_op::MapOp,
    mut_op::MutOp,
    op::Op,
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

impl<'txmap, K, V> TxBuildable<'txmap, K, V> for TxBuildableImpl<'txmap, K, V> where K: Hash + Eq {}

impl<'txmap, K, V> TxOpBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static;
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        V: Default;
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static;
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        context_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static;
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static;
    fn modify_peek_or_insert_with<const N: usize, M, G>(
        self,
        key: K,
        context_keys: [K; N],
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        G: Fn(&K) -> V + 'static;
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        V: Default;
    fn modify_peek_or_default<const N: usize, M>(
        self,
        key: K,
        context_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        V: Default;
    fn map<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static;
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static;

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, K, V>;
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, K, V>;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn remove_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool;
    fn retain<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn retain_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool;

    // global ops
    fn clear(self) -> impl TxBuildable<'txmap, K, V>;
    fn remove_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool;
    fn retain_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool;

    // fn insert_with<G>(mut self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    // where
    //     G: Fn() -> V + 'static,
    // {
    //     let map_op = MapOp::new(&self.indexer, key, move |_| Some(value_generator()));
    //     self.ops.push(Op::Map(map_op));
    //     self
    // }
    // fn remove(mut self, key: K) -> impl TxBuildable<'txmap, K, V> {
    //     let map_op = MapOp::new(&self.indexer, key, |_| None);
    //     self.ops.push(Op::Map(map_op));
    //     self
    // }
    // fn modify<M>(mut self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    // where
    //     M: Fn(&mut V) + 'static,
    // {
    //     let mut_op = MutOp::new(&self.indexer, key, mutate);
    //     self.ops.push(Op::Mut(mut_op));
    //     self
    // }
    // fn modify_or_insert_with<M, G>(
    //     mut self,
    //     key: K,
    //     mutate: M,
    //     value_generator: G,
    // ) -> impl TxBuildable<'txmap, K, V>
    // where
    //     M: Fn(&mut V) + 'static,
    //     G: Fn() -> V + 'static,
    // {
    //     let mut_op = MutOp::new_or_insert_with(&self.indexer, key, mutate, value_generator);
    //     self.ops.push(Op::Mut(mut_op));
    //     self
    // }
    // fn map<T>(mut self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    // where
    //     T: Fn(Option<&V>) -> Option<V> + 'static,
    // {
    //     let map_op = MapOp::new(&self.indexer, key, transform);
    //     self.ops.push(Op::Map(map_op));
    //     self
    // }
    // fn map_peek<const N: usize, T>(
    //     mut self,
    //     key: K,
    //     transform: T,
    //     context_keys: [K; N],
    // ) -> impl TxBuildable<'txmap, K, V>
    // where
    //     T: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    // {
    //     let map_op = MapOp::new_with_context(&self.indexer, key, transform, context_keys);
    //     self.ops.push(Op::Map(map_op));
    //     self
    // }
}

impl<'txmap, K, V> TxResultBuilder<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn get<T, R>(self, key: K, transform: T) -> impl IntoTransaction<'txmap, K, V, Option<R>>
    where
        T: Fn(&K, &V) -> R,
    {
    }
    fn get_all<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, Vec<Option<R>>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R,
    {
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
        }
    }
}
