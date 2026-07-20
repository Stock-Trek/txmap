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
    fn insert_default_if_absent(self, key: K) -> impl TxParamBuildable<'txmap, K, V, P>
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
        builder.insert_default_if_absent(key)
    }
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
    fn insert_with_if_absent<G>(
        self,
        key: K,
        value_generator: G,
    ) -> impl TxParamBuildable<'txmap, K, V, P>
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
        builder.insert_with_if_absent(key, value_generator)
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
    fn update<T>(self, key: K, transform: T) -> impl TxParamBuildable<'txmap, K, V, P>
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
        builder.update(key, transform)
    }
    fn update_peek<const N: usize, T>(
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
        builder.update_peek(key, transform, peek_keys)
    }

    // multi key ops
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
