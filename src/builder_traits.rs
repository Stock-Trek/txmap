use crate::{
    finishers::{
        finisher_trait::FinisherTrait, none_finisher::NoneFinisher, value_finisher::ValueFinisher,
        values_finisher::ValuesFinisher,
    },
    parameterized_transaction::ParameterizedTransaction,
    transaction::Transaction,
};
use std::hash::Hash;

pub trait TxBuilder<'txmap, K, V>:
    TxGuardBuilder<'txmap, K, V> + TxParamBuilder<'txmap, K, V> + TxOpBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
}

pub trait TxBuildable<'txmap, K, V>:
    TxOpBuilder<'txmap, K, V>
    + TxResultBuilder<'txmap, K, V>
    + IntoTransaction<'txmap, K, V, NoneFinisher>
where
    K: Hash + Eq,
{
}

pub trait TxParamBuildable<'txmap, K, V, P>:
    TxOpParamBuilder<'txmap, K, V, P>
    + TxResultParamBuilder<'txmap, K, V, P>
    + IntoParamTransaction<'txmap, K, V, P, NoneFinisher>
where
    K: Hash + Eq,
{
}

pub trait TxGuardBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn require<const N: usize, C>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        C: Fn([Option<&V>; N]) -> bool + 'static;
}

pub trait TxParamBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_param<P>(self) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        P: 'static;
}

pub trait TxOpBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone;
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone,
        V: Default;
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static;
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        K: Clone;
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static,
        K: Clone;
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
        K: Clone;
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        K: Clone,
        V: Default;
    fn modify_peek_or_default<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        K: Clone,
        V: Default;
    fn map<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
        K: Clone;
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
        K: Clone;

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone;
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, K, V>
    where
        K: Clone;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static;
    fn retain_only<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn retain_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static;

    // global ops
    fn clear(self) -> impl TxBuildable<'txmap, K, V>;
    fn remove_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static;
    fn retain<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static;
}

pub trait TxOpParamBuilder<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        G: Fn(&K, &P) -> V + 'static,
        K: Clone;
    fn insert_default(self, key: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone,
        V: Default;
    fn modify<M>(self, key: K, mutate: M) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static;
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        K: Clone;
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static,
        G: Fn(&K, &P) -> V + 'static,
        K: Clone;
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
        K: Clone;
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static,
        K: Clone,
        V: Default;
    fn modify_peek_or_default<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        K: Clone,
        V: Default;
    fn map<T>(self, key: K, transform: T) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        T: Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
        K: Clone;
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
        K: Clone;

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone;
    fn move_value(self, from: K, to: K) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        K: Clone;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>;
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static;
    fn retain_only<I>(self, keys: I) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>;
    fn retain_where<I, C>(self, keys: I, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static;

    // global ops
    fn clear(self) -> impl TxParamBuildable<'txmap, K, V, P>;
    fn remove_if<C>(self, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        C: Fn(&K, &V, &P) -> bool + 'static;
    fn retain<C>(self, condition: C) -> impl TxParamBuildable<'txmap, K, V, P>
    where
        C: Fn(&K, &V, &P) -> bool + 'static;
}

pub trait TxResultBuilder<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn get<T, R>(
        self,
        key: K,
        transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, ValueFinisher<K, V, R>>
    where
        T: Fn(&K, &V) -> R + 'static;
    fn get_all<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoTransaction<'txmap, K, V, ValuesFinisher<K, V, R>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static;
}

pub trait TxResultParamBuilder<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    fn get<T, R>(
        self,
        key: K,
        transform: T,
    ) -> impl IntoParamTransaction<'txmap, K, V, P, ValueFinisher<K, V, R>>
    where
        T: Fn(&K, &V) -> R + 'static;
    fn get_all<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoParamTransaction<'txmap, K, V, P, ValuesFinisher<K, V, R>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static;
}

pub trait IntoTransaction<'txmap, K, V, F>
where
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, F>;
}

pub trait IntoParamTransaction<'txmap, K, V, P, F>
where
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, K, V, P, F>;
}
