use crate::parameterized_transaction::ParameterizedTransaction;
use std::hash::Hash;

pub trait ParameterizedTxBuilder<'txmap, K, V, P>:
    WithParameterizedPrerequisite<'txmap, K, V, P> + WithParameterizedOperation<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
}

pub trait ParameterizedTxBuildable<'txmap, K, V, P>:
    WithParameterizedOperation<'txmap, K, V, P> + IntoParameterizedTransaction<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
}

pub trait WithParameterizedPrerequisite<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn with_prerequisite<const N: usize, F>(
        self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> impl ParameterizedTxBuilder<'txmap, K, V, P>
    where
        F: Fn([Option<&V>; N], &P) -> bool + 'static;
}

pub trait WithParameterizedOperation<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn with_operation<F>(
        self,
        key: K,
        operator: F,
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static;
    fn with_operation_and_context<const N: usize, F>(
        self,
        key: K,
        operator: F,
        peek_keys: [K; N],
    ) -> impl ParameterizedTxBuildable<'txmap, K, V, P>
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static;
}

pub trait IntoParameterizedTransaction<'txmap, K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, K, V, P>;
}
