use crate::{
    lock_policies::lock_policy::LockPolicy,
    params::{TxKey, TxKeySelector},
    transaction::Transaction,
};
use std::hash::Hash;

pub trait TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>:
    TxGuardBuilder<'tx, K, V, L, KEYS, PARAMS, STATE> + TxOpBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
}

pub trait TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>:
    TxOpBuilder<'tx, K, V, L, KEYS, PARAMS, STATE> + IntoTransaction<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
}

pub trait TxGuardBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn require(
        self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> impl TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>;
}

pub trait TxOpBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn insert_default(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
        V: Default;
    fn insert_default_if_absent(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
        V: Default;
    fn insert_with(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone;
    fn insert_with_if_absent(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone;
    fn modify(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>;
    fn move_value(
        self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone;
    fn remove(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>;
    fn remove_where(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>;
    fn swap_value(
        self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone;
    fn update(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone;
}

pub trait IntoTransaction<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    #[must_use]
    fn into_transaction(self) -> Transaction<'tx, K, V, L, KEYS, PARAMS, STATE>;
}
