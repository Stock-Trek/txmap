use crate::transaction::Transaction;
use std::hash::Hash;

pub trait TxBuilder<'txmap, K, V>:
    WithPrerequisite<'txmap, K, V> + WithOperation<'txmap, K, V>
where
    K: Hash + Eq,
{
}

pub trait TxBuildable<'txmap, K, V>:
    WithOperation<'txmap, K, V> + IntoTransaction<'txmap, K, V>
where
    K: Hash + Eq,
{
}

pub trait WithPrerequisite<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_prerequisite<const N: usize, F>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> impl TxBuilder<'txmap, K, V>
    where
        F: Fn([Option<&V>; N]) -> bool + 'static;
}

pub trait WithOperation<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn with_operation<F>(self, key: K, operator: F) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>) -> Option<V> + 'static;
    fn with_operation_and_context<const N: usize, F>(
        self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> impl TxBuildable<'txmap, K, V>
    where
        F: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static;
}

pub trait IntoTransaction<'txmap, K, V>
where
    K: Hash + Eq,
{
    fn build(self) -> Transaction<'txmap, K, V>;
}
