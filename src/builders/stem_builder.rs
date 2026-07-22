use crate::{
    builders::{
        buildable_impl::TxBuildableImpl,
        builder_impl::TxBuilderImpl,
        builder_traits::{
            TxBuildable, TxBuilder, TxGuardBuilder, TxOpBuilder, TxParamBuilder, TxParameterizer,
        },
        param_builder_impl::TxParamBuilderImpl,
    },
    custodian::Custodian,
    locks::lock_policy::LockPolicy,
};
use std::hash::Hash;

pub struct TxStemBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
{
    pub(crate) custodian: &'txmap Custodian<L, K, V>,
}

impl<'txmap, L, K, V> TxBuilder<'txmap, L, K, V> for TxStemBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
}

impl<'txmap, L, K, V> TxParameterizer<'txmap, L, K, V> for TxStemBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
    fn with_param<P>(self) -> impl TxParamBuilder<'txmap, L, K, V, P>
    where
        P: 'static,
    {
        let Self { custodian } = self;
        TxParamBuilderImpl {
            custodian,
            guards: Vec::new(),
        }
    }
}

impl<'txmap, L, K, V> TxGuardBuilder<'txmap, L, K, V> for TxStemBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
    fn require<const N: usize, C>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxBuilder<'txmap, L, K, V>
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let Self { custodian } = self;
        let builder = TxBuilderImpl {
            custodian,
            guards: Vec::new(),
        };
        builder.require(name, keys, condition)
    }
}

impl<'txmap, L, K, V> TxOpBuilder<'txmap, L, K, V> for TxStemBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
{
    // single key ops
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.insert_default(key)
    }
    fn insert_default_if_absent(self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.insert_default_if_absent(key)
    }
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, L, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn insert_with_if_absent<G>(
        self,
        key: K,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.insert_with_if_absent(key, value_generator)
    }
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, L, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.modify(key, mutate)
    }
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.modify_peek(key, peek_keys, mutate)
    }
    fn update<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, L, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.update(key, transform)
    }
    fn update_peek<const N: usize, T>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.update_peek(key, peek_keys, transform)
    }

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.move_value(from, to)
    }
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.swap_value(a, b)
    }

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, L, K, V>
    where
        I: IntoIterator<Item = K>,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.remove(keys)
    }
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, L, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        let Self { custodian } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards: Vec::new(),
            ops: Vec::new(),
        };
        builder.remove_where(keys, condition)
    }
}
