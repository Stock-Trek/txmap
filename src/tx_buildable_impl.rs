use crate::{
    builder_traits::{IntoTransaction, TxBuildable, WithOperation},
    custodian::Custodian,
    indexer::Indexer,
    operation::Operation,
    prerequisite::Prerequisite,
    transaction::Transaction,
};
use std::hash::Hash;

pub struct TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) prerequisites: Vec<Prerequisite<K, V>>,
    pub(crate) operations: Vec<Operation<K, V>>,
}

impl<'txmap, K, V> TxBuildable<'txmap, K, V> for TxBuildableImpl<'txmap, K, V> where K: Hash + Eq {}

impl<'txmap, K, V> WithOperation<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_operation<F>(mut self, key: K, operator: F) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>) -> Option<V> + 'static,
    {
        let operation = Operation::new(&self.indexer, key, operator);
        self.operations.push(operation);
        self
    }
    fn with_operation_and_context<const N: usize, F>(
        mut self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let operation = Operation::new_with_context(&self.indexer, key, operator, context_keys);
        self.operations.push(operation);
        self
    }
}

impl<'txmap, K, V> IntoTransaction<'txmap, K, V> for TxBuildableImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V> {
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
        Transaction {
            owned_key,
            custodian,
            guards_bitmask,
            prerequisites,
            operations,
        }
    }
}
