use crate::{
    builder_traits::{TxBuildable, TxBuilder, TxGuardBuilder, TxOpBuilder},
    custodian::Custodian,
    indexer::Indexer,
    parameterized_tx_builder_impl::ParameterizedTxBuilderImpl,
    tx_buildable_impl::TxBuildableImpl,
    tx_builder_impl::TxBuilderImpl,
};
use std::hash::Hash;

pub struct TxStemBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
}

impl<'txmap, K, V> TxStemBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
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

impl<'txmap, K, V> TxGuardBuilder<'txmap, K, V> for TxStemBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn require<const N: usize, C>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
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
            guards: Vec::new(),
        };
        builder.require(name, keys, condition)
    }
}

impl<'txmap, K, V> TxOpBuilder<'txmap, K, V> for TxStemBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn() -> V + 'static,
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
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn remove(self, key: K) -> impl TxBuildable<'txmap, K, V> {
        let Self {
            indexer,
            owned_key,
            custodian,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            owned_key,
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.remove(key)
    }
    fn map<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(Option<&V>) -> Option<V> + 'static,
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
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.map(key, transform)
    }
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
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
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.map_peek(key, transform, context_keys)
    }
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&mut V) + 'static,
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
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.modify(key, mutate)
    }
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&mut V) + 'static,
        G: Fn() -> V + 'static,
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
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.modify_or_insert_with(key, mutate, value_generator)
    }
}
