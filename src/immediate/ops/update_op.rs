use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct UpdateOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub transform: Box<dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
}
