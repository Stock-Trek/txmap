use crate::{
    builders::{
        builder_traits::{TxGuardParamBuilder, TxOpParamBuilder, TxParamBuildable, TxParamBuilder},
        param_buildable_impl::TxParamBuildableImpl,
    },
    custodian::Custodian,
    guard::Guard,
    indexer::Indexer,
};
use std::hash::Hash;

pub struct TxParamBuilderImpl<'txmap, K, V, P> {
    pub(crate) indexer: Indexer,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V, P>>,
}

impl<'txmap, K, V, P> TxParamBuilder<'txmap, K, V, P> for TxParamBuilderImpl<'txmap, K, V, P>
where
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
}

impl<'txmap, K, V, P> TxGuardParamBuilder<'txmap, K, V, P> for TxParamBuilderImpl<'txmap, K, V, P>
where
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    fn require<const N: usize, C>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxParamBuilder<'txmap, K, V, P>
    where
        C: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let guard = Guard::new_with_params(self.indexer, name.as_ref().into(), keys, condition);
        self.guards.push(guard);
        self
    }
}

impl<'txmap, K, V, P> TxOpParamBuilder<'txmap, K, V, P> for TxParamBuilderImpl<'txmap, K, V, P>
where
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        G: Fn(&K, &P) -> V + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn insert_default(self, key: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone,
        V: Default,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default(key)
    }
    fn modify<M>(self, key: K, mutate: M) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
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
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
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
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static,
        G: Fn(&K, &P) -> V + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
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
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        G: Fn(&K, &P) -> V + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek_or_insert_with(key, peek_keys, mutate, value_generator)
    }
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static,
        K: Clone,
        V: Default,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
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
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        K: Clone,
        V: Default,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek_or_default(key, peek_keys, mutate)
    }
    fn map<T>(self, key: K, transform: T) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        T: Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
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
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.map_peek(key, transform, peek_keys)
    }

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.swap_value(a, b)
    }
    fn move_value(self, from: K, to: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.move_value(from, to)
    }

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove(keys)
    }
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_where(keys, condition)
    }
    fn retain_only<I>(self, keys: I) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain_only(keys)
    }
    fn retain_where<I, C>(self, keys: I, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain_where(keys, condition)
    }

    // global ops
    fn clear(self) -> impl TxParamBuildable<'txmap, K, V, P> {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.clear()
    }
    fn remove_if<C>(self, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_if(condition)
    }
    fn retain<C>(self, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let Self {
            indexer,
            custodian,
            guards,
        } = self;
        let builder = TxParamBuildableImpl {
            indexer,
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.retain(condition)
    }
}
