use crate::{
    finishers::{
        clone_all_finisher::CloneAllFinisher, clone_finisher::CloneFinisher,
        copy_all_finisher::CopyAllFinisher, copy_finisher::CopyFinisher,
        finisher_trait::FinisherTrait, none_finisher::NoneFinisher, value_finisher::ValueFinisher,
        values_finisher::ValuesFinisher,
    },
    locks::lock_policy::LockPolicy,
    transaction::{ParameterizedTransaction, Transaction},
};
use std::hash::Hash;

pub trait TxBuilder<'txmap, L, K, V>:
    TxGuardBuilder<'txmap, L, K, V> + TxOpBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
}

pub trait TxParamBuilder<'txmap, L, K, V, P>:
    TxGuardParamBuilder<'txmap, L, K, V, P> + TxOpParamBuilder<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
}

pub trait TxBuildable<'txmap, L, K, V>:
    TxOpBuilder<'txmap, L, K, V>
    + TxResultBuilder<'txmap, L, K, V>
    + IntoTransaction<'txmap, L, K, V, NoneFinisher>
where
    L: LockPolicy,
    K: Hash + Eq,
{
}

pub trait TxParamBuildable<'txmap, L, K, V, P>:
    TxOpParamBuilder<'txmap, L, K, V, P>
    + TxResultParamBuilder<'txmap, L, K, V, P>
    + IntoParamTransaction<'txmap, L, K, V, P, NoneFinisher>
where
    L: LockPolicy,
    K: Hash + Eq,
{
}

pub trait TxParameterizer<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn with_param<P>(self) -> impl TxParamBuilder<'txmap, L, K, V, P>
    where
        P: 'static;
}

pub trait TxGuardBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn require<const N: usize, C>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxBuilder<'txmap, L, K, V>
    where
        C: Fn([Option<&V>; N]) -> bool + 'static;
}

pub trait TxGuardParamBuilder<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn require<const N: usize, C>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: C,
    ) -> impl TxParamBuilder<'txmap, L, K, V, P>
    where
        C: Fn([Option<&V>; N], &P) -> bool + 'static;
}

pub trait TxOpBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    // single key ops
    fn insert_default(self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default;
    fn insert_default_if_absent(self, key: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone,
        V: Default;
    fn insert_with<G>(self, key: K, value_generator: G) -> impl TxBuildable<'txmap, L, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone;
    fn insert_with_if_absent<G>(
        self,
        key: K,
        value_generator: G,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        G: Fn(&K) -> V + 'static,
        K: Clone;
    fn modify<M>(self, key: K, mutate: M) -> impl TxBuildable<'txmap, L, K, V>
    where
        M: Fn(&K, &mut V) + 'static;
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
        K: Clone;
    fn update<T>(self, key: K, transform: T) -> impl TxBuildable<'txmap, L, K, V>
    where
        T: Fn(&K, Option<&V>) -> Option<V> + 'static,
        K: Clone;
    fn update_peek<const N: usize, T>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
        K: Clone;

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxBuildable<'txmap, L, K, V>
    where
        I: IntoIterator<Item = K>;
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxBuildable<'txmap, L, K, V>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static;
}

pub trait TxOpParamBuilder<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    // single key ops
    fn insert_default(self, key: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
        V: Default;
    fn insert_default_if_absent(self, key: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone,
        V: Default;
    fn insert_with<G>(
        self,
        key: K,
        value_generator: G,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        G: Fn(&K, &P) -> V + 'static,
        K: Clone;
    fn insert_with_if_absent<G>(
        self,
        key: K,
        value_generator: G,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        G: Fn(&K, &P) -> V + 'static,
        K: Clone;
    fn modify<M>(self, key: K, mutate: M) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        M: Fn(&K, &mut V, &P) + 'static;
    fn modify_peek<const N: usize, M>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
        K: Clone;
    fn update<T>(self, key: K, transform: T) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        T: Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
        K: Clone;
    fn update_peek<const N: usize, T>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
        K: Clone;

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn swap_value(self, a: K, b: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;

    // batch ops
    fn remove<I>(self, keys: I) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        I: IntoIterator<Item = K>;
    fn remove_where<I, C>(self, keys: I, condition: C) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static;
}

pub trait TxResultBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn get_copied(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CopyFinisher<K, V>>
    where
        V: Copy;
    fn get_all_copied<I>(
        self,
        keys: I,
    ) -> impl IntoTransaction<'txmap, L, K, V, CopyAllFinisher<K, V>>
    where
        I: IntoIterator<Item = K>,
        V: Copy;
    fn get_cloned(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CloneFinisher<K, V>>
    where
        V: Clone;
    fn get_all_cloned<I>(
        self,
        keys: I,
    ) -> impl IntoTransaction<'txmap, L, K, V, CloneAllFinisher<K, V>>
    where
        I: IntoIterator<Item = K>,
        V: Clone;
    fn get_with<T, R>(
        self,
        key: K,
        transform: T,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValueFinisher<K, V, R>>
    where
        T: Fn(&K, &V) -> R + 'static;
    fn get_all_with<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValuesFinisher<K, V, R>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static;
}

pub trait TxResultParamBuilder<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn get_copied(
        self,
        key: K,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CopyFinisher<K, V>>
    where
        V: Copy;
    fn get_all_copied<I>(
        self,
        keys: I,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CopyAllFinisher<K, V>>
    where
        I: IntoIterator<Item = K>,
        V: Copy;
    fn get_cloned(
        self,
        key: K,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CloneFinisher<K, V>>
    where
        V: Clone;
    fn get_all_cloned<I>(
        self,
        keys: I,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CloneAllFinisher<K, V>>
    where
        I: IntoIterator<Item = K>,
        V: Clone;
    fn get_with<T, R>(
        self,
        key: K,
        transform: T,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, ValueFinisher<K, V, R>>
    where
        T: Fn(&K, &V) -> R + 'static;
    fn get_all_with<I, T, R>(
        self,
        keys: I,
        transform: T,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, ValuesFinisher<K, V, R>>
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static;
}

pub trait IntoTransaction<'txmap, L, K, V, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    #[must_use]
    fn into_transaction(self) -> Transaction<'txmap, L, K, V, F>;
}

pub trait IntoParamTransaction<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    #[must_use]
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, L, K, V, P, F>;
}
