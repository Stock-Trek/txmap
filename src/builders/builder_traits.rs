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
    fn require<const N: usize>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: impl Fn([Option<&V>; N]) -> bool + 'static,
    ) -> impl TxBuilder<'txmap, L, K, V>;
}

pub trait TxGuardParamBuilder<'txmap, L, K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn require<const N: usize>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        condition: impl Fn([Option<&V>; N], &P) -> bool + 'static,
    ) -> impl TxParamBuilder<'txmap, L, K, V, P>;
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
    fn insert_with(
        self,
        key: K,
        value_generator: impl Fn(&K) -> V + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn insert_with_if_absent(
        self,
        key: K,
        value_generator: impl Fn(&K) -> V + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn modify(
        self,
        key: K,
        mutate: impl Fn(&K, &mut V) + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>;
    fn modify_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: impl Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn update(
        self,
        key: K,
        transform: impl Fn(&K, Option<&V>) -> Option<V> + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn update_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: impl Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;
    fn swap_value(self, a: K, b: K) -> impl TxBuildable<'txmap, L, K, V>
    where
        K: Clone;

    // batch ops
    fn remove(self, keys: impl IntoIterator<Item = K>) -> impl TxBuildable<'txmap, L, K, V>;
    fn remove_where(
        self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V) -> bool + 'static,
    ) -> impl TxBuildable<'txmap, L, K, V>;
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
    fn insert_with(
        self,
        key: K,
        value_generator: impl Fn(&K, &P) -> V + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn insert_with_if_absent(
        self,
        key: K,
        value_generator: impl Fn(&K, &P) -> V + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn modify(
        self,
        key: K,
        mutate: impl Fn(&K, &mut V, &P) + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>;
    fn modify_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        mutate: impl Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn update(
        self,
        key: K,
        transform: impl Fn(&K, Option<&V>, &P) -> Option<V> + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn update_peek<const N: usize>(
        self,
        key: K,
        peek_keys: [K; N],
        transform: impl Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;

    // multi key ops
    fn move_value(self, from: K, to: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;
    fn swap_value(self, a: K, b: K) -> impl TxParamBuildable<'txmap, L, K, V, P>
    where
        K: Clone;

    // batch ops
    fn remove(self, keys: impl IntoIterator<Item = K>)
    -> impl TxParamBuildable<'txmap, L, K, V, P>;
    fn remove_where(
        self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V, &P) -> bool + 'static,
    ) -> impl TxParamBuildable<'txmap, L, K, V, P>;
}

pub trait TxResultBuilder<'txmap, L, K, V>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn get_copied(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CopyFinisher<K, V>>
    where
        V: Copy;
    fn get_all_copied(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoTransaction<'txmap, L, K, V, CopyAllFinisher<K, V>>
    where
        V: Copy;
    fn get_cloned(self, key: K) -> impl IntoTransaction<'txmap, L, K, V, CloneFinisher<K, V>>
    where
        V: Clone;
    fn get_all_cloned(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoTransaction<'txmap, L, K, V, CloneAllFinisher<K, V>>
    where
        V: Clone;
    fn get_with<R>(
        self,
        key: K,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValueFinisher<K, V, R>>;
    fn get_all_with<R>(
        self,
        keys: impl IntoIterator<Item = K>,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoTransaction<'txmap, L, K, V, ValuesFinisher<K, V, R>>;
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
    fn get_all_copied(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CopyAllFinisher<K, V>>
    where
        V: Copy;
    fn get_cloned(
        self,
        key: K,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CloneFinisher<K, V>>
    where
        V: Clone;
    fn get_all_cloned(
        self,
        keys: impl IntoIterator<Item = K>,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, CloneAllFinisher<K, V>>
    where
        V: Clone;
    fn get_with<R>(
        self,
        key: K,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, ValueFinisher<K, V, R>>;
    fn get_all_with<R>(
        self,
        keys: impl IntoIterator<Item = K>,
        transform: impl Fn(&K, &V) -> R + 'static,
    ) -> impl IntoParamTransaction<'txmap, L, K, V, P, ValuesFinisher<K, V, R>>;
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
