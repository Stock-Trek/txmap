use crate::{
    finishers::{
        finisher_trait::FinisherTrait, none_finisher::NoneFinisher, value_finisher::ValueFinisher,
        values_finisher::ValuesFinisher,
    },
    transaction::Transaction,
};
use std::hash::Hash;

pub trait TxBuilder<'txmap, K, V>:
    TxGuardBuilder<'txmap, K, V> + TxOpBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
}

pub trait TxBuildable<'txmap, K, V>:
    TxOpBuilder<'txmap, K, V>
    + TxResultBuilder<'txmap, K, V>
    + IntoTransaction<'txmap, K, V, NoneFinisher>
where
    K: Clone + Hash + Eq,
{
}

pub trait TxGuardBuilder<'txmap, K, V>
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
        C: Fn([Option<&V>; N]) -> bool + 'static;
}

pub trait TxOpBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
{
    // single key ops
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, K, V>
    where
        G: Fn(&K) -> V + 'static;
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, K, V>
    where
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
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static;
    fn modify_or_insert_with<M, G>(
        self,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        G: Fn(&K) -> V + 'static;
    fn modify_peek_or_insert_with<const N: usize, M, G>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        G: Fn(&K) -> V + 'static;
    fn modify_or_default<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V) + 'static,
        V: Default;
    fn modify_peek_or_default<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        V: Default;
    fn map<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static;
    fn map_peek<const N: usize, T>(
        self,
        key: K,
        transform: T,
        peek_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static;

    // multi key ops
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, K, V>;
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, K, V>;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn remove_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static;
    fn retain<I>(self, keys: I) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>;
    fn retain_if<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static;

    // global ops
    fn clear(self) -> impl TxBuildable<'txmap, K, V>;
    fn remove_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static;
    fn retain_any_if<C>(self, condition: C) -> impl TxBuildable<'txmap, K, V>
    where
        C: Fn(&K, &V) -> bool + 'static;
}

pub trait TxResultBuilder<'txmap, K, V>
where
    K: Clone + Hash + Eq,
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

pub trait IntoTransaction<'txmap, K, V, F>
where
    K: Clone + Hash + Eq,
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, F>;
}
