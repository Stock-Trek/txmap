use crate::{
    builder_traits::{TxBuildable, TxBuilder, TxGuardBuilder, TxOpBuilder},
    custodian::Custodian,
    guard::Guard,
    indexer::Indexer,
    tx_buildable_impl::TxBuildableImpl,
};
use std::hash::Hash;

pub struct TxBuilderImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    pub(crate) indexer: Indexer,
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
}

impl<'txmap, K, V> TxBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V> where K: Clone + Hash + Eq
{}

impl<'txmap, K, V> TxGuardBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    fn require<const N: usize, C>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let guard = Guard::new(self.indexer, name.as_ref().into(), keys, condition);
        self.guards.push(guard);
        self
    }
}

impl<'txmap, K, V> TxOpBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        V: Default,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default(key)
    }
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify(key, mutate)
    }
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek(key, peek_keys, mutate)
    }
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify_or_insert_with(key, mutate, value_generator)
    }
    fn modify_peek_or_insert_with<const N: usize, M, G>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        G: Fn(&K) -> V + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek_or_insert_with(key, peek_keys, mutate, value_generator)
    }
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        V: Default,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify_or_default(key, mutate)
    }
    fn modify_peek_or_default<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        V: Default,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek_or_default(key, peek_keys, mutate)
    }
    fn map<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.map(key, transform)
    }
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.map_peek(key, transform, peek_keys)
    }

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, K, V> {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.swap_value(a, b)
    }
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, K, V> {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.move_value(from, to)
    }

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.remove(keys)
    }
    fn remove_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.remove_if(keys, condition)
    }
    fn retain<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.retain(keys)
    }
    fn retain_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.retain_if(keys, condition)
    }

    // global ops
    fn clear(self) -> impl TxBuildable<'txmap, K, V> {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.clear()
    }
    fn remove_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.remove_any_if(condition)
    }
    fn retain_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            owned_key,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            owned_key,
            guards,
            ops: Vec::new(),
        };
        builder.retain_any_if(condition)
    }
}
