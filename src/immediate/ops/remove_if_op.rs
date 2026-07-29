use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct RemoveIfOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, &V, &mut STATE) -> bool + 'tx>,
}
