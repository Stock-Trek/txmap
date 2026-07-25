use crate::{
    builders::{
        buildable_impl::TxBuildableImpl,
        builder_traits::{TxBuildable, TxBuilder, TxGuardBuilder, TxOpBuilder},
    },
    custodian::Custodian,
    guard::Guard,
    lock_policies::lock_policy::LockPolicy,
    params::{TxKey, TxKeySelector},
};
use std::{hash::Hash, marker::PhantomData};

pub struct TxBuilderImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuilderImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxGuardBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuilderImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn require(
        mut self,
        name: impl AsRef<str>,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(Option<&V>, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> impl TxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let guard = Guard {
            name: name.as_ref().into(),
            key_selector: Box::new(key_selector),
            condition: Box::new(condition),
            _phantom: PhantomData,
        };
        self.guards.push(guard);
        self
    }
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> TxOpBuilder<'tx, K, V, L, KEYS, PARAMS, STATE>
    for TxBuilderImpl<'tx, K, V, L, KEYS, PARAMS, STATE>
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
        V: Default,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl::<'tx, K, V, L, KEYS, PARAMS, STATE> {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default(key_selector)
    }
    fn insert_default_if_absent(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
        V: Default,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_default_if_absent(key_selector)
    }
    fn insert_with(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with(key_selector, value_generator)
    }
    fn insert_with_if_absent(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        value_generator: impl Fn(&K, &PARAMS, &mut STATE) -> V + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.insert_with_if_absent(key_selector, value_generator)
    }
    fn modify(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        mutate: impl Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.modify(key_selector, mutate)
    }
    fn move_value(
        self,
        key_selector_from: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_to: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.move_value(key_selector_from, key_selector_to)
    }
    fn remove(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove(key_selector)
    }
    fn remove_where(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        condition: impl Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE> {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.remove_where(key_selector, condition)
    }
    fn swap_value(
        self,
        key_selector_a: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        key_selector_b: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.swap_value(key_selector_a, key_selector_b)
    }
    fn update(
        self,
        key_selector: impl TxKeySelector<TxKey<K>, KEYS> + 'tx,
        transform: impl Fn(&K, Option<&V>, &PARAMS, &mut STATE) -> Option<V> + 'tx,
    ) -> impl TxBuildable<'tx, K, V, L, KEYS, PARAMS, STATE>
    where
        K: Clone,
    {
        let Self { custodian, guards } = self;
        let builder = TxBuildableImpl {
            custodian,
            guards,
            ops: Vec::new(),
        };
        builder.update(key_selector, transform)
    }
}
