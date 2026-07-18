use crate::{
    builder_traits::{TxBuildable, TxBuilder, WithOperation, WithPrerequisite},
    custodian::Custodian,
    indexer::Indexer,
    parameterized_tx_builder_impl::ParameterizedTxBuilderImpl,
    tx_buildable_impl::TxBuildableImpl,
    tx_builder_impl::TxBuilderImpl,
};
use std::hash::Hash;

pub struct TxStemBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
}

impl<'txmap, K, V> TxStemBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub fn with_param<P>(self) -> ParameterizedTxBuilderImpl<'txmap, K, V, P> {
        let Self {
            indexer,
            owned_key,
            custodian,
        } = self;
        ParameterizedTxBuilderImpl {
            indexer,
            owned_key,
            custodian,
            prerequisites: Vec::new(),
        }
    }
}

impl<'txmap, K, V> TxBuilder<'txmap, K, V> for TxStemBuilder<'txmap, K, V> where K: Hash + Eq {}

impl<'txmap, K, V> WithPrerequisite<'txmap, K, V> for TxStemBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_prerequisite<const N: usize, F>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        F: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
        } = self;
        let builder = TxBuilderImpl {
            indexer,
            owned_key,
            custodian,
            prerequisites: Vec::new(),
        };
        builder.with_prerequisite(name, keys, prerequisite)
    }
}

impl<'txmap, K, V> WithOperation<'txmap, K, V> for TxStemBuilder<'txmap, K, V>
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
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            owned_key,
            custodian,
            prerequisites: Vec::new(),
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
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            owned_key,
            custodian,
            prerequisites: Vec::new(),
            operations: Vec::new(),
        };
        builder.with_operation_and_context(key, operator, context_keys)
    }
}
