use crate::{
    builder_traits::{TxBuildable, TxBuilder, WithOperation, WithPrerequisite},
    custodian::Custodian,
    indexer::Indexer,
    prerequisite::Prerequisite,
    tx_buildable_impl::TxBuildableImpl,
};
use std::hash::Hash;

pub struct TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) prerequisites: Vec<Prerequisite<K, V>>,
}

impl<'txmap, K, V> TxBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V> where K: Hash + Eq {}

impl<'txmap, K, V> WithPrerequisite<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_prerequisite<const N: usize, F>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        F: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let prerequisite =
            Prerequisite::new(self.indexer, name.as_ref().into(), keys, prerequisite);
        self.prerequisites.push(prerequisite);
        self
    }
}

impl<'txmap, K, V> WithOperation<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_operation<F>(self, key: K, operator: F) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            prerequisites,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            prerequisites,
            operations: Vec::new(),
        };
        builder.with_operation(key, operator)
    }
    fn with_operation_and_context<const N: usize, F>(
        self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            prerequisites,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            prerequisites,
            operations: Vec::new(),
        };
        builder.with_operation_and_context(key, operator, context_keys)
    }
}
