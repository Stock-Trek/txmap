use crate::{
    builders::{
        buildable_impl::TxBuildableImpl,
        builder_traits::{
            TxBuildable, TxBuilder, TxGuardBuilder, TxOpBuilder, TxParamBuilder, TxParameterizer,
        },
        param_builder_impl::TxParamBuilderImpl,
    },
    custodian::Custodian,
    guard::Guard,
    indexer::Indexer,
};
use std::hash::Hash;

pub struct TxBuilderImpl<'txmap, K, V> {
    pub(crate) indexer: Indexer,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
}

impl<'txmap, K, V> TxBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq + 'static,
    V: 'static,
{
}

impl<'txmap, K, V> TxParameterizer<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq + 'static,
    V: 'static,
{
    fn with_param<P>(self) -> impl TxParamBuilder<'txmap, K, V, P>
    where
        P: 'static,
    {
        let Self {
            indexer, custodian, ..
        } = self;
        TxParamBuilderImpl {
            custodian,
            guards: Vec::new(),
            indexer,
        }
    }
}

impl<'txmap, K, V> TxGuardBuilder<'txmap, K, V> for TxBuilderImpl<'txmap, K, V>
where
    K: Hash + Eq + 'static,
    V: 'static,
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
    K: Hash + Eq + 'static,
    V: 'static,
{
    // single key ops
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
        V: Default,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default(key)
    }
    fn insert_default_if_absent(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
        V: Default,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default_if_absent(key)
    }
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn insert_with_if_absent<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with_if_absent(key, value_generator)
    }
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
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
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek(key, peek_keys, mutate)
    }
    fn update<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.update(key, transform)
    }
    fn update_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.update_peek(key, transform, peek_keys)
    }

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.move_value(from, to)
    }
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.swap_value(a, b)
    }

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove(keys)
    }
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_where(keys, condition)
    }
    fn retain_only<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain_only(keys)
    }
    fn retain_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain_where(keys, condition)
    }

    // global ops
    fn clear(self) -> impl TxBuildable<'txmap, K, V> {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.clear()
    }
    fn remove_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_if(condition)
    }
    fn retain<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain(condition)
    }
}
