use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct RemoveOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub on_remove: Box<dyn Fn(Option<(K, V)>, &mut STATE) + 'tx>,
}
