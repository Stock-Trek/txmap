use crate::{
    custodian::Custodian,
    indexer::Indexer,
    parameterized_builder_traits::{
        IntoParameterizedTransaction, ParameterizedTxBuildable, WithParameterizedOperation,
    },
    parameterized_operation::ParameterizedOperation,
    parameterized_prerequisite::ParameterizedPrerequisite,
    parameterized_transaction::ParameterizedTransaction,
};
use std::hash::Hash;

pub struct ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
    pub(crate) operations: Vec<ParameterizedOperation<K, V, P>>,
}

impl<'txmap, K, V, P> ParameterizedTxBuildable<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Hash + Eq,
{
}

impl<'txmap, K, V, P> WithParameterizedOperation<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    fn with_operation<F>(
        mut self,
        key: K,
        operator: F,
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let operation = ParameterizedOperation::new(&self.indexer, key, operator);
        self.operations.push(operation);
        self
    }
    fn with_operation_and_context<const N: usize, F>(
        mut self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let operation =
            ParameterizedOperation::new_with_context(&self.indexer, key, operator, context_keys);
        self.operations.push(operation);
        self
    }
}

impl<'txmap, K, V, P> IntoParameterizedTransaction<'txmap, K, V, P>
    for ParameterizedTxBuildableImpl<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, K, V, P> {
        let Self {
            owned_key,
            custodian,
            prerequisites,
            operations,
            ..
        } = self;
        let mut guards_bitmask: u128 = 0;
        for prerequisite in &prerequisites {
            guards_bitmask |= prerequisite.guards_bitmask;
        }
        for operation in &operations {
            guards_bitmask |= operation.guards_bitmask;
        }
        ParameterizedTransaction {
            owned_key,
            custodian,
            guards_bitmask,
            prerequisites,
            operations,
        }
    }
}
