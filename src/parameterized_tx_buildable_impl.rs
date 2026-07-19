use crate::{
    custodian::Custodian,
    indexer::Indexer,
    ops::{map_op::MapOp, map_peek_op::MapPeekOp, op_trait::ParameterizedOpTrait},
    parameterized_builder_traits::{
        IntoParameterizedTransaction, ParameterizedTxBuildable, WithParameterizedOperation,
    },
    parameterized_prerequisite::ParameterizedPrerequisite,
    transaction::Transaction,
};
use std::hash::Hash;

pub struct ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    pub(crate) indexer: Indexer,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
    pub(crate) ops: Vec<Box<dyn ParameterizedOpTrait<K, V, P>>>,
}

impl<'txmap, K, V, P> ParameterizedTxBuildable<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
}

impl<'txmap, K, V, P> WithParameterizedOperation<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    fn with_operation<F>(
        mut self,
        key: K,
        operator: F,
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let op = MapOp::new_with_param(&self.indexer, key, move |_, v, params| operator(v, params));
        self.ops.push(Box::new(op));
        self
    }
    fn with_operation_and_context<const N: usize, F>(
        mut self,
        key: K,
        operator: F,
        peek_keys: [K; N],
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let op =
            MapPeekOp::new_with_param(&self.indexer, key, peek_keys, move |_, v, pks, params| {
                operator(v, pks, params)
            });
        self.ops.push(Box::new(op));
        self
    }
}

impl<'txmap, K, V, P> IntoParameterizedTransaction<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, P> {
        let Self {
            custodian,
            prerequisites,
            ops,
            ..
        } = self;
        let mut guards_bitmask: u128 = 0;
        for prerequisite in &prerequisites {
            guards_bitmask |= prerequisite.guards_bitmask;
        }
        for op in &ops {
            guards_bitmask |= op.guards_bitmask();
        }
        Transaction {
            custodian,
            guards_bitmask,
            guards: Vec::new(),
            param_prerequisites: prerequisites,
            ops,
            finisher: crate::finisher::Finisher::new(crate::finishers::none_finisher::NoneFinisher),
        }
    }
}
