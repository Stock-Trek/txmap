use crate::{
    custodian::Custodian, indexer::Indexer,
    parameterized_transaction::ParameterizedTransactionBuilder, transaction::TransactionBuilder,
};
use std::hash::Hash;

pub struct TransactionBuilderStem<'txmap, K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    owned_key: fn(&K) -> K,
    custodian: &'txmap Custodian<K, V>,
}

impl<'txmap, K, V> TransactionBuilderStem<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub(crate) fn new(
        indexer: Indexer,
        owned_key: fn(&K) -> K,
        custodian: &'txmap Custodian<K, V>,
    ) -> Self {
        Self {
            indexer,
            owned_key,
            custodian,
        }
    }
    pub fn with_param<P>(self) -> ParameterizedTransactionBuilder<'txmap, K, V, P> {
        let TransactionBuilderStem {
            indexer,
            owned_key,
            custodian,
        } = self;
        ParameterizedTransactionBuilder::new(indexer, owned_key, custodian)
    }
    pub fn with_prerequisite<const N: usize, F>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> TransactionBuilder<'txmap, K, V>
    where
        F: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let TransactionBuilderStem {
            indexer,
            owned_key,
            custodian,
        } = self;
        let builder = TransactionBuilder::new(indexer, owned_key, custodian);
        builder.with_prerequisite(name, keys, prerequisite)
    }
    pub fn with_operation<F>(self, key: K, operator: F) -> TransactionBuilder<'txmap, K, V>
    where
        F: Fn(Option<&V>) -> Option<V> + 'static,
    {
        let TransactionBuilderStem {
            indexer,
            owned_key,
            custodian,
        } = self;
        let builder = TransactionBuilder::new(indexer, owned_key, custodian);
        builder.with_operation(key, operator)
    }
    pub fn with_operation_and_context<const N: usize, F>(
        self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> TransactionBuilder<'txmap, K, V>
    where
        F: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let TransactionBuilderStem {
            indexer,
            owned_key,
            custodian,
        } = self;
        let builder = TransactionBuilder::new(indexer, owned_key, custodian);
        builder.with_operation_and_context(key, operator, context_keys)
    }
}
