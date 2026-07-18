use crate::{
    custodian::Custodian,
    indexer::Indexer,
    parameterized_builder_traits::{
        ParameterizedTxBuildable, ParameterizedTxBuilder, WithParameterizedOperation,
        WithParameterizedPrerequisite,
    },
    parameterized_prerequisite::ParameterizedPrerequisite,
    parameterized_tx_buildable_impl::ParameterizedTxBuildableImpl,
};
use std::hash::Hash;

pub struct ParameterizedTxBuilderImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
}

impl<'txmap, K, V, P> ParameterizedTxBuilder<'txmap, K, V, P>
    for ParameterizedTxBuilderImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
}

impl<'txmap, K, V, P> WithParameterizedPrerequisite<'txmap, K, V, P>
    for ParameterizedTxBuilderImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn with_prerequisite<const N: usize, F>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> impl ParameterizedTxBuilder<'txmap, K, V, P>
    where
        F: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let prerequisite =
            ParameterizedPrerequisite::new(self.indexer, name.as_ref().into(), keys, prerequisite);
        self.prerequisites.push(prerequisite);
        self
    }
}

impl<'txmap, K, V, P> WithParameterizedOperation<'txmap, K, V, P>
    for ParameterizedTxBuilderImpl<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn with_operation<F>(
        self,
        key: K,
        operator: F,
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            prerequisites,
        } = self;
        let builder = ParameterizedTxBuildableImpl {
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
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            prerequisites,
        } = self;
        let builder = ParameterizedTxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            prerequisites,
            operations: Vec::new(),
        };
        builder.with_operation_and_context(key, operator, context_keys)
    }
}
