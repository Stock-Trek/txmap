use crate::{
    builders::{
        builder_traits::{TxGuardParamBuilder, TxOpParamBuilder, TxParamBuildable, TxParamBuilder},
        param_buildable_impl::TxParamBuildableImpl,
    },
    custodian::Custodian,
    guard::Guard,
    locks::lock_policy::LockPolicy,
};
use std::hash::Hash;

pub struct TxParamBuilderImpl<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    pub(crate) custodian: &'txmap Custodian<L, K, V>,
    pub(crate) guards: Vec<Guard<K, V, P>>,
}

impl<'txmap, L, K, V, P> TxParamBuilder<'txmap, L, K, V, P>
    for TxParamBuilderImpl<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
}

impl<'txmap, L, K, V, P> TxGuardParamBuilder<'txmap, L, K, V, P>
    for TxParamBuilderImpl<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    fn require<const N: usize>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: impl Fn([Option<&V>; N], &P) -> bool + 'static,
    ) -> impl TxParamBuilder<'txmap, L, K, V, P> {
        let guard = Guard::new_with_params(
            self.custodian.shard_count,
            name.as_ref().into(),
            keys,
            condition,
        );
        self.guards.push(guard);
        self
    }
}

impl<'txmap, L, K, V, P> TxOpParamBuilder<'txmap, L, K, V, P>
    for TxParamBuilderImpl<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq + 'static,
    V: 'static,
    P: 'static,
{
    // single key ops
    fn insert_default(self, key: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
        V: Default,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default(key)
    }
    fn insert_default_if_absent(self, key: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
        V: Default,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default_if_absent(key)
    }
    fn insert_with(
        self,
        key: K,
        value_generator: impl Fn(&K, &P) -> V + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with(key, value_generator)
    }
    fn insert_with_if_absent(
        self,
        key: K,
        value_generator: impl Fn(&K, &P) -> V + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with_if_absent(key, value_generator)
    }
    fn modify(
        self,
        key: K,
        mutate: impl Fn(&K, &mut V, &P) + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P> {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify(key, mutate)
    }
    fn modify_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: impl Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify_peek(key, peek_keys, mutate)
    }
    fn update(
        self,
        key: K,
        transform: impl Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.update(key, transform)
    }
    fn update_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: impl Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.update_peek(key, peek_keys, transform)
    }

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.move_value(from, to)
    }
    fn swap_value(self, a: K, b: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.swap_value(a, b)
    }

    // batch ops
    fn remove(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P> {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove(keys)
    }
    fn remove_where(
        self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V, &P) -> bool + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P> {
        let Self { custodian, guards } = self;
        let builder = TxParamBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_where(keys, condition)
    }
}
